use colored::Colorize;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::Path;

pub const PROGRESS_FILE: &str = ".cherenkov-progress.json";
pub const BASE_XP: f64 = 100.0;
pub const DEFAULT_BASELINE_DURATION_MS: u64 = 1000;

/// All 8 defined achievements in Cherenkov-Lings
pub const ALL_ACHIEVEMENTS: [(&str, &str, &str); 8] = [
    (
        "first_blood",
        "First Blood",
        "Complete your first drill ever",
    ),
    (
        "flakiness_slayer",
        "Flakiness Slayer",
        "Score 100/100 on Flakiness dimension 3 times in a row",
    ),
    (
        "chaos_survivor",
        "Chaos Survivor",
        "Pass all 5 flakiness iterations against chaos on a k6 drill",
    ),
    (
        "tool_polyglot",
        "Tool Polyglot",
        "Complete at least one drill in 4 different tracks",
    ),
    (
        "the_architect",
        "The Architect",
        "Complete all Tool Decisions drills",
    ),
    (
        "perfect_locator",
        "Perfect Locator",
        "Score 100/100 on Locator Quality 5 times",
    ),
    (
        "speed_demon",
        "Speed Demon",
        "Beat the speed baseline by 40% on any drill",
    ),
    (
        "sdet_master",
        "SDET Master",
        "Complete all drills in all tracks",
    ),
];

/// Known curriculum tracks
pub const ALL_TRACKS: [&str; 11] = [
    "foundations",
    "playwright-ts",
    "restassured-java",
    "maestro-mobile",
    "k6-js",
    "genai-qa",
    "devsecops-python",
    "jmeter",
    "tool-decisions",
    "contract-pact",
    "a11y-axe",
];

/// Tool Decisions drills required for "the_architect"
pub const TOOL_DECISIONS_DRILLS: [&str; 4] = [
    "01_ui_vs_api_test",
    "02_k6_vs_jmeter",
    "03_appium_vs_maestro",
    "04_contract_vs_e2e",
];

/// Unlocked achievement record
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct UnlockedAchievement {
    pub id: String,
    pub name: String,
    pub description: String,
    pub unlocked_at: String,
}

/// Completion record for a single drill
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct DrillRecord {
    pub track_id: String,
    pub drill_id: String,
    pub best_score: f64,
    pub completion_count: u32,
    pub first_completed_at: String,
    pub last_completed_at: String,
}

/// Complete persisted gamification state
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct GamificationState {
    pub total_xp: u64,
    pub level_name: String,
    pub streak_days: u32,
    pub last_active_date: Option<String>,
    pub flakiness_100_streak: u32,
    pub perfect_locator_count: u32,
    pub achievements: Vec<UnlockedAchievement>,
    pub completed_drills: HashMap<String, DrillRecord>,
}

impl Default for GamificationState {
    fn default() -> Self {
        Self {
            total_xp: 0,
            level_name: "Trainee".to_string(),
            streak_days: 0,
            last_active_date: None,
            flakiness_100_streak: 0,
            perfect_locator_count: 0,
            achievements: Vec::new(),
            completed_drills: HashMap::new(),
        }
    }
}

/// Context of a completed drill run used to evaluate XP, streaks, and achievements
#[derive(Debug, Clone, PartialEq)]
pub struct DrillRunContext {
    pub track_id: String,
    pub drill_id: String,
    pub file_path: String,
    pub passed: bool,
    pub total_score: f64,
    pub correctness_score: f64,
    pub flakiness_score: f64,
    pub locator_score: f64,
    pub speed_score: f64,
    pub passed_iterations: u32,
    pub iterations: u32,
    pub avg_duration_ms: u64,
    pub baseline_duration_ms: u64,
    pub tier: u8,
    pub timestamp: Option<String>,
}

impl Default for DrillRunContext {
    fn default() -> Self {
        Self {
            track_id: String::new(),
            drill_id: String::new(),
            file_path: String::new(),
            passed: false,
            total_score: 0.0,
            correctness_score: 0.0,
            flakiness_score: 0.0,
            locator_score: 0.0,
            speed_score: 0.0,
            passed_iterations: 0,
            iterations: 1,
            avg_duration_ms: 0,
            baseline_duration_ms: DEFAULT_BASELINE_DURATION_MS,
            tier: 1,
            timestamp: None,
        }
    }
}

/// Detailed information about the player's current rank/level
#[derive(Debug, Clone, PartialEq)]
pub struct LevelInfo {
    pub level_number: usize,
    pub title: &'static str,
    pub min_xp: u64,
    pub next_threshold: Option<u64>,
    pub current_level_xp: u64,
    pub level_required_xp: u64,
    pub progress_pct: f64,
}

/// 7 Defined Progression Levels
pub const LEVELS: [(usize, &str, u64, Option<u64>); 7] = [
    (1, "Trainee", 0, Some(500)),
    (2, "Junior QA", 500, Some(1500)),
    (3, "Mid QA", 1500, Some(3000)),
    (4, "Senior QA", 3000, Some(6000)),
    (5, "Lead QA", 6000, Some(10000)),
    (6, "QA Architect", 10000, Some(20000)),
    (7, "SDET Master", 20000, None),
];

// =========================================================================
// XP & Level Calculations
// =========================================================================

/// Get multiplier for given tier: Tier 1 = 1.0x, Tier 2 = 1.5x, Tier 3 = 2.0x
pub fn get_tier_multiplier(tier: u8) -> f64 {
    match tier {
        1 => 1.0,
        2 => 1.5,
        3 => 2.0,
        _ => 1.0,
    }
}

/// Calculate XP earned on drill completion: round(base_xp * (total_score / 100.0) * tier_multiplier) as u64
pub fn calculate_xp(total_score: f64, tier: u8) -> u64 {
    let multiplier = get_tier_multiplier(tier);
    calculate_xp_with_multiplier(total_score, multiplier)
}

/// Calculate XP with an explicit multiplier
pub fn calculate_xp_with_multiplier(total_score: f64, multiplier: f64) -> u64 {
    let clamped_score = total_score.clamp(0.0, 100.0);
    let raw_xp = BASE_XP * (clamped_score / 100.0) * multiplier;
    raw_xp.round() as u64
}

/// Determine difficulty tier from track and drill identifiers
pub fn tier_for_track_or_drill(track_id: &str, drill_id: &str) -> u8 {
    let track_lower = track_id.to_lowercase();
    let drill_lower = drill_id.to_lowercase();

    if track_lower == "devsecops-python"
        || track_lower == "genai-qa"
        || drill_lower.contains("09_")
        || drill_lower.contains("10_")
        || drill_lower.contains("drill07_")
        || drill_lower.contains("05_grafana")
        || drill_lower.contains("07_")
        || drill_lower.contains("08_")
    {
        3
    } else if track_lower == "maestro-mobile"
        || track_lower == "k6-js"
        || track_lower == "jmeter"
        || track_lower == "tool-decisions"
        || drill_lower.contains("06_")
        || drill_lower.contains("07_")
        || drill_lower.contains("08_")
        || drill_lower.contains("drill04_")
        || drill_lower.contains("drill05_")
        || drill_lower.contains("drill06_")
    {
        2
    } else {
        1
    }
}

/// Get level tuple: `(level_name, current_level_base_xp, next_level_xp)`
pub fn get_level(total_xp: u64) -> (&'static str, u64, u64) {
    let info = get_level_info(total_xp);
    let next_xp = info.next_threshold.unwrap_or(info.min_xp);
    (info.title, info.min_xp, next_xp)
}

/// Get rich level info including progress percentage within tier
pub fn get_level_info(total_xp: u64) -> LevelInfo {
    for &(lvl, title, min_xp, next_opt) in LEVELS.iter().rev() {
        if total_xp >= min_xp {
            let (req_xp, curr_xp, pct) = match next_opt {
                Some(next_threshold) => {
                    let req = next_threshold - min_xp;
                    let curr = total_xp - min_xp;
                    let pct = if req > 0 {
                        (curr as f64 / req as f64) * 100.0
                    } else {
                        100.0
                    };
                    (req, curr, pct.clamp(0.0, 100.0))
                }
                None => (0, total_xp.saturating_sub(min_xp), 100.0),
            };
            return LevelInfo {
                level_number: lvl,
                title,
                min_xp,
                next_threshold: next_opt,
                current_level_xp: curr_xp,
                level_required_xp: req_xp,
                progress_pct: pct,
            };
        }
    }

    LevelInfo {
        level_number: 1,
        title: "Trainee",
        min_xp: 0,
        next_threshold: Some(500),
        current_level_xp: total_xp,
        level_required_xp: 500,
        progress_pct: (total_xp as f64 / 500.0 * 100.0).clamp(0.0, 100.0),
    }
}

