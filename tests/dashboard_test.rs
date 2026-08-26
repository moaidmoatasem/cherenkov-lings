use cherenkov_lings::config::{self, Config, EvaluationConfig, PlatformConfig, UiConfig};
use cherenkov_lings::gamification::{
    DrillRunContext, GamificationState, default_curriculum_tracks, discover_track_drills,
    extract_drill_id_from_path, get_next_recommended_drill, get_track_summaries, render_dashboard,
};
use std::process::Command;

fn sample_config() -> Config {
    Config {
        platform: PlatformConfig {
            name: "cherenkov-lings".to_string(),
            version: "1.0.0".to_string(),
            sandbox_port: 8080,
            chaos_proxy_port: 8086,
            telemetry: false,
        },
        evaluation: EvaluationConfig {
            pass_threshold: 85.0,
            flakiness_iterations: 5,
            flakiness_timeout_ms: 5000,
            chaos_latency_ms: 200,
            chaos_jitter_ms: 75,
        },
        ui: UiConfig {
            theme: "cherenkov-blue".to_string(),
            show_hints_on_failure: true,
            enable_audio_bell: false,
            language: "en".to_string(),
        },
        tracks: default_curriculum_tracks(),
    }
}

#[test]
fn test_dashboard_cli_subcommand_exit_code_zero() {
    let output = Command::new(env!("CARGO_BIN_EXE_cherenkov-lings"))
        .arg("dashboard")
        .output()
        .expect("Failed to execute cherenkov-lings dashboard binary");

    assert!(
        output.status.success(),
        "cherenkov-lings dashboard must exit with code 0"
    );

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("CHERENKOV-LINGS"),
        "Dashboard must contain header branding"
    );
    assert!(
        stdout.contains("PROGRESS DASHBOARD"),
        "Dashboard must contain dashboard title"
    );
    assert!(
        stdout.contains("PLAYER RANK & PROGRESSION"),
        "Dashboard must contain player rank section"
    );
    assert!(
        stdout.contains("XP Progress"),
        "Dashboard must contain XP progress bar"
    );
    assert!(
        stdout.contains("ACTIVITY & ENGAGEMENT STATS"),
        "Dashboard must contain activity stats"
    );
    assert!(
        stdout.contains("Streak"),
        "Dashboard must contain streak info"
    );
    assert!(
        stdout.contains("CURRICULUM TRACK PROGRESSION"),
        "Dashboard must contain track progress table"
    );
    assert!(
        stdout.contains("RECENT ACHIEVEMENTS"),
        "Dashboard must contain recent achievements section"
    );
    assert!(
        stdout.contains("NEXT RECOMMENDED DRILL"),
        "Dashboard must contain next recommended drill section"
    );
}

#[test]
fn test_render_dashboard_default_initial_state() {
    let state = GamificationState::default();
    let config = sample_config();

    let output = render_dashboard(&state, &config);

    // Header & Rank
    assert!(output.contains("Trainee (Level 1)"));
    assert!(output.contains("0 XP total"));

    // Activity stats
    assert!(output.contains("0 days"));
    assert!(output.contains("0 /"));
    assert!(output.contains("0 / 8 achievements"));

    // Table contains all 9 tracks with ⏳ status
    assert!(output.contains("Automation Foundations"));
    assert!(output.contains("Modern Web Automation"));
    assert!(output.contains("API Resilience & Security"));
    assert!(output.contains("Mobile UI Automation"));
    assert!(output.contains("High-Concurrency Load Testing"));
    assert!(output.contains("GenAI QA Testing"));
    assert!(output.contains("Cloud-Native & DevSecOps"));
    assert!(output.contains("Enterprise Performance Testing"));
    assert!(output.contains("Cross-Tool Decision Framework"));
    assert!(output.contains("⏳"));

    // Achievements placeholder
    assert!(output.contains("No achievements unlocked yet"));

    // Next recommended drill
    assert!(output.contains("NEXT RECOMMENDED DRILL"));
    assert!(output.contains("01_what_is_a_test") || output.contains("Automation Foundations"));
}

#[test]
fn test_render_dashboard_partially_completed_state() {
    let mut state = GamificationState::default();
    let config = sample_config();

    // Complete Foundations Track (5 drills)
    for i in 1..=5 {
        let drill_id = format!("0{}_test", i);
        let ctx = DrillRunContext {
            track_id: "foundations".to_string(),
            drill_id: drill_id.clone(),
            file_path: format!("exercises/00_foundations/{}/exercise.py", drill_id),
            passed: true,
            total_score: 100.0,
            flakiness_score: 100.0,
            tier: 1,
            timestamp: Some("2026-08-20T12:00:00Z".to_string()),
            ..Default::default()
        };
        state.record_drill_run(&ctx);
    }

    // Complete 3 drills in Playwright
    for i in 1..=3 {
        let drill_id = format!("0{}_playwright", i);
        let ctx = DrillRunContext {
            track_id: "playwright-ts".to_string(),
            drill_id: drill_id.clone(),
            file_path: format!("exercises/01_web_playwright_ts/{}/exercise.ts", drill_id),
            passed: true,
            total_score: 92.0,
            tier: 1,
            timestamp: Some("2026-08-21T12:00:00Z".to_string()),
            ..Default::default()
        };
        state.record_drill_run(&ctx);
    }

    // Advance streak to day 3
    state.update_streak("2026-08-22T12:00:00Z");

    let output = render_dashboard(&state, &config);

    // Rank & XP
    assert!(state.total_xp >= 700);
    assert!(output.contains("Junior QA (Level 2)"));

    // Streak
    assert!(output.contains("3 days"));

    // Status emojis in table: ✅ for foundations, 🟡 for playwright, ⏳ for others
    assert!(output.contains("✅"));
    assert!(output.contains("🟡"));
    assert!(output.contains("⏳"));

    // Top 3 achievements display
    assert!(output.contains("First Blood"));
    assert!(output.contains("Flakiness Slayer"));

    // Next recommended drill is the 4th drill in Playwright or remaining incomplete
    assert!(output.contains("NEXT RECOMMENDED DRILL"));
    assert!(
        output.contains("playwright-ts")
            || output.contains("Modern Web Automation")
            || output.contains("04_")
    );
}

