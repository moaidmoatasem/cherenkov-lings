use cherenkov_lings::gamification::{
    ALL_ACHIEVEMENTS, ALL_TRACKS, DEFAULT_BASELINE_DURATION_MS, DrillRunContext, GamificationState,
    PROGRESS_FILE, TOOL_DECISIONS_DRILLS, UnlockedAchievement, calculate_xp,
    calculate_xp_with_multiplier, check_achievements, civil_from_days, current_utc_date_string,
    current_utc_iso_timestamp, days_from_civil, get_level, get_level_info, get_tier_multiplier,
    load_progress, parse_date_to_days, render_badge_reveal, render_gamification_scorecard,
    render_gamification_scorecard_with_tier, render_gamification_summary,
    render_level_progress_bar, save_progress, tier_for_track_or_drill,
};
use std::fs;
use std::path::PathBuf;

#[test]
fn test_constants_and_tier_mapping() {
    assert_eq!(PROGRESS_FILE, ".cherenkov-progress.json");
    assert_eq!(ALL_ACHIEVEMENTS.len(), 8);
    assert_eq!(ALL_TRACKS.len(), 11);
    assert_eq!(TOOL_DECISIONS_DRILLS.len(), 4);

    assert_eq!(tier_for_track_or_drill("foundations", "01_assert"), 1);
    assert_eq!(tier_for_track_or_drill("playwright-ts", "06_pom"), 2);
    assert_eq!(tier_for_track_or_drill("devsecops-python", "01_zap"), 3);
}

#[test]
fn test_direct_check_achievements_function() {
    let mut state = GamificationState::default();
    let ctx = DrillRunContext {
        track_id: "playwright-ts".to_string(),
        drill_id: "01_hydration".to_string(),
        passed: true,
        total_score: 100.0,
        flakiness_score: 100.0,
        locator_score: 100.0,
        locator_applies: true,
        tier: 1,
        ..Default::default()
    };
    state.record_drill_run(&ctx);

    // Call check_achievements directly
    let direct_unlocked = check_achievements(&mut state, &ctx);
    // Already unlocked during record_drill_run, so direct call returns empty
    assert!(direct_unlocked.is_empty());
}

fn get_unique_temp_file(prefix: &str) -> PathBuf {
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    std::env::temp_dir().join(format!("{}_{}_{}.json", prefix, std::process::id(), nanos))
}

#[test]
fn test_default_gamification_state() {
    let state = GamificationState::default();
    assert_eq!(state.total_xp, 0);
    assert_eq!(state.level_name, "Trainee");
    assert_eq!(state.streak_days, 0);
    assert_eq!(state.last_active_date, None);
    assert_eq!(state.flakiness_100_streak, 0);
    assert_eq!(state.perfect_locator_count, 0);
    assert!(state.achievements.is_empty());
    assert!(state.completed_drills.is_empty());
    assert_eq!(state.distinct_tracks_count(), 0);
}

#[test]
fn test_xp_tier_multipliers() {
    assert!((get_tier_multiplier(1) - 1.0).abs() < f64::EPSILON);
    assert!((get_tier_multiplier(2) - 1.5).abs() < f64::EPSILON);
    assert!((get_tier_multiplier(3) - 2.0).abs() < f64::EPSILON);
    assert!((get_tier_multiplier(99) - 1.0).abs() < f64::EPSILON);

    assert_eq!(calculate_xp(100.0, 1), 100);
    assert_eq!(calculate_xp(100.0, 2), 150);
    assert_eq!(calculate_xp(100.0, 3), 200);

    assert_eq!(calculate_xp(80.0, 1), 80);
    assert_eq!(calculate_xp(80.0, 2), 120);
    assert_eq!(calculate_xp(80.0, 3), 160);

    // Rounding verification
    // 93.3 * 1.5 = 139.95 -> 140
    assert_eq!(calculate_xp_with_multiplier(93.3, 1.5), 140);
    // 93.1 * 1.5 = 139.65 -> 140
    assert_eq!(calculate_xp_with_multiplier(93.1, 1.5), 140);
    // 50.0 * 1.5 = 75.0 -> 75
    assert_eq!(calculate_xp_with_multiplier(50.0, 1.5), 75);
}

