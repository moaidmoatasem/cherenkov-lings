use cherenkov_lings::feedback::{AstReport, analyze_file, evaluate_feedback};
use cherenkov_lings::runner::{DrillResponse, RunResult};
use std::fs;
use std::path::Path;

#[test]
fn test_genai_qa_drill_files_exist_and_hints_contracts() {
    let drill_dirs = [
        "exercises/05_genai_qa/01_rag_context_faithfulness",
        "exercises/05_genai_qa/02_llm_assertion_flakiness",
        "exercises/05_genai_qa/03_llm_hallucination_eval",
        "exercises/05_genai_qa/04_prompt_injection_red_teaming",
        "exercises/05_genai_qa/05_latency_streaming_ttft",
    ];

    for dir in drill_dirs {
        let p = Path::new(dir);
        assert!(p.exists(), "Drill directory {} must exist", dir);

        let exercise = p.join("exercise.ts");
        let solution = p.join("solution.ts");
        let hints = p.join("hints.md");

        assert!(exercise.exists(), "exercise.ts must exist in {}", dir);
        assert!(solution.exists(), "solution.ts must exist in {}", dir);
        assert!(hints.exists(), "hints.md must exist in {}", dir);

        let ex_content = fs::read_to_string(&exercise).expect("read exercise.ts");
        let sol_content = fs::read_to_string(&solution).expect("read solution.ts");
        let hint_content = fs::read_to_string(&hints).expect("read hints.md");

        assert!(
            !ex_content.trim().is_empty(),
            "exercise.ts in {} cannot be empty",
            dir
        );
        assert!(
            !sol_content.trim().is_empty(),
            "solution.ts in {} cannot be empty",
            dir
        );
        assert!(
            !hint_content.trim().is_empty(),
            "hints.md in {} cannot be empty",
            dir
        );

        assert!(
            hint_content.contains("Hint 1"),
            "hints.md in {} must contain Hint 1",
            dir
        );
        assert!(
            hint_content.contains("Hint 2"),
            "hints.md in {} must contain Hint 2",
            dir
        );
        assert!(
            hint_content.contains("Hint 3"),
            "hints.md in {} must contain Hint 3",
            dir
        );
    }
}

#[test]
fn test_genai_qa_drill01_rag_faithfulness_contract() {
    let ex_path = "exercises/05_genai_qa/01_rag_context_faithfulness/exercise.ts";
    let sol_path = "exercises/05_genai_qa/01_rag_context_faithfulness/solution.ts";

    let ex_content = fs::read_to_string(ex_path).expect("read drill 01 exercise.ts");
    let sol_content = fs::read_to_string(sol_path).expect("read drill 01 solution.ts");

    assert!(
        ex_content.contains("body.answer") && ex_content.contains(".toBe("),
        "Drill 01 exercise.ts must contain exact string equality assertion anti-pattern"
    );

    assert!(
        sol_content.contains("body.grounded"),
        "Drill 01 solution.ts must assert body.grounded"
    );
    assert!(
        sol_content.contains("body.source_facts"),
        "Drill 01 solution.ts must assert body.source_facts"
    );
    assert!(
        sol_content.contains("body.document_title"),
        "Drill 01 solution.ts must assert document_title"
    );
    assert!(
        sol_content.contains(".toContain") || sol_content.contains("toContain"),
        "Drill 01 solution.ts must use semantic/fact containment checks"
    );
}

#[test]
fn test_genai_qa_drill02_llm_flakiness_contract() {
    let ex_path = "exercises/05_genai_qa/02_llm_assertion_flakiness/exercise.ts";
    let sol_path = "exercises/05_genai_qa/02_llm_assertion_flakiness/solution.ts";

    let ex_content = fs::read_to_string(ex_path).expect("read drill 02 exercise.ts");
    let sol_content = fs::read_to_string(sol_path).expect("read drill 02 solution.ts");

    assert!(
        ex_content.contains("body.raw_text") && ex_content.contains(".toBe("),
        "Drill 02 exercise.ts must contain raw_text exact equality assertion anti-pattern"
    );

    assert!(
        sol_content.contains("body.intent"),
        "Drill 02 solution.ts must assert body.intent"
    );
    assert!(
        sol_content.contains("body.entities.action") || sol_content.contains("body.entities"),
        "Drill 02 solution.ts must assert structured entities"
    );
    assert!(
        sol_content.contains("body.confidence"),
        "Drill 02 solution.ts must assert body.confidence"
    );
}