#[test]
fn test_render_dashboard_sdet_master_state() {
    let mut state = GamificationState::default();
    let config = sample_config();

    // Complete all tracks with high scores
    for track in &config.tracks {
        let drills = discover_track_drills(&track.id, &track.exercise_dir);
        for drill in drills {
            let ctx = DrillRunContext {
                track_id: track.id.clone(),
                drill_id: drill.clone(),
                file_path: format!("{}/{}/exercise", track.exercise_dir, drill),
                passed: true,
                total_score: 100.0,
                flakiness_score: 100.0,
                locator_score: 100.0,
                tier: 3,
                timestamp: Some("2026-08-24T12:00:00Z".to_string()),
                ..Default::default()
            };
            state.record_drill_run(&ctx);
        }
    }

    // Award bonus XP to exceed SDET Master threshold (20000 XP)
    state.total_xp = 25000;
    state.level_name = "SDET Master".to_string();

    let output = render_dashboard(&state, &config);

    assert!(output.contains("SDET Master (Level 7)"));
    assert!(output.contains("MAX LEVEL REACHED"));
    assert!(output.contains("✅"));

    let summaries = get_track_summaries(&config, &state);
    assert!(summaries.iter().all(|s| s.status_emoji == "✅"));

    let rec = get_next_recommended_drill(&config, &state);
    assert!(
        rec.reason.contains("All drills mastered")
            || rec.reason.contains("SDET Master")
            || rec.reason.contains("100.0%")
    );
}

#[test]
fn test_track_drill_discovery_all_curriculum_tracks() {
    let config = sample_config();
    let summaries = get_track_summaries(&config, &GamificationState::default());

    assert_eq!(
        summaries.len(),
        config.tracks.len(),
        "Should produce summary for all curriculum tracks"
    );

    for sum in summaries {
        assert!(
            sum.drills_total > 0,
            "Track '{}' must have discovered drills > 0",
            sum.track_id
        );
        assert_eq!(sum.drills_completed, 0);
        assert_eq!(sum.status_emoji, "⏳");
        assert!(sum.next_incomplete_drill.is_some());
    }
}

#[test]
fn test_next_drill_recommendation_prioritization() {
    let mut state = GamificationState::default();
    let config = sample_config();

    // Default: recommends first drill in Foundations
    let rec1 = get_next_recommended_drill(&config, &state);
    assert_eq!(rec1.track_id, "foundations");
    assert_eq!(rec1.drill_id, "01_what_is_a_test");

    // Complete first drill in Foundations
    let ctx1 = DrillRunContext {
        track_id: "foundations".to_string(),
        drill_id: "01_what_is_a_test".to_string(),
        file_path: "exercises/00_foundations/01_what_is_a_test/exercise.py".to_string(),
        passed: true,
        total_score: 95.0,
        tier: 1,
        ..Default::default()
    };
    state.record_drill_run(&ctx1);

    // Now recommends second drill in Foundations
    let rec2 = get_next_recommended_drill(&config, &state);
    assert_eq!(rec2.track_id, "foundations");
    assert_eq!(rec2.drill_id, "02_test_naming_matters");
}

#[test]
fn test_extract_drill_id_from_path_robustness() {
    assert_eq!(
        extract_drill_id_from_path(
            "exercises\\01_web_playwright_ts\\01_hydration_timing\\exercise.ts"
        ),
        "01_hydration_timing"
    );
    assert_eq!(
        extract_drill_id_from_path(
            "exercises/02_api_restassured_java/src/test/java/com/cherenkov/drill06_graphql_assertions/Solution.java"
        ),
        "drill06_graphql_assertions"
    );
    assert_eq!(
        extract_drill_id_from_path("exercises/05_perf_jmeter/08_jtl_dashboard/theory.md"),
        "08_jtl_dashboard"
    );
    assert_eq!(
        extract_drill_id_from_path(
            "exercises/03_mobile_maestro/04_scroll_to_element/exercise.yaml"
        ),
        "04_scroll_to_element"
    );
    assert_eq!(
        extract_drill_id_from_path("exercises/08_tool_decisions/03_appium_vs_maestro/exercise.py"),
        "03_appium_vs_maestro"
    );
}

#[test]
fn test_dashboard_with_lings_toml_file_integration() {
    let cfg = config::load_config("lings.toml").expect("lings.toml must load cleanly");
    let state = GamificationState::default();

    let output = render_dashboard(&state, &cfg);
    assert!(output.contains("CHERENKOV-LINGS"));
    assert!(output.contains("CURRICULUM TRACK PROGRESSION"));
    assert!(output.contains("Status Legend"));
}