#[test]
fn test_all_seven_levels_progression_and_thresholds() {
    let test_cases = [
        (0, "Trainee", 0, 500, 0.0),
        (250, "Trainee", 0, 500, 50.0),
        (499, "Trainee", 0, 500, 99.8),
        (500, "Junior QA", 500, 1500, 0.0),
        (1000, "Junior QA", 500, 1500, 50.0),
        (1499, "Junior QA", 500, 1500, 99.9),
        (1500, "Mid QA", 1500, 3000, 0.0),
        (2250, "Mid QA", 1500, 3000, 50.0),
        (3000, "Senior QA", 3000, 6000, 0.0),
        (4500, "Senior QA", 3000, 6000, 50.0),
        (6000, "Lead QA", 6000, 10000, 0.0),
        (8000, "Lead QA", 6000, 10000, 50.0),
        (10000, "QA Architect", 10000, 20000, 0.0),
        (15000, "QA Architect", 10000, 20000, 50.0),
        (20000, "SDET Master", 20000, 20000, 100.0),
        (99999, "SDET Master", 20000, 20000, 100.0),
    ];

    for (xp, expected_title, expected_min, expected_next, expected_pct) in test_cases {
        let (title, min_xp, next_xp) = get_level(xp);
        assert_eq!(title, expected_title, "Failed level title for xp={}", xp);
        assert_eq!(min_xp, expected_min, "Failed min xp for xp={}", xp);
        assert_eq!(next_xp, expected_next, "Failed next xp for xp={}", xp);

        let info = get_level_info(xp);
        assert_eq!(info.title, expected_title);
        assert_eq!(info.min_xp, expected_min);
        assert!(
            (info.progress_pct - expected_pct).abs() < 0.2,
            "Failed progress_pct for xp={}: expected {}, got {}",
            xp,
            expected_pct,
            info.progress_pct
        );
    }
}

#[test]
fn test_render_level_progress_bar_output() {
    let bar_0 = render_level_progress_bar(0, 10);
    assert!(bar_0.contains("0/500 XP"));
    assert!(bar_0.contains("Trainee"));
    assert!(bar_0.contains("(0.0%)"));

    let bar_mid = render_level_progress_bar(1840, 12);
    assert!(bar_mid.contains("1840/3000 XP"));
    assert!(bar_mid.contains("Mid QA"));

    let bar_max = render_level_progress_bar(25000, 10);
    assert!(bar_max.contains("25000 XP"));
    assert!(bar_max.contains("SDET Master"));
    assert!(bar_max.contains("MAX LEVEL"));
}

#[test]
fn test_civil_date_conversions_and_leap_years() {
    // 2000 was a leap year
    let days_2000 = days_from_civil(2000, 2, 29);
    assert_eq!(civil_from_days(days_2000), "2000-02-29");

    // 2024 was a leap year
    let days_2024 = days_from_civil(2024, 2, 29);
    assert_eq!(civil_from_days(days_2024), "2024-02-29");

    // Current date helpers
    let today = current_utc_date_string();
    assert_eq!(today.len(), 10);
    assert_eq!(today.chars().nth(4), Some('-'));
    assert_eq!(today.chars().nth(7), Some('-'));

    let iso_ts = current_utc_iso_timestamp();
    assert!(iso_ts.ends_with('Z'));
    assert!(iso_ts.contains('T'));

    // Date parsing
    assert_eq!(
        parse_date_to_days(&today),
        Some(days_from_civil(
            today[0..4].parse().unwrap(),
            today[5..7].parse().unwrap(),
            today[8..10].parse().unwrap(),
        ))
    );
    assert_eq!(parse_date_to_days(""), None);
    assert_eq!(parse_date_to_days("2026-99-99"), None);
}

