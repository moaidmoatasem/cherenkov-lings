mod config;
pub mod feedback;
pub mod gamification;
pub mod mcp;
pub mod proxy;
pub mod runner;
mod watcher;

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
    /// Diagnose why an exercise is failing (AST & anti-pattern root cause)
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
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    match &cli.command {
        Commands::Init { name } => {
            let project_name = name.clone().unwrap_or_else(|| "my-sdet-journey".to_string());

            // Create exercise directories
            let exercise_dirs = vec![
                "exercises/00_foundations",
                "exercises/01_web_playwright_ts",
                "exercises/02_api_restassured_java",
                "exercises/03_mobile_maestro",
                "exercises/04_perf_k6_js",
                "exercises/05_perf_jmeter",
                "exercises/06_tool_decisions",
            ];
            for dir in &exercise_dirs {
                let p = Path::new(dir);
                if !p.exists() {
                    std::fs::create_dir_all(p)?;
                }
            }

            // Rich Manual-QA-first welcome banner
            println!();
            println!("{}", "╔══════════════════════════════════════════════════════════════════╗".bright_cyan());
            println!("{}", "║      ⚡  CHERENKOV-LINGS  — Interactive QA Learning Platform      ║".bright_cyan());
            println!("{}", "╚══════════════════════════════════════════════════════════════════╝".bright_cyan());
            println!();
            println!("  {} Workspace {} ready.", "✓".green(), project_name.bright_yellow());
            println!();
            println!("{}", "  YOUR LEARNING PATH (start here, go in order):".bold().white());
            println!();
            println!("  {}  {}  {}",
                "STEP 1".bright_white().bold(),
                "Foundations — What IS an automated test?".bright_yellow(),
                "(no tools needed, just Python)".dimmed()
            );
            println!("         {}", "cherenkov-lings watch --track=foundations".bright_cyan());
            println!("         Open: {}", "exercises/00_foundations/01_what_is_a_test/exercise.py".dimmed());
            println!();
            println!("  {}  {}  {}",
                "STEP 2".bright_white().bold(),
                "UI Automation — Playwright TypeScript".bright_yellow(),
                "(needs Node.js)".dimmed()
            );
            println!("         {}", "cherenkov-lings watch --track=playwright-ts".bright_cyan());
            println!("         Open: {}", "exercises/01_web_playwright_ts/04_first_playwright_test/exercise.ts".dimmed());
            println!();
            println!("  {}  {}  {}",
                "STEP 3".bright_white().bold(),
                "API Automation — REST Assured Java".bright_yellow(),
                "(needs Java + Maven)".dimmed()
            );
            println!("         {}", "cherenkov-lings watch --track=restassured-java".bright_cyan());
            println!();
            println!("  {}  {}  {}",
                "STEP 4".bright_white().bold(),
                "Mobile Automation — Maestro YAML".bright_yellow(),
                "(needs Maestro CLI)".dimmed()
            );
            println!("         {}", "cherenkov-lings watch --track=maestro-mobile".bright_cyan());
            println!();
            println!("  {}  {}  {}",
                "STEP 5".bright_white().bold(),
                "Performance — k6 (modern) or JMeter (enterprise)".bright_yellow(),
                "(needs k6 or JMeter)".dimmed()
            );
            println!("         {}   or   {}",
                "cherenkov-lings watch --track=k6-js".bright_cyan(),
                "cherenkov-lings watch --track=jmeter".bright_cyan()
            );
            println!();
            println!("  {}  {}",
                "STEP 6".bright_white().bold(),
                "Which tool is right for which job?".bright_yellow()
            );
            println!("         {}", "cherenkov-lings watch --track=tool-decisions".bright_cyan());
            println!();
            println!("{}", "  ─────────────────────────────────────────────────────────────────".dimmed());
            println!();
            println!("  {} Start the Micro-Crucible sandbox FIRST:", "⚡".yellow());
            println!("    {}", ".\\crucible\\start.bat".bright_white());
            println!();
            println!("  {} When you are stuck on any drill:", "💡".yellow());
            println!("    Check {} in the same folder as your exercise.", "hints.md".bright_white());
            println!("    Or run: {}", "cherenkov-lings diagnose --file=<path/to/exercise>".bright_cyan());
            println!();
            println!("{}", "  🚀 Begin your journey:".bold().green());
            println!("     {}", "cherenkov-lings watch --track=foundations".bold().bright_cyan());
            println!();
        }

        Commands::Watch { track } => {
            println!("Starting watcher for track: {}", track.bright_cyan());

            // Load configuration
            let cfg = config::load_config("lings.toml")?;
            let track_cfg = cfg.tracks.iter().find(|t| t.id == *track);

            if let Some(track_config) = track_cfg {
                println!("{} Track loaded: {}", "✓".green(), track_config.name.bold());

                // Auto-spawn background Chaos Proxy if configured in lings.toml
                let _proxy_shutdown = if cfg.platform.chaos_proxy_port > 0 {
                    let proxy_listen: std::net::SocketAddr = format!("127.0.0.1:{}", cfg.platform.chaos_proxy_port)
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
                    println!("{} Created exercise directory: {:?}", "✓".green(), exercise_dir);
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
                    println!("{} REST Assured JVM Runner initialized (Maven: {}).", "✓".green(), runner.maven_cmd().bright_yellow());
                    Some(Arc::new(runner::AnyRunner::Jvm(Arc::new(runner))))
                } else if track_config.runner == "k6" {
                    println!("{} Initializing k6 Load Testing Runner...", "⚡".yellow());
                    let runner = runner::K6Runner::new();
                    println!("{} k6 Load Testing Runner initialized.", "✓".green());
                    Some(Arc::new(runner::AnyRunner::K6(Arc::new(runner))))
                } else if track_config.runner == "maestro" {
                    println!("{} Initializing Maestro Mobile Runner...", "⚡".yellow());
                    let runner = runner::MaestroRunner::new();
                    println!("{} Maestro Mobile Definition Runner initialized.", "✓".green());
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
                        if watcher::should_ignore_path(Path::new(&path)) || !path.ends_with(&track_ext) {
                            continue;
                        }

                        println!("\n{}", "========================================================================================".cyan());
                        println!(" {} v{}  |  Track: [{}]", "CHERENKOV-LINGS".bold().bright_cyan(), platform_version, track_name.bright_yellow());
                        println!(" File saved: {}", path.bright_white());
                        println!("{}", "========================================================================================".cyan());

                        if let Some(ref runner) = runner_for_task {
                            let chaos_header = format!("delay={}ms;jitter={}ms", chaos_latency, chaos_jitter);
                            let total_timeout = timeout_per_iter * (flakiness_iterations as u64);

                            println!("{} Running {} test suite ({} iterations with chaos: {})...", "⏳".yellow(), track_name, flakiness_iterations, chaos_header);

                            match runner.run_drill(&path, &chaos_header, flakiness_iterations, total_timeout).await {
                                Ok(response) => {
                                    // Perform static AST analysis of the modified exercise file
                                    let ast_report = feedback::analyze_file(&path).unwrap_or_else(|_| {
                                        feedback::AstReport {
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
                                    let tier = gamification::tier_for_track_or_drill(&task_track_id, &drill_id);

                                    if scorecard.passed {
                                        let mut state = gamification::load_progress(None::<&Path>).unwrap_or_default();
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
                                                response.total_duration_ms / (response.iterations as u64)
                                            } else {
                                                response.total_duration_ms
                                            },
                                            baseline_duration_ms: 1000,
                                            tier,
                                            timestamp: None,
                                        };

                                        let (xp_earned, newly_unlocked) = state.record_drill_run(&run_ctx);
                                        if let Err(e) = gamification::save_progress(&state, None::<&Path>) {
                                            eprintln!("{} Failed to save progress to {}: {}", "⚠️".yellow(), gamification::PROGRESS_FILE, e);
                                        }

                                        let gamification_footer = gamification::render_gamification_scorecard_with_tier(
                                            xp_earned,
                                            tier,
                                            &state,
                                            &newly_unlocked,
                                        );
                                        print!("{}", gamification_footer);
                                    }
                                }
                                Err(err) => {
                                    eprintln!("{} Runner communication error: {}", "✗".bold().red(), err);
                                }
                            }
                        } else {
                            println!(" Runner for track '{}' is not currently active.", track_name);
                        }
                    }
                });

                watcher::watch_exercises(exercise_dir, tx).await?;
            } else {
                eprintln!("Error: Track '{}' not found in lings.toml", track);
                eprintln!("Available tracks: playwright-ts, restassured-java, k6-js, maestro-mobile, genai-qa");
            }
        }
        Commands::Diagnose { file } => {
            let target = file.clone().unwrap_or_else(|| {
                let default_exercise = "exercises/01_web_playwright_ts/01_hydration_timing/exercise.ts";
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
            let track_name = if target.ends_with(".yaml") || target.ends_with(".yml") || target.contains("maestro") {
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
            println!(" {} v1.0.0  |  Programmable Chaos Proxy", "CHERENKOV-LINGS".bold().bright_cyan());
            println!(" Listening on:     {}", format!("http://{}", listen_addr).bright_yellow());
            println!(" Forwarding to:    {}", format!("http://{}", upstream_addr).bright_white());
            if *latency > 0 || *jitter > 0 {
                println!(" Default Latency:  {}ms (jitter: ±{}ms)", latency, jitter);
            }
            if *drop_rate > 0.0 {
                println!(" Default Drop Rate: {:.1}%", drop_rate * 100.0);
            }
            println!("{}", "========================================================================================".cyan());
            println!("{} Proxy running. Press Ctrl+C to terminate.", "⚡".yellow());

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
            let cfg = config::load_config("lings.toml").unwrap_or_else(|_| {
                config::Config {
                    platform: config::PlatformConfig {
                        name: "cherenkov-lings".to_string(),
                        version: "1.0.0".to_string(),
                        sandbox_port: 8080,
                        chaos_proxy_port: 8086,
                        telemetry: false,
                    },
                    evaluation: config::EvaluationConfig {
                        pass_threshold: 85.0,
                        flakiness_iterations: 5,
                        flakiness_timeout_ms: 5000,
                        chaos_latency_ms: 200,
                        chaos_jitter_ms: 75,
                    },
                    ui: config::UiConfig {
                        theme: "cherenkov-blue".to_string(),
                        show_hints_on_failure: true,
                        enable_audio_bell: false,
                        language: "en".to_string(),
                    },
                    tracks: gamification::default_curriculum_tracks(),
                }
            });

            let state = gamification::load_progress(None::<&Path>).unwrap_or_default();
            let dashboard_output = gamification::render_dashboard(&state, &cfg);
            print!("{}", dashboard_output);
        }
    }

    Ok(())
}
