mod config;
pub mod feedback;
pub mod gamification;
pub mod mcp;
pub mod pipeline;
pub mod proxy;
pub mod reports;
pub mod review;
pub mod runner;
pub mod triage;
mod watcher;
pub mod device_manager;

use clap::{Parser, Subcommand};
use colored::Colorize;
use std::path::Path;
use std::sync::Arc;
use tokio::sync::mpsc;

#[derive(Parser)]
#[command(name = "cherenkov-lings")]
#[command(version = "1.0.0")]
#[command(about = "Interactive Quality Engineering & SDET Learning Platform", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Initialize a new learning workspace
    Init {
        #[arg(short, long)]
        name: Option<String>,
    },
    /// Start the interactive learning watcher on a track
    Watch {
        #[arg(short, long)]
        track: String,
    },
    /// Diagnose why an exercise is failing (static analysis & anti-pattern root cause)
    Diagnose {
        #[arg(short, long)]
        file: Option<String>,
    },
    /// Start the standalone Programmable Chaos Proxy
    Proxy {
        /// Port to listen on
        #[arg(short, long, default_value_t = 8086)]
        port: u16,
        /// Upstream server address to forward traffic to
        #[arg(short, long, default_value = "127.0.0.1:8081")]
        upstream: String,
        /// Base artificial latency in milliseconds
        #[arg(short, long, default_value_t = 0)]
        latency: u64,
        /// Latency variance/jitter in milliseconds
        #[arg(short, long, default_value_t = 0)]
        jitter: u64,
        /// Probability of Layer 4 raw TCP connection drop (0.0 to 1.0)
        #[arg(short, long, default_value_t = 0.0)]
        drop_rate: f64,
    },
    /// Start the Model Context Protocol (MCP) JSON-RPC stdio server
    Mcp,
    /// View interactive QA learning progress, XP level, badges, and curriculum completion
    Dashboard,
    /// Audit and verify the integrity, contract completeness, and health of the entire curriculum
    Audit,
    /// Scaffold a new drill with standard contracts (exercise, solution, hints, and theory)
    NewDrill {
        /// Track ID (e.g. playwright-ts, restassured-java, k6-js, maestro-mobile, genai-qa, devsecops-python, foundations, jmeter, tool-decisions, contract-pact, a11y-axe)
        #[arg(short, long)]
        track: String,
        /// Drill folder name (e.g. 11_network_throttling)
        #[arg(short, long)]
        name: String,
        /// Human-readable title
        #[arg(long)]
        title: Option<String>,
    },
    /// [Sprint 4] AI-powered virtual Senior QA code review and architectural mentorship
    Review {
        /// Target file or exercise path to review (positional)
        #[arg(value_name = "TARGET")]
        target: Option<String>,
        /// Optional flag alias for target file
        #[arg(short, long)]
        file: Option<String>,
        /// Optional local LLM endpoint (e.g., http://localhost:11434/api/generate)
        #[arg(long)]
        llm: Option<String>,
        /// Model name for LLM (e.g., llama3, mistral, codellama)
        #[arg(long)]
        model: Option<String>,
        /// Automatically apply fixes without interactive prompt
        #[arg(long)]
        fix: bool,
        /// Strict mode (fails on any warning or error)
        #[arg(long)]
        strict: bool,
    },
    /// [Sprint 4] Local CI/CD Pipeline Simulator for GitHub Actions / GitLab
    Pipeline {
        /// Pipeline action ('run' or 'validate') or path to workflow YAML file
        #[arg(default_value = "run")]
        action: String,
        /// Path to workflow YAML file (defaults to .github/workflows/ci.yml if omitted)
        path: Option<String>,
        /// Enforce strict SDET validation failure
        #[arg(short, long)]
        strict: bool,
        /// Verbose output with full runner logs
        #[arg(short, long)]
        verbose: bool,
    },
    /// [Sprint 4] Interactive Root-Cause Triage Hypothesis Engine
    Triage {
        /// Test ID to investigate or evaluate (e.g. BUG-101, FLAKE-201, ANTI-301)
        #[arg(short, long)]
        test_id: Option<String>,
        /// Category hypothesis (real_bug, flaky_infra, anti_pattern)
        #[arg(short, long)]
        category: Option<String>,
        /// Root-cause explanation
        #[arg(short, long)]
        explanation: Option<String>,
        /// Suggested remediation / fix
        #[arg(long)]
        fix: Option<String>,
        /// Generate Allure JSON results and interactive HTML report
        #[arg(short, long)]
        report: bool,
        /// Output directory for Allure report
        #[arg(short, long, default_value = "target/allure-report")]
        output_dir: String,
        /// List all chaotic test failures available for triage
        #[arg(short, long)]
        list: bool,
    },
    /// [Sprint 4] Generate Allure-compatible JSON test results and interactive HTML report
    Report {
        /// Output directory for Allure report
        #[arg(short, long, default_value = "target/allure-report")]
        output_dir: String,
    },
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    match &cli.command {
        Commands::Init { name } => {
            let project_name = name
                .clone()
                .unwrap_or_else(|| "my-sdet-journey".to_string());

            // Create exercise directories
            let exercise_dirs = vec![
                "exercises/00_foundations",
                "exercises/01_web_playwright_ts",
                "exercises/02_api_restassured_java",
                "exercises/03_mobile_maestro",
                "exercises/04_perf_k6_js",
                "exercises/05_perf_jmeter",
                "exercises/06_genai_qa",
                "exercises/08_tool_decisions",
                "exercises/07_cloud_devsecops",
                "exercises/09_contract_pact",
                "exercises/10_a11y_axe",
            ];
            for dir in &exercise_dirs {
                let p = Path::new(dir);
                if !p.exists() {
                    std::fs::create_dir_all(p)?;
                }
            }

            // Rich Manual-QA-first welcome banner
            println!();
            println!(
                "{}",
                "╔══════════════════════════════════════════════════════════════════╗"
                    .bright_cyan()
            );
            println!(
                "{}",
                "║      ⚡  CHERENKOV-LINGS  — Interactive QA Learning Platform      ║"
                    .bright_cyan()
            );
            println!(
                "{}",
                "╚══════════════════════════════════════════════════════════════════╝"
                    .bright_cyan()
            );
            println!();
            println!(
                "  {} Workspace {} ready.",
                "✓".green(),
                project_name.bright_yellow()
            );
            println!();
            println!(
                "{}",
                "  YOUR LEARNING PATH (start here, go in order):"
                    .bold()
                    .white()
            );
            println!();
            println!(
                "  {}  {}  {}",
                "STEP 1".bright_white().bold(),
                "Foundations — What IS an automated test?".bright_yellow(),
                "(no tools needed, just Python)".dimmed()
            );
            println!(
                "         {}",
                "cherenkov-lings watch --track=foundations".bright_cyan()
            );
            println!(
                "         Open: {}",
                "exercises/00_foundations/01_what_is_a_test/exercise.py".dimmed()
            );
            println!();
            println!(
                "  {}  {}  {}",
                "STEP 2".bright_white().bold(),
                "UI Automation — Playwright TypeScript".bright_yellow(),
                "(needs Node.js)".dimmed()
            );
            println!(
                "         {}",
                "cherenkov-lings watch --track=playwright-ts".bright_cyan()
            );
            println!(
                "         Open: {}",
                "exercises/01_web_playwright_ts/04_first_playwright_test/exercise.ts".dimmed()
            );
            println!();
            println!(
                "  {}  {}  {}",
                "STEP 3".bright_white().bold(),
                "API Automation — REST Assured Java".bright_yellow(),
                "(needs Java + Maven)".dimmed()
            );
            println!(
                "         {}",
                "cherenkov-lings watch --track=restassured-java".bright_cyan()
            );
            println!();
            println!(
                "  {}  {}  {}",
                "STEP 4".bright_white().bold(),
                "Mobile Automation — Maestro YAML".bright_yellow(),
                "(needs Maestro CLI)".dimmed()
            );
            println!(
                "         {}",
                "cherenkov-lings watch --track=maestro-mobile".bright_cyan()
            );
            println!();
            println!(
                "  {}  {}  {}",
                "STEP 5".bright_white().bold(),
                "Performance — k6 (modern) or JMeter (enterprise)".bright_yellow(),
                "(needs k6 or JMeter)".dimmed()
            );
            println!(
                "         {}   or   {}",
                "cherenkov-lings watch --track=k6-js".bright_cyan(),
                "cherenkov-lings watch --track=jmeter".bright_cyan()
            );
            println!();
            println!(
                "  {}  {}",
                "STEP 6".bright_white().bold(),
                "Which tool is right for which job?".bright_yellow()
            );
            println!(
                "         {}",
                "cherenkov-lings watch --track=tool-decisions".bright_cyan()
            );
            println!();
            println!(
                "{}",
                "  ─────────────────────────────────────────────────────────────────".dimmed()
            );
            println!();
            println!(
                "  {} Start the Micro-Crucible sandbox FIRST:",
                "⚡".yellow()
            );
            println!("    {}", ".\\crucible\\start.bat".bright_white());
            println!();
            println!("  {} When you are stuck on any drill:", "💡".yellow());
            println!(
                "    Check {} in the same folder as your exercise.",
                "hints.md".bright_white()
            );
            println!(
                "    Or run: {}",
                "cherenkov-lings diagnose --file=<path/to/exercise>".bright_cyan()
            );
            println!();
            println!("{}", "  🚀 Begin your journey:".bold().green());
            println!(
                "     {}",
                "cherenkov-lings watch --track=foundations"
                    .bold()
                    .bright_cyan()
            );
            println!();
        }

        Commands::Watch { track } => {
            println!("Starting watcher for track: {}", track.bright_cyan());

            // Load configuration
            let cfg = config::load_config("lings.toml")?;
            let track_cfg = cfg.tracks.iter().find(|t| t.id == *track);

            if let Some(track_config) = track_cfg {
                println!("{} Track loaded: {}", "✓".green(), track_config.name.bold());

                let device_manager = crate::device_manager::DeviceManager::new();
                if track_config.runner == "maestro" {
                    // device_manager.start_android_emulator("Pixel_6_Pro_API_33");
                } else if track_config.runner == "node" {
                    device_manager.start_browser_node("chromium");
                }

                // Auto-spawn background Chaos Proxy if configured in lings.toml
                let _proxy_shutdown = if cfg.platform.chaos_proxy_port > 0 {
                    let proxy_listen: std::net::SocketAddr =
                        format!("127.0.0.1:{}", cfg.platform.chaos_proxy_port)
                            .parse()
                            .unwrap_or_else(|_| "127.0.0.1:8086".parse().unwrap());
                    let proxy_upstream: std::net::SocketAddr = "127.0.0.1:8081".parse().unwrap();

                    let proxy_cfg = proxy::ProxyConfig {
                        listen_addr: proxy_listen,
                        upstream_addr: proxy_upstream,
                        default_latency_ms: 0,
                        default_jitter_ms: 0,
                        default_drop_rate: 0.0,
                        default_fault_rate: 0.0,
                        upstream_timeout_ms: 5000,
                    };

                    match proxy::ProxyServer::spawn_background(proxy_cfg).await {
                        Ok((_handle, shutdown_tx)) => {
                            println!(
                                "{} Chaos Proxy active on {} -> {}",
                                "✓".green(),
                                proxy_listen.to_string().bright_yellow(),
                                proxy_upstream.to_string().bright_white()
                            );
                            Some(shutdown_tx)
                        }
                        Err(e) => {
                            eprintln!(
                                "{} Could not start Chaos Proxy on port {}: {}",
                                "⚠️".yellow(),
                                cfg.platform.chaos_proxy_port,
                                e
                            );
                            None
                        }
                    }
                } else {
                    None
                };

                let platform_version = cfg.platform.version.clone();
                let track_id = track_config.id.clone();
                let track_name = track_config.name.clone();
                let track_ext = track_config.extension.clone();
                let chaos_latency = cfg.evaluation.chaos_latency_ms;
                let chaos_jitter = cfg.evaluation.chaos_jitter_ms;
                let flakiness_iterations = cfg.evaluation.flakiness_iterations;
                let timeout_per_iter = cfg.evaluation.flakiness_timeout_ms as u64;
                let pass_threshold = cfg.evaluation.pass_threshold as f64;

                let exercise_dir = Path::new(&track_config.exercise_dir);
                if !exercise_dir.exists() {
                    std::fs::create_dir_all(exercise_dir)?;
                    println!(
                        "{} Created exercise directory: {:?}",
                        "✓".green(),
                        exercise_dir
                    );
                }

                // Initialize Runner based on track runner configuration
                let runner_arc: Option<Arc<runner::AnyRunner>> = if track_config.runner == "node" {
                    let worker_script = Path::new("workers/node_worker.js");
                    if !worker_script.exists() {
                        eprintln!(
                            "{} Worker script not found at {:?}. Please verify worker installation.",
                            "✗".red(),
                            worker_script
                        );
                        return Ok(());
                    }
                    println!("{} Spawning Node.js IPC Worker...", "⚡".yellow());
                    let runner = runner::NodeRunner::start(worker_script).await?;
                    println!("{} Node.js IPC Worker connected and ready.", "✓".green());
                    Some(Arc::new(runner::AnyRunner::Node(Arc::new(runner))))
                } else if track_config.runner == "jvm" {
                    println!("{} Initializing REST Assured JVM Runner...", "⚡".yellow());
                    let runner = runner::JvmRunner::new(&track_config.exercise_dir);
                    println!(
                        "{} REST Assured JVM Runner initialized (Maven: {}).",
                        "✓".green(),
                        runner.maven_cmd().bright_yellow()
                    );
                    Some(Arc::new(runner::AnyRunner::Jvm(Arc::new(runner))))
                } else if track_config.runner == "k6" {
                    println!("{} Initializing k6 Load Testing Runner...", "⚡".yellow());
                    let runner = runner::K6Runner::new();
                    println!("{} k6 Load Testing Runner initialized.", "✓".green());
                    Some(Arc::new(runner::AnyRunner::K6(Arc::new(runner))))
                } else if track_config.runner == "maestro" {
                    println!("{} Initializing Maestro Mobile Runner...", "⚡".yellow());
                    let runner = runner::MaestroRunner::new();
                    println!(
                        "{} Maestro Mobile Definition Runner initialized.",
                        "✓".green()
                    );
                    Some(Arc::new(runner::AnyRunner::Maestro(Arc::new(runner))))
                } else if track_config.runner == "python" {
                    println!("{} Initializing Pytest Runner...", "⚡".yellow());
                    let runner = runner::PytestRunner::new();
                    println!("{} Pytest Runner initialized.", "✓".green());
                    Some(Arc::new(runner::AnyRunner::Pytest(Arc::new(runner))))
                } else if track_config.runner == "jmeter" {
                    println!("{} Initializing JMeter Runner...", "⚡".yellow());
                    let runner = runner::JMeterRunner::new();
                    println!("{} JMeter Runner initialized.", "✓".green());
                    Some(Arc::new(runner::AnyRunner::Jmeter(Arc::new(runner))))
                } else if track_config.runner == "pipeline" {
                    println!("{} Initializing CI/CD Pipeline Simulator...", "⚡".yellow());
                    let runner = runner::PipelineRunner::new();
                    println!(
                        "{} CI/CD Pipeline Simulator initialized (in-process, pass score {}/100).",
                        "✓".green(),
                        runner.pass_score().to_string().bright_yellow()
                    );
                    Some(Arc::new(runner::AnyRunner::Pipeline(Arc::new(runner))))
                } else {
                    None
                };

                let (tx, mut rx) = mpsc::channel::<String>(100);

                // Spawn a background task to process file-change events from the watcher
                let runner_for_task = runner_arc.clone();
                let task_track_id = track_id.clone();
                tokio::spawn(async move {
                    while let Some(path) = rx.recv().await {
                        // Filter out build artifacts, target/ directory, .class files, or non-matching extensions
                        if watcher::should_ignore_path(Path::new(&path))
                            || !path.ends_with(&track_ext)
                        {
                            continue;
                        }

                        println!("\n{}", "========================================================================================".cyan());
                        println!(
                            " {} v{}  |  Track: [{}]",
                            "CHERENKOV-LINGS".bold().bright_cyan(),
                            platform_version,
                            track_name.bright_yellow()
                        );
                        println!(" File saved: {}", path.bright_white());
                        println!("{}", "========================================================================================".cyan());

                        if let Some(ref runner) = runner_for_task {
                            let chaos_header =
                                format!("delay={}ms;jitter={}ms", chaos_latency, chaos_jitter);
                            let total_timeout = timeout_per_iter * (flakiness_iterations as u64);

                            println!(
                                "{} Running {} test suite ({} iterations with chaos: {})...",
                                "⏳".yellow(),
                                track_name,
                                flakiness_iterations,
                                chaos_header
                            );

                            match runner
                                .run_drill(
                                    &path,
                                    &chaos_header,
                                    flakiness_iterations,
                                    total_timeout,
                                )
                                .await
                            {
                                Ok(response) => {
                                    // Perform static analysis of the modified exercise file
                                    let ast_report =
                                        feedback::analyze_file(&path).unwrap_or_else(|_| {
                                            feedback::StaticAnalysisReport {
                                                file_path: path.clone(),
                                                ..Default::default()
                                            }
                                        });

                                    // Evaluate the 4D Feedback Matrix
                                    let scorecard = feedback::evaluate_feedback(
                                        &response,
                                        &ast_report,
                                        &track_name,
                                        &platform_version,
                                        pass_threshold,
                                        1000,
                                    );

                                    // Print the full terminal scorecard
                                    let rendered = feedback::render_scorecard(&scorecard);
                                    print!("{}", rendered);

                                    // Gamification: Record progress and render progression scorecard on pass
                                    let drill_id = gamification::extract_drill_id_from_path(&path);
                                    let tier = gamification::tier_for_track_or_drill(
                                        &task_track_id,
                                        &drill_id,
                                    );

                                    if scorecard.passed {
                                        let mut state = match gamification::load_progress(
                                            None::<&Path>,
                                        ) {
                                            Ok(s) => s,
                                            Err(e) => {
                                                eprintln!(
                                                    "{} Progress file unreadable ({}); starting fresh from zero XP.",
                                                    "⚠️".yellow(),
                                                    e
                                                );
                                                Default::default()
                                            }
                                        };
                                        let run_ctx = gamification::DrillRunContext {
                                            track_id: task_track_id.clone(),
                                            drill_id: drill_id.clone(),
                                            file_path: path.clone(),
                                            passed: true,
                                            total_score: scorecard.total_score,
                                            correctness_score: scorecard.correctness.score,
                                            flakiness_score: scorecard.flakiness.score,
                                            locator_score: scorecard.locator_quality.score,
                                            speed_score: scorecard.speed.score,
                                            passed_iterations: response.passed_iterations,
                                            iterations: response.iterations,
                                            avg_duration_ms: if response.iterations > 0 {
                                                response.total_duration_ms
                                                    / (response.iterations as u64)
                                            } else {
                                                response.total_duration_ms
                                            },
                                            baseline_duration_ms: 1000,
                                            tier,
                                            timestamp: None,
                                        };

                                        let (xp_earned, newly_unlocked) =
                                            state.record_drill_run(&run_ctx);
                                        if let Err(e) =
                                            gamification::save_progress(&state, None::<&Path>)
                                        {
                                            eprintln!(
                                                "{} Failed to save progress to {}: {}",
                                                "⚠️".yellow(),
                                                gamification::PROGRESS_FILE,
                                                e
                                            );
                                        }

                                        let gamification_footer =
                                            gamification::render_gamification_scorecard_with_tier(
                                                xp_earned,
                                                tier,
                                                &state,
                                                &newly_unlocked,
                                            );
                                        print!("{}", gamification_footer);
                                    }
                                }
                                Err(err) => {
                                    eprintln!(
                                        "{} Runner communication error: {}",
                                        "✗".bold().red(),
                                        err
                                    );
                                }
                            }
                        } else {
                            println!(
                                " Runner for track '{}' is not currently active.",
                                track_name
                            );
                        }
                    }
                });

                watcher::watch_exercises(exercise_dir, tx).await?;
            } else {
                eprintln!("Error: Track '{}' not found in lings.toml", track);
                eprintln!(
                    "Available tracks: playwright-ts, restassured-java, k6-js, maestro-mobile, genai-qa, devsecops-python, foundations, jmeter, tool-decisions, contract-pact, a11y-axe"
                );
            }
        }
        Commands::Diagnose { file } => {
            let target = file.clone().unwrap_or_else(|| {
                let default_exercise =
                    "exercises/01_web_playwright_ts/01_hydration_timing/exercise.ts";
                default_exercise.to_string()
            });

            let target_path = Path::new(&target);
            if !target_path.exists() {
                eprintln!(
                    "{} File not found: {:?}\n   Usage: cherenkov-lings diagnose --file=<path/to/exercise>",
                    "✗".red().bold(),
                    target
                );
                return Ok(());
            }

            // Determine track name dynamically based on file path / extension
            let track_name = if target.ends_with(".yaml")
                || target.ends_with(".yml")
                || target.contains("maestro")
            {
                "Mobile UI Automation (Maestro YAML)"
            } else if target.ends_with(".java") || target.contains("restassured") {
                "API Resilience & Security (REST Assured Java)"
            } else if target.ends_with(".js") || target.contains("k6") {
                "High-Concurrency Load Testing (k6 JS)"
            } else if target.contains("genai") {
                "GenAI QA Testing (Playwright TypeScript)"
            } else {
                "Modern Web Automation (Playwright TypeScript)"
            };

            match feedback::analyze_file(target_path) {
                Ok(report) => {
                    let version = "1.0.0";
                    let rendered = feedback::render_diagnostic(&report, track_name, version);
                    print!("{}", rendered);
                }
                Err(err) => {
                    eprintln!("{} Failed to analyze file: {}", "✗".red().bold(), err);
                }
            }
        }
        Commands::Proxy {
            port,
            upstream,
            latency,
            jitter,
            drop_rate,
        } => {
            let listen_addr: std::net::SocketAddr = format!("127.0.0.1:{}", port).parse()?;
            let upstream_addr: std::net::SocketAddr = if upstream.contains(':') {
                upstream.parse()?
            } else {
                format!("127.0.0.1:{}", upstream).parse()?
            };

            let config = proxy::ProxyConfig {
                listen_addr,
                upstream_addr,
                default_latency_ms: *latency,
                default_jitter_ms: *jitter,
                default_drop_rate: *drop_rate,
                default_fault_rate: 0.0,
                upstream_timeout_ms: 5000,
            };

            println!("{}", "========================================================================================".cyan());
            println!(
                " {} v1.0.0  |  Programmable Chaos Proxy",
                "CHERENKOV-LINGS".bold().bright_cyan()
            );
            println!(
                " Listening on:     {}",
                format!("http://{}", listen_addr).bright_yellow()
            );
            println!(
                " Forwarding to:    {}",
                format!("http://{}", upstream_addr).bright_white()
            );
            if *latency > 0 || *jitter > 0 {
                println!(" Default Latency:  {}ms (jitter: ±{}ms)", latency, jitter);
            }
            if *drop_rate > 0.0 {
                println!(" Default Drop Rate: {:.1}%", drop_rate * 100.0);
            }
            println!("{}", "========================================================================================".cyan());
            println!(
                "{} Proxy running. Press Ctrl+C to terminate.",
                "⚡".yellow()
            );

            let (shutdown_tx, shutdown_rx) = tokio::sync::oneshot::channel();
            let server = proxy::ProxyServer::new(config);

            tokio::spawn(async move {
                tokio::signal::ctrl_c().await.ok();
                let _ = shutdown_tx.send(());
            });

            if let Err(e) = server.run(shutdown_rx).await {
                eprintln!("{} Proxy server error: {}", "✗".red(), e);
            }
            println!("\n{} Proxy stopped cleanly.", "✓".green());
        }
        Commands::Mcp => {
            mcp::run_mcp_server();
        }
        Commands::Dashboard => {
            let cfg = config::load_config("lings.toml")
                .unwrap_or_else(|_| gamification::embedded_config());

            let state = match gamification::load_progress(None::<&Path>) {
                Ok(s) => s,
                Err(e) => {
                    eprintln!(
                        "{} Progress file unreadable ({}); showing default state.",
                        "⚠️".yellow(),
                        e
                    );
                    Default::default()
                }
            };
            let dashboard_output = gamification::render_dashboard(&state, &cfg);
            print!("{}", dashboard_output);
        }
        Commands::Audit => {
            run_curriculum_audit();
        }
        Commands::NewDrill { track, name, title } => {
            run_new_drill(track, name, title.as_deref());
        }
        Commands::Review {
            target,
            file,
            llm,
            model,
            fix,
            strict,
        } => {
            run_review(
                target.as_deref().or(file.as_deref()),
                llm.as_deref(),
                model.as_deref(),
                *fix,
                *strict,
            );
        }
        Commands::Pipeline {
            action,
            path,
            strict,
            verbose,
        } => {
            run_pipeline(action, path.as_deref(), *strict, *verbose);
        }
        Commands::Triage {
            test_id,
            category,
            explanation,
            fix,
            report,
            output_dir,
            list,
        } => {
            run_triage_cmd(
                test_id.as_deref(),
                category.as_deref(),
                explanation.as_deref(),
                fix.as_deref(),
                *report,
                output_dir,
                *list,
            )?;
        }
        Commands::Report { output_dir } => {
            run_report_cmd(output_dir)?;
        }
    }

    Ok(())
}

