//! Local device orchestration for the mobile and web automation tracks.
//!
//! Two very different things live here. `start_android_emulator` shells out to
//! a real `emulator` binary and is therefore opt-in; `start_browser_node` is a
//! deliberate mock that reports the container it *would* start, so the browser
//! tracks stay runnable on a machine with no Docker daemon.
//!
//! The command-building halves (`emulator_command`, `browser_node_command`) are
//! pure and separated from the spawning halves on purpose: they are what the
//! tests assert against, without launching anything.

use std::process::{Child, Command};
use thiserror::Error;

/// Browser images the mock node supports. Anything else is a caller mistake
/// worth surfacing rather than silently "starting".
const SUPPORTED_BROWSERS: [&str; 3] = ["chromium", "chrome", "firefox"];

#[derive(Debug, Error, PartialEq, Eq)]
pub enum DeviceError {
    #[error("AVD name must not be empty")]
    EmptyAvdName,

    #[error("browser name must not be empty")]
    EmptyBrowserName,

    #[error("unsupported browser '{name}' (supported: {})", SUPPORTED_BROWSERS.join(", "))]
    UnsupportedBrowser { name: String },

    #[error("failed to launch '{binary}': {source_message} — is the Android SDK emulator on PATH?")]
    SpawnFailed {
        binary: String,
        source_message: String,
    },
}

#[derive(Debug, Clone)]
pub struct DeviceManager {
    emulator_binary: String,
}

impl Default for DeviceManager {
    fn default() -> Self {
        Self::new()
    }
}

impl DeviceManager {
    pub fn new() -> Self {
        DeviceManager {
            emulator_binary: "emulator".to_string(),
        }
    }

    /// Point the manager at a specific emulator binary. Tests use this to aim
    /// at a binary that is guaranteed not to exist and assert on the error.
    pub fn with_emulator_binary(binary: impl Into<String>) -> Self {
        DeviceManager {
            emulator_binary: binary.into(),
        }
    }

    /// The argv the manager would execute for `avd_name`, without spawning.
    pub fn emulator_command(&self, avd_name: &str) -> Result<Vec<String>, DeviceError> {
        if avd_name.trim().is_empty() {
            return Err(DeviceError::EmptyAvdName);
        }
        Ok(vec![
            self.emulator_binary.clone(),
            "-avd".to_string(),
            avd_name.to_string(),
        ])
    }

    /// Launch an Android emulator and hand back the child process.
    ///
    /// The handle is returned rather than dropped: a dropped `Child` leaves the
    /// emulator running with nobody to reap it, and the caller is the only one
    /// that knows when the device is finished with.
    ///
    /// A missing `emulator` binary is the common case on a CI runner or a
    /// laptop without the Android SDK, so it is reported as a typed error the
    /// caller can degrade on — never a panic that takes the CLI session down.
    pub fn start_android_emulator(&self, avd_name: &str) -> Result<Child, DeviceError> {
        let argv = self.emulator_command(avd_name)?;
        println!("Starting Android emulator: {}", avd_name);

        Command::new(&argv[0])
            .args(&argv[1..])
            .spawn()
            .map_err(|e| DeviceError::SpawnFailed {
                binary: self.emulator_binary.clone(),
                source_message: e.to_string(),
            })
    }

    /// The `docker run` argv a real implementation would execute.
    pub fn browser_node_command(&self, browser: &str) -> Result<Vec<String>, DeviceError> {
        let normalized = browser.trim().to_lowercase();
        if normalized.is_empty() {
            return Err(DeviceError::EmptyBrowserName);
        }
        if !SUPPORTED_BROWSERS.contains(&normalized.as_str()) {
            return Err(DeviceError::UnsupportedBrowser {
                name: browser.to_string(),
            });
        }
        Ok(vec![
            "docker".to_string(),
            "run".to_string(),
            "-d".to_string(),
            "--shm-size=2g".to_string(),
            "-p".to_string(),
            "4444:4444".to_string(),
            format!("selenium/standalone-{}:latest", normalized),
        ])
    }

    /// Mock-start a standalone browser container, returning its node name.
    ///
    /// Nothing is actually spawned: the browser tracks drive a local Playwright
    /// install, and pretending otherwise would make the CLI fail on any machine
    /// without a Docker daemon.
    pub fn start_browser_node(&self, browser: &str) -> Result<String, DeviceError> {
        let argv = self.browser_node_command(browser)?;
        let node = format!("crucible-node-{}", browser.trim().to_lowercase());
        println!(
            "Mocking start of standalone browser container for: {} ({})",
            browser,
            argv.join(" ")
        );
        Ok(node)
    }
}