#[test]
fn test_streak_calculation_scenarios() {
    let mut state = GamificationState::default();

    // Start streak
    state.update_streak("2026-08-01T10:00:00Z");
    assert_eq!(state.streak_days, 1);
    assert_eq!(state.last_active_date.as_deref(), Some("2026-08-01"));

    // Multiple drills on the same day -> streak remains 1
    state.update_streak("2026-08-01T14:30:00Z");
    state.update_streak("2026-08-01T23:59:59Z");
    assert_eq!(state.streak_days, 1);

    // Consecutive day 2
    state.update_streak("2026-08-02");
    assert_eq!(state.streak_days, 2);

    // Consecutive day 3
    state.update_streak("2026-08-03T08:00:00Z");
    assert_eq!(state.streak_days, 3);

    // Month boundary: 2026-08-31 to 2026-09-01
    state.last_active_date = Some("2026-08-31".to_string());
    state.streak_days = 10;
    state.update_streak("2026-09-01");
    assert_eq!(state.streak_days, 11);

    // Broken streak (skip 2 days)
    state.update_streak("2026-09-04");
    assert_eq!(state.streak_days, 1);
    assert_eq!(state.last_active_date.as_deref(), Some("2026-09-04"));
}

#[test]
fn test_achievement_all_8_badges_unlocking() {
    let mut state = GamificationState::default();

    // 1. first_blood
    let ctx1 = DrillRunContext {
        track_id: "foundations".to_string(),
        drill_id: "01_assert".to_string(),
        passed: true,
        total_score: 90.0,
        tier: 1,
        ..Default::default()
    };
    let (xp1, un1) = state.record_drill_run(&ctx1);
    assert_eq!(xp1, 90);
    assert!(un1.iter().any(|a| a.id == "first_blood"));
    assert!(state.has_achievement("first_blood"));

    // 2. flakiness_slayer
    let ctx_flaky = DrillRunContext {
        track_id: "playwright-ts".to_string(),
        drill_id: "01_hydration".to_string(),
        passed: true,
        total_score: 100.0,
        flakiness_score: 100.0,
        tier: 1,
        ..Default::default()
    };
    state.record_drill_run(&ctx_flaky);
    state.record_drill_run(&ctx_flaky);
    let (_, un_flaky) = state.record_drill_run(&ctx_flaky);
    assert!(un_flaky.iter().any(|a| a.id == "flakiness_slayer"));
    assert!(state.has_achievement("flakiness_slayer"));

    // 3. chaos_survivor
    let ctx_chaos = DrillRunContext {
        track_id: "k6-js".to_string(),
        drill_id: "03_chaos_sla".to_string(),
        passed: true,
        total_score: 95.0,
        passed_iterations: 5,
        iterations: 5,
        tier: 2,
        ..Default::default()
    };
    let (_, un_chaos) = state.record_drill_run(&ctx_chaos);
    assert!(un_chaos.iter().any(|a| a.id == "chaos_survivor"));
    assert!(state.has_achievement("chaos_survivor"));

    // 4. tool_polyglot (4 distinct tracks)
    let ctx_java = DrillRunContext {
        track_id: "restassured-java".to_string(),
        drill_id: "drill01_auth".to_string(),
        passed: true,
        total_score: 90.0,
        tier: 1,
        ..Default::default()
    };
    let (_, un_poly) = state.record_drill_run(&ctx_java);
    // Tracks so far: foundations, playwright-ts, k6-js, restassured-java -> 4 tracks!
    assert!(un_poly.iter().any(|a| a.id == "tool_polyglot"));
    assert!(state.has_achievement("tool_polyglot"));

    // 5. the_architect (Tool Decisions)
    for drill in TOOL_DECISIONS_DRILLS {
        let ctx_tool = DrillRunContext {
            track_id: "tool-decisions".to_string(),
            drill_id: drill.to_string(),
            passed: true,
            total_score: 100.0,
            tier: 2,
            ..Default::default()
        };
        state.record_drill_run(&ctx_tool);
    }
    assert!(state.has_achievement("the_architect"));

    // 6. perfect_locator (5 times 100 on locator quality)
    for i in 1..=5 {
        let ctx_loc = DrillRunContext {
            track_id: "playwright-ts".to_string(),
            drill_id: format!("loc_drill_{}", i),
            passed: true,
            total_score: 100.0,
            locator_score: 100.0,
            locator_applies: true,
            tier: 1,
            ..Default::default()
        };
        state.record_drill_run(&ctx_loc);
    }
    assert!(state.has_achievement("perfect_locator"));

    // 7. speed_demon (duration <= 600ms on 1000ms baseline)
    let ctx_speed = DrillRunContext {
        track_id: "foundations".to_string(),
        drill_id: "02_speed".to_string(),
        passed: true,
        total_score: 98.0,
        avg_duration_ms: 450,
        baseline_duration_ms: 1000,
        tier: 1,
        ..Default::default()
    };
    let (_, un_speed) = state.record_drill_run(&ctx_speed);
    assert!(un_speed.iter().any(|a| a.id == "speed_demon"));
    assert!(state.has_achievement("speed_demon"));

    // 8. sdet_master (all 9 tracks completed)
    for track in ALL_TRACKS {
        let ctx_track = DrillRunContext {
            track_id: track.to_string(),
            drill_id: "drill_master".to_string(),
            passed: true,
            total_score: 100.0,
            tier: 3,
            ..Default::default()
        };
        state.record_drill_run(&ctx_track);
    }
    assert!(state.has_achievement("sdet_master"));

    // Total 8 achievements unlocked
    assert_eq!(state.achievements.len(), 8);
    for &(id, _, _) in &ALL_ACHIEVEMENTS {
        assert!(state.has_achievement(id), "Missing badge: {}", id);
    }
}