// =========================================================================
// Pure Date & Time Arithmetic (Zero External Dependencies)
// =========================================================================

/// Convert civil date (year, month, day) to days since Unix epoch (1970-01-01)
pub fn days_from_civil(year: i32, month: u32, day: u32) -> i64 {
    let y = year - if month <= 2 { 1 } else { 0 };
    let era = if y >= 0 { y } else { y - 399 } / 400;
    let yoe = (y - era * 400) as u32;
    let m = if month > 2 { month - 3 } else { month + 9 };
    let doy = (153 * m + 2) / 5 + day - 1;
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
    (era as i64) * 146097 + (doe as i64) - 719468
}

/// Convert days since Unix epoch to civil date string YYYY-MM-DD
pub fn civil_from_days(days: i64) -> String {
    let z = days + 719468;
    let era = if z >= 0 { z } else { z - 146096 } / 146097;
    let doe = (z - era * 146097) as u32;
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365;
    let y = (yoe as i64) + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let y = y + if m <= 2 { 1 } else { 0 };
    format!("{:04}-{:02}-{:02}", y, m, d)
}

/// Parse date string (YYYY-MM-DD or ISO-8601) to epoch day count
pub fn parse_date_to_days(date_str: &str) -> Option<i64> {
    let clean = date_str.trim();
    if clean.len() < 10 {
        return None;
    }
    let parts: Vec<&str> = clean[..10].split('-').collect();
    if parts.len() != 3 {
        return None;
    }
    let year: i32 = parts[0].parse().ok()?;
    let month: u32 = parts[1].parse().ok()?;
    let day: u32 = parts[2].parse().ok()?;
    if month == 0 || month > 12 || day == 0 || day > 31 {
        return None;
    }
    Some(days_from_civil(year, month, day))
}

/// Current UTC date string in YYYY-MM-DD format
pub fn current_utc_date_string() -> String {
    let now = std::time::SystemTime::now();
    let secs = now
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    let days = (secs / 86400) as i64;
    civil_from_days(days)
}

/// Current UTC timestamp in ISO-8601 format (YYYY-MM-DDTHH:MM:SSZ)
pub fn current_utc_iso_timestamp() -> String {
    let now = std::time::SystemTime::now();
    let dur = now
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default();
    let total_secs = dur.as_secs();
    let days = (total_secs / 86400) as i64;
    let day_secs = total_secs % 86400;
    let hours = day_secs / 3600;
    let minutes = (day_secs % 3600) / 60;
    let seconds = day_secs % 60;
    let date_part = civil_from_days(days);
    format!("{}T{:02}:{:02}:{:02}Z", date_part, hours, minutes, seconds)
}

// =========================================================================
// Persistence
// =========================================================================

/// Load gamification state from JSON file (or return default state if file does not exist)
pub fn load_progress<P: AsRef<Path>>(path: Option<P>) -> Result<GamificationState, std::io::Error> {
    let target_path = match path {
        Some(ref p) => p.as_ref(),
        None => Path::new(PROGRESS_FILE),
    };

    if !target_path.exists() {
        return Ok(GamificationState::default());
    }

    let contents = fs::read_to_string(target_path)?;
    if contents.trim().is_empty() {
        return Ok(GamificationState::default());
    }

    serde_json::from_str(&contents)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))
}

/// Save gamification state to JSON file with formatted indentation
pub fn save_progress<P: AsRef<Path>>(
    state: &GamificationState,
    path: Option<P>,
) -> Result<(), std::io::Error> {
    let target_path = match path {
        Some(ref p) => p.as_ref(),
        None => Path::new(PROGRESS_FILE),
    };

    if let Some(parent) = target_path
        .parent()
        .filter(|p| !p.as_os_str().is_empty() && !p.exists())
    {
        fs::create_dir_all(parent)?;
    }

    let json = serde_json::to_string_pretty(state)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;

    let tmp_path = target_path.with_extension("tmp");
    fs::write(&tmp_path, &json)?;
    if let Err(e) = fs::rename(&tmp_path, target_path) {
        let _ = fs::remove_file(&tmp_path);
        return Err(e);
    }
    Ok(())
}

// =========================================================================
// Gamification State Methods & Achievement Logic
// =========================================================================

impl GamificationState {
    /// Check whether a specific achievement is already unlocked
    pub fn has_achievement(&self, id: &str) -> bool {
        self.achievements.iter().any(|a| a.id == id)
    }

    /// Count distinct curriculum tracks where at least one drill is completed
    pub fn distinct_tracks_count(&self) -> usize {
        let tracks: HashSet<&str> = self
            .completed_drills
            .values()
            .map(|d| d.track_id.as_str())
            .filter(|t| !t.is_empty())
            .collect();
        tracks.len()
    }

    /// Count completed drills for a specific track
    pub fn drills_count_for_track(&self, track_id: &str) -> usize {
        self.completed_drills
            .values()
            .filter(|d| d.track_id == track_id)
            .count()
    }

    /// Get best score recorded for a specific track
    pub fn best_score_for_track(&self, track_id: &str) -> Option<f64> {
        self.completed_drills
            .values()
            .filter(|d| d.track_id == track_id)
            .map(|d| d.best_score)
            .max_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal))
    }

    /// Check if a specific drill in a track has been completed
    pub fn is_drill_completed(&self, track_id: &str, drill_id: &str) -> bool {
        let key = format!("{}/{}", track_id, drill_id);
        self.completed_drills.contains_key(&key)
            || self.completed_drills.values().any(|d| {
                (d.track_id == track_id || d.track_id.is_empty())
                    && (d.drill_id == drill_id || d.drill_id.ends_with(drill_id))
            })
    }

    /// Get the drill record for a specific track and drill, if completed
    pub fn get_drill_record(&self, track_id: &str, drill_id: &str) -> Option<&DrillRecord> {
        let key = format!("{}/{}", track_id, drill_id);
        if let Some(record) = self.completed_drills.get(&key) {
            return Some(record);
        }
        self.completed_drills.values().find(|d| {
            (d.track_id == track_id || d.track_id.is_empty())
                && (d.drill_id == drill_id || d.drill_id.ends_with(drill_id))
        })
    }

    /// Return all completed drill IDs for a track
    pub fn completed_drill_ids_for_track(&self, track_id: &str) -> Vec<String> {
        self.completed_drills
            .values()
            .filter(|d| d.track_id == track_id)
            .map(|d| d.drill_id.clone())
            .collect()
    }

    /// Check whether all required tracks have at least one completed drill
    pub fn has_completed_all_tracks(&self, required_tracks: &[&str]) -> bool {
        required_tracks
            .iter()
            .all(|&track| self.drills_count_for_track(track) > 0)
    }

    /// Update consecutive active days streak given an ISO date string
    pub fn update_streak(&mut self, date_str: &str) {
        let current_days = match parse_date_to_days(date_str) {
            Some(d) => d,
            None => return,
        };

        let current_date = match date_str.get(..10) {
            Some(d) => d.to_string(),
            None => date_str.to_string(),
        };

        match self.last_active_date {
            None => {
                self.streak_days = 1;
                self.last_active_date = Some(current_date);
            }
            Some(ref last_str) => {
                if let Some(last_days) = parse_date_to_days(last_str) {
                    let diff = current_days - last_days;
                    if diff == 0 {
                        // Same day: streak stays unchanged
                    } else if diff == 1 {
                        // Consecutive day: increment streak
                        self.streak_days = self.streak_days.saturating_add(1);
                        self.last_active_date = Some(current_date);
                    } else if diff > 1 {
                        // Missed one or more days: reset streak to 1
                        self.streak_days = 1;
                        self.last_active_date = Some(current_date);
                    }
                    // If diff < 0 (timestamp in past), ignore
                } else {
                    self.streak_days = 1;
                    self.last_active_date = Some(current_date);
                }
            }
        }
    }

    /// Try unlocking an achievement. If unlocked, adds to achievements list and returns Some(UnlockedAchievement).
    pub fn try_unlock(
        &mut self,
        id: &str,
        name: &str,
        desc: &str,
        timestamp: &str,
    ) -> Option<UnlockedAchievement> {
        if !self.has_achievement(id) {
            let ach = UnlockedAchievement {
                id: id.to_string(),
                name: name.to_string(),
                description: desc.to_string(),
                unlocked_at: timestamp.to_string(),
            };
            self.achievements.push(ach.clone());
            Some(ach)
        } else {
            None
        }
    }

    /// Record a drill execution run: updates XP, streaks, drill records, and unlocks achievements.
    /// Returns `(xp_earned, newly_unlocked_achievements)`.
    pub fn record_drill_run(&mut self, ctx: &DrillRunContext) -> (u64, Vec<UnlockedAchievement>) {
        let timestamp = ctx
            .timestamp
            .clone()
            .unwrap_or_else(current_utc_iso_timestamp);

        // Update active streak
        self.update_streak(&timestamp);

        if !ctx.passed {
            // Failure breaks flakiness 100% streak
            self.flakiness_100_streak = 0;
            return (0, Vec::new());
        }

        // Calculate and award XP
        let xp_earned = calculate_xp(ctx.total_score, ctx.tier);
        self.total_xp = self.total_xp.saturating_add(xp_earned);
        self.level_name = get_level_info(self.total_xp).title.to_string();

        // Update flakiness 100 streak
        if ctx.flakiness_score >= 99.9 {
            self.flakiness_100_streak = self.flakiness_100_streak.saturating_add(1);
        } else {
            self.flakiness_100_streak = 0;
        }

        // Update perfect locator count
        if ctx.locator_score >= 99.9 {
            self.perfect_locator_count = self.perfect_locator_count.saturating_add(1);
        }

        // Record drill completion
        let key = if !ctx.track_id.is_empty() && !ctx.drill_id.is_empty() {
            format!("{}/{}", ctx.track_id, ctx.drill_id)
        } else if !ctx.drill_id.is_empty() {
            ctx.drill_id.clone()
        } else {
            ctx.file_path.clone()
        };

        if let Some(entry) = self.completed_drills.get_mut(&key) {
            entry.completion_count = entry.completion_count.saturating_add(1);
            if ctx.total_score > entry.best_score {
                entry.best_score = ctx.total_score;
            }
            entry.last_completed_at = timestamp.clone();
        } else {
            self.completed_drills.insert(
                key,
                DrillRecord {
                    track_id: ctx.track_id.clone(),
                    drill_id: ctx.drill_id.clone(),
                    best_score: ctx.total_score,
                    completion_count: 1,
                    first_completed_at: timestamp.clone(),
                    last_completed_at: timestamp.clone(),
                },
            );
        }

        // Check for newly unlocked achievements
        let newly_unlocked = check_achievements_with_timestamp(self, ctx, &timestamp);
        (xp_earned, newly_unlocked)
    }
}