#[test]
fn test_genai_qa_drill03_hallucination_eval_contract() {
    let ex_path = "exercises/05_genai_qa/03_llm_hallucination_eval/exercise.ts";
    let sol_path = "exercises/05_genai_qa/03_llm_hallucination_eval/solution.ts";

    let ex_content = fs::read_to_string(ex_path).expect("read drill 03 exercise.ts");
    let sol_content = fs::read_to_string(sol_path).expect("read drill 03 solution.ts");

    assert!(
        ex_content.contains("answer.length") && ex_content.contains(".toBeGreaterThan("),
        "Drill 03 exercise.ts must contain naive length check anti-pattern"
    );

    assert!(
        sol_content.contains("grounded"),
        "Drill 03 solution.ts must assert grounded property"
    );
    assert!(
        sol_content.contains("document_title") || sol_content.contains("source_facts"),
        "Drill 03 solution.ts must assert citation metadata"
    );
}

#[test]
fn test_genai_qa_drill04_prompt_injection_contract() {
    let ex_path = "exercises/05_genai_qa/04_prompt_injection_red_teaming/exercise.ts";
    let sol_path = "exercises/05_genai_qa/04_prompt_injection_red_teaming/solution.ts";

    let ex_content = fs::read_to_string(ex_path).expect("read drill 04 exercise.ts");
    let sol_content = fs::read_to_string(sol_path).expect("read drill 04 solution.ts");

    assert!(
        ex_content.contains("res.status()") && ex_content.contains(".toBeLessThan("),
        "Drill 04 exercise.ts must have naive status check anti-pattern"
    );

    assert!(
        sol_content.contains("PROMPT_INJECTION_DETECTED") || sol_content.contains("blocked"),
        "Drill 04 solution.ts must check for injection detection"
    );
    assert!(
        sol_content.contains("400"),
        "Drill 04 solution.ts must assert 400 status"
    );
}

#[test]
fn test_genai_qa_drill05_ttft_streaming_contract() {
    let ex_path = "exercises/05_genai_qa/05_latency_streaming_ttft/exercise.ts";
    let sol_path = "exercises/05_genai_qa/05_latency_streaming_ttft/solution.ts";

    let ex_content = fs::read_to_string(ex_path).expect("read drill 05 exercise.ts");
    let sol_content = fs::read_to_string(sol_path).expect("read drill 05 solution.ts");

    assert!(
        ex_content.contains("body.length") && ex_content.contains(".toBeGreaterThan("),
        "Drill 05 exercise.ts must have naive length check anti-pattern"
    );

    assert!(
        sol_content.contains("ttft") || sol_content.contains("start") && sol_content.contains("Date.now()"),
        "Drill 05 solution.ts must measure TTFT"
    );
    assert!(
        sol_content.contains("text/event-stream") || sol_content.contains("stream"),
        "Drill 05 solution.ts must verify streaming format"
    );
}

#[test]
fn test_playwright_config_discovers_genai_drills() {
    let config_path = "playwright.config.ts";
    assert!(
        Path::new(config_path).exists(),
        "playwright.config.ts must exist"
    );

    let content = fs::read_to_string(config_path).expect("read playwright.config.ts");
    assert!(
        content.contains("testDir: './exercises'")
            || content.contains("testDir: \"./exercises\"")
            || content.contains("exercises"),
        "playwright.config.ts must configure exercises directory"
    );
    assert!(
        content.contains(".ts"),
        "playwright.config.ts must match TypeScript test files"
    );

    let d1_ex = Path::new("exercises/05_genai_qa/01_rag_context_faithfulness/exercise.ts");
    let d2_ex = Path::new("exercises/05_genai_qa/02_llm_assertion_flakiness/exercise.ts");
    assert!(d1_ex.exists() && d1_ex.extension().unwrap() == "ts");
    assert!(d2_ex.exists() && d2_ex.extension().unwrap() == "ts");
}

