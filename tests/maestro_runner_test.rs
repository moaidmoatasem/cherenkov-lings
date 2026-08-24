use cherenkov_lings::feedback::{self, AntiPatternKind};
use cherenkov_lings::runner::{AnyRunner, DrillResponse, MaestroRunner, RunResult};
use std::path::Path;
use std::sync::Arc;

#[test]
fn test_maestro_runner_initialization_and_options() {
    let runner = MaestroRunner::new();
    assert_eq!(runner.maestro_cmd(), "maestro");

    let custom = MaestroRunner::with_maestro_cmd("custom-maestro");
    assert_eq!(custom.maestro_cmd(), "custom-maestro");
}

#[test]
fn test_any_runner_maestro_wrapping() {
    let runner = MaestroRunner::new();
    let any_runner = AnyRunner::Maestro(Arc::new(runner));

    match any_runner {
        AnyRunner::Maestro(r) => {
            assert_eq!(r.maestro_cmd(), "maestro");
        }
        _ => panic!("Expected AnyRunner::Maestro"),
    }
}

#[test]
fn test_maestro_runner_validate_flow_definition_syntax() {
    let valid_yaml = r#"
---
- launchApp:
    appId: com.cherenkov.bankapp
- tapOn:
    text: Login with Biometric
- runFlow:
    when:
      visible:
        text: Biometric unavailable
    file: pin_fallback_flow.yaml
- assertVisible:
    text: Welcome, SDET Engineer
"#;
    assert!(MaestroRunner::validate_flow_definition(valid_yaml).is_ok());

    let invalid_tabs = "-\tlaunchApp:\n\tappId: com.cherenkov.bankapp\n";
    let tab_err = MaestroRunner::validate_flow_definition(invalid_tabs);
    assert!(tab_err.is_err());
    assert!(tab_err.unwrap_err().contains("tabs are not allowed"));

    let empty_yaml = "   \n\n  ";
    assert!(MaestroRunner::validate_flow_definition(empty_yaml).is_err());

    let comments_only = "# Just comments\n# Line 2\n";
    assert!(MaestroRunner::validate_flow_definition(comments_only).is_err());
}

#[test]
fn test_maestro_drill01_biometric_fallback_ast_and_scorecard() {
    let ex_path = "exercises/03_mobile_maestro/01_biometric_fallback/exercise.yaml";
    let sol_path = "exercises/03_mobile_maestro/01_biometric_fallback/solution.yaml";

    if !Path::new(ex_path).exists() || !Path::new(sol_path).exists() {
        eprintln!("Skipping: Drill 01 files not found");
        return;
    }

    // Exercise AST analysis
    let ast_ex = feedback::analyze_file(ex_path).expect("Analyze exercise.yaml");
    assert!(
        ast_ex.has_wait_for_timeout,
        "Drill 01 Exercise must trigger anti-pattern flakiness flag"
    );
    assert_eq!(ast_ex.anti_patterns.len(), 1);
    assert!(matches!(
        ast_ex.anti_patterns[0].kind,
        AntiPatternKind::MissingWhenCondition { .. }
    ));

    // Solution AST analysis
    let ast_sol = feedback::analyze_file(sol_path).expect("Analyze solution.yaml");
    assert!(
        !ast_sol.has_wait_for_timeout,
        "Drill 01 Solution must not trigger anti-pattern flag"
    );
    assert_eq!(ast_sol.anti_patterns.len(), 0);

    let mock_response = DrillResponse {
        id: "maestro-mock-1".to_string(),
        ok: true,
        passed: true,
        iterations: 1,
        passed_iterations: 1,
        failed_iterations: 0,
        total_duration_ms: 50,
        runs: vec![RunResult {
            iteration: 1,
            passed: true,
            duration_ms: 50,
            error: None,
        }],
        error: None,
    };

    // Exercise evaluation must cap flakiness at 40.0 and fail
    let card_ex = feedback::evaluate_feedback(
        &mock_response,
        &ast_ex,
        "Mobile UI Automation (Maestro YAML)",
        "1.0.0",
        85.0,
        1000,
    );
    assert_eq!(card_ex.flakiness.score, 40.0);
    assert!(!card_ex.passed);
    assert!(card_ex
        .diagnostics
        .iter()
        .any(|d| d.contains("Missing 'when:' condition")));

    // Solution evaluation must score >= pass threshold (85.0) and pass
    let card_sol = feedback::evaluate_feedback(
        &mock_response,
        &ast_sol,
        "Mobile UI Automation (Maestro YAML)",
        "1.0.0",
        85.0,
        1000,
    );
    assert_eq!(card_sol.flakiness.score, 100.0);
    assert!(card_sol.total_score >= 85.0);
    assert!(card_sol.passed);
}

