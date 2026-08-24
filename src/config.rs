use serde::Deserialize;
use std::fs;
use std::path::Path;

// Sprint 0: All fields are scaffolded here and will be consumed fully
// by the feedback engine, runner pool, and TUI in Sprint 1 and beyond.

#[allow(dead_code)]
#[derive(Debug, Deserialize)]
pub struct Config {
    pub platform: PlatformConfig,
    pub evaluation: EvaluationConfig,
    pub ui: UiConfig,
    pub tracks: Vec<TrackConfig>,
}

#[allow(dead_code)]
#[derive(Debug, Deserialize)]
pub struct PlatformConfig {
    pub name: String,
    pub version: String,
    pub sandbox_port: u16,
    pub chaos_proxy_port: u16,
    pub telemetry: bool,
}

#[allow(dead_code)]
#[derive(Debug, Deserialize)]
pub struct EvaluationConfig {
    pub pass_threshold: f32,
    pub flakiness_iterations: u32,
    pub flakiness_timeout_ms: u32,
    pub chaos_latency_ms: u32,
    pub chaos_jitter_ms: u32,
}

#[allow(dead_code)]
#[derive(Debug, Deserialize)]
pub struct UiConfig {
    pub theme: String,
    pub show_hints_on_failure: bool,
    pub enable_audio_bell: bool,
    pub language: String,
}

#[allow(dead_code)]
#[derive(Debug, Deserialize)]
pub struct TrackConfig {
    pub id: String,
    pub name: String,
    pub runner: String,
    pub exercise_dir: String,
    pub extension: String,
    pub command: String,
}

pub fn load_config<P: AsRef<Path>>(path: P) -> Result<Config, Box<dyn std::error::Error>> {
    let contents = fs::read_to_string(path)?;
    let config: Config = toml::from_str(&contents)?;
    Ok(config)
}