#[test]
fn test_genai_qa_ast_analysis_clean_locators() {
    let d1_sol = "exercises/05_genai_qa/01_rag_context_faithfulness/solution.ts";
    let d2_sol = "exercises/05_genai_qa/02_llm_assertion_flakiness/solution.ts";

    let rep1 = analyze_file(d1_sol).expect("analyze drill 01 solution");
    assert!(!rep1.has_wait_for_timeout);
    assert_eq!(rep1.anti_patterns.len(), 0);

    let rep2 = analyze_file(d2_sol).expect("analyze drill 02 solution");
    assert!(!rep2.has_wait_for_timeout);
    assert_eq!(rep2.anti_patterns.len(), 0);
}

#[test]
fn test_genai_qa_feedback_matrix_scorecard_evaluation() {
    let ast = AstReport {
        file_path: "exercises/05_genai_qa/01_rag_context_faithfulness/solution.ts".to_string(),
        locator_quality_score: 100.0,
        ..Default::default()
    };

    let passed_response = DrillResponse {
        id: "genai-eval-pass".to_string(),
        ok: true,
        passed: true,
        iterations: 5,
        passed_iterations: 5,
        failed_iterations: 0,
        total_duration_ms: 250,
        runs: vec![
            RunResult {
                iteration: 1,
                passed: true,
                duration_ms: 50,
                error: None,
            },
            RunResult {
                iteration: 2,
                passed: true,
                duration_ms: 50,
                error: None,
            },
            RunResult {
                iteration: 3,
                passed: true,
                duration_ms: 50,
                error: None,
            },
            RunResult {
                iteration: 4,
                passed: true,
                duration_ms: 50,
                error: None,
            },
            RunResult {
                iteration: 5,
                passed: true,
                duration_ms: 50,
                error: None,
            },
        ],
        error: None,
    };

    let scorecard = evaluate_feedback(
        &passed_response,
        &ast,
        "GenAI QA Testing (Playwright TypeScript)",
        "1.0.0",
        85.0,
        1000,
    );

    assert!(scorecard.passed);
    assert_eq!(scorecard.correctness.score, 100.0);
    assert_eq!(scorecard.flakiness.score, 100.0);
    assert_eq!(scorecard.speed.score, 100.0);
    assert_eq!(scorecard.total_score, 100.0);

    let flaky_response = DrillResponse {
        id: "genai-eval-flaky".to_string(),
        ok: true,
        passed: false,
        iterations: 5,
        passed_iterations: 3,
        failed_iterations: 2,
        total_duration_ms: 250,
        runs: vec![
            RunResult {
                iteration: 1,
                passed: true,
                duration_ms: 50,
                error: None,
            },
            RunResult {
                iteration: 2,
                passed: false,
                duration_ms: 50,
                error: Some("raw_text mismatch".to_string()),
            },
            RunResult {
                iteration: 3,
                passed: true,
                duration_ms: 50,
                error: None,
            },
            RunResult {
                iteration: 4,
                passed: false,
                duration_ms: 50,
                error: Some("raw_text mismatch".to_string()),
            },
            RunResult {
                iteration: 5,
                passed: true,
                duration_ms: 50,
                error: None,
            },
        ],
        error: Some("raw_text mismatch".to_string()),
    };

    let scorecard_flaky = evaluate_feedback(
        &flaky_response,
        &ast,
        "GenAI QA Testing (Playwright TypeScript)",
        "1.0.0",
        85.0,
        1000,
    );

    assert!(!scorecard_flaky.passed);
    assert_eq!(scorecard_flaky.correctness.score, 60.0);
    assert_eq!(scorecard_flaky.flakiness.score, 60.0);
    assert!(scorecard_flaky.total_score < 85.0);
}