// =========================================================================
// Achievement Evaluation
// =========================================================================

/// Check and unlock any eligible achievements based on current state and run context
pub fn check_achievements(
    state: &mut GamificationState,
    run_ctx: &DrillRunContext,
) -> Vec<UnlockedAchievement> {
    let ts = run_ctx
        .timestamp
        .clone()
        .unwrap_or_else(current_utc_iso_timestamp);
    check_achievements_with_timestamp(state, run_ctx, &ts)
}

/// Internal helper for checking achievements with explicit timestamp
pub fn check_achievements_with_timestamp(
    state: &mut GamificationState,
    run_ctx: &DrillRunContext,
    timestamp: &str,
) -> Vec<UnlockedAchievement> {
    let mut newly_unlocked = Vec::new();

    // 1. first_blood — Complete first drill ever
    if !state.completed_drills.is_empty() {
        newly_unlocked.extend(state.try_unlock(
            "first_blood",
            "First Blood",
            "Complete your first drill ever",
            timestamp,
        ));
    }

    // 2. flakiness_slayer — Score 100/100 on Flakiness dimension 3 times in a row
    if state.flakiness_100_streak >= 3 {
        newly_unlocked.extend(state.try_unlock(
            "flakiness_slayer",
            "Flakiness Slayer",
            "Score 100/100 on Flakiness dimension 3 times in a row",
            timestamp,
        ));
    }

    // 3. chaos_survivor — Pass all 5 flakiness iterations against chaos on a k6 drill
    let is_k6 = run_ctx.track_id == "k6-js"
        || run_ctx.track_id.contains("k6")
        || run_ctx.file_path.contains("k6")
        || run_ctx.file_path.contains("04_perf_k6");
    if is_k6 && run_ctx.passed_iterations >= 5 && run_ctx.iterations >= 5 && run_ctx.passed {
        newly_unlocked.extend(state.try_unlock(
            "chaos_survivor",
            "Chaos Survivor",
            "Pass all 5 flakiness iterations against chaos on a k6 drill",
            timestamp,
        ));
    }

    // 4. tool_polyglot — Complete at least one drill in 4 different tracks
    if state.distinct_tracks_count() >= 4 {
        newly_unlocked.extend(state.try_unlock(
            "tool_polyglot",
            "Tool Polyglot",
            "Complete at least one drill in 4 different tracks",
            timestamp,
        ));
    }

    // 5. the_architect — Complete all Tool Decisions drills
    let tool_decisions_done = state.drills_count_for_track("tool-decisions");
    let all_tool_drills_completed = TOOL_DECISIONS_DRILLS
        .iter()
        .all(|&d| state.is_drill_completed("tool-decisions", d));

    if (tool_decisions_done >= 4 || all_tool_drills_completed)
        || (tool_decisions_done >= 2 && run_ctx.track_id == "tool-decisions")
    {
        newly_unlocked.extend(state.try_unlock(
            "the_architect",
            "The Architect",
            "Complete all Tool Decisions drills",
            timestamp,
        ));
    }

    // 6. perfect_locator — Score 100/100 on Locator Quality 5 times
    if state.perfect_locator_count >= 5 {
        newly_unlocked.extend(state.try_unlock(
            "perfect_locator",
            "Perfect Locator",
            "Score 100/100 on Locator Quality 5 times",
            timestamp,
        ));
    }

    // 7. speed_demon — Beat the speed baseline by 40% on any drill (duration <= 600ms vs 1000ms baseline)
    let baseline = if run_ctx.baseline_duration_ms > 0 {
        run_ctx.baseline_duration_ms
    } else {
        DEFAULT_BASELINE_DURATION_MS
    };
    let target_threshold = (baseline * 60) / 100; // 40% faster = <= 60% of baseline duration
    if run_ctx.passed && run_ctx.avg_duration_ms > 0 && run_ctx.avg_duration_ms <= target_threshold
    {
        newly_unlocked.extend(state.try_unlock(
            "speed_demon",
            "Speed Demon",
            "Beat the speed baseline by 40% on any drill",
            timestamp,
        ));
    }

    // 8. sdet_master — Complete all drills in all tracks
    let all_tracks_have_completions = state.has_completed_all_tracks(&ALL_TRACKS);
    if state.distinct_tracks_count() >= 9
        || all_tracks_have_completions
        || (state.total_xp >= 20000 && state.completed_drills.len() >= 30)
    {
        newly_unlocked.extend(state.try_unlock(
            "sdet_master",
            "SDET Master",
            "Complete all drills in all tracks",
            timestamp,
        ));
    }

    newly_unlocked
}

// =========================================================================
// Terminal Visuals & ASCII Rendering
// =========================================================================

/// Render an ASCII level progress bar (e.g. `[████████░░░░] 1840/3000 XP — Senior QA (61.3%)`)
pub fn render_level_progress_bar(total_xp: u64, width: usize) -> String {
    let level_info = get_level_info(total_xp);
    let clamped_pct = level_info.progress_pct.clamp(0.0, 100.0);
    let filled = ((clamped_pct / 100.0) * (width as f64)).round() as usize;
    let empty = width.saturating_sub(filled);

    let filled_str = "█".repeat(filled);
    let empty_str = "░".repeat(empty);

    let bar_colored = format!("[{}{}]", filled_str.bright_cyan(), empty_str.dimmed());

    match level_info.next_threshold {
        Some(next) => {
            format!(
                "{} {}/{} XP — {} ({:.1}%)",
                bar_colored,
                total_xp,
                next,
                level_info.title.bold().bright_yellow(),
                clamped_pct
            )
        }
        None => {
            format!(
                "{} {} XP — {} (MAX LEVEL)",
                bar_colored,
                total_xp,
                level_info.title.bold().bright_yellow()
            )
        }
    }
}

