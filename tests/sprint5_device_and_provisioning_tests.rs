//! Sprint 5 Phase 1 & 2 coverage: local device orchestration and the CI/CD
//! infrastructure provisioning simulator.
//!
//! `start_android_emulator` is the only function here that touches the OS, and
//! it is exercised only through a binary guaranteed not to exist — these tests
//! must never launch a real emulator or Docker container.

use cherenkov_lings::device_manager::{DeviceError, DeviceManager};
use cherenkov_lings::pipeline::{PROVISIONING_TOOLS, provisioning_steps};

// ---------------------------------------------------------------------------
// DeviceManager — Android emulator
// ---------------------------------------------------------------------------

#[test]
fn emulator_command_builds_the_documented_argv() {
    let dm = DeviceManager::new();
    let argv = dm
        .emulator_command("Pixel_6_Pro_API_33")
        .expect("a non-empty AVD name is valid");

    assert_eq!(
        argv,
        vec![
            "emulator".to_string(),
            "-avd".to_string(),
            "Pixel_6_Pro_API_33".to_string()
        ],
        "the emulator is launched as `emulator -avd <name>`"
    );
}

#[test]
fn emulator_command_rejects_empty_and_whitespace_avd_names() {
    let dm = DeviceManager::new();

    assert_eq!(dm.emulator_command(""), Err(DeviceError::EmptyAvdName));
    assert_eq!(
        dm.emulator_command("   \t "),
        Err(DeviceError::EmptyAvdName),
        "a whitespace-only name would otherwise spawn `emulator -avd '   '`"
    );
}

#[test]
fn missing_emulator_binary_is_an_error_not_a_panic() {
    // The regression this pins: the original implementation called
    // .expect("Failed to start emulator"), so a laptop or CI runner without the
    // Android SDK took the whole CLI session down instead of degrading.
    let dm = DeviceManager::with_emulator_binary("cherenkov-emulator-that-does-not-exist");

    let result = dm.start_android_emulator("Pixel_6_Pro_API_33");

    match result {
        Err(DeviceError::SpawnFailed { binary, .. }) => {
            assert_eq!(binary, "cherenkov-emulator-that-does-not-exist");
        }
        Err(other) => panic!("expected SpawnFailed, got {other:?}"),
        Ok(mut child) => {
            // Should be unreachable, but never leave a stray process behind.
            let _ = child.kill();
            panic!("a nonexistent binary somehow spawned pid {}", child.id());
        }
    }
}

#[test]
fn spawn_failure_message_names_the_binary_and_points_at_path() {
    let dm = DeviceManager::with_emulator_binary("cherenkov-emulator-that-does-not-exist");
    let err = match dm.start_android_emulator("Pixel_6_Pro_API_33") {
        Err(e) => e,
        Ok(mut child) => {
            let _ = child.kill();
            panic!("binary should not exist");
        }
    };

    let rendered = err.to_string();
    assert!(
        rendered.contains("cherenkov-emulator-that-does-not-exist"),
        "error should name the binary it tried: {rendered}"
    );
    assert!(
        rendered.contains("PATH"),
        "error should tell the user how to fix it: {rendered}"
    );
}

#[test]
fn empty_avd_name_fails_before_any_process_is_spawned() {
    let dm = DeviceManager::new();
    // Validation must precede the spawn, otherwise a real `emulator` on PATH
    // would be launched with a garbage AVD on a machine that has the SDK.
    // Child is neither PartialEq nor Debug-comparable, so match on the error.
    match dm.start_android_emulator("") {
        Err(e) => assert_eq!(e, DeviceError::EmptyAvdName),
        Ok(mut child) => {
            let _ = child.kill();
            panic!("an empty AVD name reached the spawn");
        }
    }
}

// ---------------------------------------------------------------------------
// DeviceManager — browser nodes
// ---------------------------------------------------------------------------

#[test]
fn browser_node_command_targets_a_standalone_selenium_image() {
    let dm = DeviceManager::new();
    let argv = dm
        .browser_node_command("chromium")
        .expect("chromium is supported");

    assert_eq!(argv[0], "docker");
    assert_eq!(argv[1], "run");
    assert!(
        argv.contains(&"selenium/standalone-chromium:latest".to_string()),
        "expected a standalone selenium image, got {argv:?}"
    );
    assert!(
        argv.contains(&"--shm-size=2g".to_string()),
        "browser containers need an enlarged /dev/shm or Chrome crashes: {argv:?}"
    );
}