#[test]
fn test_polyglot_5_tracks_manifest_and_config() {
    let toml_content = fs::read_to_string("lings.toml").expect("read lings.toml");

    let expected_tracks = [
        ("playwright-ts", "exercises/01_web_playwright_ts", ".ts"),
        (
            "restassured-java",
            "exercises/02_api_restassured_java",
            ".java",
        ),
        ("k6-js", "exercises/04_perf_k6_js", ".js"),
        ("maestro-mobile", "exercises/03_mobile_maestro", ".yaml"),
        ("genai-qa", "exercises/05_genai_qa", ".ts"),
        ("devsecops-python", "exercises/06_cloud_devsecops", ".py"),
        ("foundations", "exercises/00_foundations", ".py"),
        ("jmeter", "exercises/05_perf_jmeter", ".jmx"),
        ("tool-decisions", "exercises/06_tool_decisions", ".py"),
        ("contract-pact", "exercises/07_contract_pact", ".py"),
        ("a11y-axe", "exercises/08_a11y_axe", ".ts"),
    ];

    for (track_id, dir, ext) in expected_tracks {
        assert!(
            toml_content.contains(&format!("id = \"{}\"", track_id)),
            "lings.toml must define track id '{}'",
            track_id
        );
        assert!(
            toml_content.contains(&format!("exercise_dir = \"{}\"", dir)),
            "lings.toml must define exercise_dir '{}'",
            dir
        );
        assert!(
            toml_content.contains(&format!("extension = \"{}\"", ext)),
            "lings.toml must define extension '{}'",
            ext
        );
    }
}

#[test]
fn test_polyglot_all_14_drills_exist_with_complete_artifacts() {
    let drill_manifest = [
        (
            "exercises/01_web_playwright_ts/01_hydration_timing",
            "exercise.ts",
            "solution.ts",
        ),
        (
            "exercises/01_web_playwright_ts/02_shadow_dom_v2",
            "exercise.ts",
            "solution.ts",
        ),
        (
            "exercises/01_web_playwright_ts/03_debounce_race_condition",
            "exercise.ts",
            "solution.ts",
        ),
        (
            "exercises/02_api_restassured_java/src/test/java/com/cherenkov/drill01_idempotency",
            "Exercise.java",
            "Solution.java",
        ),
        (
            "exercises/02_api_restassured_java/src/test/java/com/cherenkov/drill02_jwt_auth",
            "Exercise.java",
            "Solution.java",
        ),
        (
            "exercises/02_api_restassured_java/src/test/java/com/cherenkov/drill03_kafka_lag",
            "Exercise.java",
            "Solution.java",
        ),
        (
            "exercises/03_mobile_maestro/01_biometric_fallback",
            "exercise.yaml",
            "solution.yaml",
        ),
        (
            "exercises/03_mobile_maestro/02_deep_link_cold_start",
            "exercise.yaml",
            "solution.yaml",
        ),
        (
            "exercises/03_mobile_maestro/03_activity_recreation",
            "exercise.yaml",
            "solution.yaml",
        ),
        (
            "exercises/04_perf_k6_js/01_database_pool_starvation",
            "exercise.js",
            "solution.js",
        ),
        (
            "exercises/04_perf_k6_js/02_spike_profile_p99",
            "exercise.js",
            "solution.js",
        ),
        (
            "exercises/04_perf_k6_js/03_chaos_sla_assertion",
            "exercise.js",
            "solution.js",
        ),
        (
            "exercises/05_genai_qa/01_rag_context_faithfulness",
            "exercise.ts",
            "solution.ts",
        ),
        (
            "exercises/05_genai_qa/02_llm_assertion_flakiness",
            "exercise.ts",
            "solution.ts",
        ),
    ];

    assert_eq!(
        drill_manifest.len(),
        14,
        "Expected exactly 14 drills across all 5 tracks"
    );

    for (dir, ex_name, sol_name) in drill_manifest {
        let p = Path::new(dir);
        assert!(p.exists(), "Drill directory {} must exist", dir);

        let ex_path = p.join(ex_name);
        let sol_path = p.join(sol_name);
        let hint_path = p.join("hints.md");

        assert!(
            ex_path.exists(),
            "Exercise file '{}' must exist",
            ex_path.display()
        );
        assert!(
            sol_path.exists(),
            "Solution file '{}' must exist",
            sol_path.display()
        );
        assert!(
            hint_path.exists(),
            "Hints file '{}' must exist",
            hint_path.display()
        );

        let hints = fs::read_to_string(&hint_path).expect("read hints.md");
        assert!(
            hints.contains("Hint 1"),
            "Missing Hint 1 in {}",
            hint_path.display()
        );
        assert!(
            hints.contains("Hint 2"),
            "Missing Hint 2 in {}",
            hint_path.display()
        );
        assert!(
            hints.contains("Hint 3"),
            "Missing Hint 3 in {}",
            hint_path.display()
        );
    }
}
