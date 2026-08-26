pub mod interactive;
pub mod llm;
pub mod rules;

use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

pub use interactive::run_interactive_review;
pub use llm::{AiMentorClient, MentorReview};
pub use rules::{apply_automated_fixes, AstViolation, RuleScanner, Severity, SupportedLanguage};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReviewConfig {
    pub llm_endpoint: Option<String>,
    pub llm_model: Option<String>,
    pub offline_fallback: bool,
    pub strict_mode: bool,
    pub score_threshold: u32,
}

impl Default for ReviewConfig {
    fn default() -> Self {
        Self {
            llm_endpoint: None,
            llm_model: None,
            offline_fallback: true,
            strict_mode: false,
            score_threshold: 80,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReviewReport {
    pub exercise_name: String,
    pub score: u32,
    pub passed: bool,
    pub violations: Vec<AstViolation>,
    pub mentor_critique: String,
    pub socratic_questions: Vec<String>,
    pub suggested_diff: Option<String>,
}

#[derive(Debug, Clone)]
pub enum ReviewError {
    IoError(String),
    AnalysisError(String),
    FixError(String),
}

impl std::fmt::Display for ReviewError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ReviewError::IoError(msg) => write!(f, "I/O Error: {}", msg),
            ReviewError::AnalysisError(msg) => write!(f, "Analysis Error: {}", msg),
            ReviewError::FixError(msg) => write!(f, "Fix Application Error: {}", msg),
        }
    }
}

impl std::error::Error for ReviewError {}

impl From<std::io::Error> for ReviewError {
    fn from(err: std::io::Error) -> Self {
        ReviewError::IoError(err.to_string())
    }
}

/// Computes the review quality score (0 to 100) based on detected violations
pub fn calculate_score(violations: &[AstViolation]) -> u32 {
    let mut score: i32 = 100;

    for violation in violations {
        match violation.severity {
            Severity::Error => score -= 25,
            Severity::Warning => score -= 10,
            Severity::Info => score -= 5,
        }
    }

    if score < 0 {
        0
    } else {
        score as u32
    }
}

/// Runs a static rule review and AI mentor consultation on a target file
pub fn run_review(
    file_path: &Path,
    config: &ReviewConfig,
) -> Result<ReviewReport, ReviewError> {
    if !file_path.exists() {
        return Err(ReviewError::IoError(format!(
            "Target file '{}' does not exist",
            file_path.display()
        )));
    }

    let content = fs::read_to_string(file_path)?;
    let file_str = file_path.to_string_lossy().to_string();
    run_review_on_content(&file_str, &content, config)
}

/// Runs a static rule review on raw code content
pub fn run_review_on_content(
    file_path: &str,
    content: &str,
    config: &ReviewConfig,
) -> Result<ReviewReport, ReviewError> {
    let exercise_name = Path::new(file_path)
        .file_name()
        .map(|f| f.to_string_lossy().to_string())
        .unwrap_or_else(|| file_path.to_string());

    let violations = RuleScanner::scan_content(file_path, content);
    let score = calculate_score(&violations);

    let has_error = violations.iter().any(|v| v.severity == Severity::Error);
    let passed = if config.strict_mode {
        score >= config.score_threshold && !has_error && violations.is_empty()
    } else {
        score >= config.score_threshold && !has_error
    };

    // AI Mentor consultation (sync wrapper around async call or offline fallback)
    let mentor_client = AiMentorClient::new(
        config.llm_endpoint.as_deref(),
        config.llm_model.as_deref(),
        config.offline_fallback,
    );

    let mentor_review = mentor_client.generate_offline_mentor_review(
        &exercise_name,
        content,
        &violations,
    );

    // Generate proposed unified diff if violations have suggested fixes
    let suggested_diff = if !violations.is_empty() {
        let fixed_content = apply_automated_fixes(content, &violations);
        if fixed_content != content {
            Some(generate_unified_diff(content, &fixed_content, &exercise_name))
        } else {
            None
        }
    } else {
        None
    };

    Ok(ReviewReport {
        exercise_name,
        score,
        passed,
        violations,
        mentor_critique: mentor_review.critique,
        socratic_questions: mentor_review.socratic_questions,
        suggested_diff,
    })
}

