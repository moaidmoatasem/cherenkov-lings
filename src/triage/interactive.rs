use crate::gamification::{render_level_progress_bar, GamificationState};
use crate::reports::chaos_dataset::{
    get_failing_tests, get_test_by_id, ChaosTestResult, FailureCategory,
};
use crate::triage::evaluator::{evaluate_and_record_progress, TriageResult, TriageSubmission};
use colored::Colorize;
use std::io::{self, BufRead, Write};
use std::path::Path;

/// Parse category string from CLI arguments or user input
pub fn parse_category_from_str(s: &str) -> Option<FailureCategory> {
    let clean = s.trim().to_lowercase().replace('-', "_").replace(' ', "_");
    match clean.as_str() {
        "1" | "real_bug" | "bug" | "realbug" | "product_bug" | "defect" => {
            Some(FailureCategory::RealBug)
        }
        "2" | "flaky_infra" | "flaky" | "infra" | "flakyinfra" | "network" | "proxy" => {
            Some(FailureCategory::FlakyInfra)
        }
        "3" | "anti_pattern" | "antipattern" | "anti" | "test_flaw" | "brittle" => {
            Some(FailureCategory::AntiPattern)
        }
        "none" | "pass" | "healthy" => Some(FailureCategory::None),
        _ => None,
    }
}

/// Run the interactive Root-Cause Triage terminal flow
pub fn run_interactive_triage(
    test_id_filter: Option<&str>,
    category_flag: Option<&str>,
    explanation_flag: Option<&str>,
    fix_flag: Option<&str>,
) -> std::io::Result<()> {
    let failures = get_failing_tests();

    println!();
    println!(
        "{}",
        "╔══════════════════════════════════════════════════════════════════════════════════╗"
            .bright_cyan()
    );
    println!(
        "{}",
        "║     🔬  CHERENKOV-LINGS ROOT-CAUSE TRIAGE HYPOTHESIS ENGINE  🔬                 ║"
            .bold()
            .bright_cyan()
    );
    println!(
        "{}",
        "║     Diagnose chaotic test failures, investigate telemetry, and earn SDET XP!     ║"
            .dimmed()
    );
    println!(
        "{}",
        "╚══════════════════════════════════════════════════════════════════════════════════╝"
            .bright_cyan()
    );
    println!();

    let target_test = match test_id_filter {
        Some(id) => match get_test_by_id(id) {
            Some(t) => t,
            None => {
                eprintln!(
                    "{} Test ID '{}' not found in chaotic test dataset.",
                    "✗".bold().red(),
                    id
                );
                display_failure_summary_table(&failures[..failures.len().min(10)]);
                return Ok(());
            }
        },
        None => {
            // Interactive mode: show table and prompt user to select
            display_failure_summary_table(&failures);
            println!();
            print!(
                "{} Enter Test ID to investigate (or press Enter for '{}'): ",
                "▶".bright_yellow(),
                failures.first().map(|f| f.test_id.as_str()).unwrap_or("BUG-101").cyan()
            );
            io::stdout().flush()?;
            let mut input = String::new();
            io::stdin().lock().read_line(&mut input)?;
            let chosen = input.trim();
            let chosen_id = if chosen.is_empty() {
                failures.first().map(|f| f.test_id.as_str()).unwrap_or("BUG-101")
            } else {
                chosen
            };

            match get_test_by_id(chosen_id) {
                Some(t) => t,
                None => {
                    eprintln!("{} Unknown test ID '{}'.", "✗".bold().red(), chosen_id);
                    return Ok(());
                }
            }
        }
    };

    // Render test investigation card
    display_test_investigation_card(&target_test);

    // Get learner category
    let category = match category_flag.and_then(parse_category_from_str) {
        Some(cat) => cat,
        None => {
            println!(
                "{}",
                "SELECT ROOT-CAUSE HYPOTHESIS CATEGORY:".bold().bright_white()
            );
            println!(
                "  [1] {} — Server 500, RBAC bypass, DB constraint / deadlock, integer overflow",
                "Genuine Product Defect (RealBug)".bright_red().bold()
            );
            println!(
                "  [2] {} — Proxy latency spikes, TCP resets, 502/504 gateway errors, DNS drops",
                "Flaky Infrastructure (FlakyInfra)".bright_blue().bold()
            );
            println!(
                "  [3] {} — Hardcoded sleep races, stale element references, brittle locators",
                "Test Automation Anti-Pattern (AntiPattern)".bright_magenta().bold()
            );
            println!();
            print!("{} Select [1, 2, 3]: ", "▶".bright_yellow());
            io::stdout().flush()?;

            let mut choice_str = String::new();
            io::stdin().lock().read_line(&mut choice_str)?;
            match parse_category_from_str(&choice_str) {
                Some(cat) => cat,
                None => {
                    eprintln!("{} Invalid category choice. Aborting triage.", "✗".red());
                    return Ok(());
                }
            }
        }
    };

    // Get explanation
    let explanation = match explanation_flag {
        Some(exp) => exp.to_string(),
        None => {
            println!();
            println!(
                "{} Provide your root-cause analysis (what specifically triggered this failure?):",
                "💡".yellow()
            );
            print!("{} Explanation: ", "▶".bright_yellow());
            io::stdout().flush()?;
            let mut exp_input = String::new();
            io::stdin().lock().read_line(&mut exp_input)?;
            exp_input.trim().to_string()
        }
    };

    // Get suggested fix
    let fix = match fix_flag {
        Some(f) => f.to_string(),
        None => {
            println!();
            println!(
                "{} Suggest an engineering remediation or test refactoring pattern:",
                "🔧".yellow()
            );
            print!("{} Suggested Fix: ", "▶".bright_yellow());
            io::stdout().flush()?;
            let mut fix_input = String::new();
            io::stdin().lock().read_line(&mut fix_input)?;
            fix_input.trim().to_string()
        }
    };

    let submission = TriageSubmission {
        test_id: target_test.test_id.clone(),
        learner_category: category,
        root_cause_explanation: explanation,
        suggested_fix: fix,
    };

    println!();
    println!("{} Evaluating hypothesis against telemetry ground truth...", "⏳".yellow());
    let (result, state) = evaluate_and_record_progress(&submission, None::<&Path>);

    display_triage_scorecard(&result, &submission, &state);

    Ok(())
}