#[test]
fn test_persistence_full_lifecycle_and_serialization() {
    let temp_file = get_unique_temp_file("test_persistence");

    // File missing -> returns default
    let initial = load_progress(Some(&temp_file)).unwrap();
    assert_eq!(initial, GamificationState::default());

    let mut state = GamificationState::default();
    let ctx = DrillRunContext {
        track_id: "playwright-ts".to_string(),
        drill_id: "01_hydration".to_string(),
        file_path: "exercises/01_web_playwright_ts/01_hydration_timing/exercise.ts".to_string(),
        passed: true,
        total_score: 92.5,
        correctness_score: 100.0,
        flakiness_score: 100.0,
        locator_score: 85.0,
        locator_applies: true,
        speed_score: 90.0,
        passed_iterations: 5,
        iterations: 5,
        avg_duration_ms: 650,
        baseline_duration_ms: DEFAULT_BASELINE_DURATION_MS,
        tier: 2,
        timestamp: Some("2026-08-24T09:30:00Z".to_string()),
    };

    let (xp, unlocked) = state.record_drill_run(&ctx);
    assert_eq!(xp, 139); // 100 * 0.925 * 1.5 = 138.75 -> 139
    assert_eq!(state.total_xp, 139);
    assert_eq!(state.level_name, "Trainee");
    assert_eq!(state.streak_days, 1);
    assert_eq!(unlocked.len(), 1);

    // Save to disk
    save_progress(&state, Some(&temp_file)).unwrap();
    assert!(temp_file.exists());

    // Re-load from disk
    let loaded = load_progress(Some(&temp_file)).unwrap();
    assert_eq!(loaded.total_xp, 139);
    assert_eq!(loaded.level_name, "Trainee");
    assert_eq!(loaded.streak_days, 1);
    assert_eq!(loaded.achievements.len(), 1);
    assert_eq!(loaded.completed_drills.len(), 1);

    let drill = loaded
        .completed_drills
        .get("playwright-ts/01_hydration")
        .unwrap();
    assert_eq!(drill.best_score, 92.5);
    assert_eq!(drill.completion_count, 1);
    assert_eq!(drill.first_completed_at, "2026-08-24T09:30:00Z");

    // Clean up
    let _ = fs::remove_file(&temp_file);
}