fn run_curriculum_audit() {
    use std::fs;

    println!(
        "{}",
        "════════════════════════════════════════════════════════════════════════════════════════"
            .bright_cyan()
    );
    println!(
        "{}",
        "   🔬  CHERENKOV-LINGS CURRICULUM AUDIT & INTEGRITY VERIFICATION  🔬"
            .bold()
            .bright_white()
    );
    println!(
        "{}",
        "════════════════════════════════════════════════════════════════════════════════════════"
            .bright_cyan()
    );
    println!();

    let cfg = config::load_config("lings.toml").unwrap_or_else(|_| gamification::embedded_config());

    let mut total_drills = 0;
    let mut complete_drills = 0;
    let mut total_checks = 0;
    let mut passed_checks = 0;
    let mut issues: Vec<String> = Vec::new();

    println!(
        " {:<46} │ {:<8} │ {:<14} │ {:<8}",
        "Track Name", "Drills", "Contract Checks", "Health"
    );
    println!(
        " ───────────────────────────────────────────────┼──────────┼────────────────┼────────"
    );

    for track in &cfg.tracks {
        let track_path = Path::new(&track.exercise_dir);
        let mut track_drill_count = 0;
        let mut track_complete_count = 0;

        if track_path.exists() {
            // Find drill subdirectories
            let mut subdirs: Vec<_> = fs::read_dir(track_path)
                .into_iter()
                .flatten()
                .filter_map(|e| e.ok())
                .filter(|e| e.path().is_dir())
                .collect();

            // Tracks whose drills do not sit directly under exercise_dir (the
            // Maven-structured Java track) declare a drill_root in lings.toml.
            let drill_root = Path::new(track.drill_root());
            if drill_root != track_path && drill_root.exists() {
                subdirs = fs::read_dir(drill_root)
                    .into_iter()
                    .flatten()
                    .filter_map(|e| e.ok())
                    .filter(|e| e.path().is_dir())
                    .collect();
            }

            subdirs.sort_by_key(|a| a.file_name());

            for dir in subdirs {
                let p = dir.path();
                let dir_name = p.file_name().unwrap_or_default().to_string_lossy();
                if dir_name.starts_with('.') || dir_name == "target" || dir_name == "src" {
                    continue;
                }

                track_drill_count += 1;
                total_drills += 1;

                let ext = &track.extension;
                let (ex_name, sol_name) = (track.exercise_file(), track.solution_file());

                let has_exercise = p.join(&ex_name).exists();
                let has_solution =
                    p.join(&sol_name).exists() || (ext == ".jmx" && p.join("solution.sh").exists());
                let has_hints = p.join("hints.md").exists();
                let has_theory = p.join("theory.md").exists();

                total_checks += 4;
                let mut dir_passed = 0;
                if has_exercise {
                    passed_checks += 1;
                    dir_passed += 1;
                } else {
                    issues.push(format!("Missing {}: {}", ex_name, p.display()));
                }
                if has_solution {
                    passed_checks += 1;
                    dir_passed += 1;
                } else {
                    issues.push(format!("Missing {}: {}", sol_name, p.display()));
                }
                if has_hints {
                    passed_checks += 1;
                    dir_passed += 1;
                } else {
                    issues.push(format!("Missing hints.md: {}", p.display()));
                }
                if has_theory {
                    passed_checks += 1;
                    dir_passed += 1;
                    // Check theory length
                    if let Ok(content) = fs::read_to_string(p.join("theory.md")) {
                        let word_count = content.split_whitespace().count();
                        if word_count < 150 {
                            issues.push(format!(
                                "Short theory.md ({} words): {}",
                                word_count,
                                p.display()
                            ));
                        }
                    }
                } else {
                    issues.push(format!("Missing theory.md: {}", p.display()));
                }

                total_checks += 1;
                if has_solution {
                    let sol_path = p.join(&sol_name);
                    if let Ok(content) = fs::read_to_string(&sol_path) {
                        let has_content = match ext.as_str() {
                            ".ts" => {
                                content.contains("import")
                                    || content.contains("test(")
                                    || content.contains("expect(")
                            }
                            ".py" => {
                                content.contains("def test_")
                                    || content.contains("class Test")
                                    || content.contains("import pytest")
                            }
                            ".java" => content.contains("import") || content.contains("@Test"),
                            ".jmx" => {
                                content.contains("<HTTPSamplerProxy")
                                    || content.contains("<TestPlan")
                            }
                            ".js" => {
                                content.contains("import")
                                    || content.contains("export")
                                    || content.contains("http.get")
                            }
                            ".yaml" => {
                                content.contains("launchApp")
                                    || content.contains("openLink")
                                    || content.contains("tapOn")
                            }
                            ".yml" => {
                                content.contains("jobs:")
                                    && (content.contains("runs-on") || content.contains("uses:"))
                            }
                            _ => !content.trim().is_empty(),
                        };
                        if has_content {
                            passed_checks += 1;
                            dir_passed += 1;
                        } else {
                            issues.push(format!(
                                "Solution file appears empty or invalid: {}",
                                sol_path.display()
                            ));
                        }
                    } else {
                        issues.push(format!("Cannot read solution file: {}", sol_path.display()));
                    }
                }

                if dir_passed == 5 {
                    track_complete_count += 1;
                    complete_drills += 1;
                }
            }
        }

        let status_emoji = if track_drill_count > 0 && track_complete_count == track_drill_count {
            "✅ 100%".green()
        } else if track_drill_count > 0 {
            format!("🟡 {}/{}", track_complete_count, track_drill_count).yellow()
        } else {
            "⏳ N/A".bright_black()
        };

        let file_summary = format!("{}/{} drills OK", track_complete_count, track_drill_count);
        println!(
            " {:<46} │ {:<8} │ {:<14} │ {}",
            track.name, track_drill_count, file_summary, status_emoji
        );
    }

    println!(
        " ───────────────────────────────────────────────┴──────────┴────────────────┴────────"
    );
    println!();

    let health_pct = if total_checks > 0 {
        (passed_checks as f64 / total_checks as f64) * 100.0
    } else {
        0.0
    };
    println!(" 📊 AUDIT SUMMARY:");
    println!(
        "    Total Tracks Scanned:     {}",
        cfg.tracks.len().to_string().cyan()
    );
    println!(
        "    Total Drills Validated:   {}",
        total_drills.to_string().green()
    );
    println!(
        "    Fully Compliant Drills:   {}/{} ({}%)",
        complete_drills,
        total_drills,
        if total_drills > 0 {
            (complete_drills as f64 / total_drills as f64 * 100.0).round() as u32
        } else {
            0
        }
    );
    println!(
        "    Contract Verification:    {}/{} checks passed",
        passed_checks, total_checks
    );
    println!("    Overall Curriculum Health: {:.1}%", health_pct);
    println!();

    if issues.is_empty() {
        println!(
            " {}",
            "✓ ALL DRILL CONTRACTS VERIFIED (100% HEALTHY). ZERO DISCREPANCIES DETECTED."
                .bold()
                .green()
        );
    } else {
        println!(" {} Issues Found:", "✗".red());
        for issue in &issues {
            println!("   - {}", issue.yellow());
        }
    }
    println!(
        "{}",
        "════════════════════════════════════════════════════════════════════════════════════════"
            .bright_cyan()
    );
}