/// Display a formatted table of chaotic test failures available for triage
pub fn display_failure_summary_table(failures: &[ChaosTestResult]) {
    println!(
        " {:<10} │ {:<40} │ {:<18} │ {:<8}",
        "Test ID".bold().bright_white(),
        "Test Name".bold().bright_white(),
        "Track".bold().bright_white(),
        "Status".bold().bright_white()
    );
    println!(
        " ───────────┼──────────────────────────────────────────┼────────────────────┼─────────"
    );

    for t in failures {
        let status_colored = match t.status {
            crate::reports::chaos_dataset::TestStatus::Failed => "FAILED".bright_red().bold(),
            crate::reports::chaos_dataset::TestStatus::Broken => "BROKEN".bright_yellow().bold(),
            crate::reports::chaos_dataset::TestStatus::Flaky => "FLAKY".bright_magenta().bold(),
            _ => "PASSED".green(),
        };

        let short_name = if t.name.len() > 38 {
            format!("{}...", &t.name[..35])
        } else {
            t.name.clone()
        };

        println!(
            " {:<10} │ {:<40} │ {:<18} │ {}",
            t.test_id.bright_cyan().bold(),
            short_name,
            t.track_id.dimmed(),
            status_colored
        );
    }
}

/// Display full investigation card with telemetry and stack traces in terminal
pub fn display_test_investigation_card(test: &ChaosTestResult) {
    println!();
    println!(
        "{}",
        "──────────────────────────────────────────────────────────────────────────────────"
            .cyan()
    );
    println!(
        "  {} [{}] {}",
        "INVESTIGATION TARGET:".bold().bright_white(),
        test.test_id.bright_cyan().bold(),
        test.name.bright_yellow().bold()
    );
    println!(
        "  Track: {}  |  Suite: {}  |  Duration: {}ms",
        test.track_id.cyan(),
        test.suite.white(),
        test.duration_ms
    );
    println!(
        "{}",
        "──────────────────────────────────────────────────────────────────────────────────"
            .cyan()
    );

    if let Some(ref err) = test.error_message {
        println!();
        println!("  {}", "ERROR MESSAGE:".bold().bright_red());
        println!("    {}", err.bright_white());
    }

    if let Some(ref trace) = test.stack_trace {
        println!();
        println!("  {}", "STACK TRACE:".bold().bright_yellow());
        for line in trace.lines().take(6) {
            println!("    {}", line.dimmed());
        }
        if trace.lines().count() > 6 {
            println!("    {}", "... (truncated for brevity)".dimmed());
        }
    }

    if let Some(ref chaos) = test.chaos_event {
        println!();
        println!("  {}", "CORRELATED L4/L7 CHAOS TELEMETRY:".bold().bright_cyan());
        println!("    Layer:             {}", chaos.layer.bright_white());
        println!("    Event Type:        {}", chaos.event_type.bright_white());
        if chaos.latency_ms > 0 {
            println!(
                "    Injected Latency:  {}ms (jitter: ±{}ms)",
                chaos.latency_ms.to_string().yellow(),
                chaos.jitter_ms
            );
        }
        if chaos.packet_loss_rate > 0.0 {
            println!(
                "    Packet Loss Rate:  {:.1}%",
                chaos.packet_loss_rate * 100.0
            );
        }
        if let Some(ref log) = chaos.proxy_log {
            println!("    Proxy Log Snippet: {}", log.bright_cyan());
        }
    }

    println!();
    println!(
        "{}",
        "──────────────────────────────────────────────────────────────────────────────────"
            .cyan()
    );
    println!();
}