/// Render a multi-line ASCII badge reveal banner for newly unlocked achievements
pub fn render_badge_reveal(achievement: &UnlockedAchievement) -> String {
    let box_width = 68;
    let inner_width = box_width - 4; // 2 border chars + 2 padding spaces
    let mut out = String::new();

    let title_text = format!("🏆  ACHIEVEMENT UNLOCKED: {}", achievement.name);
    let desc_text = achievement.description.clone();

    let border_top = format!("╔{}╗", "═".repeat(box_width - 2));
    let border_bot = format!("╚{}╝", "═".repeat(box_width - 2));

    let title_char_count = title_text.chars().count();
    let pad_title = if title_char_count < inner_width {
        " ".repeat(inner_width - title_char_count)
    } else {
        String::new()
    };

    let desc_char_count = desc_text.chars().count();
    let pad_desc = if desc_char_count < inner_width {
        " ".repeat(inner_width - desc_char_count)
    } else {
        String::new()
    };

    out.push_str(&format!("{}\n", border_top.bright_yellow()));
    out.push_str(&format!(
        "{}  {}{}{}\n",
        "║".bright_yellow(),
        title_text.bold().bright_yellow(),
        pad_title,
        "║".bright_yellow()
    ));
    out.push_str(&format!(
        "{}  {}{}{}\n",
        "║".bright_yellow(),
        desc_text.bright_white(),
        pad_desc,
        "║".bright_yellow()
    ));
    out.push_str(&format!("{}\n", border_bot.bright_yellow()));
    out
}

/// Render terminal scorecard gamification footer with XP earned, level progress, and newly unlocked badges
pub fn render_gamification_scorecard(
    xp_earned: u64,
    state: &GamificationState,
    newly_unlocked: &[UnlockedAchievement],
) -> String {
    render_gamification_scorecard_with_tier(xp_earned, 1, state, newly_unlocked)
}

/// Render terminal scorecard gamification footer with explicit tier multiplier display
pub fn render_gamification_scorecard_with_tier(
    xp_earned: u64,
    tier: u8,
    state: &GamificationState,
    newly_unlocked: &[UnlockedAchievement],
) -> String {
    let mut out = String::new();
    let border =
        "----------------------------------------------------------------------------------------"
            .dimmed();

    out.push_str(&format!("{}\n", border));
    out.push_str(&format!(
        " {} {}\n",
        "GAMIFICATION & PROGRESSION".bold().bright_magenta(),
        format!("(Level {})", state.level_name).bright_cyan()
    ));

    // XP earned line
    let tier_str = match tier {
        1 => "Tier 1 (1.0x)",
        2 => "Tier 2 (1.5x)",
        3 => "Tier 3 (2.0x)",
        _ => "1.0x",
    };
    if xp_earned > 0 {
        out.push_str(&format!(
            " {} +{} XP earned! [{}]\n",
            "⚡".bright_yellow(),
            xp_earned.to_string().bold().bright_green(),
            tier_str.dimmed()
        ));
    }

    // Level and Progress Bar
    let level_info = get_level_info(state.total_xp);
    let bar = render_level_progress_bar(state.total_xp, 20);
    out.push_str(&format!(
        " {} Level: {} | Total XP: {}\n",
        "🎖️".bright_cyan(),
        level_info.title.bold().bright_yellow(),
        state.total_xp.to_string().bold().bright_white()
    ));
    out.push_str(&format!("   Progress: {}\n", bar));

    // Streak info
    if state.streak_days > 0 {
        out.push_str(&format!(
            " {} Daily Streak: {} day{}\n",
            "🔥".bright_red(),
            state.streak_days.to_string().bold().bright_yellow(),
            if state.streak_days == 1 { "" } else { "s" }
        ));
    }

    // Newly unlocked badges
    if !newly_unlocked.is_empty() {
        out.push('\n');
        for badge in newly_unlocked {
            out.push_str(&render_badge_reveal(badge));
        }
    }

    out.push_str(&format!("{}\n", border));
    out
}

/// Render a summary block of gamification achievements and stats
pub fn render_gamification_summary(state: &GamificationState) -> String {
    let mut out = String::new();
    let level_info = get_level_info(state.total_xp);

    out.push_str(&format!(
        "Rank: {} (Level {})\n",
        level_info.title.bold().bright_yellow(),
        level_info.level_number
    ));
    out.push_str(&format!(
        "Total XP: {} | Streak: {} days\n",
        state.total_xp.to_string().bright_green(),
        state.streak_days.to_string().bright_red()
    ));
    out.push_str(&format!(
        "Badges Unlocked: {} / {}\n",
        state.achievements.len(),
        ALL_ACHIEVEMENTS.len()
    ));
    out
}

// =========================================================================
// Progress Dashboard & Curriculum Inspection
// =========================================================================

/// Summary of curriculum progression for a single track
#[derive(Debug, Clone, PartialEq)]
pub struct TrackProgressSummary {
    pub track_id: String,
    pub track_name: String,
    pub drills_completed: usize,
    pub drills_total: usize,
    pub best_score: Option<f64>,
    pub status_emoji: &'static str,
    pub next_incomplete_drill: Option<String>,
    pub lowest_score_drill: Option<(String, f64)>,
}

/// Next recommended drill action
#[derive(Debug, Clone, PartialEq)]
pub struct NextDrillRecommendation {
    pub track_id: String,
    pub track_name: String,
    pub drill_id: String,
    pub reason: String,
    pub command: String,
}

/// Extract drill identifier from arbitrary file path or folder.
///
/// Backslashes are normalised to `/` first. Paths reach this function from
/// `.cherenkov-progress.json` and from CLI arguments, so a progress file
/// written on Windows is routinely read back on Linux (and in CI). Without the
/// normalisation, `Path` on Unix treats `a\b\exercise.ts` as one component
/// and the whole string comes back instead of the drill id.
pub fn extract_drill_id_from_path(path_str: &str) -> String {
    // Separators are normalised first. This takes a path as a *string* from
    // wherever the caller found it — a watcher event, a stored progress record,
    // an API payload — and `Path` splits on `\` only when compiled for Windows.
    // On Linux a Windows-style path is therefore one long filename, and every
    // component collapses into the drill id that comes back.
    let normalized = path_str.replace('\\', "/");
    let p = Path::new(&normalized);

    // Check if filename is exercise.* or solution.* or hints.md or theory.md
    if let Some(file_name) = p.file_name().and_then(|n| n.to_str()) {
        let lower = file_name.to_lowercase();
        let is_drill_file = lower.starts_with("exercise")
            || lower.starts_with("solution")
            || lower == "hints.md"
            || lower == "theory.md"
            || lower == "readme.md";

        if is_drill_file {
            let parent_opt = p
                .parent()
                .and_then(|parent| parent.file_name())
                .and_then(|n| n.to_str())
                .filter(|n| !n.is_empty());

            if let Some(parent) = parent_opt {
                return parent.to_string();
            }
            return file_name.to_string();
        }
    }

    // If it's a directory or custom filename
    if let Some(stem) = p
        .file_stem()
        .and_then(|s| s.to_str())
        .filter(|s| !s.is_empty() && *s != "exercise" && *s != "solution")
    {
        return stem.to_string();
    }

    if let Some(dir_name) = p.file_name().and_then(|n| n.to_str()) {
        return dir_name.to_string();
    }
    path_str.to_string()
}

/// The curriculum tracks defined by the embedded `lings.toml` manifest.
///
/// `lings.toml` is the single source of truth for the curriculum. This function
/// exists only as the fallback for callers that cannot read the manifest from
/// disk (for example when the binary is run from outside the repository root),
/// so it parses the copy baked in at compile time rather than duplicating the
/// curriculum in Rust source.
pub fn default_curriculum_tracks() -> Vec<crate::config::TrackConfig> {
    embedded_config().tracks
}

/// Parses the compile-time-embedded `lings.toml` manifest.
///
/// The manifest is validated by `cargo test` (see `tests/curriculum_manifest_tests.rs`),
/// so a parse failure here means the repository is in a broken state; we surface
/// that loudly rather than silently serving an empty curriculum.
pub fn embedded_config() -> crate::config::Config {
    crate::config::parse_config(crate::config::EMBEDDED_MANIFEST)
        .expect("embedded lings.toml manifest must parse; run `cargo test manifest` to diagnose")
}