/// Applies a single fix matching `fix_id` (or rule_id / line number) to a file on disk
pub fn apply_fix(file_path: &Path, fix_id: &str) -> Result<String, ReviewError> {
    let content = fs::read_to_string(file_path)?;
    let violations = RuleScanner::scan_content(&file_path.to_string_lossy(), &content);

    // Find violation matching fix_id (can be rule_id, rule_id@line, or line number)
    let target_violations: Vec<AstViolation> = violations
        .into_iter()
        .filter(|v| {
            v.rule_id == fix_id
                || format!("{}@{}", v.rule_id, v.line_number) == fix_id
                || v.line_number.to_string() == fix_id
                || fix_id == "all"
        })
        .collect();

    if target_violations.is_empty() {
        if fix_id == "all" {
            return Ok(content);
        }
        return Err(ReviewError::FixError(format!(
            "No matching violation found for fix identifier '{}'",
            fix_id
        )));
    }

    let modified = apply_automated_fixes(&content, &target_violations);
    fs::write(file_path, &modified)?;
    Ok(modified)
}

/// Applies all automated rule fixes to a file on disk
pub fn apply_all_fixes(file_path: &Path) -> Result<String, ReviewError> {
    apply_fix(file_path, "all")
}

/// Generates a standard unified diff representation between original and patched code
pub fn generate_unified_diff(original: &str, modified: &str, file_name: &str) -> String {
    let orig_lines: Vec<&str> = original.lines().collect();
    let mod_lines: Vec<&str> = modified.lines().collect();

    let mut diff = Vec::new();
    diff.push(format!("--- a/{}", file_name));
    diff.push(format!("+++ b/{}", file_name));

    let max_len = orig_lines.len().max(mod_lines.len());
    let mut in_hunk = false;
    let mut hunk_orig_start = 0;
    let mut hunk_lines = Vec::new();

    for i in 0..max_len {
        let orig_line = orig_lines.get(i).copied();
        let mod_line = mod_lines.get(i).copied();

        if orig_line != mod_line {
            if !in_hunk {
                in_hunk = true;
                hunk_orig_start = i + 1;
                // Add context line before if available
                if i > 0 {
                    if let Some(ctx) = orig_lines.get(i - 1) {
                        hunk_lines.push(format!(" {}", ctx));
                    }
                }
            }
            if let Some(o) = orig_line {
                hunk_lines.push(format!("-{}", o));
            }
            if let Some(m) = mod_line {
                hunk_lines.push(format!("+{}", m));
            }
        } else if in_hunk {
            // Add trailing context line
            if let Some(ctx) = orig_line {
                hunk_lines.push(format!(" {}", ctx));
            }
            // Flush hunk
            diff.push(format!(
                "@@ -{},{} +{},{} @@",
                hunk_orig_start,
                hunk_lines.iter().filter(|l| l.starts_with('-') || l.starts_with(' ')).count(),
                hunk_orig_start,
                hunk_lines.iter().filter(|l| l.starts_with('+') || l.starts_with(' ')).count()
            ));
            diff.extend(hunk_lines.drain(..));
            in_hunk = false;
        }
    }

    if in_hunk && !hunk_lines.is_empty() {
        diff.push(format!(
            "@@ -{},{} +{},{} @@",
            hunk_orig_start,
            hunk_lines.iter().filter(|l| l.starts_with('-') || l.starts_with(' ')).count(),
            hunk_orig_start,
            hunk_lines.iter().filter(|l| l.starts_with('+') || l.starts_with(' ')).count()
        ));
        diff.extend(hunk_lines);
    }

    diff.join("\n")
}
