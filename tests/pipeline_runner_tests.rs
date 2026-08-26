//! Tests for the in-process CI/CD pipeline runner and the drills it scores.
//!
//! The curriculum contract for this track is stricter than "the solution
//! passes": every `exercise.yml` must fail for its *own* stated reason, so a
//! learner sees one focused finding rather than a wall of unrelated policy
//! noise.

use cherenkov_lings::config;
use cherenkov_lings::pipeline::validate_workflow;
use cherenkov_lings::runner::{PIPELINE_PASS_SCORE, PipelineRunner};
use std::path::{Path, PathBuf};

const TRACK_ID: &str = "ci-pipeline";

/// The policy code each drill is designed to provoke, and nothing else.
const EXPECTED_FINDINGS: &[(&str, &str)] = &[
    ("01_leaked_secret_in_workflow", "HARDCODED_SECRET"),
    ("02_missing_matrix_strategy", "MISSING_MATRIX_STRATEGY"),
    ("03_lost_failure_artifacts", "MISSING_ARTIFACT_UPLOAD"),
    ("04_runaway_job_timeout", "EXCESSIVE_TIMEOUT"),
    (
        "05_redundant_concurrent_runs",
        "CONCURRENCY_CANCEL_DISABLED",
    ),
];

fn track() -> config::TrackConfig {
    config::load_config("lings.toml")
        .expect("lings.toml must parse")
        .tracks
        .into_iter()
        .find(|t| t.id == TRACK_ID)
        .expect("ci-pipeline track must exist in the manifest")
}

fn drill_dir(drill_id: &str) -> PathBuf {
    PathBuf::from(track().drill_path(drill_id))
}

fn all_finding_codes(yaml: &str) -> Vec<String> {
    let v = validate_workflow(yaml);
    v.errors
        .iter()
        .map(|e| e.code.clone())
        .chain(v.warnings.iter().map(|w| w.code.clone()))
        .collect()
}

#[tokio::test]
async fn every_reference_solution_scores_a_perfect_policy_result() {
    let runner = PipelineRunner::new();

    for (drill_id, _) in EXPECTED_FINDINGS {
        let solution = drill_dir(drill_id).join("solution.yml");
        let response = runner
            .run_drill(&solution.to_string_lossy(), "", 1, 30_000)
            .await
            .expect("runner must not error on a well-formed workflow");

        assert!(
            response.passed,
            "solution for {drill_id} did not pass: {:?}",
            response.error
        );
    }
}

#[tokio::test]
async fn every_starter_workflow_fails_out_of_the_box() {
    let runner = PipelineRunner::new();

    for (drill_id, _) in EXPECTED_FINDINGS {
        let exercise = drill_dir(drill_id).join("exercise.yml");
        let response = runner
            .run_drill(&exercise.to_string_lossy(), "", 1, 30_000)
            .await
            .expect("runner must parse the starter workflow");

        assert!(
            !response.passed,
            "starter workflow for {drill_id} already passes — the drill teaches nothing"
        );
        assert!(
            response.error.is_some(),
            "failing drill {drill_id} must explain why it failed"
        );
    }
}

#[test]
fn each_drill_provokes_exactly_its_own_finding() {
    for (drill_id, expected_code) in EXPECTED_FINDINGS {
        let yaml = std::fs::read_to_string(drill_dir(drill_id).join("exercise.yml"))
            .expect("exercise.yml must be readable");
        let codes = all_finding_codes(&yaml);

        assert!(
            codes.iter().any(|c| c == expected_code),
            "{drill_id} should raise {expected_code}, got {codes:?}"
        );
        assert_eq!(
            codes.len(),
            1,
            "{drill_id} must raise exactly one finding so the lesson stays focused, got {codes:?}"
        );
    }
}

#[test]
fn reference_solutions_raise_no_findings_at_all() {
    for (drill_id, _) in EXPECTED_FINDINGS {
        let yaml = std::fs::read_to_string(drill_dir(drill_id).join("solution.yml"))
            .expect("solution.yml must be readable");
        let validation = validate_workflow(&yaml);

        assert_eq!(
            validation.sdet_score, PIPELINE_PASS_SCORE,
            "solution for {drill_id} scored {}: {:?} / {:?}",
            validation.sdet_score, validation.errors, validation.warnings
        );
    }
}

#[tokio::test]
async fn scoring_is_deterministic_across_flakiness_iterations() {
    let runner = PipelineRunner::new();
    let solution = drill_dir("01_leaked_secret_in_workflow").join("solution.yml");

    let response = runner
        .run_drill(
            &solution.to_string_lossy(),
            "delay=200ms;jitter=75ms",
            5,
            30_000,
        )
        .await
        .expect("runner must succeed");

    assert_eq!(response.iterations, 5);
    assert_eq!(
        response.passed_iterations, 5,
        "static policy analysis must be deterministic under chaos, got {:?}",
        response.runs
    );
    assert_eq!(response.failed_iterations, 0);
}

#[tokio::test]
async fn a_missing_workflow_file_is_reported_not_silently_passed() {
    let runner = PipelineRunner::new();

    let response = runner
        .run_drill("exercises/09_ci_pipeline/does_not_exist.yml", "", 1, 30_000)
        .await
        .expect("a missing file is a drill failure, not a runner crash");

    assert!(!response.passed);
    assert!(
        response.error.is_some_and(|e| !e.is_empty()),
        "missing workflow must surface an explanation"
    );
}

#[tokio::test]
async fn malformed_yaml_fails_without_panicking() {
    let runner = PipelineRunner::new();
    let dir = std::env::temp_dir().join("cherenkov_pipeline_runner_tests");
    std::fs::create_dir_all(&dir).expect("temp dir");
    let path = dir.join("malformed.yml");
    std::fs::write(&path, "name: broken\n  jobs:\n    - this: [is\n").expect("write temp workflow");

    let response = runner
        .run_drill(&path.to_string_lossy(), "", 1, 30_000)
        .await
        .expect("malformed YAML is a drill failure, not a runner crash");

    assert!(!response.passed);

    let _ = std::fs::remove_file(&path);
}

#[test]
fn the_track_is_wired_to_the_in_process_pipeline_runner() {
    let t = track();
    assert_eq!(
        t.runner, "pipeline",
        "the ci-pipeline track must use the in-process simulator"
    );
    assert_eq!(t.extension, ".yml");
    assert!(
        Path::new(&t.exercise_dir).is_dir(),
        "exercise_dir {} must exist",
        t.exercise_dir
    );
    assert_eq!(
        t.drills.len(),
        EXPECTED_FINDINGS.len(),
        "this test file must cover every drill in the track"
    );
}