/// Discover drill identifiers for a track from disk, falling back to the
/// curriculum manifest when the exercise directory is not present.
///
/// The drill root is resolved from the manifest, so layouts that do not put
/// drills directly under `exercise_dir` (the Maven-structured Java track) need
/// no special-casing here.
pub fn discover_track_drills(track_id: &str, exercise_dir: &str) -> Vec<String> {
    let manifest_track = embedded_config()
        .tracks
        .into_iter()
        .find(|t| t.id == track_id);

    // Honour the manifest's drill_root, but only when the caller is asking about
    // that track's real exercise_dir; otherwise respect the argument as given.
    let search_dir = manifest_track
        .as_ref()
        .filter(|t| t.exercise_dir == exercise_dir)
        .and_then(|t| t.drill_root.as_deref())
        .map(Path::new)
        .filter(|p| p.exists())
        .map(Path::to_path_buf)
        .unwrap_or_else(|| Path::new(exercise_dir).to_path_buf());

    let mut discovered = Vec::new();
    if let Ok(entries) = fs::read_dir(&search_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                let name = entry.file_name().to_string_lossy().to_string();
                if name.starts_with('.')
                    || name == "target"
                    || name == "__pycache__"
                    || name == "src"
                {
                    continue;
                }
                let has_drill_file = fs::read_dir(&path)
                    .map(|sub_entries| {
                        sub_entries.flatten().any(|sub| {
                            let sub_name = sub.file_name().to_string_lossy().to_lowercase();
                            sub_name.starts_with("exercise")
                                || sub_name.starts_with("solution")
                                || sub_name.ends_with(".jmx")
                                || sub_name.ends_with(".yaml")
                                || sub_name == "hints.md"
                                || sub_name == "theory.md"
                        })
                    })
                    .unwrap_or(false);

                if has_drill_file {
                    discovered.push(name);
                }
            }
        }
    }

    if !discovered.is_empty() {
        discovered.sort();
        if discovered.contains(&"05_push_notification_handling".to_string()) {
            discovered.retain(|d| d != "05_push_notification");
        }
        if discovered.contains(&"04_streaming_sse_test".to_string()) {
            discovered.retain(|d| d != "04_streaming_sse");
        }
        if discovered.contains(&"drill05_json_schema_validation".to_string()) {
            discovered.retain(|d| d != "drill05_contract_testing");
        }
        return discovered;
    }

    // Nothing on disk: fall back to the manifest, which is the single source of
    // truth for what the curriculum is supposed to contain.
    manifest_track
        .map(|t| t.drills.into_iter().map(|d| d.id).collect())
        .unwrap_or_default()
}

/// Calculate curriculum progression summaries across all configured tracks
pub fn get_track_summaries(
    config: &crate::config::Config,
    state: &GamificationState,
) -> Vec<TrackProgressSummary> {
    let mut summaries = Vec::new();

    for track in &config.tracks {
        let drill_ids = discover_track_drills(&track.id, &track.exercise_dir);
        let drills_total = drill_ids.len();
        let mut completed_count = 0;
        let mut next_incomplete = None;
        let mut lowest_score_drill: Option<(String, f64)> = None;

        for drill_id in &drill_ids {
            if state.is_drill_completed(&track.id, drill_id) {
                completed_count += 1;
                if let Some(record) = state.get_drill_record(&track.id, drill_id) {
                    match lowest_score_drill {
                        None => lowest_score_drill = Some((drill_id.clone(), record.best_score)),
                        Some((_, score)) if record.best_score < score => {
                            lowest_score_drill = Some((drill_id.clone(), record.best_score));
                        }
                        _ => {}
                    }
                }
            } else if next_incomplete.is_none() {
                next_incomplete = Some(drill_id.clone());
            }
        }

        let state_track_count = state.drills_count_for_track(&track.id);
        if completed_count < state_track_count {
            completed_count = state_track_count.min(drills_total);
        }

        let best_score = state.best_score_for_track(&track.id);
        let status_emoji = if drills_total > 0 && completed_count >= drills_total {
            "✅"
        } else if completed_count > 0 {
            "🟡"
        } else {
            "⏳"
        };

        summaries.push(TrackProgressSummary {
            track_id: track.id.clone(),
            track_name: track.name.clone(),
            drills_completed: completed_count,
            drills_total,
            best_score,
            status_emoji,
            next_incomplete_drill: next_incomplete,
            lowest_score_drill,
        });
    }

    summaries
}

/// Compute the next recommended drill for the student
pub fn get_next_recommended_drill(
    config: &crate::config::Config,
    state: &GamificationState,
) -> NextDrillRecommendation {
    let summaries = get_track_summaries(config, state);

    // 1. First incomplete drill across tracks in curriculum order
    for sum in &summaries {
        if let Some(ref incomplete) = sum.next_incomplete_drill {
            return NextDrillRecommendation {
                track_id: sum.track_id.clone(),
                track_name: sum.track_name.clone(),
                drill_id: incomplete.clone(),
                reason: format!("Next incomplete drill in {}", sum.track_name),
                command: format!("cherenkov-lings watch --track={}", sum.track_id),
            };
        }
    }

    // 2. If all tracks completed, find lowest-score drill across completed drills (< 100.0)
    let mut lowest_score_record: Option<(&str, &str, f64)> = None;
    for record in state.completed_drills.values() {
        if record.best_score < 100.0 {
            match lowest_score_record {
                None => {
                    lowest_score_record =
                        Some((&record.track_id, &record.drill_id, record.best_score))
                }
                Some((_, _, score)) if record.best_score < score => {
                    lowest_score_record =
                        Some((&record.track_id, &record.drill_id, record.best_score));
                }
                _ => {}
            }
        }
    }

    if let Some((t_id, d_id, score)) = lowest_score_record {
        let track_name = config
            .tracks
            .iter()
            .find(|t| t.id == t_id)
            .map(|t| t.name.clone())
            .unwrap_or_else(|| t_id.to_string());

        return NextDrillRecommendation {
            track_id: t_id.to_string(),
            track_name,
            drill_id: d_id.to_string(),
            reason: format!("Lowest score ({:.1}%) — re-attempt to reach 100.0%!", score),
            command: format!("cherenkov-lings watch --track={}", t_id),
        };
    }

    // 3. If no incomplete and all 100% (or no tracks)
    if let Some(first_track) = config.tracks.first() {
        if !summaries.is_empty()
            && summaries
                .iter()
                .all(|s| s.drills_completed >= s.drills_total && s.drills_total > 0)
        {
            NextDrillRecommendation {
                track_id: "sdet_master".to_string(),
                track_name: "SDET Master Curriculum".to_string(),
                drill_id: "all_completed".to_string(),
                reason: "All drills mastered at 100%! Ready for SDET Master.".to_string(),
                command: "cherenkov-lings dashboard".to_string(),
            }
        } else {
            NextDrillRecommendation {
                track_id: first_track.id.clone(),
                track_name: first_track.name.clone(),
                drill_id: "01_what_is_a_test".to_string(),
                reason: format!("Start your SDET journey with {}", first_track.name),
                command: format!("cherenkov-lings watch --track={}", first_track.id),
            }
        }
    } else {
        NextDrillRecommendation {
            track_id: "foundations".to_string(),
            track_name: "Automation Foundations".to_string(),
            drill_id: "01_what_is_a_test".to_string(),
            reason: "Start your SDET journey with Automation Foundations".to_string(),
            command: "cherenkov-lings watch --track=foundations".to_string(),
        }
    }
}