fn run_new_drill(track_id: &str, drill_name: &str, title: Option<&str>) {
    use std::fs;

    let cfg = config::load_config("lings.toml").unwrap_or_else(|_| gamification::embedded_config());

    let track = match cfg.tracks.iter().find(|t| t.id == track_id) {
        Some(t) => t,
        None => {
            eprintln!(
                "{} Unknown track '{}'. Available tracks: {}",
                "✗".red(),
                track_id,
                cfg.tracks
                    .iter()
                    .map(|t| t.id.as_str())
                    .collect::<Vec<_>>()
                    .join(", ")
            );
            return;
        }
    };

    let target_dir = Path::new(&track.exercise_dir).join(drill_name);
    if target_dir.exists() {
        eprintln!(
            "{} Directory already exists: {}",
            "✗".red(),
            target_dir.display()
        );
        return;
    }

    if let Err(e) = fs::create_dir_all(&target_dir) {
        eprintln!("{} Failed to create directory: {}", "✗".red(), e);
        return;
    }

    let human_title = title.unwrap_or(drill_name);
    let ext = &track.extension;

    // 1. exercise file
    let exercise_content = format!(
        "/**\n * PRODUCTION STORY:\n * Real-world incident case study for {human_title}.\n * Brief summary of the outage or flakiness pattern.\n */\n\n// TODO: Implement fix for {human_title} anti-pattern.\n"
    );
    let _ = fs::write(
        target_dir.join(format!("exercise{}", ext)),
        exercise_content,
    );

    // 2. solution file
    let solution_content = format!(
        "/**\n * SDET Resilient Reference Solution for {human_title}.\n * Demonstrates resilient synchronization and robust assertions.\n */\n"
    );
    let _ = fs::write(
        target_dir.join(format!("solution{}", ext)),
        solution_content,
    );

    // 3. hints.md
    let hints_content = "## Hint 1 (Architectural Nudge)\nUnderstand why this pattern fails under asynchronous latency and network jitter.\n\n## Hint 2 (API Pattern)\nLook at the recommended synchronization or assertion pattern.\n\n## Hint 3 (Code Diff)\n```diff\n- // Old brittle code\n+ // New resilient code\n```\n";
    let _ = fs::write(target_dir.join("hints.md"), hints_content);

    // 4. theory.md
    let theory_content = format!(
        "# Theoretical Context: {human_title}\n\n## Real-World Incident Case Study\nIn a high-profile production incident, unhandled timing discrepancies caused significant disruption...\n\n## Protocol & Runtime Mechanism\nUnder the hood, asynchronous event dispatching and network race conditions lead to state desynchronization...\n\n```\n  [ Client / Test ] ────────► [ Network Buffer ] ────────► [ Target Service ]\n           │                           │                          │\n           ▼                           ▼                          ▼\n     Fast Dispatch              Variable Jitter           Asynchronous State\n```\n\n## You will now simulate this in the Crucible\nExecute this drill using `cherenkov-lings watch --track={track_id}` and verify the resilient pattern under injected chaos.\n"
    );
    let _ = fs::write(target_dir.join("theory.md"), theory_content);

    println!(
        "{} Scaffolding complete for new drill '{}'!",
        "✓".green(),
        human_title.bold()
    );
    println!("   Directory: {}", target_dir.display().to_string().cyan());
    println!("   Files created:");
    println!("   - exercise{}", ext);
    println!("   - solution{}", ext);
    println!("   - hints.md");
    println!("   - theory.md");
    println!();
    println!(
        "   Start watching with: {}",
        format!("cherenkov-lings watch --track={}", track_id).yellow()
    );
}