#[test]
fn test_drill_re_attempt_updates_best_score_and_count() {
    let mut state = GamificationState::default();

    // First attempt: score 70.0 (passed=false, fail threshold is 85.0)
    let ctx_fail = DrillRunContext {
        track_id: "playwright-ts".to_string(),
        drill_id: "01_hydration".to_string(),
        passed: false,
        total_score: 70.0,
        tier: 1,
        ..Default::default()
    };
    let (xp_fail, _) = state.record_drill_run(&ctx_fail);
    assert_eq!(xp_fail, 0);
    assert!(state.completed_drills.is_empty());

    // Second attempt: score 88.0 (passed)
    let ctx_pass1 = DrillRunContext {
        track_id: "playwright-ts".to_string(),
        drill_id: "01_hydration".to_string(),
        passed: true,
        total_score: 88.0,
        tier: 1,
        timestamp: Some("2026-08-24T10:00:00Z".to_string()),
        ..Default::default()
    };
    let (xp1, _) = state.record_drill_run(&ctx_pass1);
    assert_eq!(xp1, 88);
    assert_eq!(state.drills_count_for_track("playwright-ts"), 1);
    assert_eq!(state.best_score_for_track("playwright-ts"), Some(88.0));

    // Third attempt: score 98.0 (improvement)
    let ctx_pass2 = DrillRunContext {
        track_id: "playwright-ts".to_string(),
        drill_id: "01_hydration".to_string(),
        passed: true,
        total_score: 98.0,
        tier: 1,
        timestamp: Some("2026-08-24T10:30:00Z".to_string()),
        ..Default::default()
    };
    let (xp2, _) = state.record_drill_run(&ctx_pass2);
    assert_eq!(xp2, 98);
    assert_eq!(state.total_xp, 88 + 98);

    let drill_entry = state
        .completed_drills
        .get("playwright-ts/01_hydration")
        .unwrap();
    assert_eq!(drill_entry.best_score, 98.0);
    assert_eq!(drill_entry.completion_count, 2);
    assert_eq!(drill_entry.first_completed_at, "2026-08-24T10:00:00Z");
    assert_eq!(drill_entry.last_completed_at, "2026-08-24T10:30:00Z");
}

#[test]
fn test_render_scorecard_and_summaries_no_panic() {
    let mut state = GamificationState {
        total_xp: 4250,
        level_name: "Senior QA".to_string(),
        streak_days: 7,
        ..Default::default()
    };

    let ach = UnlockedAchievement {
        id: "speed_demon".to_string(),
        name: "Speed Demon".to_string(),
        description: "Beat the speed baseline by 40% on any drill".to_string(),
        unlocked_at: "2026-08-24T12:00:00Z".to_string(),
    };
    state.achievements.push(ach.clone());

    let scorecard = render_gamification_scorecard(150, &state, std::slice::from_ref(&ach));
    assert!(scorecard.contains("GAMIFICATION & PROGRESSION"));
    assert!(scorecard.contains("+150 XP earned!"));
    assert!(scorecard.contains("Senior QA"));
    assert!(scorecard.contains("Daily Streak: 7 days"));
    assert!(scorecard.contains("ACHIEVEMENT UNLOCKED: Speed Demon"));

    let scorecard_tier =
        render_gamification_scorecard_with_tier(150, 2, &state, std::slice::from_ref(&ach));
    assert!(scorecard_tier.contains("Tier 2 (1.5x)"));

    let reveal = render_badge_reveal(&ach);
    assert!(reveal.contains("╔════"));
    assert!(reveal.contains("Speed Demon"));
    assert!(reveal.contains("╚════"));

    let summary = render_gamification_summary(&state);
    assert!(summary.contains("Senior QA"));
    assert!(summary.contains("Total XP: 4250"));
    assert!(summary.contains("Streak: 7 days"));
    assert!(summary.contains("Badges Unlocked: 1 / 8"));
}