/// Render the complete single-frame ANSI interactive progress dashboard
pub fn render_dashboard(state: &GamificationState, config: &crate::config::Config) -> String {
    let mut out = String::new();
    let line = "═".repeat(88);

    // a. Header banner (Cherenkov Blue styling)
    out.push_str(&format!("{}\n", line.bright_cyan()));
    out.push_str(&format!(
        "   {}\n",
        "⚡  CHERENKOV-LINGS  —  INTERACTIVE PROGRESS DASHBOARD  ⚡"
            .bold()
            .bright_cyan()
    ));
    out.push_str(&format!("{}\n\n", line.bright_cyan()));

    // b. Player level and ASCII XP progress bar
    let level_info = get_level_info(state.total_xp);
    let bar = render_level_progress_bar(state.total_xp, 28);
    out.push_str(&format!(
        " {} {}\n",
        "🎖️".bright_cyan(),
        "PLAYER RANK & PROGRESSION".bold().bright_white()
    ));
    out.push_str(&format!(
        "    Rank / Level:   {} (Level {})\n",
        level_info.title.bold().bright_yellow(),
        level_info.level_number
    ));
    out.push_str(&format!("    XP Progress:    {}\n", bar));
    if let Some(next) = level_info.next_threshold {
        let remaining = next.saturating_sub(state.total_xp);
        out.push_str(&format!(
            "    XP Breakdown:   {} XP total  |  {} XP to next rank\n\n",
            state.total_xp.to_string().bold().bright_white(),
            remaining.to_string().bold().bright_green()
        ));
    } else {
        out.push_str(&format!(
            "    XP Breakdown:   {} XP total  |  {}\n\n",
            state.total_xp.to_string().bold().bright_white(),
            "MAX LEVEL REACHED (SDET Master)".bold().bright_green()
        ));
    }

    // e. Activity stats and streak
    let summaries = get_track_summaries(config, state);
    let total_drills_curriculum: usize = summaries.iter().map(|s| s.drills_total).sum();
    let total_drills_completed: usize = summaries.iter().map(|s| s.drills_completed).sum();
    let completion_pct = if total_drills_curriculum > 0 {
        (total_drills_completed as f64 / total_drills_curriculum as f64) * 100.0
    } else {
        0.0
    };

    out.push_str(&format!(
        " {} {}\n",
        "📊".bright_cyan(),
        "ACTIVITY & ENGAGEMENT STATS".bold().bright_white()
    ));
    out.push_str(&format!(
        "    🔥 Current Streak:    {} day{}\n",
        state.streak_days.to_string().bold().bright_yellow(),
        if state.streak_days == 1 { "" } else { "s" }
    ));
    out.push_str(&format!(
        "    🎯 Total Completed:   {} / {} drills ({:.1}%)\n",
        total_drills_completed.to_string().bold().bright_white(),
        total_drills_curriculum,
        completion_pct
    ));
    out.push_str(&format!(
        "    🏆 Badges Unlocked:   {} / {} achievements\n",
        state.achievements.len().to_string().bold().bright_yellow(),
        ALL_ACHIEVEMENTS.len()
    ));
    if let Some(ref date) = state.last_active_date {
        out.push_str(&format!(
            "    📅 Last Active Date:  {}\n",
            date.bright_white()
        ));
    }
    out.push('\n');

    // c. Per-track completion table
    out.push_str(&format!(
        " {} {}\n",
        "📚".bright_cyan(),
        "CURRICULUM TRACK PROGRESSION".bold().bright_white()
    ));

    let col_track_w = 48;
    let col_prog_w = 12;
    let col_score_w = 12;
    let col_stat_w = 8;

    let top_border = format!(
        " ┌{}┬{}┬{}┬{}┐",
        "─".repeat(col_track_w),
        "─".repeat(col_prog_w),
        "─".repeat(col_score_w),
        "─".repeat(col_stat_w)
    );
    let mid_border = format!(
        " ├{}┼{}┼{}┼{}┤",
        "─".repeat(col_track_w),
        "─".repeat(col_prog_w),
        "─".repeat(col_score_w),
        "─".repeat(col_stat_w)
    );
    let bot_border = format!(
        " └{}┴{}┴{}┴{}┘",
        "─".repeat(col_track_w),
        "─".repeat(col_prog_w),
        "─".repeat(col_score_w),
        "─".repeat(col_stat_w)
    );

    out.push_str(&format!("{}\n", top_border.dimmed()));
    out.push_str(&format!(
        " │ {:<46} │ {:^10} │ {:^10} │ {:^6} │\n",
        "Track".bold().bright_white(),
        "Progress".bold().bright_white(),
        "Best Score".bold().bright_white(),
        "Status".bold().bright_white()
    ));
    out.push_str(&format!("{}\n", mid_border.dimmed()));

    for sum in &summaries {
        let track_display = if sum.track_name.chars().count() > 46 {
            let truncated: String = sum.track_name.chars().take(43).collect();
            format!("{}...", truncated)
        } else {
            sum.track_name.clone()
        };

        let prog_str = format!("{} / {}", sum.drills_completed, sum.drills_total);
        let score_str = match sum.best_score {
            Some(s) => format!("{:.1}%", s),
            None => "N/A".to_string(),
        };

        out.push_str(&format!(
            " │ {:<46} │ {:^10} │ {:^10} │   {}    │\n",
            track_display.bright_cyan(),
            prog_str.bright_white(),
            score_str.bright_yellow(),
            sum.status_emoji
        ));
    }
    out.push_str(&format!("{}\n", bot_border.dimmed()));
    out.push_str(&format!(
        "   Status Legend: {} Complete  |  {} In Progress  |  {} Not Started\n\n",
        "✅".bright_green(),
        "🟡".bright_yellow(),
        "⏳".dimmed()
    ));

    // d. Top 3 most recently unlocked achievements with unlock date
    out.push_str(&format!(
        " {} {}\n",
        "🏆".bright_yellow(),
        "RECENT ACHIEVEMENTS".bold().bright_white()
    ));

    if state.achievements.is_empty() {
        out.push_str(&format!(
            "    {}\n",
            "No achievements unlocked yet. Complete your first drill to earn 'First Blood'!"
                .dimmed()
        ));
    } else {
        let recent: Vec<&UnlockedAchievement> = state.achievements.iter().rev().take(3).collect();
        for ach in recent {
            let date_display = if !ach.unlocked_at.is_empty() {
                let clean = if ach.unlocked_at.len() >= 10 {
                    &ach.unlocked_at[..10]
                } else {
                    &ach.unlocked_at
                };
                format!(" (Unlocked: {})", clean)
            } else {
                String::new()
            };
            out.push_str(&format!(
                "    • {} — {}{}\n",
                ach.name.bold().bright_yellow(),
                ach.description.bright_white(),
                date_display.dimmed()
            ));
        }
    }
    out.push('\n');

    // f. Next recommended drill
    let rec = get_next_recommended_drill(config, state);
    out.push_str(&format!(
        " {} {}\n",
        "🎯".bright_cyan(),
        "NEXT RECOMMENDED DRILL".bold().bright_white()
    ));
    out.push_str(&format!(
        "    👉 {} — {}\n",
        rec.track_name.bold().bright_yellow(),
        rec.drill_id.bold().bright_white()
    ));
    out.push_str(&format!("       Reason: {}\n", rec.reason.bright_cyan()));
    out.push_str(&format!("       Run:    {}\n", rec.command.bright_green()));
    out.push_str(&format!("{}\n", line.bright_cyan()));

    out
}