fn run_review(
    file_or_target: Option<&str>,
    llm: Option<&str>,
    model: Option<&str>,
    fix: bool,
    strict: bool,
) {
    let target = file_or_target
        .unwrap_or("exercises/01_web_playwright_ts/04_first_playwright_test/exercise.ts");
    let target_path = std::path::Path::new(target);

    let config = review::ReviewConfig {
        llm_endpoint: llm.map(|s| s.to_string()),
        llm_model: model.map(|s| s.to_string()),
        offline_fallback: true,
        strict_mode: strict,
        score_threshold: 80,
    };

    if fix {
        match review::apply_all_fixes(target_path) {
            Ok(_) => {
                println!(
                    "{} Successfully applied automated lint fixes to '{}'!",
                    "✓".green().bold(),
                    target.bright_white()
                );
            }
            Err(e) => {
                eprintln!(
                    "{} Failed to apply automated fixes: {}",
                    "✗".red().bold(),
                    e
                );
                std::process::exit(1);
            }
        }
        return;
    }

    if let Err(e) = review::run_interactive_review(target, &config) {
        eprintln!("{} Review error: {}", "✗".red().bold(), e);
        std::process::exit(1);
    }
}

fn run_pipeline(action: &str, path: Option<&str>, strict: bool, verbose: bool) {
    let (actual_action, workflow_path) = match (action, path) {
        ("run", Some(p)) => ("run", std::path::PathBuf::from(p)),
        ("validate", Some(p)) => ("validate", std::path::PathBuf::from(p)),
        ("run", None) => {
            let default_path = std::path::PathBuf::from(".github/workflows/ci.yml");
            if default_path.exists() {
                ("run", default_path)
            } else if let Ok(entries) = std::fs::read_dir(".github/workflows") {
                let mut found = None;
                for entry in entries.flatten() {
                    let p = entry.path();
                    if p.extension()
                        .is_some_and(|ext| ext == "yml" || ext == "yaml")
                    {
                        found = Some(p);
                        break;
                    }
                }
                if let Some(f) = found {
                    ("run", f)
                } else {
                    eprintln!(
                        "{} No GitHub Actions workflow found at .github/workflows/ci.yml or in .github/workflows/",
                        "✗".red()
                    );
                    std::process::exit(1);
                }
            } else {
                eprintln!(
                    "{} No .github/workflows directory found. Specify a path explicitly: cherenkov-lings pipeline run <path>",
                    "✗".red()
                );
                std::process::exit(1);
            }
        }
        ("validate", None) => {
            let default_path = std::path::PathBuf::from(".github/workflows/ci.yml");
            if default_path.exists() {
                ("validate", default_path)
            } else {
                eprintln!(
                    "{} No workflow file found to validate at .github/workflows/ci.yml",
                    "✗".red()
                );
                std::process::exit(1);
            }
        }
        (other_path, None) => {
            // User provided path as first argument (e.g. `cherenkov-lings pipeline my_workflow.yml`)
            ("run", std::path::PathBuf::from(other_path))
        }
        (act, Some(p)) => (act, std::path::PathBuf::from(p)),
    };

    if !workflow_path.exists() {
        eprintln!(
            "{} Workflow file does not exist: {}",
            "✗".red(),
            workflow_path.display()
        );
        std::process::exit(1);
    }

    match actual_action {
        "validate" => {
            let content = match std::fs::read_to_string(&workflow_path) {
                Ok(c) => c,
                Err(e) => {
                    eprintln!("{} Failed to read workflow file: {}", "✗".red(), e);
                    std::process::exit(1);
                }
            };
            let validation = pipeline::validate_workflow(&content);
            println!("{}", "========================================================================================".cyan());
            println!(
                " {} v1.0.0  |  {}",
                "CHERENKOV-LINGS".bold().bright_cyan(),
                "CI/CD Policy Validator".bright_yellow()
            );
            println!("{}", "========================================================================================".cyan());
            println!();
            println!(
                "{} File: {}",
                "▶".bright_blue(),
                workflow_path.display().to_string().bold()
            );
            println!("  SDET Policy Score: {}/100", validation.sdet_score);
            println!(
                "  Status: {}",
                if validation.valid {
                    "VALID".green().bold()
                } else {
                    "POLICY VIOLATIONS FOUND".red().bold()
                }
            );
            println!();
            for err in &validation.errors {
                println!(
                    "  {} [{}] {}",
                    "✗".red().bold(),
                    err.code.bright_red(),
                    err.message
                );
                if let Some(ref s) = err.suggestion {
                    println!("    {} {}", "💡 Fix:".yellow(), s.italic());
                }
            }
            for warn in &validation.warnings {
                println!(
                    "  {} [{}] {}",
                    "⚠".yellow(),
                    warn.code.bright_yellow(),
                    warn.message
                );
                if let Some(ref s) = warn.suggestion {
                    println!("    {} {}", "💡 Suggestion:".yellow(), s.italic());
                }
            }
            println!();
            if !validation.valid {
                std::process::exit(1);
            }
        }
        _ => {
            let opts = pipeline::PipelineRunOptions {
                parallel: true,
                fail_fast: false,
                animated: true,
                max_parallel: None,
                verbose,
                strict_validation: strict,
            };
            match pipeline::run_pipeline(&workflow_path, &opts) {
                Ok(result) => {
                    pipeline::render_pipeline_summary(&result);
                    if verbose {
                        println!("{}", "─── Detailed Runner Execution Logs ──────────────────────────────────────────────".dimmed());
                        for log in &result.logs {
                            let lvl_str = match log.level {
                                pipeline::LogLevel::Info => "INFO".bright_blue(),
                                pipeline::LogLevel::Warn => "WARN".yellow(),
                                pipeline::LogLevel::Error => "ERROR".red(),
                                pipeline::LogLevel::Debug => "DEBUG".dimmed(),
                            };
                            println!(
                                "  [{}] [{}] ({}) {}",
                                lvl_str,
                                log.runner.dimmed(),
                                log.step.dimmed(),
                                log.message
                            );
                        }
                        println!();
                    }
                    if !result.success {
                        std::process::exit(1);
                    }
                }
                Err(e) => {
                    eprintln!("{} Failed to execute pipeline: {}", "✗".red(), e);
                    std::process::exit(1);
                }
            }
        }
    }
}

