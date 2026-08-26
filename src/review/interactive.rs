use crate::review::{ReviewConfig, ReviewReport, Severity, apply_all_fixes, run_review};
use colored::Colorize;
use std::io::{self, BufRead, Write};
use std::path::Path;

/// Renders a formatted scorecard for the review report to stdout
pub fn display_review_report(report: &ReviewReport) {
    println!();
    println!(
        "{}",
        "╔══════════════════════════════════════════════════════════════════════════════╗"
            .bright_cyan()
    );
    println!(
        "{}",
        "║           ⚡  CHERENKOV-LINGS VIRTUAL SENIOR QA CODE REVIEW  ⚡             ║"
            .bold()
            .bright_cyan()
    );
    println!(
        "{}",
        "╚══════════════════════════════════════════════════════════════════════════════╝"
            .bright_cyan()
    );
    println!();

    // Score Badge & Status
    let score_bar = render_score_bar(report.score);
    let status_str = if report.passed {
        "✓ PASSED ENTERPRISE SDET STANDARDS".green().bold()
    } else {
        "✗ ACTION REQUIRED (ANTI-PATTERNS DETECTED)".red().bold()
    };

    println!("  Target: {}", report.exercise_name.bright_white().bold());
    println!(
        "  Quality Score: {} [{}]",
        format!("{}/100", report.score).bright_yellow().bold(),
        score_bar
    );
    println!("  Assessment:    {}", status_str);
    println!();

    // Violations List
    if report.violations.is_empty() {
        println!(
            "  {} {}",
            "✓".green().bold(),
            "Zero AST rule violations detected! Code is clean, resilient, and deterministic."
                .green()
        );
        println!();
    } else {
        println!(
            "{}",
            format!(
                "  ── Detected Rule Violations ({}) ─────────────────────────────",
                report.violations.len()
            )
            .bright_yellow()
        );
        println!();

        for (i, v) in report.violations.iter().enumerate() {
            let severity_badge = match v.severity {
                Severity::Error => "[ERROR]".red().bold(),
                Severity::Warning => "[WARN] ".yellow().bold(),
                Severity::Info => "[INFO] ".cyan().bold(),
            };

            println!(
                "  {}. {} {} (Line {})",
                (i + 1).to_string().bold(),
                severity_badge,
                v.rule_id.bright_white().bold(),
                v.line_number.to_string().cyan()
            );
            println!("     {}", v.message.dimmed());
            println!("     Offending Code: {}", v.code_snippet.trim().red());
            if let Some(fix) = &v.suggested_fix {
                println!("     Suggested Fix:  {}", fix.trim().green());
            }
            println!();
        }
    }

    // Senior QA Mentor Critique
    println!(
        "{}",
        "  ── 🧠 Senior QA Mentor Critique ──────────────────────────────────────".bright_cyan()
    );
    println!();
    for line in report.mentor_critique.lines() {
        println!("  {}", line);
    }
    println!();

    // Socratic Questions
    if !report.socratic_questions.is_empty() {
        println!(
            "{}",
            "  ── 💡 Socratic Thinking Questions (Reflect & Learn) ─────────────────"
                .bright_magenta()
        );
        println!();
        for (idx, q) in report.socratic_questions.iter().enumerate() {
            println!("  Q{}: {}", idx + 1, q.bright_white());
        }
        println!();
    }
}

