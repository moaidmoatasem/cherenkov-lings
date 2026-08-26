use colored::Colorize;
use regex::Regex;
use std::fs;
use std::path::Path;
use std::sync::LazyLock;

use crate::runner::DrillResponse;

/// Weights for the 4D Feedback Matrix
pub const WEIGHT_CORRECTNESS: f64 = 0.35;
pub const WEIGHT_FLAKINESS: f64 = 0.35;
pub const WEIGHT_LOCATOR_QUALITY: f64 = 0.15;
pub const WEIGHT_SPEED: f64 = 0.15;
pub const DEFAULT_PASS_THRESHOLD: f64 = 85.0;
pub const DEFAULT_BASELINE_DURATION_MS: u64 = 1000;
pub const FLAKINESS_PENALTY_CAP: f64 = 40.0;

// Lazy static regex patterns for source scanning
static RE_WAIT_FOR_TIMEOUT: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?:page|locator|\b)\s*\.\s*waitForTimeout\s*\(\s*(\d+)?\s*\)|waitForTimeout\s*\(\s*(\d+)?\s*\)").expect("Valid regex")
});

static RE_SET_TIMEOUT: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?:window\.)?setTimeout\s*\(\s*(?:[^,]+,\s*)?(\d+)\s*\)").expect("Valid regex")
});

static RE_THREAD_SLEEP: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?:\bThread\s*\.\s*sleep|\bTimeUnit\s*\.\s*[A-Za-z_]+\s*\.\s*sleep|\bjava\.lang\.Thread\s*\.\s*sleep)\s*\(\s*(\d+)?").expect("Valid regex")
});

static RE_GET_BY_ROLE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r#"(?:page|locator|\b)\s*\.\s*getByRole\s*\(\s*(?:'((?:\\.|[^'\\])*)'|"((?:\\.|[^"\\])*)"|`((?:\\.|[^`\\])*)`)"#).expect("Valid regex")
});

static RE_GET_BY_TEST_ID: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r#"(?:page|locator|\b)\s*\.\s*getByTestId\s*\(\s*(?:'((?:\\.|[^'\\])*)'|"((?:\\.|[^"\\])*)"|`((?:\\.|[^`\\])*)`)"#).expect("Valid regex")
});

static RE_GET_BY_TEXT_OR_LABEL: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r#"(?:page|locator|\b)\s*\.\s*(?:getByText|getByLabel|getByPlaceholder|getByAltText|getByTitle)\s*\(\s*(?:'((?:\\.|[^'\\])*)'|"((?:\\.|[^"\\])*)"|`((?:\\.|[^`\\])*)`)"#).expect("Valid regex")
});

static RE_LOCATOR_CALL: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r#"(?:page|locator|\b)\s*\.\s*(?:locator|\$|\$\$)\s*\(\s*(?:'((?:\\.|[^'\\])*)'|"((?:\\.|[^"\\])*)"|`((?:\\.|[^`\\])*)`)\s*\)"#).expect("Valid regex")
});

fn extract_string_arg<'a>(caps: &'a regex::Captures<'a>, start_group: usize) -> Option<&'a str> {
    caps.get(start_group)
        .or_else(|| caps.get(start_group + 1))
        .or_else(|| caps.get(start_group + 2))
        .map(|m| m.as_str())
}

/// Classification of locator quality according to testing best practices
#[derive(Debug, Clone, PartialEq)]
pub enum LocatorKind {
    /// Semantic accessible role: page.getByRole('button', ...) -> 100 pts
    GetByRole,
    /// User-visible text/label: page.getByText / getByLabel / getByPlaceholder / Maestro text: -> 90 pts
    GetByTextOrLabel,
    /// Resilient test id: page.getByTestId(...) or Maestro id: -> 85 pts
    GetByTestId,
    /// Fragile CSS class/id selector: page.locator('.btn-primary') -> 40 pts
    CssSelector,
    /// Absolute XPath / Fragile DOM path: page.locator('/html/body/...') -> 0 pts
    AbsoluteXPath,
}

impl LocatorKind {
    pub fn score(&self) -> f64 {
        match self {
            Self::GetByRole => 100.0,
            Self::GetByTextOrLabel => 90.0,
            Self::GetByTestId => 85.0,
            Self::CssSelector => 40.0,
            Self::AbsoluteXPath => 0.0,
        }
    }

    pub fn display_name(&self) -> &'static str {
        match self {
            Self::GetByRole => "Semantic Role (getByRole)",
            Self::GetByTextOrLabel => "Text / Label (getByText/Label)",
            Self::GetByTestId => "Test ID (getByTestId)",
            Self::CssSelector => "CSS Selector (class/id)",
            Self::AbsoluteXPath => "Absolute XPath",
        }
    }
}

/// Recorded locator occurrence in source code
#[derive(Debug, Clone, PartialEq)]
pub struct LocatorOccurrence {
    pub kind: LocatorKind,
    pub selector: String,
    pub line: usize,
    pub snippet: String,
    pub score: f64,
}

/// Anti-pattern classification detected by static source analysis
#[derive(Debug, Clone, PartialEq)]
pub enum AntiPatternKind {
    WaitForTimeout { duration_ms: Option<u64> },
    HardcodedSleep { duration_ms: Option<u64> },
    ThreadSleep { duration_ms: Option<u64> },
    FragileXPath { selector: String },
    FragileCss { selector: String },
    // Maestro Mobile YAML Anti-Patterns:
    MissingWhenCondition { flow_type: String },
    MissingColdStartDeepLink { command: String },
    MissingActivityRecreation { state: String },
    UnconditionalFallbackAssert { selector: String },
}

/// Diagnostic details for a detected anti-pattern
#[derive(Debug, Clone, PartialEq)]
pub struct AntiPattern {
    pub kind: AntiPatternKind,
    pub line: usize,
    pub snippet: String,
    pub explanation: String,
    pub recommendation: String,
}

/// Static analysis report of a test or flow definition file
#[derive(Debug, Clone, Default)]
pub struct StaticAnalysisReport {
    pub file_path: String,
    pub total_lines: usize,
    pub anti_patterns: Vec<AntiPattern>,
    pub locators: Vec<LocatorOccurrence>,
    pub locator_quality_score: f64,
    pub has_wait_for_timeout: bool,
}

impl StaticAnalysisReport {
    pub fn has_anti_patterns(&self) -> bool {
        !self.anti_patterns.is_empty()
    }
}

/// Strip single-line (`//`) and block (`/* ... */`) comments while preserving
/// exact line numbers and byte positions. String literals are protected so
/// URLs and internal strings containing `//` or `/*` are not stripped.
pub fn strip_comments(source: &str) -> String {
    let mut result = String::with_capacity(source.len());
    let chars: Vec<char> = source.chars().collect();
    let len = chars.len();
    let mut i = 0;
    let mut in_single_quote = false;
    let mut in_double_quote = false;
    let mut in_template = false;

    while i < len {
        let c = chars[i];
        let next = if i + 1 < len {
            Some(chars[i + 1])
        } else {
            None
        };

        // Handle escape character inside strings
        if (in_single_quote || in_double_quote || in_template) && c == '\\' {
            result.push(c);
            if let Some(n) = next {
                result.push(n);
                i += 2;
                continue;
            }
            i += 1;
            continue;
        }

        // Single-quote toggle
        if !in_double_quote && !in_template && c == '\'' {
            in_single_quote = !in_single_quote;
            result.push(c);
            i += 1;
            continue;
        }

        // Double-quote toggle
        if !in_single_quote && !in_template && c == '"' {
            in_double_quote = !in_double_quote;
            result.push(c);
            i += 1;
            continue;
        }

        // Template string toggle
        if !in_single_quote && !in_double_quote && c == '`' {
            in_template = !in_template;
            result.push(c);
            i += 1;
            continue;
        }

        // When not inside any string literal, detect comment starters
        if !in_single_quote && !in_double_quote && !in_template {
            // Single-line comment: // ...
            if c == '/' && next == Some('/') {
                while i < len && chars[i] != '\n' {
                    result.push(' ');
                    i += 1;
                }
                continue;
            }

            // Multi-line block comment: /* ... */
            if c == '/' && next == Some('*') {
                i += 2;
                result.push(' ');
                result.push(' ');
                while i < len {
                    if chars[i] == '\n' {
                        result.push('\n');
                        i += 1;
                    } else if chars[i] == '*' && i + 1 < len && chars[i + 1] == '/' {
                        result.push(' ');
                        result.push(' ');
                        i += 2;
                        break;
                    } else {
                        result.push(' ');
                        i += 1;
                    }
                }
                continue;
            }
        }

        result.push(c);
        i += 1;
    }

    result
}

/// Strip YAML comments (`# ...`) while preserving line numbers and string literals
pub fn strip_yaml_comments(source: &str) -> String {
    let mut result = String::with_capacity(source.len());
    for line in source.lines() {
        let mut in_single = false;
        let mut in_double = false;
        let mut stripped_line = String::with_capacity(line.len());
        let chars: Vec<char> = line.chars().collect();
        let mut i = 0;
        while i < chars.len() {
            let c = chars[i];
            if c == '\'' && !in_double {
                in_single = !in_single;
                stripped_line.push(c);
            } else if c == '"' && !in_single {
                in_double = !in_double;
                stripped_line.push(c);
            } else if c == '#' && !in_single && !in_double {
                while i < chars.len() {
                    stripped_line.push(' ');
                    i += 1;
                }
                break;
            } else {
                stripped_line.push(c);
            }
            i += 1;
        }
        result.push_str(&stripped_line);
        result.push('\n');
    }
    result
}