fn run_triage_cmd(
    test_id: Option<&str>,
    category: Option<&str>,
    explanation: Option<&str>,
    fix: Option<&str>,
    report: bool,
    output_dir: &str,
    list: bool,
) -> std::io::Result<()> {
    if report {
        return run_report_cmd(output_dir);
    }

    if list {
        let failures = reports::get_failing_tests();
        println!();
        println!(
            "{}",
            "─── CHERENKOV-LINGS CHAOTIC TEST FAILURES FOR TRIAGE ───"
                .bold()
                .bright_cyan()
        );
        println!();
        triage::display_failure_summary_table(&failures);
        println!();
        println!("Run `cherenkov-lings triage --test-id <ID>` to investigate a specific failure.");
        println!();
        return Ok(());
    }

    triage::run_interactive_triage(test_id, category, explanation, fix)
}

fn run_report_cmd(output_dir: &str) -> std::io::Result<()> {
    let out_path = Path::new(output_dir);
    println!(
        "{}",
        "========================================================================================"
            .cyan()
    );
    println!(
        " {} v1.0.0  |  {}",
        "CHERENKOV-LINGS".bold().bright_cyan(),
        "Enterprise Allure Chaos Reporter".bright_yellow()
    );
    println!(
        "{}",
        "========================================================================================"
            .cyan()
    );
    println!();
    println!(
        "{} Generating Allure JSON test results and interactive HTML report...",
        "⏳".yellow()
    );

    match reports::generate_chaos_allure_report(out_path) {
        Ok(summary) => {
            println!(
                "{} Allure Report generated successfully!",
                "✓".bold().green()
            );
            println!(
                "   Total Tests:         {}",
                summary.total_tests.to_string().cyan()
            );
            println!(
                "   Pass Rate:           {:.1}% ({} passed, {} failed, {} broken, {} flaky)",
                summary.pass_percentage,
                summary.passed,
                summary.failed,
                summary.broken,
                summary.flaky
            );
            println!(
                "   Taxonomy Breakdown:  {} Product Bugs, {} Flaky Infra, {} Anti-Patterns",
                summary.real_bugs.to_string().bright_red(),
                summary.flaky_infra.to_string().bright_blue(),
                summary.anti_patterns.to_string().bright_magenta()
            );
            println!(
                "   Allure Results Dir:  {}",
                summary.results_dir.bright_yellow()
            );
            println!(
                "   Interactive HTML:    {}",
                summary.report_html_path.bright_white().bold()
            );
            println!();
            println!("Open the report in your browser:");
            println!("   {}", summary.report_html_path.cyan());
            println!();
        }
        Err(e) => {
            eprintln!("{} Failed to generate report: {}", "✗".red(), e);
            return Err(e);
        }
    }

    Ok(())
}