/// Runs the interactive Fix-It-Together flow in the terminal
pub fn run_interactive_review(
    target_path: &str,
    config: &ReviewConfig,
) -> Result<(), Box<dyn std::error::Error>> {
    let p = resolve_target_path(target_path)?;

    loop {
        let report = run_review(&p, config)?;
        display_review_report(&report);

        if report.violations.is_empty() {
            println!(
                "{} {}",
                "🎉".green(),
                "Your test implementation meets all Enterprise SDET criteria! Great work."
                    .bold()
                    .green()
            );
            break;
        }

        println!(
            "{}",
            "══════════════════════════════════════════════════════════════════════════════"
                .bright_cyan()
        );
        println!(
            "{}",
            "  Interactive Fix-It-Together Wizard:"
                .bold()
                .bright_white()
        );
        println!("  [1] Preview unified diff of proposed automated fixes");
        println!("  [2] Apply all automated fixes to file");
        println!("  [3] Re-run review");
        println!("  [q] Exit review wizard");
        print!("\n  Choose an option: ");
        let _ = io::stdout().flush();

        let mut input = String::new();
        let stdin = io::stdin();
        let mut handle = stdin.lock();
        if handle.read_line(&mut input).is_err() || input.trim().is_empty() {
            // Non-interactive or EOF
            break;
        }

        match input.trim() {
            "1" => {
                println!();
                println!(
                    "{}",
                    "── Proposed Unified Diff ──────────────────────────────────────"
                        .bright_yellow()
                );
                if let Some(diff) = &report.suggested_diff {
                    for line in diff.lines() {
                        if line.starts_with('+') && !line.starts_with("+++") {
                            println!("{}", line.green());
                        } else if line.starts_with('-') && !line.starts_with("---") {
                            println!("{}", line.red());
                        } else if line.starts_with('@') {
                            println!("{}", line.cyan());
                        } else {
                            println!("{}", line);
                        }
                    }
                } else {
                    println!("No automated diff available for detected violations.");
                }
                println!();
                println!("Press Enter to continue...");
                let mut pause = String::new();
                let _ = handle.read_line(&mut pause);
            }
            "2" => match apply_all_fixes(&p) {
                Ok(_) => {
                    println!();
                    println!(
                        "{} Successfully applied automated fixes to {}!",
                        "✓".green().bold(),
                        p.display().to_string().bright_yellow()
                    );
                    println!();
                }
                Err(e) => {
                    println!();
                    println!("{} Failed to apply fixes: {}", "✗".red().bold(), e);
                    println!();
                }
            },
            "3" => {
                // Re-runs on loop start
                continue;
            }
            "q" | "Q" | "exit" => {
                println!("\nExiting review wizard. Happy testing!\n");
                break;
            }
            _ => {
                println!("\nInvalid selection. Please choose 1, 2, 3, or q.\n");
            }
        }
    }

    Ok(())
}

fn render_score_bar(score: u32) -> String {
    let total_blocks: usize = 10;
    let filled_blocks = (score as f32 / 100.0 * total_blocks as f32).round() as usize;
    let empty_blocks = total_blocks.saturating_sub(filled_blocks);

    let filled = "█".repeat(filled_blocks);
    let empty = "░".repeat(empty_blocks);

    if score >= 80 {
        format!("{}{}", filled.green(), empty.dimmed())
    } else if score >= 50 {
        format!("{}{}", filled.yellow(), empty.dimmed())
    } else {
        format!("{}{}", filled.red(), empty.dimmed())
    }
}

fn resolve_target_path(
    target_path: &str,
) -> Result<std::path::PathBuf, Box<dyn std::error::Error>> {
    let p = Path::new(target_path);
    if p.is_file() {
        return Ok(p.to_path_buf());
    }

    if p.is_dir() {
        // Look for common drill file names
        for candidate in &[
            "exercise.ts",
            "exercise.py",
            "exercise.java",
            "exercise.rs",
            "exercise.js",
            "solution.ts",
            "solution.py",
        ] {
            let candidate_path = p.join(candidate);
            if candidate_path.is_file() {
                return Ok(candidate_path);
            }
        }
    }

    // Try finding within exercises/ directory
    let exercises_root = Path::new("exercises");
    if exercises_root.exists() {
        let nested = exercises_root.join(target_path);
        if nested.is_file() {
            return Ok(nested);
        }
        if nested.is_dir() {
            for candidate in &[
                "exercise.ts",
                "exercise.py",
                "exercise.java",
                "exercise.rs",
                "exercise.js",
            ] {
                let candidate_path = nested.join(candidate);
                if candidate_path.is_file() {
                    return Ok(candidate_path);
                }
            }
        }
    }

    if p.exists() {
        Ok(p.to_path_buf())
    } else {
        Err(format!(
            "Could not find target file or exercise at '{}'",
            target_path
        )
        .into())
    }
}
