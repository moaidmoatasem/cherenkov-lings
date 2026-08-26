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
#[derive(Debug, Clone, Deserialize)]
pub struct TrackConfig {
    pub id: String,
    pub name: String,
    pub runner: String,
    pub exercise_dir: String,
    pub extension: String,
    pub command: String,
    /// Directory that actually holds the drill sub-directories. Defaults to
    /// `exercise_dir`; the Maven-layout Java track overrides it because its
    /// drills live under `src/test/java/com/cherenkov`.
    #[serde(default)]
    pub drill_root: Option<String>,
    #[serde(default)]
    pub stack: String,
    #[serde(default)]
    pub tier: String,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub drills: Vec<DrillConfig>,
    /// Starter-code filename inside each drill directory. Defaults to
    /// `exercise{extension}`; the Java track overrides it because Maven
    /// requires the file name to match the public class (`Exercise.java`).
    #[serde(default)]
    pub exercise_file: Option<String>,
    /// Reference-solution filename inside each drill directory. Defaults to
    /// `solution{extension}`.
    #[serde(default)]
    pub solution_file: Option<String>,
}

impl TrackConfig {
    /// Directory holding this track's drill sub-directories.
    pub fn drill_root(&self) -> &str {
        self.drill_root.as_deref().unwrap_or(&self.exercise_dir)
    }

    /// Filesystem path of a drill within this track.
    pub fn drill_path(&self, drill_id: &str) -> String {
        format!("{}/{}", self.drill_root(), drill_id)
    }

    /// Starter-code filename for drills in this track.
    pub fn exercise_file(&self) -> String {
        self.exercise_file
            .clone()
            .unwrap_or_else(|| format!("exercise{}", self.extension))
    }

    /// Reference-solution filename for drills in this track.
    pub fn solution_file(&self) -> String {
        self.solution_file
            .clone()
            .unwrap_or_else(|| format!("solution{}", self.extension))
    }
}

#[allow(dead_code)]
#[derive(Debug, Clone, Deserialize)]
pub struct DrillConfig {
    pub id: String,
    pub name: String,
}

pub fn load_config<P: AsRef<Path>>(path: P) -> Result<Config, Box<dyn std::error::Error>> {
    let contents = fs::read_to_string(path)?;
    parse_config(&contents)
}

pub fn parse_config(contents: &str) -> Result<Config, Box<dyn std::error::Error>> {
    let config: Config = toml::from_str(contents)?;
    Ok(config)
}

/// The curriculum manifest baked into the binary at compile time.
///
/// `lings.toml` is the single source of truth for tracks and drills. Embedding
/// it keeps commands like `dashboard` and `audit` working when the binary is
/// invoked from outside the repository root, without duplicating the
/// curriculum in Rust source.
pub const EMBEDDED_MANIFEST: &str = include_str!("../lings.toml");