#[test]
fn browser_names_are_case_and_whitespace_insensitive() {
    let dm = DeviceManager::new();

    let padded = dm.browser_node_command("  ChRoMiUm  ").expect("normalized");
    let plain = dm.browser_node_command("chromium").expect("plain");

    assert_eq!(
        padded, plain,
        "casing and padding must not change the image"
    );
}

#[test]
fn every_supported_browser_produces_a_command() {
    let dm = DeviceManager::new();
    for browser in ["chromium", "chrome", "firefox"] {
        let argv = dm
            .browser_node_command(browser)
            .unwrap_or_else(|e| panic!("{browser} should be supported: {e}"));
        assert!(argv.last().expect("image").contains(browser));
    }
}

#[test]
fn unsupported_browser_is_rejected_rather_than_silently_started() {
    let dm = DeviceManager::new();

    let err = dm
        .browser_node_command("internet-explorer")
        .expect_err("IE is not a supported node");

    assert_eq!(
        err,
        DeviceError::UnsupportedBrowser {
            name: "internet-explorer".to_string()
        }
    );
    assert!(
        err.to_string().contains("firefox"),
        "the error should list what IS supported: {err}"
    );
}

#[test]
fn empty_browser_name_is_rejected() {
    let dm = DeviceManager::new();
    assert_eq!(
        dm.browser_node_command("   "),
        Err(DeviceError::EmptyBrowserName)
    );
}

#[test]
fn start_browser_node_returns_a_named_node_without_touching_docker() {
    let dm = DeviceManager::new();
    // This is a mock by design: the browser tracks run a local Playwright
    // install, so the CLI must work on a machine with no Docker daemon.
    let node = dm
        .start_browser_node("chromium")
        .expect("mock start succeeds");
    assert_eq!(node, "crucible-node-chromium");
}

#[test]
fn start_browser_node_propagates_validation_errors() {
    let dm = DeviceManager::new();
    assert_eq!(
        dm.start_browser_node("netscape"),
        Err(DeviceError::UnsupportedBrowser {
            name: "netscape".to_string()
        })
    );
}

// ---------------------------------------------------------------------------
// Pipeline provisioning simulator
// ---------------------------------------------------------------------------

#[test]
fn provisioning_simulates_terraform_before_docker() {
    let steps = provisioning_steps();
    let first_docker = steps
        .iter()
        .position(|s| s.tool == "docker")
        .expect("docker phase exists");
    let last_terraform = steps
        .iter()
        .rposition(|s| s.tool == "terraform")
        .expect("terraform phase exists");

    assert!(
        last_terraform < first_docker,
        "infrastructure must be applied before containers are brought up"
    );
}

#[test]
fn provisioning_covers_both_declared_tools() {
    let steps = provisioning_steps();
    for tool in PROVISIONING_TOOLS {
        assert!(
            steps.iter().any(|s| s.tool == tool),
            "no simulated step for '{tool}'"
        );
    }
}

#[test]
fn provisioning_reports_a_terraform_apply_and_a_started_container() {
    let steps = provisioning_steps();

    assert!(
        steps
            .iter()
            .any(|s| s.tool == "terraform" && s.message.contains("Apply complete!")),
        "terraform phase should end with an apply summary"
    );
    assert!(
        steps
            .iter()
            .any(|s| s.tool == "docker" && s.message.contains("Started")),
        "docker phase should report at least one started container"
    );
}

#[test]
fn provisioning_steps_are_deterministic() {
    // The pipeline renders these into a transcript that tests diff against;
    // a random or time-dependent step would make that transcript flaky.
    assert_eq!(provisioning_steps(), provisioning_steps());
}

#[test]
fn provisioning_steps_carry_no_empty_messages() {
    for step in provisioning_steps() {
        assert!(
            !step.message.trim().is_empty(),
            "step for '{}' has an empty message",
            step.tool
        );
        assert!(
            PROVISIONING_TOOLS.contains(&step.tool),
            "step declares undeclared tool '{}'",
            step.tool
        );
    }
}