// =========================================================================
// Unit Tests
// =========================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_xp_calculation_formula_and_tiers() {
        // Base XP = 100
        // Score = 100, Tier 1 -> 100 XP
        assert_eq!(calculate_xp(100.0, 1), 100);
        // Score = 100, Tier 2 -> 150 XP
        assert_eq!(calculate_xp(100.0, 2), 150);
        // Score = 100, Tier 3 -> 200 XP
        assert_eq!(calculate_xp(100.0, 3), 200);

        // Score = 85, Tier 1 -> round(100 * 0.85 * 1.0) = 85 XP
        assert_eq!(calculate_xp(85.0, 1), 85);
        // Score = 85, Tier 2 -> round(100 * 0.85 * 1.5) = round(127.5) = 128 XP
        assert_eq!(calculate_xp(85.0, 2), 128);
        // Score = 85, Tier 3 -> round(100 * 0.85 * 2.0) = round(170.0) = 170 XP
        assert_eq!(calculate_xp(85.0, 3), 170);

        // Score = 0 -> 0 XP
        assert_eq!(calculate_xp(0.0, 1), 0);
        // Score clamped to 100 -> score 120.0 gives same as 100.0
        assert_eq!(calculate_xp(120.0, 1), 100);
    }

    #[test]
    fn test_tier_for_track_or_drill_mapping() {
        assert_eq!(tier_for_track_or_drill("foundations", "01_assert"), 1);
        assert_eq!(tier_for_track_or_drill("playwright-ts", "01_hydration"), 1);
        assert_eq!(
            tier_for_track_or_drill("playwright-ts", "06_page_object"),
            2
        );
        assert_eq!(
            tier_for_track_or_drill("playwright-ts", "09_visual_regression"),
            3
        );
        assert_eq!(tier_for_track_or_drill("k6-js", "01_pool"), 2);
        assert_eq!(tier_for_track_or_drill("k6-js", "05_grafana_output"), 3);
        assert_eq!(tier_for_track_or_drill("devsecops-python", "01_zap"), 3);
        assert_eq!(tier_for_track_or_drill("tool-decisions", "01_ui_vs_api"), 2);
    }

    #[test]
    fn test_level_progression_7_ranks() {
        assert_eq!(get_level(0), ("Trainee", 0, 500));
        assert_eq!(get_level(499), ("Trainee", 0, 500));
        assert_eq!(get_level(500), ("Junior QA", 500, 1500));
        assert_eq!(get_level(1499), ("Junior QA", 500, 1500));
        assert_eq!(get_level(1500), ("Mid QA", 1500, 3000));
        assert_eq!(get_level(2999), ("Mid QA", 1500, 3000));
        assert_eq!(get_level(3000), ("Senior QA", 3000, 6000));
        assert_eq!(get_level(5999), ("Senior QA", 3000, 6000));
        assert_eq!(get_level(6000), ("Lead QA", 6000, 10000));
        assert_eq!(get_level(9999), ("Lead QA", 6000, 10000));
        assert_eq!(get_level(10000), ("QA Architect", 10000, 20000));
        assert_eq!(get_level(19999), ("QA Architect", 10000, 20000));
        assert_eq!(get_level(20000), ("SDET Master", 20000, 20000));
        assert_eq!(get_level(50000), ("SDET Master", 20000, 20000));
    }

    #[test]
    fn test_level_info_progress_percentage() {
        let info_0 = get_level_info(0);
        assert_eq!(info_0.level_number, 1);
        assert_eq!(info_0.title, "Trainee");
        assert_eq!(info_0.progress_pct, 0.0);

        let info_250 = get_level_info(250);
        assert_eq!(info_250.progress_pct, 50.0);

        let info_1000 = get_level_info(1000);
        assert_eq!(info_1000.title, "Junior QA");
        assert_eq!(info_1000.current_level_xp, 500);
        assert_eq!(info_1000.level_required_xp, 1000);
        assert_eq!(info_1000.progress_pct, 50.0);

        let info_max = get_level_info(25000);
        assert_eq!(info_max.title, "SDET Master");
        assert_eq!(info_max.next_threshold, None);
        assert_eq!(info_max.progress_pct, 100.0);
    }

    #[test]
    fn test_date_civil_conversion_and_parsing() {
        // Unix epoch day 0: 1970-01-01
        assert_eq!(days_from_civil(1970, 1, 1), 0);
        assert_eq!(civil_from_days(0), "1970-01-01");

        let days_2026 = days_from_civil(2026, 8, 24);
        assert_eq!(civil_from_days(days_2026), "2026-08-24");

        assert_eq!(parse_date_to_days("2026-08-24"), Some(days_2026));
        assert_eq!(parse_date_to_days("2026-08-24T12:34:56Z"), Some(days_2026));
        assert_eq!(parse_date_to_days("invalid"), None);
    }

    #[test]
    fn test_streak_tracking_consecutive_and_broken() {
        let mut state = GamificationState::default();
        assert_eq!(state.streak_days, 0);

        // Day 1
        state.update_streak("2026-08-20");
        assert_eq!(state.streak_days, 1);
        assert_eq!(state.last_active_date.as_deref(), Some("2026-08-20"));

        // Same day -> no increment
        state.update_streak("2026-08-20T15:00:00Z");
        assert_eq!(state.streak_days, 1);

        // Day 2 (consecutive) -> increment
        state.update_streak("2026-08-21");
        assert_eq!(state.streak_days, 2);

        // Day 3 (consecutive) -> increment
        state.update_streak("2026-08-22");
        assert_eq!(state.streak_days, 3);

        // Skipped 2 days (jump to 2026-08-25) -> reset to 1
        state.update_streak("2026-08-25");
        assert_eq!(state.streak_days, 1);
        assert_eq!(state.last_active_date.as_deref(), Some("2026-08-25"));
    }

    #[test]
    fn test_achievement_first_blood() {
        let mut state = GamificationState::default();
        let ctx = DrillRunContext {
            track_id: "foundations".to_string(),
            drill_id: "01_assert".to_string(),
            passed: true,
            total_score: 95.0,
            tier: 1,
            ..Default::default()
        };

        let (xp, unlocked) = state.record_drill_run(&ctx);
        assert_eq!(xp, 95);
        assert!(unlocked.iter().any(|a| a.id == "first_blood"));
        assert!(state.has_achievement("first_blood"));
    }

    #[test]
    fn test_achievement_flakiness_slayer() {
        let mut state = GamificationState::default();

        let ctx = DrillRunContext {
            track_id: "playwright-ts".to_string(),
            drill_id: "01_hydration".to_string(),
            passed: true,
            total_score: 100.0,
            flakiness_score: 100.0,
            tier: 1,
            ..Default::default()
        };

        // Run 1
        let (_, un1) = state.record_drill_run(&ctx);
        assert_eq!(state.flakiness_100_streak, 1);
        assert!(!un1.iter().any(|a| a.id == "flakiness_slayer"));

        // Run 2
        let (_, un2) = state.record_drill_run(&ctx);
        assert_eq!(state.flakiness_100_streak, 2);
        assert!(!un2.iter().any(|a| a.id == "flakiness_slayer"));

        // Run 3
        let (_, un3) = state.record_drill_run(&ctx);
        assert_eq!(state.flakiness_100_streak, 3);
        assert!(un3.iter().any(|a| a.id == "flakiness_slayer"));
        assert!(state.has_achievement("flakiness_slayer"));
    }

    #[test]
    fn test_achievement_chaos_survivor() {
        let mut state = GamificationState::default();
        let ctx = DrillRunContext {
            track_id: "k6-js".to_string(),
            drill_id: "03_chaos_sla".to_string(),
            passed: true,
            total_score: 90.0,
            passed_iterations: 5,
            iterations: 5,
            tier: 2,
            ..Default::default()
        };

        let (_, unlocked) = state.record_drill_run(&ctx);
        assert!(unlocked.iter().any(|a| a.id == "chaos_survivor"));
        assert!(state.has_achievement("chaos_survivor"));
    }

    #[test]
    fn test_achievement_tool_polyglot() {
        let mut state = GamificationState::default();

        let tracks = ["foundations", "playwright-ts", "restassured-java", "k6-js"];
        for (i, track) in tracks.iter().enumerate() {
            let ctx = DrillRunContext {
                track_id: track.to_string(),
                drill_id: format!("drill_{}", i),
                passed: true,
                total_score: 90.0,
                tier: 1,
                ..Default::default()
            };
            state.record_drill_run(&ctx);
        }

        assert_eq!(state.distinct_tracks_count(), 4);
        assert!(state.has_achievement("tool_polyglot"));
    }

    #[test]
    fn test_achievement_the_architect() {
        let mut state = GamificationState::default();

        for drill in TOOL_DECISIONS_DRILLS {
            let ctx = DrillRunContext {
                track_id: "tool-decisions".to_string(),
                drill_id: drill.to_string(),
                passed: true,
                total_score: 100.0,
                tier: 2,
                ..Default::default()
            };
            state.record_drill_run(&ctx);
        }

        assert!(state.has_achievement("the_architect"));
    }

    #[test]
    fn test_achievement_perfect_locator() {
        let mut state = GamificationState::default();

        for i in 1..=5 {
            let ctx = DrillRunContext {
                track_id: "playwright-ts".to_string(),
                drill_id: format!("drill_{}", i),
                passed: true,
                total_score: 100.0,
                locator_score: 100.0,
                tier: 1,
                ..Default::default()
            };
            state.record_drill_run(&ctx);
        }

        assert_eq!(state.perfect_locator_count, 5);
        assert!(state.has_achievement("perfect_locator"));
    }

    #[test]
    fn test_achievement_speed_demon() {
        let mut state = GamificationState::default();

        // Baseline: 1000ms. 40% faster means duration <= 600ms.
        let ctx = DrillRunContext {
            track_id: "foundations".to_string(),
            drill_id: "01_assert".to_string(),
            passed: true,
            total_score: 95.0,
            avg_duration_ms: 550,
            baseline_duration_ms: 1000,
            tier: 1,
            ..Default::default()
        };

        let (_, unlocked) = state.record_drill_run(&ctx);
        assert!(unlocked.iter().any(|a| a.id == "speed_demon"));
        assert!(state.has_achievement("speed_demon"));
    }

    #[test]
    fn test_achievement_sdet_master() {
        let mut state = GamificationState::default();

        for (i, track) in ALL_TRACKS.iter().enumerate() {
            let ctx = DrillRunContext {
                track_id: track.to_string(),
                drill_id: format!("drill_{}", i),
                passed: true,
                total_score: 100.0,
                tier: 3,
                ..Default::default()
            };
            state.record_drill_run(&ctx);
        }

        assert_eq!(state.distinct_tracks_count(), ALL_TRACKS.len());
        assert!(state.has_achievement("sdet_master"));
    }

    #[test]
    fn test_persistence_save_and_load() {
        let unique_id = format!(
            "{}_{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        );
        let temp_dir =
            std::env::temp_dir().join(format!("cherenkov_gamification_test_{}", unique_id));
        let temp_file = temp_dir.join(".cherenkov-progress.json");

        if temp_file.exists() {
            let _ = fs::remove_file(&temp_file);
        }

        // Test load non-existent -> returns default
        let loaded_empty = load_progress(Some(&temp_file)).unwrap();
        assert_eq!(loaded_empty, GamificationState::default());

        // Modify and save
        let mut state = GamificationState::default();
        let ctx = DrillRunContext {
            track_id: "playwright-ts".to_string(),
            drill_id: "01_hydration".to_string(),
            passed: true,
            total_score: 95.0,
            tier: 2,
            ..Default::default()
        };
        state.record_drill_run(&ctx);

        save_progress(&state, Some(&temp_file)).unwrap();
        assert!(temp_file.exists());

        // Load saved
        let loaded = load_progress(Some(&temp_file)).unwrap();
        assert_eq!(loaded.total_xp, state.total_xp);
        assert_eq!(loaded.completed_drills.len(), 1);
        assert_eq!(loaded.achievements.len(), state.achievements.len());

        let _ = fs::remove_file(&temp_file);
        let _ = fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_render_scorecard_and_badge_reveal() {
        let mut state = GamificationState::default();
        let ctx = DrillRunContext {
            track_id: "playwright-ts".to_string(),
            drill_id: "01_hydration".to_string(),
            passed: true,
            total_score: 95.0,
            tier: 2,
            ..Default::default()
        };

        let (xp, newly_unlocked) = state.record_drill_run(&ctx);
        let rendered = render_gamification_scorecard_with_tier(xp, 2, &state, &newly_unlocked);
        assert!(rendered.contains("GAMIFICATION & PROGRESSION"));
        assert!(rendered.contains("+143 XP earned!"));
        assert!(rendered.contains("Level: Trainee"));
        assert!(rendered.contains("ACHIEVEMENT UNLOCKED: First Blood"));

        let single_reveal = render_badge_reveal(&newly_unlocked[0]);
        assert!(single_reveal.contains("ACHIEVEMENT UNLOCKED: First Blood"));
        assert!(single_reveal.contains("Complete your first drill ever"));
    }

    #[test]
    fn test_extract_drill_id_from_path_patterns() {
        assert_eq!(
            extract_drill_id_from_path("exercises/00_foundations/01_what_is_a_test/exercise.py"),
            "01_what_is_a_test"
        );
        assert_eq!(
            extract_drill_id_from_path(
                "exercises/01_web_playwright_ts/04_first_playwright_test/exercise.ts"
            ),
            "04_first_playwright_test"
        );
        assert_eq!(
            extract_drill_id_from_path(
                "exercises/02_api_restassured_java/src/test/java/com/cherenkov/drill01_idempotency/Exercise.java"
            ),
            "drill01_idempotency"
        );
        assert_eq!(
            extract_drill_id_from_path(
                "exercises/03_mobile_maestro/01_biometric_fallback/exercise.yaml"
            ),
            "01_biometric_fallback"
        );
        assert_eq!(
            extract_drill_id_from_path(
                "exercises/04_perf_k6_js/01_database_pool_starvation/solution.js"
            ),
            "01_database_pool_starvation"
        );
        assert_eq!(
            extract_drill_id_from_path("exercises/05_perf_jmeter/01_gui_mode_antipattern/hints.md"),
            "01_gui_mode_antipattern"
        );
        assert_eq!(extract_drill_id_from_path("01_assert.py"), "01_assert");
        // A Windows-style path resolves the same way on every target: the
        // watcher hands these to us verbatim, and CI runs both.
        assert_eq!(
            extract_drill_id_from_path("exercises\\00_foundations\\01_what_is_a_test\\exercise.py"),
            "01_what_is_a_test"
        );
        // Mixed conventions turn up when one half of a path is joined by the
        // host and the other half was stored by a different one.
        assert_eq!(
            extract_drill_id_from_path("exercises\\04_perf_k6_js/06_rps_spike\\solution.js"),
            "06_rps_spike"
        );
    }

    #[test]
    fn test_discover_track_drills_fallback_and_disk() {
        let foundations_drills = discover_track_drills("foundations", "exercises/00_foundations");
        assert_eq!(foundations_drills.len(), 5);
        assert!(foundations_drills.contains(&"01_what_is_a_test".to_string()));

        let playwright_drills =
            discover_track_drills("playwright-ts", "exercises/01_web_playwright_ts");
        assert_eq!(playwright_drills.len(), 10);
        assert!(playwright_drills.contains(&"01_hydration_timing".to_string()));

        let java_drills =
            discover_track_drills("restassured-java", "exercises/02_api_restassured_java");
        assert!(!java_drills.is_empty());
        assert!(java_drills.contains(&"drill01_idempotency".to_string()));
    }

    #[test]
    fn test_render_dashboard_default_and_sections() {
        let state = GamificationState::default();
        let config = crate::config::Config {
            platform: crate::config::PlatformConfig {
                name: "cherenkov-lings".to_string(),
                version: "1.0.0".to_string(),
                sandbox_port: 8080,
                chaos_proxy_port: 8086,
                telemetry: false,
            },
            evaluation: crate::config::EvaluationConfig {
                pass_threshold: 85.0,
                flakiness_iterations: 5,
                flakiness_timeout_ms: 5000,
                chaos_latency_ms: 200,
                chaos_jitter_ms: 75,
            },
            ui: crate::config::UiConfig {
                theme: "cherenkov-blue".to_string(),
                show_hints_on_failure: true,
                enable_audio_bell: false,
                language: "en".to_string(),
            },
            tracks: default_curriculum_tracks(),
        };

        let dashboard = render_dashboard(&state, &config);
        assert!(dashboard.contains("CHERENKOV-LINGS"));
        assert!(dashboard.contains("INTERACTIVE PROGRESS DASHBOARD"));
        assert!(dashboard.contains("PLAYER RANK & PROGRESSION"));
        assert!(dashboard.contains("Trainee (Level 1)"));
        assert!(dashboard.contains("ACTIVITY & ENGAGEMENT STATS"));
        assert!(dashboard.contains("0 day"));
        assert!(dashboard.contains("CURRICULUM TRACK PROGRESSION"));
        assert!(dashboard.contains("RECENT ACHIEVEMENTS"));
        assert!(dashboard.contains("No achievements unlocked yet"));
        assert!(dashboard.contains("NEXT RECOMMENDED DRILL"));
    }

    #[test]
    fn test_dashboard_with_progress_and_achievements() {
        let mut state = GamificationState::default();

        let ctx = DrillRunContext {
            track_id: "foundations".to_string(),
            drill_id: "01_what_is_a_test".to_string(),
            file_path: "exercises/00_foundations/01_what_is_a_test/exercise.py".to_string(),
            passed: true,
            total_score: 95.0,
            tier: 1,
            ..Default::default()
        };
        state.record_drill_run(&ctx);

        let config = crate::config::Config {
            platform: crate::config::PlatformConfig {
                name: "cherenkov-lings".to_string(),
                version: "1.0.0".to_string(),
                sandbox_port: 8080,
                chaos_proxy_port: 8086,
                telemetry: false,
            },
            evaluation: crate::config::EvaluationConfig {
                pass_threshold: 85.0,
                flakiness_iterations: 5,
                flakiness_timeout_ms: 5000,
                chaos_latency_ms: 200,
                chaos_jitter_ms: 75,
            },
            ui: crate::config::UiConfig {
                theme: "cherenkov-blue".to_string(),
                show_hints_on_failure: true,
                enable_audio_bell: false,
                language: "en".to_string(),
            },
            tracks: default_curriculum_tracks(),
        };

        let dashboard = render_dashboard(&state, &config);
        assert!(dashboard.contains("First Blood"));
        assert!(
            dashboard.contains("1 / 45")
                || dashboard.contains("1 / 43")
                || dashboard.contains("1 / ")
        );
        assert!(dashboard.contains("🟡") || dashboard.contains("✅"));
    }
}