/// Analyze YAML source code for Maestro flow definitions, locators, and anti-patterns
pub fn analyze_yaml_source(source: &str, file_path: &str) -> StaticAnalysisReport {
    let stripped = strip_yaml_comments(source);
    let original_lines: Vec<&str> = source.lines().collect();
    let stripped_lines: Vec<&str> = stripped.lines().collect();

    let mut anti_patterns = Vec::new();
    let mut locators = Vec::new();
    let mut has_wait_for_timeout = false;

    // Pattern matching for YAML locators
    let re_text =
        Regex::new(r#"(?:^|\s+)text:\s*(?:'((?:\\.|[^'\\])*)'|"((?:\\.|[^"\\])*)"|([^\r\n#]+))"#)
            .expect("Valid regex");
    let re_id =
        Regex::new(r#"(?:^|\s+)id:\s*(?:'((?:\\.|[^'\\])*)'|"((?:\\.|[^"\\])*)"|([^\r\n#]+))"#)
            .expect("Valid regex");

    for (idx, stripped_line) in stripped_lines.iter().enumerate() {
        let line_num = idx + 1;
        let original_snippet = original_lines.get(idx).unwrap_or(&"").trim().to_string();

        // Extract text: locators (exclude appId or config properties)
        if !stripped_line.contains("appId:")
            && let Some(caps) = re_text.captures(stripped_line)
            && let Some(txt) = extract_string_arg(&caps, 1)
        {
            let clean_txt = txt.trim();
            if !clean_txt.is_empty() {
                locators.push(LocatorOccurrence {
                    kind: LocatorKind::GetByTextOrLabel,
                    selector: clean_txt.to_string(),
                    line: line_num,
                    snippet: original_snippet.clone(),
                    score: LocatorKind::GetByTextOrLabel.score(),
                });
            }
        }

        // Extract id: locators (exclude appId:)
        if !stripped_line.contains("appId:")
            && let Some(caps) = re_id.captures(stripped_line)
            && let Some(id_val) = extract_string_arg(&caps, 1)
        {
            let clean_id = id_val.trim();
            if !clean_id.is_empty() {
                locators.push(LocatorOccurrence {
                    kind: LocatorKind::GetByTestId,
                    selector: clean_id.to_string(),
                    line: line_num,
                    snippet: original_snippet.clone(),
                    score: LocatorKind::GetByTestId.score(),
                });
            }
        }
    }

    // 1. Biometric Fallback Anti-Pattern:
    // Flow triggers Biometric auth without conditional `runFlow` with `when:` clause
    let has_biometric = stripped.contains("Biometric") || stripped.contains("biometric");
    let has_when_condition = stripped.contains("when:") && stripped.contains("runFlow:");
    if has_biometric && !has_when_condition {
        has_wait_for_timeout = true;
        let bio_line = stripped_lines
            .iter()
            .enumerate()
            .find(|(_, l)| l.contains("Biometric") || l.contains("biometric"))
            .map(|(i, _)| i + 1)
            .unwrap_or(1);
        let bio_snippet = original_lines
            .get(bio_line.saturating_sub(1))
            .unwrap_or(&"")
            .trim()
            .to_string();

        anti_patterns.push(AntiPattern {
            kind: AntiPatternKind::MissingWhenCondition { flow_type: "biometric_fallback".to_string() },
            line: bio_line,
            snippet: bio_snippet,
            explanation: "Biometric authentication flow lacks a conditional fallback (missing 'when:' clause in runFlow). Biometric prompts fail in headless CI or simulator environments without fallback handling.".to_string(),
            recommendation: "Add a 'runFlow' step with a 'when: visible: text: Biometric unavailable' condition to handle the PIN fallback screen.".to_string(),
        });
    }

    // 2. Cold Start Deep Link Anti-Pattern:
    // Uses `openLink` which fails or opens browser on unlaunched/cold-start app
    let has_open_link = stripped.contains("openLink:");
    let has_launch_app_deeplink = stripped.contains("launchApp:")
        && (stripped.contains("deeplink:") || stripped.contains("arguments:"));
    if has_open_link && !has_launch_app_deeplink {
        has_wait_for_timeout = true;
        let open_line = stripped_lines
            .iter()
            .enumerate()
            .find(|(_, l)| l.contains("openLink:"))
            .map(|(i, _)| i + 1)
            .unwrap_or(1);
        let open_snippet = original_lines
            .get(open_line.saturating_sub(1))
            .unwrap_or(&"")
            .trim()
            .to_string();

        anti_patterns.push(AntiPattern {
            kind: AntiPatternKind::MissingColdStartDeepLink { command: "openLink".to_string() },
            line: open_line,
            snippet: open_snippet,
            explanation: "Using 'openLink' assumes the application is already running in the foreground (warm start). In cold-start or CI environments, openLink may open a web browser instead of the native app.".to_string(),
            recommendation: "Use 'launchApp' with 'clearState: true' and 'arguments: { deeplink: ... }' to ensure deterministic cold-start initialization.".to_string(),
        });
    }

    // 3. Activity Recreation Anti-Pattern:
    // Asserts state without testing Activity destruction and recreation across screen orientation changes
    let is_activity_drill = file_path.contains("03_activity_recreation")
        || (stripped.contains("Balance") && stripped.contains("assertVisible"));
    let has_orientation_change = stripped.contains("setOrientation:")
        || (stripped.contains("orientation:") && stripped.contains("landscape"));
    if is_activity_drill && !has_orientation_change {
        has_wait_for_timeout = true;
        let assert_line = stripped_lines
            .iter()
            .enumerate()
            .find(|(_, l)| l.contains("assertVisible:"))
            .map(|(i, _)| i + 1)
            .unwrap_or(1);
        let assert_snippet = original_lines
            .get(assert_line.saturating_sub(1))
            .unwrap_or(&"")
            .trim()
            .to_string();

        anti_patterns.push(AntiPattern {
            kind: AntiPatternKind::MissingActivityRecreation { state: "screen_rotation".to_string() },
            line: assert_line,
            snippet: assert_snippet,
            explanation: "UI state is asserted only once without testing Activity destruction and recreation across screen orientation changes. On Android/iOS, config changes recreate the activity and can wipe unpersisted UI state.".to_string(),
            recommendation: "Use 'setOrientation: landscape' followed by 'assertVisible' to verify state persistence across activity recreation, then restore orientation.".to_string(),
        });
    }

    let locator_quality_score = if locators.is_empty() {
        100.0
    } else {
        let sum: f64 = locators.iter().map(|l| l.score).sum();
        sum / (locators.len() as f64)
    };

    StaticAnalysisReport {
        file_path: file_path.to_string(),
        total_lines: original_lines.len(),
        anti_patterns,
        locators,
        locator_quality_score,
        has_wait_for_timeout,
    }
}

/// Analyze source code (TypeScript, Java, or Maestro YAML) for locators, anti-patterns, and quality
pub fn analyze_source(source: &str, file_path: &str) -> StaticAnalysisReport {
    let is_yaml = file_path.ends_with(".yaml")
        || file_path.ends_with(".yml")
        || source.trim_start().starts_with("---");
    if is_yaml {
        return analyze_yaml_source(source, file_path);
    }
    let stripped = strip_comments(source);
    let original_lines: Vec<&str> = source.lines().collect();
    let stripped_lines: Vec<&str> = stripped.lines().collect();

    let mut anti_patterns = Vec::new();
    let mut locators = Vec::new();
    let mut has_wait_for_timeout = false;

    for (idx, stripped_line) in stripped_lines.iter().enumerate() {
        let line_num = idx + 1;
        let original_snippet = original_lines.get(idx).unwrap_or(&"").trim().to_string();

        // 1. Detect waitForTimeout anti-pattern
        if let Some(caps) = RE_WAIT_FOR_TIMEOUT.captures(stripped_line) {
            has_wait_for_timeout = true;
            let duration_ms = caps
                .get(1)
                .or_else(|| caps.get(2))
                .and_then(|m| m.as_str().parse::<u64>().ok());
            anti_patterns.push(AntiPattern {
                kind: AntiPatternKind::WaitForTimeout { duration_ms },
                line: line_num,
                snippet: original_snippet.clone(),
                explanation: "Fixed sleep makes the test timing-dependent and causes click drops or race conditions during hydration delay and network jitter.".to_string(),
                recommendation: "Replace page.waitForTimeout() with event-driven web-first assertions like await expect(locator).toBeVisible() or await expect(locator).toBeEnabled().".to_string(),
            });
        }

        // 2. Detect hardcoded setTimeout anti-pattern
        if let Some(caps) = RE_SET_TIMEOUT.captures(stripped_line) {
            let duration_ms = caps.get(1).and_then(|m| m.as_str().parse::<u64>().ok());
            anti_patterns.push(AntiPattern {
                kind: AntiPatternKind::HardcodedSleep { duration_ms },
                line: line_num,
                snippet: original_snippet.clone(),
                explanation: "Manual setTimeout blocks or delays test execution without synchronizing with DOM readiness.".to_string(),
                recommendation: "Use Playwright auto-retrying assertions instead of manual timers.".to_string(),
            });
        }

        // 3. Detect Java Thread.sleep / TimeUnit sleep anti-pattern
        if let Some(caps) = RE_THREAD_SLEEP.captures(stripped_line) {
            has_wait_for_timeout = true;
            let duration_ms = caps.get(1).and_then(|m| m.as_str().parse::<u64>().ok());
            anti_patterns.push(AntiPattern {
                kind: AntiPatternKind::ThreadSleep { duration_ms },
                line: line_num,
                snippet: original_snippet.clone(),
                explanation: "Fixed Thread.sleep() makes tests brittle under asynchronous processing lag and eventual consistency.".to_string(),
                recommendation: "Replace Thread.sleep() with dynamic polling assertions using Awaitility (e.g. await().atMost(5, SECONDS).untilAsserted(...)).".to_string(),
            });
        }

        // 3. Detect getByRole (Semantic standard - 100 pts)
        for caps in RE_GET_BY_ROLE.captures_iter(stripped_line) {
            if let Some(role) = extract_string_arg(&caps, 1) {
                locators.push(LocatorOccurrence {
                    kind: LocatorKind::GetByRole,
                    selector: role.to_string(),
                    line: line_num,
                    snippet: original_snippet.clone(),
                    score: LocatorKind::GetByRole.score(),
                });
            }
        }

        // 4. Detect getByTestId (Resilient test contract - 85 pts)
        for caps in RE_GET_BY_TEST_ID.captures_iter(stripped_line) {
            if let Some(tid) = extract_string_arg(&caps, 1) {
                locators.push(LocatorOccurrence {
                    kind: LocatorKind::GetByTestId,
                    selector: tid.to_string(),
                    line: line_num,
                    snippet: original_snippet.clone(),
                    score: LocatorKind::GetByTestId.score(),
                });
            }
        }

        // 5. Detect getByText / getByLabel / getByPlaceholder (90 pts)
        for caps in RE_GET_BY_TEXT_OR_LABEL.captures_iter(stripped_line) {
            if let Some(val) = extract_string_arg(&caps, 1) {
                locators.push(LocatorOccurrence {
                    kind: LocatorKind::GetByTextOrLabel,
                    selector: val.to_string(),
                    line: line_num,
                    snippet: original_snippet.clone(),
                    score: LocatorKind::GetByTextOrLabel.score(),
                });
            }
        }

        // 6. Detect page.locator(...) / $(...) calls
        for caps in RE_LOCATOR_CALL.captures_iter(stripped_line) {
            if let Some(selector_match) = extract_string_arg(&caps, 1) {
                let sel = selector_match.trim();

                if sel.starts_with('/') || sel.starts_with("//") || sel.starts_with("xpath=") {
                    // Absolute XPath (0 pts)
                    locators.push(LocatorOccurrence {
                        kind: LocatorKind::AbsoluteXPath,
                        selector: sel.to_string(),
                        line: line_num,
                        snippet: original_snippet.clone(),
                        score: LocatorKind::AbsoluteXPath.score(),
                    });
                    anti_patterns.push(AntiPattern {
                        kind: AntiPatternKind::FragileXPath { selector: sel.to_string() },
                        line: line_num,
                        snippet: original_snippet.clone(),
                        explanation: "Absolute XPath coupling to DOM hierarchy breaks immediately upon UI layout changes.".to_string(),
                        recommendation: "Use accessible semantic locators such as page.getByRole() or resilient data-testid attributes.".to_string(),
                    });
                } else if sel.starts_with("[data-testid") || sel.starts_with("[data-test") {
                    // TestId attribute selector (85 pts)
                    locators.push(LocatorOccurrence {
                        kind: LocatorKind::GetByTestId,
                        selector: sel.to_string(),
                        line: line_num,
                        snippet: original_snippet.clone(),
                        score: LocatorKind::GetByTestId.score(),
                    });
                } else if sel.contains('.')
                    || sel.contains('#')
                    || sel.contains('>')
                    || sel.contains(' ')
                    || sel.contains(':')
                    || sel.contains("[class")
                {
                    // CSS class / ID selector (40 pts)
                    locators.push(LocatorOccurrence {
                        kind: LocatorKind::CssSelector,
                        selector: sel.to_string(),
                        line: line_num,
                        snippet: original_snippet.clone(),
                        score: LocatorKind::CssSelector.score(),
                    });
                    anti_patterns.push(AntiPattern {
                        kind: AntiPatternKind::FragileCss { selector: sel.to_string() },
                        line: line_num,
                        snippet: original_snippet.clone(),
                        explanation: "CSS class and element selectors are brittle across styling updates and framework hydration cycles.".to_string(),
                        recommendation: format!("Replace CSS locator '{}' with page.getByRole() or page.getByTestId().", sel),
                    });
                } else {
                    // Generic element selector (e.g. 'button')
                    locators.push(LocatorOccurrence {
                        kind: LocatorKind::CssSelector,
                        selector: sel.to_string(),
                        line: line_num,
                        snippet: original_snippet.clone(),
                        score: LocatorKind::CssSelector.score(),
                    });
                }
            }
        }
    }

    let locator_quality_score = if locators.is_empty() {
        100.0
    } else {
        let sum: f64 = locators.iter().map(|l| l.score).sum();
        sum / (locators.len() as f64)
    };

    StaticAnalysisReport {
        file_path: file_path.to_string(),
        total_lines: original_lines.len(),
        anti_patterns,
        locators,
        locator_quality_score,
        has_wait_for_timeout,
    }
}

/// Analyze a file on disk
pub fn analyze_file<P: AsRef<Path>>(path: P) -> Result<StaticAnalysisReport, std::io::Error> {
    let p = path.as_ref();
    let content = fs::read_to_string(p)?;
    Ok(analyze_source(&content, &p.to_string_lossy()))
}

/// Individual dimension evaluation score
#[derive(Debug, Clone, PartialEq)]
pub struct DimensionScore {
    pub name: &'static str,
    pub score: f64,
    pub weight: f64,
    pub weighted_score: f64,
    pub passed: bool,
    pub detail: String,
}

/// Comprehensive 4D Feedback Scorecard
#[derive(Debug, Clone, PartialEq)]
pub struct Scorecard {
    pub file_path: String,
    pub track_name: String,
    pub platform_version: String,
    pub total_score: f64,
    pub pass_threshold: f64,
    pub passed: bool,
    pub correctness: DimensionScore,
    pub flakiness: DimensionScore,
    pub locator_quality: DimensionScore,
    pub speed: DimensionScore,
    pub diagnostics: Vec<String>,
    pub hint: Option<String>,
}

/// Progressive Hints system loader
#[derive(Debug, Clone, Default)]
pub struct ProgressiveHints {
    pub hints: Vec<String>,
}

impl ProgressiveHints {
    /// Load progressive hints from `hints.md` located next to the exercise file
    pub fn load_from_exercise_path<P: AsRef<Path>>(exercise_path: P) -> Option<Self> {
        Self::load_from_dir(exercise_path.as_ref().parent()?)
    }

    /// Load progressive hints from the `hints.md` inside a drill directory
    pub fn load_from_dir<P: AsRef<Path>>(exercise_dir: P) -> Option<Self> {
        let hints_file = exercise_dir.as_ref().join("hints.md");
        if !hints_file.exists() {
            return None;
        }
        let content = fs::read_to_string(hints_file).ok()?;
        Some(Self::parse(&content))
    }

    /// Split a `hints.md` document into its individual hint levels.
    ///
    /// Each `## `/`### ` heading (or a bold `**Hint` marker) opens a new level. A
    /// leading `# ` document title is a preamble, not a hint — counting it would
    /// shift every level by one and hand a struggling learner nothing but a title.
    pub fn parse(content: &str) -> Self {
        fn starts_hint(line: &str) -> bool {
            line.starts_with("## ") || line.starts_with("### ") || line.starts_with("**Hint")
        }

        let mut hints = Vec::new();
        let mut current_hint = String::new();
        let mut in_hint = false;

        for line in content.lines() {
            if starts_hint(line) {
                if in_hint && !current_hint.trim().is_empty() {
                    hints.push(current_hint.trim().to_string());
                }
                current_hint.clear();
                in_hint = true;
            }
            // Lines before the first hint heading are preamble and are dropped.
            if in_hint {
                current_hint.push_str(line);
                current_hint.push('\n');
            }
        }
        if in_hint && !current_hint.trim().is_empty() {
            hints.push(current_hint.trim().to_string());
        }

        if hints.is_empty() {
            // No recognizable hint headings — fall back to the whole document.
            Self {
                hints: vec![content.trim().to_string()],
            }
        } else {
            Self { hints }
        }
    }

    /// Number of hint levels available
    pub fn len(&self) -> usize {
        self.hints.len()
    }

    /// Whether the drill has no hints at all
    pub fn is_empty(&self) -> bool {
        self.hints.is_empty()
    }

    /// Select a specific 1-based hint level, clamped to the available range.
    /// Returns `(level, total_levels, hint_text)`.
    pub fn get_hint_at_level(&self, level: usize) -> Option<(usize, usize, &str)> {
        if self.hints.is_empty() {
            return None;
        }
        let total = self.hints.len();
        let index = level.max(1).min(total) - 1;
        Some((index + 1, total, self.hints[index].as_str()))
    }

    /// Select the appropriate hint based on current total score
    pub fn get_hint_for_score(&self, total_score: f64) -> Option<(usize, usize, &str)> {
        if self.hints.is_empty() {
            return None;
        }

        let total = self.hints.len();
        let index = if total_score < 50.0 {
            0
        } else if total_score < 75.0 {
            if total >= 2 { 1 } else { 0 }
        } else if total_score < 85.0 {
            if total >= 3 {
                2
            } else {
                total.saturating_sub(1)
            }
        } else {
            0
        };

        Some((index + 1, total, self.hints[index].as_str()))
    }
}

/// Calculate wall-clock execution speed score vs baseline duration (1000ms)
pub fn calculate_speed_score(
    total_duration_ms: u64,
    iterations: u32,
    baseline_ms: u64,
) -> (f64, u64) {
    let num_iter = iterations.max(1) as u64;
    let avg_duration_ms = total_duration_ms / num_iter;

    let score = if avg_duration_ms <= baseline_ms {
        100.0
    } else {
        let diff = avg_duration_ms - baseline_ms;
        let penalty = (diff as f64) / 50.0;
        (100.0 - penalty).clamp(0.0, 100.0)
    };

    (score, avg_duration_ms)
}

/// Evaluate the 4D Feedback Matrix against test execution results and static analysis
pub fn evaluate_feedback(
    response: &DrillResponse,
    ast: &StaticAnalysisReport,
    track_name: &str,
    platform_version: &str,
    pass_threshold: f64,
    baseline_duration_ms: u64,
) -> Scorecard {
    let iterations = response.iterations.max(1);

    // 1. Correctness Dimension (0.35)
    let correctness_score = if response.passed && response.failed_iterations == 0 {
        100.0
    } else {
        (response.passed_iterations as f64 / iterations as f64) * 100.0
    };
    let correctness_detail = if response.passed {
        format!(
            "All {}/{} iterations passed",
            response.passed_iterations, iterations
        )
    } else {
        format!(
            "{}/{} passed, {} failed",
            response.passed_iterations, iterations, response.failed_iterations
        )
    };

    // 2. Flakiness Resistance Dimension (0.35)
    let raw_flakiness = (response.passed_iterations as f64 / iterations as f64) * 100.0;
    let (flakiness_score, flakiness_detail) = if ast.has_wait_for_timeout {
        let capped = raw_flakiness.min(FLAKINESS_PENALTY_CAP);
        (
            capped,
            format!(
                "{}/{} passed under chaos (Capped at {:.0} due to waitForTimeout anti-pattern)",
                response.passed_iterations, iterations, FLAKINESS_PENALTY_CAP
            ),
        )
    } else {
        (
            raw_flakiness,
            format!(
                "{}/{} passed under chaos (200ms delay + 75ms jitter)",
                response.passed_iterations, iterations
            ),
        )
    };

    // 3. Locator Quality Dimension (0.15)
    let locator_score = ast.locator_quality_score;
    let locator_detail = if ast.locators.is_empty() {
        "No locators detected in source (Default: 100)".to_string()
    } else {
        let roles = ast
            .locators
            .iter()
            .filter(|l| l.kind == LocatorKind::GetByRole)
            .count();
        let test_ids = ast
            .locators
            .iter()
            .filter(|l| l.kind == LocatorKind::GetByTestId)
            .count();
        let css = ast
            .locators
            .iter()
            .filter(|l| l.kind == LocatorKind::CssSelector)
            .count();
        let xpath = ast
            .locators
            .iter()
            .filter(|l| l.kind == LocatorKind::AbsoluteXPath)
            .count();
        format!(
            "{} locators ({} getByRole, {} testId, {} CSS, {} XPath)",
            ast.locators.len(),
            roles,
            test_ids,
            css,
            xpath
        )
    };

    // 4. Execution Speed Dimension (0.15)
    let (speed_score, avg_duration_ms) =
        calculate_speed_score(response.total_duration_ms, iterations, baseline_duration_ms);
    let speed_detail = format!(
        "{}ms total (avg: {}ms/iter, baseline: {}ms)",
        response.total_duration_ms, avg_duration_ms, baseline_duration_ms
    );

    // Total Composite Score
    let weighted_c = correctness_score * WEIGHT_CORRECTNESS;
    let weighted_f = flakiness_score * WEIGHT_FLAKINESS;
    let weighted_l = locator_score * WEIGHT_LOCATOR_QUALITY;
    let weighted_s = speed_score * WEIGHT_SPEED;
    let total_score = weighted_c + weighted_f + weighted_l + weighted_s;

    let passed = total_score >= pass_threshold && response.passed && !ast.has_wait_for_timeout;

    // Diagnostics & Root Causes
    let mut diagnostics = Vec::new();
    for ap in &ast.anti_patterns {
        match &ap.kind {
            AntiPatternKind::WaitForTimeout { duration_ms } => {
                let ms_str = duration_ms
                    .map(|d| format!("({}ms)", d))
                    .unwrap_or_default();
                diagnostics.push(format!(
                    "✗ Anti-pattern: page.waitForTimeout{} on line {}\n  → {}\n  → Recommendation: {}",
                    ms_str, ap.line, ap.explanation, ap.recommendation
                ));
            }
            AntiPatternKind::HardcodedSleep { duration_ms } => {
                let ms_str = duration_ms
                    .map(|d| format!("({}ms)", d))
                    .unwrap_or_default();
                diagnostics.push(format!(
                    "✗ Anti-pattern: setTimeout{} on line {}\n  → {}\n  → Recommendation: {}",
                    ms_str, ap.line, ap.explanation, ap.recommendation
                ));
            }
            AntiPatternKind::ThreadSleep { duration_ms } => {
                let ms_str = duration_ms
                    .map(|d| format!("({}ms)", d))
                    .unwrap_or_default();
                diagnostics.push(format!(
                    "✗ Anti-pattern: Thread.sleep{} on line {}\n  → {}\n  → Recommendation: {}",
                    ms_str, ap.line, ap.explanation, ap.recommendation
                ));
            }
            AntiPatternKind::FragileXPath { selector } => {
                diagnostics.push(format!(
                    "✗ Fragile XPath: '{}' on line {}\n  → {}\n  → Recommendation: {}",
                    selector, ap.line, ap.explanation, ap.recommendation
                ));
            }
            AntiPatternKind::FragileCss { selector } => {
                diagnostics.push(format!(
                    "⚠ Fragile CSS locator: '{}' on line {}\n  → {}\n  → Recommendation: {}",
                    selector, ap.line, ap.explanation, ap.recommendation
                ));
            }
            AntiPatternKind::MissingWhenCondition { flow_type } => {
                diagnostics.push(format!(
                    "✗ Anti-pattern: Missing 'when:' condition ({}) on line {}\n  → {}\n  → Recommendation: {}",
                    flow_type, ap.line, ap.explanation, ap.recommendation
                ));
            }
            AntiPatternKind::MissingColdStartDeepLink { command } => {
                diagnostics.push(format!(
                    "✗ Anti-pattern: Cold-start deep link issue with '{}' on line {}\n  → {}\n  → Recommendation: {}",
                    command, ap.line, ap.explanation, ap.recommendation
                ));
            }
            AntiPatternKind::MissingActivityRecreation { state } => {
                diagnostics.push(format!(
                    "✗ Anti-pattern: Missing activity recreation check ({}) on line {}\n  → {}\n  → Recommendation: {}",
                    state, ap.line, ap.explanation, ap.recommendation
                ));
            }
            AntiPatternKind::UnconditionalFallbackAssert { selector } => {
                diagnostics.push(format!(
                    "✗ Anti-pattern: Unconditional fallback assert '{}' on line {}\n  → {}\n  → Recommendation: {}",
                    selector, ap.line, ap.explanation, ap.recommendation
                ));
            }
        }
    }

    if let Some(ref err) = response.error {
        diagnostics.push(format!("✗ Runner Error: {}", err));
    }

    // Load progressive hint if available
    let hint = if let Some(ph) = ProgressiveHints::load_from_exercise_path(&ast.file_path) {
        ph.get_hint_for_score(total_score)
            .map(|(num, total, text)| format!("💡 HINT ({}/{}):\n{}", num, total, text))
    } else if !ast.anti_patterns.is_empty() {
        let first_ap = &ast.anti_patterns[0];
        Some(format!(
            "💡 HINT (1/1):\n{}\nReplace with: {}",
            first_ap.explanation, first_ap.recommendation
        ))
    } else {
        None
    };

    Scorecard {
        file_path: ast.file_path.clone(),
        track_name: track_name.to_string(),
        platform_version: platform_version.to_string(),
        total_score,
        pass_threshold,
        passed,
        correctness: DimensionScore {
            name: "Correctness",
            score: correctness_score,
            weight: WEIGHT_CORRECTNESS,
            weighted_score: weighted_c,
            passed: correctness_score >= 100.0,
            detail: correctness_detail,
        },
        flakiness: DimensionScore {
            name: "Flakiness Resistance",
            score: flakiness_score,
            weight: WEIGHT_FLAKINESS,
            weighted_score: weighted_f,
            passed: flakiness_score >= 80.0,
            detail: flakiness_detail,
        },
        locator_quality: DimensionScore {
            name: "Locator Quality",
            score: locator_score,
            weight: WEIGHT_LOCATOR_QUALITY,
            weighted_score: weighted_l,
            passed: locator_score >= 80.0,
            detail: locator_detail,
        },
        speed: DimensionScore {
            name: "Execution Speed",
            score: speed_score,
            weight: WEIGHT_SPEED,
            weighted_score: weighted_s,
            passed: speed_score >= 80.0,
            detail: speed_detail,
        },
        diagnostics,
        hint,
    }
}

/// Render an ANSI progress bar (e.g. `[████████░░]`)
pub fn render_progress_bar(score: f64, width: usize) -> String {
    let clamped = score.clamp(0.0, 100.0);
    let filled = ((clamped / 100.0) * (width as f64)).round() as usize;
    let empty = width.saturating_sub(filled);

    let filled_str = "█".repeat(filled);
    let empty_str = "░".repeat(empty);

    if score >= 85.0 {
        format!("[{}{}]", filled_str.green(), empty_str.dimmed())
    } else if score >= 50.0 {
        format!("[{}{}]", filled_str.yellow(), empty_str.dimmed())
    } else {
        format!("[{}{}]", filled_str.red(), empty_str.dimmed())
    }
}

/// Render the complete ANSI Cherenkov Blue terminal scorecard
pub fn render_scorecard(card: &Scorecard) -> String {
    let mut out = String::new();
    let border =
        "========================================================================================"
            .cyan();

    out.push_str(&format!("{}\n", border));
    out.push_str(&format!(
        " {} v{}  |  Track: [{}]\n",
        "CHERENKOV-LINGS".bold().bright_cyan(),
        card.platform_version,
        card.track_name.bright_yellow()
    ));
    out.push_str(&format!(" Drill: {}\n", card.file_path.bright_white()));
    out.push_str(&format!("{}\n", border));

    // Status Banner
    if card.passed {
        out.push_str(&format!(
            " {} {}\n\n",
            "[STATUS]:".bold(),
            format!(
                "✓ PASSED (Score: {:.1} / 100.0 - Threshold: {:.1})",
                card.total_score, card.pass_threshold
            )
            .bold()
            .green()
        ));
    } else {
        out.push_str(&format!(
            " {} {}\n\n",
            "[STATUS]:".bold(),
            format!(
                "❌ REJECTED (Score: {:.1} / 100.0 - Threshold: {:.1})",
                card.total_score, card.pass_threshold
            )
            .bold()
            .red()
        ));
    }

    // Dimension Scores
    out.push_str(&format!(" {}\n", "DIMENSION SCORES:".bold().bright_cyan()));

    let dimensions = [
        (&card.correctness, "35%"),
        (&card.flakiness, "35%"),
        (&card.locator_quality, "15%"),
        (&card.speed, "15%"),
    ];

    for (dim, weight_str) in dimensions {
        let mark = if dim.passed {
            "✓".green()
        } else {
            "✗".red()
        };
        let bar = render_progress_bar(dim.score, 10);
        let score_fmt = format!("{:5.1} / 100", dim.score);

        out.push_str(&format!(
            " {} {:<21} {} {} (Weight: {})  {}\n",
            mark,
            dim.name.bright_white(),
            bar,
            score_fmt.bold(),
            weight_str.dimmed(),
            dim.detail
        ));
    }

    // Diagnostics & Root Causes
    if !card.diagnostics.is_empty() {
        out.push_str(&format!(
            "\n {}\n",
            "DIAGNOSTICS & ROOT CAUSE:".bold().bright_yellow()
        ));
        for diag in &card.diagnostics {
            out.push_str(&format!(" {}\n", diag));
        }
    }

    // Progressive Hints
    if let Some(ref hint) = card.hint {
        out.push_str(&format!("\n {}\n", hint.bright_cyan()));
    }

    out.push_str(&format!("{}\n", border));
    out
}

/// Render standalone diagnostic view (for `cherenkov-lings diagnose`)
pub fn render_diagnostic(
    ast: &StaticAnalysisReport,
    track_name: &str,
    platform_version: &str,
) -> String {
    let mut out = String::new();
    let border =
        "========================================================================================"
            .cyan();

    out.push_str(&format!("{}\n", border));
    out.push_str(&format!(
        " {} v{}  |  Track: [{}]\n",
        "CHERENKOV-LINGS DIAGNOSTIC".bold().bright_cyan(),
        platform_version,
        track_name.bright_yellow()
    ));
    out.push_str(&format!(" Target File: {}\n", ast.file_path.bright_white()));
    out.push_str(&format!("{}\n", border));

    // Locator Quality Breakdown
    out.push_str(&format!(
        " {}\n",
        "STATIC SOURCE ANALYSIS:".bold().bright_cyan()
    ));
    let bar = render_progress_bar(ast.locator_quality_score, 10);
    out.push_str(&format!(
        "   Locator Quality Score: {} {:5.1} / 100  ({} locators analyzed)\n\n",
        bar,
        ast.locator_quality_score,
        ast.locators.len()
    ));

    if !ast.locators.is_empty() {
        out.push_str(&format!("   {}\n", "Discovered Locators:".underline()));
        for loc in &ast.locators {
            let badge = match loc.kind {
                LocatorKind::GetByRole => "[getByRole (100)]".green(),
                LocatorKind::GetByTextOrLabel => "[text/label (90)]".green(),
                LocatorKind::GetByTestId => "[getByTestId (85)]".cyan(),
                LocatorKind::CssSelector => "[CSS class (40)]".yellow(),
                LocatorKind::AbsoluteXPath => "[XPath (0)]".red(),
            };
            out.push_str(&format!(
                "   - Line {:2}: {:<22} selector: '{}'\n     Snippet: {}\n",
                loc.line,
                badge,
                loc.selector,
                loc.snippet.dimmed()
            ));
        }
        out.push('\n');
    }

    // Detected Anti-patterns
    if ast.anti_patterns.is_empty() {
        out.push_str(&format!(
            " {}\n",
            "✓ No anti-patterns detected.".green().bold()
        ));
    } else {
        out.push_str(&format!(
            " {}\n",
            "DETECTED ANTI-PATTERNS & ROOT CAUSES:"
                .bold()
                .bright_yellow()
        ));
        for ap in &ast.anti_patterns {
            out.push_str(&format!(
                "   ✗ Line {:2}: {}\n     Root Cause: {}\n     Recommendation: {}\n\n",
                ap.line,
                ap.snippet.bright_red(),
                ap.explanation,
                ap.recommendation.bright_green()
            ));
        }
    }

    // Progressive Hints
    if let Some(ph) = ProgressiveHints::load_from_exercise_path(&ast.file_path)
        && let Some((idx, total, text)) = ph.get_hint_for_score(ast.locator_quality_score)
    {
        out.push_str(&format!(
            " {}\n{}\n",
            format!("💡 PROGRESSIVE HINTS ({}/{}):", idx, total)
                .bright_cyan()
                .bold(),
            text
        ));
    }

    out.push_str(&format!("{}\n", border));
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::runner::RunResult;

    #[test]
    fn test_comment_stripper_preserves_line_numbers_and_strings() {
        let ts_code = r#"
// This is a single-line comment on line 2
const url = "http://localhost:8080/checkout"; // Trailing comment
/*
  Multi-line block comment
  Line 6
*/
await page.waitForTimeout(2000);
"#;
        let stripped = strip_comments(ts_code);
        let orig_lines: Vec<&str> = ts_code.lines().collect();
        let stripped_lines: Vec<&str> = stripped.lines().collect();

        assert_eq!(orig_lines.len(), stripped_lines.len());
        assert!(!stripped.contains("This is a single-line comment"));
        assert!(!stripped.contains("Multi-line block comment"));
        // URL with // inside quotes must be preserved
        assert!(stripped.contains("http://localhost:8080/checkout"));
        assert!(stripped.contains("waitForTimeout(2000)"));
    }

    #[test]
    fn test_ast_analyzer_wait_for_timeout_detection() {
        let ts_code = r#"
import { test, expect } from '@playwright/test';

test('checkout hydration timing', async ({ page }) => {
    await page.goto('http://localhost:8080/checkout');
    // TODO: Fix the anti-pattern below
    await page.waitForTimeout(2000);
    await page.locator('.btn-checkout').click();
});
"#;
        let report = analyze_source(ts_code, "exercises/01_hydration/exercise.ts");
        assert!(report.has_wait_for_timeout);
        assert_eq!(report.anti_patterns.len(), 2); // waitForTimeout and CSS selector

        let timeout_ap = report
            .anti_patterns
            .iter()
            .find(|ap| matches!(ap.kind, AntiPatternKind::WaitForTimeout { .. }))
            .expect("Found waitForTimeout");
        assert_eq!(timeout_ap.line, 7);
        assert!(timeout_ap.snippet.contains("waitForTimeout(2000)"));
        assert!(timeout_ap.explanation.contains("timing-dependent"));
    }

    #[test]
    fn test_ast_analyzer_locator_quality_scoring() {
        // 1. Semantic getByRole (100)
        let role_code = "await page.getByRole('button', { name: 'Submit' }).click();";
        let r_role = analyze_source(role_code, "test.ts");
        assert_eq!(r_role.locators.len(), 1);
        assert_eq!(r_role.locators[0].kind, LocatorKind::GetByRole);
        assert_eq!(r_role.locator_quality_score, 100.0);

        // 2. getByTestId (85)
        let tid_code = "await page.getByTestId('order-submit').click();";
        let r_tid = analyze_source(tid_code, "test.ts");
        assert_eq!(r_tid.locators.len(), 1);
        assert_eq!(r_tid.locators[0].kind, LocatorKind::GetByTestId);
        assert_eq!(r_tid.locator_quality_score, 85.0);

        // 3. getByText / getByLabel (90)
        let text_code = "await page.getByText('Welcome').toBeVisible();";
        let r_text = analyze_source(text_code, "test.ts");
        assert_eq!(r_text.locators.len(), 1);
        assert_eq!(r_text.locators[0].kind, LocatorKind::GetByTextOrLabel);
        assert_eq!(r_text.locator_quality_score, 90.0);

        // 4. CSS Class selector (40)
        let css_code = "await page.locator('.btn-primary').click();";
        let r_css = analyze_source(css_code, "test.ts");
        assert_eq!(r_css.locators.len(), 1);
        assert_eq!(r_css.locators[0].kind, LocatorKind::CssSelector);
        assert_eq!(r_css.locator_quality_score, 40.0);

        // 5. Absolute XPath (0)
        let xpath_code = "await page.locator('/html/body/div[1]/button').click();";
        let r_xpath = analyze_source(xpath_code, "test.ts");
        assert_eq!(r_xpath.locators.len(), 1);
        assert_eq!(r_xpath.locators[0].kind, LocatorKind::AbsoluteXPath);
        assert_eq!(r_xpath.locator_quality_score, 0.0);

        // 6. Mixed locators (Role 100 + CSS 40 = 70.0 avg)
        let mixed_code = r#"
            await page.getByRole('button', { name: 'OK' }).click();
            await page.locator('.modal-close').click();
        "#;
        let r_mixed = analyze_source(mixed_code, "test.ts");
        assert_eq!(r_mixed.locators.len(), 2);
        assert_eq!(r_mixed.locator_quality_score, 70.0);
    }

    #[test]
    fn test_speed_scoring_baseline_and_penalties() {
        // Fast execution (500ms avg, baseline 1000ms) -> 100.0
        let (s1, avg1) = calculate_speed_score(2500, 5, 1000);
        assert_eq!(avg1, 500);
        assert_eq!(s1, 100.0);

        // Baseline execution (1000ms avg) -> 100.0
        let (s2, avg2) = calculate_speed_score(5000, 5, 1000);
        assert_eq!(avg2, 1000);
        assert_eq!(s2, 100.0);

        // Delayed execution (1500ms avg -> penalty 500/50 = 10 -> score 90.0)
        let (s3, avg3) = calculate_speed_score(7500, 5, 1000);
        assert_eq!(avg3, 1500);
        assert_eq!(s3, 90.0);

        // Slow execution (2450ms avg -> penalty 1450/50 = 29 -> score 71.0)
        let (s4, avg4) = calculate_speed_score(12250, 5, 1000);
        assert_eq!(avg4, 2450);
        assert_eq!(s4, 71.0);

        // Very slow execution (7000ms avg -> penalty 6000/50 = 120 -> clamped to 0.0)
        let (s5, avg5) = calculate_speed_score(35000, 5, 1000);
        assert_eq!(avg5, 7000);
        assert_eq!(s5, 0.0);
    }

    #[test]
    fn test_flakiness_and_wait_for_timeout_penalty() {
        let drill_response = DrillResponse {
            id: "req-1".to_string(),
            ok: true,
            passed: true,
            iterations: 5,
            passed_iterations: 5,
            failed_iterations: 0,
            total_duration_ms: 2500,
            runs: vec![
                RunResult {
                    iteration: 1,
                    passed: true,
                    duration_ms: 500,
                    error: None,
                },
                RunResult {
                    iteration: 2,
                    passed: true,
                    duration_ms: 500,
                    error: None,
                },
                RunResult {
                    iteration: 3,
                    passed: true,
                    duration_ms: 500,
                    error: None,
                },
                RunResult {
                    iteration: 4,
                    passed: true,
                    duration_ms: 500,
                    error: None,
                },
                RunResult {
                    iteration: 5,
                    passed: true,
                    duration_ms: 500,
                    error: None,
                },
            ],
            error: None,
        };

        // When waitForTimeout is present, flakiness score is capped at 40.0
        let ast_with_sleep = StaticAnalysisReport {
            file_path: "exercise.ts".to_string(),
            has_wait_for_timeout: true,
            locator_quality_score: 100.0,
            ..Default::default()
        };

        let card_flaky = evaluate_feedback(
            &drill_response,
            &ast_with_sleep,
            "playwright-ts",
            "1.0.0",
            85.0,
            1000,
        );
        assert_eq!(card_flaky.flakiness.score, 40.0);
        assert!(!card_flaky.passed);

        // Without waitForTimeout, flakiness score is 100.0
        let ast_clean = StaticAnalysisReport {
            file_path: "solution.ts".to_string(),
            has_wait_for_timeout: false,
            locator_quality_score: 100.0,
            ..Default::default()
        };

        let card_clean = evaluate_feedback(
            &drill_response,
            &ast_clean,
            "playwright-ts",
            "1.0.0",
            85.0,
            1000,
        );
        assert_eq!(card_clean.flakiness.score, 100.0);
        assert_eq!(card_clean.total_score, 100.0);
        assert!(card_clean.passed);
    }

    #[test]
    fn test_composite_4d_scorecard_calculation_and_rendering() {
        let drill_response = DrillResponse {
            id: "req-101".to_string(),
            ok: true,
            passed: false,
            iterations: 5,
            passed_iterations: 1,
            failed_iterations: 4,
            total_duration_ms: 12250,
            runs: vec![
                RunResult {
                    iteration: 1,
                    passed: false,
                    duration_ms: 2450,
                    error: Some("Click dropped".into()),
                },
                RunResult {
                    iteration: 2,
                    passed: true,
                    duration_ms: 2450,
                    error: None,
                },
                RunResult {
                    iteration: 3,
                    passed: false,
                    duration_ms: 2450,
                    error: Some("Click dropped".into()),
                },
                RunResult {
                    iteration: 4,
                    passed: false,
                    duration_ms: 2450,
                    error: Some("Click dropped".into()),
                },
                RunResult {
                    iteration: 5,
                    passed: false,
                    duration_ms: 2450,
                    error: Some("Click dropped".into()),
                },
            ],
            error: None,
        };

        let ast = StaticAnalysisReport {
            file_path: "exercises/01_hydration/exercise.ts".to_string(),
            total_lines: 15,
            anti_patterns: vec![AntiPattern {
                kind: AntiPatternKind::WaitForTimeout {
                    duration_ms: Some(2000),
                },
                line: 10,
                snippet: "await page.waitForTimeout(2000);".to_string(),
                explanation: "Fixed sleep causes click drops".to_string(),
                recommendation: "Use toBeEnabled()".to_string(),
            }],
            locators: vec![LocatorOccurrence {
                kind: LocatorKind::CssSelector,
                selector: ".btn-checkout".to_string(),
                line: 12,
                snippet: "await page.locator('.btn-checkout').click();".to_string(),
                score: 40.0,
            }],
            locator_quality_score: 40.0,
            has_wait_for_timeout: true,
        };

        let card = evaluate_feedback(
            &drill_response,
            &ast,
            "Modern Web Automation",
            "1.0.0",
            85.0,
            1000,
        );

        // Correctness: 1/5 = 20.0 (35% -> 7.0)
        assert_eq!(card.correctness.score, 20.0);
        assert_eq!(card.correctness.weighted_score, 7.0);

        // Flakiness: 1/5 = 20.0, capped at 40 -> 20.0 (35% -> 7.0)
        assert_eq!(card.flakiness.score, 20.0);
        assert_eq!(card.flakiness.weighted_score, 7.0);

        // Locator Quality: 40.0 (15% -> 6.0)
        assert_eq!(card.locator_quality.score, 40.0);
        assert_eq!(card.locator_quality.weighted_score, 6.0);

        // Speed: 2450ms avg -> 71.0 (15% -> 10.65)
        assert_eq!(card.speed.score, 71.0);
        assert_eq!(card.speed.weighted_score, 10.65);

        // Total: 7.0 + 7.0 + 6.0 + 10.65 = 30.65
        assert!((card.total_score - 30.65).abs() < 0.001);
        assert!(!card.passed);

        let rendered = render_scorecard(&card);
        assert!(rendered.contains("CHERENKOV-LINGS"));
        assert!(rendered.contains("REJECTED"));
        assert!(rendered.contains("Correctness"));
        assert!(rendered.contains("Flakiness Resistance"));
        assert!(rendered.contains("Locator Quality"));
        assert!(rendered.contains("Execution Speed"));
        assert!(rendered.contains("waitForTimeout(2000ms)"));
    }

    #[test]
    fn test_progressive_hints_selection_thresholds() {
        let hints = ProgressiveHints {
            hints: vec![
                "Hint 1: Architectural nudge".to_string(),
                "Hint 2: API pattern".to_string(),
                "Hint 3: Code diff".to_string(),
            ],
        };

        // Score < 50 -> Hint 1
        let (idx1, tot1, h1) = hints.get_hint_for_score(30.0).expect("Got hint 1");
        assert_eq!(idx1, 1);
        assert_eq!(tot1, 3);
        assert!(h1.contains("Architectural nudge"));

        // 50 <= Score < 75 -> Hint 2
        let (idx2, tot2, h2) = hints.get_hint_for_score(65.0).expect("Got hint 2");
        assert_eq!(idx2, 2);
        assert_eq!(tot2, 3);
        assert!(h2.contains("API pattern"));

        // 75 <= Score < 85 -> Hint 3
        let (idx3, tot3, h3) = hints.get_hint_for_score(80.0).expect("Got hint 3");
        assert_eq!(idx3, 3);
        assert_eq!(tot3, 3);
        assert!(h3.contains("Code diff"));
    }

    /// The `# Hints: ...` document title must not become hint level 1, or a
    /// struggling learner is handed a bare title instead of a nudge.
    #[test]
    fn test_progressive_hints_parse_drops_document_title() {
        let doc = "# Hints: Drill 01 - Hydration Timing\n\
                   \n\
                   ## Hint 1 (Architectural Nudge)\n\
                   Hydration attaches listeners after HTML streams in.\n\
                   \n\
                   ## Hint 2 (API Pattern)\n\
                   Use an auto-retrying assertion on `data-hydrated`.\n\
                   \n\
                   ## Hint 3 (Code Diff)\n\
                   ```diff\n\
                   - await page.waitForTimeout(200);\n\
                   ```\n";

        let hints = ProgressiveHints::parse(doc);
        assert_eq!(hints.len(), 3, "title must not count as a hint level");
        assert!(hints.hints[0].starts_with("## Hint 1"));
        assert!(!hints.hints[0].contains("Hints: Drill 01"));
        assert!(hints.hints[2].contains("waitForTimeout"));

        // A struggling learner gets the conceptual nudge, not the title.
        let (idx, total, text) = hints.get_hint_for_score(10.0).expect("hint for low score");
        assert_eq!((idx, total), (1, 3));
        assert!(text.contains("Hydration attaches listeners"));
    }

    #[test]
    fn test_get_hint_at_level_clamps_and_falls_back() {
        let hints = ProgressiveHints {
            hints: vec!["one".to_string(), "two".to_string()],
        };

        assert_eq!(hints.get_hint_at_level(1).expect("level 1").2, "one");
        assert_eq!(hints.get_hint_at_level(2).expect("level 2").2, "two");
        // Out-of-range levels clamp rather than erroring.
        assert_eq!(hints.get_hint_at_level(0).expect("clamped low").2, "one");
        assert_eq!(hints.get_hint_at_level(99).expect("clamped high").2, "two");

        assert!(ProgressiveHints::default().get_hint_at_level(1).is_none());
    }

    /// Every shipped drill must expose more than one hint level, otherwise the
    /// first request would hand over the solution diff.
    #[test]
    fn test_every_drill_hints_file_has_multiple_levels() {
        fn collect(dir: &Path, out: &mut Vec<std::path::PathBuf>) {
            let Ok(entries) = fs::read_dir(dir) else {
                return;
            };
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_dir() {
                    collect(&path, out);
                } else if path.file_name().is_some_and(|n| n == "hints.md") {
                    out.push(path);
                }
            }
        }

        // Scoped to the drill directories lings.toml declares, which is what
        // "every shipped drill" means. Walking all of `exercises/` also swept up
        // scaffolding that no track declares, so an unreferenced directory could
        // fail this test while telling you nothing about the curriculum. Stray
        // directories are the audit's business; `cherenkov-lings audit` lists
        // them under "not declared in lings.toml".
        //
        // The manifest is read as text rather than through `crate::config`
        // because several integration tests pull this module in by path, where
        // that module is not in scope.
        let manifest = fs::read_to_string("lings.toml").expect("lings.toml must be readable");
        let mut roots: Vec<String> = Vec::new();
        let mut current_root: Option<String> = None;
        let mut drill_dirs: Vec<std::path::PathBuf> = Vec::new();

        for line in manifest.lines() {
            let line = line.trim();
            if let Some(value) = line.strip_prefix("exercise_dir = ") {
                current_root = Some(value.trim_matches('"').to_string());
            } else if let Some(value) = line.strip_prefix("drill_root = ") {
                current_root = Some(value.trim_matches('"').to_string());
            } else if line == "[[tracks.drills]]" {
                roots.push(
                    current_root
                        .clone()
                        .expect("drill declared before its track"),
                );
            } else if let Some(value) = line.strip_prefix("id = ") {
                // Drill ids follow their `[[tracks.drills]]` marker, so a pending
                // root means this id belongs to a drill rather than a track.
                if roots.len() > drill_dirs.len() {
                    let root = &roots[drill_dirs.len()];
                    drill_dirs.push(Path::new(root).join(value.trim_matches('"')));
                }
            }
        }

        assert!(!drill_dirs.is_empty(), "lings.toml declares no drills");

        let mut files = Vec::new();
        for dir in &drill_dirs {
            collect(dir, &mut files);
        }
        assert_eq!(
            files.len(),
            drill_dirs.len(),
            "expected one hints.md per drill declared in lings.toml"
        );

        for file in files {
            let dir = file.parent().expect("hints.md has a parent");
            let hints = ProgressiveHints::load_from_dir(dir).expect("hints load");
            assert!(
                hints.len() >= 2,
                "{} parsed into {} level(s); the first level would leak the solution",
                file.display(),
                hints.len()
            );
            let (level, _, text) = hints.get_hint_at_level(1).expect("level 1");
            assert_eq!(level, 1);
            assert!(
                !text.starts_with("# Hints:"),
                "{} exposes its document title as hint level 1",
                file.display()
            );
        }
    }

    /// A document with no `##` hint headings still yields something usable.
    #[test]
    fn test_progressive_hints_parse_without_headings() {
        let hints = ProgressiveHints::parse("Just some freeform guidance.\n");
        assert_eq!(hints.len(), 1);
        assert!(hints.hints[0].contains("freeform guidance"));
    }

    #[test]
    fn test_all_locator_kinds_and_data_testid_variations() {
        let code = r#"
            await page.getByRole('button', { name: 'Submit' }).click();
            await page.getByTestId('order-id').toBeVisible();
            await page.locator('[data-testid="order-row"]').click();
            await page.locator('[data-test="user-profile"]').click();
            await page.getByText('Success').toBeVisible();
            await page.getByLabel('Username').fill('user');
            await page.locator('#main-container').click();
            await page.locator('//button[@type="submit"]').click();
        "#;
        let report = analyze_source(code, "test.ts");

        // getByRole: 100
        // getByTestId: 85
        // [data-testid=...]: 85
        // [data-test=...]: 85
        // getByText: 90
        // getByLabel: 90
        // #main-container: 40
        // //button...: 0
        assert_eq!(report.locators.len(), 8);
        assert_eq!(report.anti_patterns.len(), 2); // 1 CSS, 1 XPath

        let expected_sum = 100.0 + 85.0 + 85.0 + 85.0 + 90.0 + 90.0 + 40.0 + 0.0;
        let expected_avg = expected_sum / 8.0;
        assert!((report.locator_quality_score - expected_avg).abs() < 0.001);
    }

    #[test]
    fn test_diagnostic_view_rendering() {
        let ast = StaticAnalysisReport {
            file_path: "exercises/01_web_playwright_ts/01_hydration/exercise.ts".to_string(),
            total_lines: 20,
            anti_patterns: vec![AntiPattern {
                kind: AntiPatternKind::WaitForTimeout {
                    duration_ms: Some(2000),
                },
                line: 12,
                snippet: "await page.waitForTimeout(2000);".to_string(),
                explanation: "Fixed sleep makes the test timing-dependent.".to_string(),
                recommendation: "Use event-driven assertions.".to_string(),
            }],
            locators: vec![LocatorOccurrence {
                kind: LocatorKind::CssSelector,
                selector: ".submit-btn".to_string(),
                line: 14,
                snippet: "await page.locator('.submit-btn').click();".to_string(),
                score: 40.0,
            }],
            locator_quality_score: 40.0,
            has_wait_for_timeout: true,
        };

        let diag = render_diagnostic(
            &ast,
            "Modern Web Automation (Playwright TypeScript)",
            "1.0.0",
        );
        assert!(diag.contains("CHERENKOV-LINGS DIAGNOSTIC"));
        assert!(diag.contains("Target File:"));
        assert!(diag.contains("STATIC SOURCE ANALYSIS"));
        assert!(diag.contains("DETECTED ANTI-PATTERNS"));
        assert!(diag.contains("waitForTimeout(2000)"));
    }

    #[test]
    fn test_ast_analyzer_java_thread_sleep_detection() {
        let java_code = r#"
package com.cherenkov.drill03_kafka_lag;

import org.junit.jupiter.api.Test;

public class Exercise {
    @Test
    void testTransfer() throws InterruptedException {
        // Step 1: Queue transfer
        doTransfer();
        // ANTI-PATTERN: Brittle sleep
        Thread.sleep(100);
        assertBalance();
    }
}
"#;
        let report = analyze_source(
            java_code,
            "exercises/02_api_restassured_java/src/test/java/com/cherenkov/drill03_kafka_lag/Exercise.java",
        );
        assert!(report.has_wait_for_timeout);
        assert_eq!(report.anti_patterns.len(), 1);

        let sleep_ap = &report.anti_patterns[0];
        assert_eq!(
            sleep_ap.kind,
            AntiPatternKind::ThreadSleep {
                duration_ms: Some(100)
            }
        );
        assert_eq!(sleep_ap.line, 12);
        assert!(sleep_ap.snippet.contains("Thread.sleep(100)"));
        assert!(
            sleep_ap
                .explanation
                .contains("brittle under asynchronous processing lag")
        );
        assert!(sleep_ap.recommendation.contains("Awaitility"));
    }

    #[test]
    fn test_ast_analyzer_java_timeunit_and_fq_sleep() {
        let code = r#"
            TimeUnit.SECONDS.sleep(5);
            java.lang.Thread.sleep(250);
        "#;
        let report = analyze_source(code, "Test.java");
        assert!(report.has_wait_for_timeout);
        assert_eq!(report.anti_patterns.len(), 2);
    }

    #[test]
    fn test_java_thread_sleep_flakiness_penalty_cap_at_40() {
        let drill_response = DrillResponse {
            id: "jvm-req-1".to_string(),
            ok: true,
            passed: true,
            iterations: 5,
            passed_iterations: 5,
            failed_iterations: 0,
            total_duration_ms: 2500,
            runs: vec![
                RunResult {
                    iteration: 1,
                    passed: true,
                    duration_ms: 500,
                    error: None,
                },
                RunResult {
                    iteration: 2,
                    passed: true,
                    duration_ms: 500,
                    error: None,
                },
                RunResult {
                    iteration: 3,
                    passed: true,
                    duration_ms: 500,
                    error: None,
                },
                RunResult {
                    iteration: 4,
                    passed: true,
                    duration_ms: 500,
                    error: None,
                },
                RunResult {
                    iteration: 5,
                    passed: true,
                    duration_ms: 500,
                    error: None,
                },
            ],
            error: None,
        };

        // When Thread.sleep is present, flakiness score is capped at 40.0
        let ast_with_sleep = StaticAnalysisReport {
            file_path: "Exercise.java".to_string(),
            has_wait_for_timeout: true,
            anti_patterns: vec![AntiPattern {
                kind: AntiPatternKind::ThreadSleep {
                    duration_ms: Some(100),
                },
                line: 10,
                snippet: "Thread.sleep(100);".to_string(),
                explanation: "Fixed sleep".to_string(),
                recommendation: "Use Awaitility".to_string(),
            }],
            locator_quality_score: 100.0,
            ..Default::default()
        };

        let card_flaky = evaluate_feedback(
            &drill_response,
            &ast_with_sleep,
            "API Resilience & Security (REST Assured Java)",
            "1.0.0",
            85.0,
            1000,
        );
        assert_eq!(card_flaky.flakiness.score, 40.0);
        assert!(!card_flaky.passed);
        assert!(
            card_flaky
                .diagnostics
                .iter()
                .any(|d| d.contains("Thread.sleep(100ms)"))
        );

        // When clean (e.g. Solution.java with Awaitility), score is 100.0
        let ast_clean = StaticAnalysisReport {
            file_path: "Solution.java".to_string(),
            has_wait_for_timeout: false,
            locator_quality_score: 100.0,
            ..Default::default()
        };

        let card_clean = evaluate_feedback(
            &drill_response,
            &ast_clean,
            "API Resilience & Security (REST Assured Java)",
            "1.0.0",
            85.0,
            1000,
        );
        assert_eq!(card_clean.flakiness.score, 100.0);
        assert_eq!(card_clean.total_score, 100.0);
        assert!(card_clean.passed);
    }

    #[test]
    fn test_strip_yaml_comments() {
        let yaml = r#"
# Header comment
- launchApp:
    appId: com.cherenkov.bankapp # inline comment
    arguments:
      deeplink: 'cherenkov://account#123' # contains # inside quotes
"#;
        let stripped = strip_yaml_comments(yaml);
        assert!(!stripped.contains("Header comment"));
        assert!(!stripped.contains("inline comment"));
        assert!(stripped.contains("cherenkov://account#123"));
        assert!(stripped.contains("launchApp:"));
    }

    #[test]
    fn test_maestro_drill_01_biometric_fallback_detection() {
        let ex_path = "exercises/03_mobile_maestro/01_biometric_fallback/exercise.yaml";
        let sol_path = "exercises/03_mobile_maestro/01_biometric_fallback/solution.yaml";

        let ex_yaml = r#"
---
- launchApp:
    appId: com.cherenkov.bankapp
- tapOn:
    text: Login with Biometric
- assertVisible:
    text: Welcome, SDET Engineer
"#;
        let report_ex = analyze_source(ex_yaml, ex_path);
        assert!(report_ex.has_wait_for_timeout);
        assert_eq!(report_ex.anti_patterns.len(), 1);
        assert!(matches!(
            report_ex.anti_patterns[0].kind,
            AntiPatternKind::MissingWhenCondition { .. }
        ));

        let sol_yaml = r#"
---
- launchApp:
    appId: com.cherenkov.bankapp
- tapOn:
    text: Login with Biometric
- runFlow:
    when:
      visible:
        text: Biometric unavailable
    file: pin_fallback_flow.yaml
- assertVisible:
    text: Welcome, SDET Engineer
"#;
        let report_sol = analyze_source(sol_yaml, sol_path);
        assert!(!report_sol.has_wait_for_timeout);
        assert_eq!(report_sol.anti_patterns.len(), 0);
    }

    #[test]
    fn test_maestro_drill_02_deep_link_cold_start_detection() {
        let ex_path = "exercises/03_mobile_maestro/02_deep_link_cold_start/exercise.yaml";
        let sol_path = "exercises/03_mobile_maestro/02_deep_link_cold_start/solution.yaml";

        let ex_yaml = r#"
---
- openLink:
    link: cherenkov://account/ACC-001
- assertVisible:
    text: Account Summary
"#;
        let report_ex = analyze_source(ex_yaml, ex_path);
        assert!(report_ex.has_wait_for_timeout);
        assert_eq!(report_ex.anti_patterns.len(), 1);
        assert!(matches!(
            report_ex.anti_patterns[0].kind,
            AntiPatternKind::MissingColdStartDeepLink { .. }
        ));

        let sol_yaml = r#"
---
- launchApp:
    appId: com.cherenkov.bankapp
    clearState: true
    arguments:
      deeplink: cherenkov://account/ACC-001
- assertVisible:
    text: Account Summary
"#;
        let report_sol = analyze_source(sol_yaml, sol_path);
        assert!(!report_sol.has_wait_for_timeout);
        assert_eq!(report_sol.anti_patterns.len(), 0);
    }

    #[test]
    fn test_maestro_drill_03_activity_recreation_detection() {
        let ex_path = "exercises/03_mobile_maestro/03_activity_recreation/exercise.yaml";
        let sol_path = "exercises/03_mobile_maestro/03_activity_recreation/solution.yaml";

        let ex_yaml = r#"
---
- launchApp:
    appId: com.cherenkov.bankapp
- tapOn:
    text: View Balance
- assertVisible:
    text: "Account Balance: USD 1000"
"#;
        let report_ex = analyze_source(ex_yaml, ex_path);
        assert!(report_ex.has_wait_for_timeout);
        assert_eq!(report_ex.anti_patterns.len(), 1);
        assert!(matches!(
            report_ex.anti_patterns[0].kind,
            AntiPatternKind::MissingActivityRecreation { .. }
        ));

        let sol_yaml = r#"
---
- launchApp:
    appId: com.cherenkov.bankapp
- tapOn:
    text: View Balance
- assertVisible:
    text: "Account Balance: USD 1000"
- setOrientation:
    orientation: landscape
- assertVisible:
    text: "Account Balance: USD 1000"
    optional: false
- setOrientation:
    orientation: portrait
- assertVisible:
    text: "Account Balance: USD 1000"
"#;
        let report_sol = analyze_source(sol_yaml, sol_path);
        assert!(!report_sol.has_wait_for_timeout);
        assert_eq!(report_sol.anti_patterns.len(), 0);
    }

    #[test]
    fn test_maestro_quoted_and_unquoted_locators() {
        let yaml_quoted = r#"
---
- launchApp:
    appId: com.cherenkov.bankapp
- tapOn:
    text: "Login Button"
- tapOn:
    id: "submit-btn"
- assertVisible:
    text: 'Account Balance: USD 1000'
- assertVisible:
    id: 'balance-card'
"#;
        let report_quoted = analyze_source(yaml_quoted, "flow.yaml");
        let selectors: Vec<(&str, &str)> = report_quoted
            .locators
            .iter()
            .map(|l| {
                (
                    match l.kind {
                        LocatorKind::GetByTextOrLabel => "text",
                        LocatorKind::GetByTestId => "id",
                        _ => "other",
                    },
                    l.selector.as_str(),
                )
            })
            .collect();
        assert_eq!(
            selectors,
            vec![
                ("text", "Login Button"),
                ("id", "submit-btn"),
                ("text", "Account Balance: USD 1000"),
                ("id", "balance-card"),
            ]
        );

        let yaml_unquoted = r#"
---
- launchApp:
    appId: com.cherenkov.bankapp
- tapOn:
    text: Login Button
- tapOn:
    id: submit-btn
"#;
        let report_unquoted = analyze_source(yaml_unquoted, "flow.yaml");
        let selectors_unquoted: Vec<(&str, &str)> = report_unquoted
            .locators
            .iter()
            .map(|l| {
                (
                    match l.kind {
                        LocatorKind::GetByTextOrLabel => "text",
                        LocatorKind::GetByTestId => "id",
                        _ => "other",
                    },
                    l.selector.as_str(),
                )
            })
            .collect();
        assert_eq!(
            selectors_unquoted,
            vec![("text", "Login Button"), ("id", "submit-btn"),]
        );
    }

    #[test]
    fn test_maestro_all_actual_drill_files_on_disk() {
        let drills = [
            (
                "exercises/03_mobile_maestro/01_biometric_fallback/exercise.yaml",
                "exercises/03_mobile_maestro/01_biometric_fallback/solution.yaml",
            ),
            (
                "exercises/03_mobile_maestro/02_deep_link_cold_start/exercise.yaml",
                "exercises/03_mobile_maestro/02_deep_link_cold_start/solution.yaml",
            ),
            (
                "exercises/03_mobile_maestro/03_activity_recreation/exercise.yaml",
                "exercises/03_mobile_maestro/03_activity_recreation/solution.yaml",
            ),
        ];

        for (ex, sol) in drills {
            if Path::new(ex).exists() && Path::new(sol).exists() {
                let report_ex = analyze_file(ex).expect("Analyze exercise YAML");
                assert!(
                    report_ex.has_wait_for_timeout,
                    "Exercise must fail anti-pattern: {}",
                    ex
                );
                assert!(!report_ex.anti_patterns.is_empty());

                let report_sol = analyze_file(sol).expect("Analyze solution YAML");
                assert!(
                    !report_sol.has_wait_for_timeout,
                    "Solution must pass cleanly: {}",
                    sol
                );
                assert!(report_sol.anti_patterns.is_empty());
            }
        }
    }
}