#[test]
fn test_maestro_drill02_deep_link_cold_start_ast_and_scorecard() {
    let ex_path = "exercises/03_mobile_maestro/02_deep_link_cold_start/exercise.yaml";
    let sol_path = "exercises/03_mobile_maestro/02_deep_link_cold_start/solution.yaml";

    if !Path::new(ex_path).exists() || !Path::new(sol_path).exists() {
        eprintln!("Skipping: Drill 02 files not found");
        return;
    }

    let ast_ex = feedback::analyze_file(ex_path).expect("Analyze exercise.yaml");
    assert!(ast_ex.has_wait_for_timeout);
    assert_eq!(ast_ex.anti_patterns.len(), 1);
    assert!(matches!(
        ast_ex.anti_patterns[0].kind,
        AntiPatternKind::MissingColdStartDeepLink { .. }
    ));

    let ast_sol = feedback::analyze_file(sol_path).expect("Analyze solution.yaml");
    assert!(!ast_sol.has_wait_for_timeout);
    assert_eq!(ast_sol.anti_patterns.len(), 0);

    let mock_response = DrillResponse {
        id: "maestro-mock-2".to_string(),
        ok: true,
        passed: true,
        iterations: 1,
        passed_iterations: 1,
        failed_iterations: 0,
        total_duration_ms: 50,
        runs: vec![RunResult {
            iteration: 1,
            passed: true,
            duration_ms: 50,
            error: None,
        }],
        error: None,
    };

    let card_ex = feedback::evaluate_feedback(
        &mock_response,
        &ast_ex,
        "Mobile UI Automation (Maestro YAML)",
        "1.0.0",
        85.0,
        1000,
    );
    assert_eq!(card_ex.flakiness.score, 40.0);
    assert!(!card_ex.passed);

    let card_sol = feedback::evaluate_feedback(
        &mock_response,
        &ast_sol,
        "Mobile UI Automation (Maestro YAML)",
        "1.0.0",
        85.0,
        1000,
    );
    assert_eq!(card_sol.flakiness.score, 100.0);
    assert!(card_sol.total_score >= 85.0);
    assert!(card_sol.passed);
}

#[test]
fn test_maestro_drill03_activity_recreation_ast_and_scorecard() {
    let ex_path = "exercises/03_mobile_maestro/03_activity_recreation/exercise.yaml";
    let sol_path = "exercises/03_mobile_maestro/03_activity_recreation/solution.yaml";

    if !Path::new(ex_path).exists() || !Path::new(sol_path).exists() {
        eprintln!("Skipping: Drill 03 files not found");
        return;
    }

    let ast_ex = feedback::analyze_file(ex_path).expect("Analyze exercise.yaml");
    assert!(ast_ex.has_wait_for_timeout);
    assert_eq!(ast_ex.anti_patterns.len(), 1);
    assert!(matches!(
        ast_ex.anti_patterns[0].kind,
        AntiPatternKind::MissingActivityRecreation { .. }
    ));

    let ast_sol = feedback::analyze_file(sol_path).expect("Analyze solution.yaml");
    assert!(!ast_sol.has_wait_for_timeout);
    assert_eq!(ast_sol.anti_patterns.len(), 0);

    let mock_response = DrillResponse {
        id: "maestro-mock-3".to_string(),
        ok: true,
        passed: true,
        iterations: 1,
        passed_iterations: 1,
        failed_iterations: 0,
        total_duration_ms: 50,
        runs: vec![RunResult {
            iteration: 1,
            passed: true,
            duration_ms: 50,
            error: None,
        }],
        error: None,
    };

    let card_ex = feedback::evaluate_feedback(
        &mock_response,
        &ast_ex,
        "Mobile UI Automation (Maestro YAML)",
        "1.0.0",
        85.0,
        1000,
    );
    assert_eq!(card_ex.flakiness.score, 40.0);
    assert!(!card_ex.passed);

    let card_sol = feedback::evaluate_feedback(
        &mock_response,
        &ast_sol,
        "Mobile UI Automation (Maestro YAML)",
        "1.0.0",
        85.0,
        1000,
    );
    assert_eq!(card_sol.flakiness.score, 100.0);
    assert!(card_sol.total_score >= 85.0);
    assert!(card_sol.passed);
}

#[test]
fn test_all_maestro_drill_files_exist_on_disk() {
    let drill_files = [
        "exercises/03_mobile_maestro/01_biometric_fallback/exercise.yaml",
        "exercises/03_mobile_maestro/01_biometric_fallback/solution.yaml",
        "exercises/03_mobile_maestro/01_biometric_fallback/hints.md",
        "exercises/03_mobile_maestro/02_deep_link_cold_start/exercise.yaml",
        "exercises/03_mobile_maestro/02_deep_link_cold_start/solution.yaml",
        "exercises/03_mobile_maestro/02_deep_link_cold_start/hints.md",
        "exercises/03_mobile_maestro/03_activity_recreation/exercise.yaml",
        "exercises/03_mobile_maestro/03_activity_recreation/solution.yaml",
        "exercises/03_mobile_maestro/03_activity_recreation/hints.md",
        "exercises/03_mobile_maestro/maestro_runner.sh",
    ];

    for file in drill_files {
        assert!(
            Path::new(file).exists(),
            "Expected Maestro track file to exist: {}",
            file
        );
    }
}