/// Display the final scorecard and XP progression footer
pub fn display_triage_scorecard(
    result: &TriageResult,
    submission: &TriageSubmission,
    state: &GamificationState,
) {
    println!();
    println!(
        "{}",
        "══════════════════════════════════════════════════════════════════════════════════"
            .bright_cyan()
    );
    if result.correct {
        println!(
            "   {} {}",
            "✓ TRIAGE HYPOTHESIS ACCEPTED!".bold().bright_green(),
            format!("(+{} XP)", result.score_awarded).bold().bright_yellow()
        );
    } else {
        println!(
            "   {} {}",
            "✗ TRIAGE HYPOTHESIS REJECTED".bold().bright_red(),
            "(0 XP)".dimmed()
        );
    }
    println!(
        "{}",
        "══════════════════════════════════════════════════════════════════════════════════"
            .bright_cyan()
    );
    println!();

    println!("  Test Under Investigation: {}", submission.test_id.bright_cyan());
    println!("  Your Hypothesis Category:  {}", submission.learner_category.display_name().bold());
    println!("  Actual Ground Truth:      {}", result.actual_category.display_name().bold().bright_yellow());
    println!();

    println!("  {}", "EXPLANATION & SENIOR QA CRITIQUE:".bold().bright_white());
    for line in result.feedback.lines() {
        println!("    {}", line);
    }
    println!();

    if !result.detailed_reasons.is_empty() {
        println!("  {}", "SCORING BREAKDOWN:".bold().bright_white());
        for r in &result.detailed_reasons {
            println!("    • {}", r.bright_cyan());
        }
        println!();
    }

    if let Some(ref badge) = result.badge_unlocked {
        println!(
            "  🏆 {} '{}'!",
            "NEW BADGE UNLOCKED:".bold().bright_yellow(),
            badge.bold().bright_white()
        );
        println!();
    }

    // Render XP bar
    let progress_bar = render_level_progress_bar(state.total_xp, 28);
    println!("  Progress: {}", progress_bar);
    println!();
}
