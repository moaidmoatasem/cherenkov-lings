use regex::Regex;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::path::Path;
use std::sync::LazyLock;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Severity {
    Info,
    Warning,
    Error,
}

impl std::fmt::Display for Severity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Severity::Info => write!(f, "INFO"),
            Severity::Warning => write!(f, "WARNING"),
            Severity::Error => write!(f, "ERROR"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AstViolation {
    pub rule_id: String,
    pub severity: Severity,
    pub file_path: String,
    pub line_number: usize,
    pub message: String,
    pub code_snippet: String,
    pub suggested_fix: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SupportedLanguage {
    TypeScript,
    JavaScript,
    Python,
    Java,
    Rust,
    Unknown,
}

impl SupportedLanguage {
    pub fn from_path(path: &Path) -> Self {
        match path.extension().and_then(|ext| ext.to_str()) {
            Some("ts") | Some("tsx") => SupportedLanguage::TypeScript,
            Some("js") | Some("jsx") | Some("mjs") | Some("cjs") => SupportedLanguage::JavaScript,
            Some("py") => SupportedLanguage::Python,
            Some("java") => SupportedLanguage::Java,
            Some("rs") => SupportedLanguage::Rust,
            _ => SupportedLanguage::Unknown,
        }
    }
}

// Regex patterns for detecting test anti-patterns across languages

// 1. Hardcoded sleeps
static RE_TS_JS_SLEEP: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r#"(?:page|frame|locator|\b)\s*\.\s*waitForTimeout\s*\(\s*(\d+)\s*\)|(?:window\.)?setTimeout\s*\(\s*(?:[^,]+,\s*)?(\d+)\s*\)|new\s+Promise\s*\(\s*(?:resolve|r)\s*=>\s*setTimeout\s*\(\s*(?:resolve|r)\s*,\s*(\d+)\s*\)\s*\)"#)
        .expect("Valid regex")
});

static RE_PY_SLEEP: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r#"(?:\btime\.sleep|\basyncio\.sleep)\s*\(\s*([0-9.]+)\s*\)"#).expect("Valid regex")
});

static RE_JAVA_SLEEP: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r#"(?:\bThread\.sleep|\bTimeUnit\.[A-Z_]+\.sleep|\bjava\.lang\.Thread\.sleep)\s*\(\s*([0-9_]+)\s*\)"#)
        .expect("Valid regex")
});

static RE_RUST_SLEEP: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r#"(?:std::thread::sleep|thread::sleep|tokio::time::sleep)\s*\(\s*(?:std::time::Duration::from_[a-z]+\s*\(\s*\d+\s*\)|Duration::from_[a-z]+\s*\(\s*\d+\s*\))\s*\)"#)
        .expect("Valid regex")
});

// 2. Fragile locators
static RE_ABSOLUTE_XPATH: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r#"['"`](?:/html/body(?:/[a-zA-Z0-9_-]+(?:\[\d+\])?)+|//(?:div|span|section|main|ul|li|form|table|tbody|tr|td)/div(?:\[\d+\])?(?:/[a-zA-Z0-9_-]+(?:\[\d+\])?)+)['"`]"#)
        .expect("Valid regex")
});

static RE_DEEP_CSS_CHAIN: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r#"['"`]\s*(?:div\s*>\s*div\s*>\s*(?:span|button|input|a)|(?:[.#:]?[a-zA-Z0-9_-]+(?:\([^\)]*\))?\s*>\s*){3,}[.#:]?[a-zA-Z0-9_-]+(?:\([^\)]*\))?|[.#]?[a-zA-Z0-9_-]+\s*>\s*:nth-child\(\d+\)(?:\s*>\s*[.#:]?[a-zA-Z0-9_-]+(?:\([^\)]*\))?)+)\s*['"`]"#)
        .expect("Valid regex")
});

static RE_AUTO_GENERATED_ID: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r#"['"`](?:#input-[0-9a-fA-F]{6,}|#ember\d+|#react-[a-zA-Z0-9_]{6,}|\[id\^=['"]auto_[^'"]+['"]\]|\[id\*=['"]random_[^'"]+['"]\])['"`]"#)
        .expect("Valid regex")
});

// 3. Unsafe Assertions & Unwraps
static RE_RUST_UNWRAP: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r#"\.(?:unwrap|expect)\s*\(\s*(?:&?"[^"]*")?\s*\)"#).expect("Valid regex")
});

static RE_TS_NON_NULL_OR_ANY: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r#"(?:as\s+any\b|:\s*any\b|!\s*\.\s*[a-zA-Z_]|\bany\s*=\s*(?:page|locator|element))"#)
        .expect("Valid regex")
});

// 4. Vacuous / Tautological Assertions
static RE_TAUTOLOGY_ASSERTION: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r#"(?:expect\s*\(\s*true\s*\)\s*\.\s*toBe\s*\(\s*true\s*\)|expect\s*\(\s*1\s*\)\s*\.\s*toBe\s*\(\s*1\s*\)|assert\s+True\b|assert\s+1\s*==\s*1\b|assertTrue\s*\(\s*true\s*\)|assert_eq!\s*\(\s*true\s*,\s*true\s*\)|assert_eq!\s*\(\s*1\s*,\s*1\s*\)|assert!\s*\(\s*true\s*\))"#)
        .expect("Valid regex")
});

// 5. Floating / Unawaited Promises (in TS/JS/Python async context)
static RE_TS_FLOATING_PROMISE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r#"^\s*(?:page|locator|element|frame)(?:\.[a-zA-Z0-9_]+(?:\([^\)]*\))?)*\s*\.\s*(?:click|fill|type|press|goto|waitForSelector|waitForURL|check|uncheck|selectOption|dblclick|hover|focus)\s*\("#)
        .expect("Valid regex")
});

static RE_TS_FLOATING_EXPECT: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r#"^\s*expect\s*\(.+?\)\s*\.\s*(?:toBeVisible|toBeHidden|toHaveTitle|toHaveURL|toHaveText|toContainText|toBeChecked|toBeDisabled|toBeEditable|toBeEmpty|toBeEnabled|toBeFocused|toBeInViewport|toHaveAttribute|toHaveClass|toHaveCount|toHaveCSS|toHaveId|toHaveValue|toHaveValues)\s*\("#)
        .expect("Valid regex")
});

// 6. Hardcoded Secrets / Plaintext Credentials
static RE_HARDCODED_SECRETS: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r#"(?i)(?:password|passwd|secret|api_key|apikey|auth_token|bearer_token|access_token|private_key)\s*(?:=|:)\s*['"`]([a-zA-Z0-9_\-!@#$%^&*+=/]{5,})['"`]"#)
        .expect("Valid regex")
});

static RE_JWT_TOKEN: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r#"['"`]eyJ[a-zA-Z0-9_\-]+\.eyJ[a-zA-Z0-9_\-]+\.[a-zA-Z0-9_\-]+['"`]"#)
        .expect("Valid regex")
});

/// Rule Scanner Implementation
pub struct RuleScanner;

impl RuleScanner {
    pub fn scan_content(file_path: &str, content: &str) -> Vec<AstViolation> {
        let path = Path::new(file_path);
        let language = SupportedLanguage::from_path(path);
        let mut violations = Vec::new();

        let lines: Vec<&str> = content.lines().collect();

        // 1. Scan line-by-line rules
        for (idx, line) in lines.iter().enumerate() {
            let line_num = idx + 1;
            let trimmed = line.trim();

            // Ignore comments
            if Self::is_comment_line(trimmed, language) {
                continue;
            }

            // Check hardcoded sleeps
            Self::check_hardcoded_sleeps(
                file_path,
                line_num,
                line,
                trimmed,
                language,
                &mut violations,
            );

            // Check fragile locators
            Self::check_fragile_locators(
                file_path,
                line_num,
                line,
                trimmed,
                language,
                &mut violations,
            );

            // Check unsafe assertions / unwraps
            Self::check_unsafe_assertions(
                file_path,
                line_num,
                line,
                trimmed,
                language,
                &mut violations,
            );

            // Check tautological / vacuous assertions
            Self::check_tautological_assertions(
                file_path,
                line_num,
                line,
                trimmed,
                language,
                &mut violations,
            );

            // Check floating unawaited promises
            Self::check_floating_promises(
                file_path,
                line_num,
                line,
                trimmed,
                language,
                &mut violations,
            );

            // Check hardcoded secrets
            Self::check_hardcoded_secrets(
                file_path,
                line_num,
                line,
                trimmed,
                language,
                &mut violations,
            );
        }

        // 2. Global file-level checks (e.g. test files with zero assertions)
        Self::check_missing_assertions_in_test_file(
            file_path,
            content,
            &lines,
            language,
            &mut violations,
        );

        // Sort violations by line number and severity
        violations.sort_by(|a, b| {
            a.line_number
                .cmp(&b.line_number)
                .then_with(|| b.severity.cmp(&a.severity))
        });

        violations
    }

    fn is_comment_line(line: &str, language: SupportedLanguage) -> bool {
        match language {
            SupportedLanguage::Python => line.starts_with('#'),
            SupportedLanguage::Rust
            | SupportedLanguage::TypeScript
            | SupportedLanguage::JavaScript
            | SupportedLanguage::Java => {
                line.starts_with("//") || line.starts_with("/*") || line.starts_with('*')
            }
            _ => line.starts_with("//") || line.starts_with('#'),
        }
    }

    fn strip_string_literals(line: &str) -> String {
        let mut result = String::with_capacity(line.len());
        let mut in_single = false;
        let mut in_double = false;
        let mut in_backtick = false;
        let mut escaped = false;

        let chars: Vec<char> = line.chars().collect();
        let mut i = 0;
        while i < chars.len() {
            let c = chars[i];
            if escaped {
                escaped = false;
                i += 1;
                continue;
            }
            if c == '\\' {
                escaped = true;
                i += 1;
                continue;
            }
            if in_single {
                if c == '\'' {
                    in_single = false;
                }
                i += 1;
                continue;
            }
            if in_double {
                if c == '"' {
                    in_double = false;
                }
                i += 1;
                continue;
            }
            if in_backtick {
                if c == '`' {
                    in_backtick = false;
                }
                i += 1;
                continue;
            }

            match c {
                '\'' => in_single = true,
                '"' => in_double = true,
                '`' => in_backtick = true,
                _ => result.push(c),
            }
            i += 1;
        }

        result
    }

    fn check_hardcoded_sleeps(
        file_path: &str,
        line_num: usize,
        original_line: &str,
        trimmed: &str,
        language: SupportedLanguage,
        violations: &mut Vec<AstViolation>,
    ) {
        let code_without_strings = Self::strip_string_literals(trimmed);
        let is_sleep = match language {
            SupportedLanguage::TypeScript | SupportedLanguage::JavaScript => {
                RE_TS_JS_SLEEP.is_match(&code_without_strings)
            }
            SupportedLanguage::Python => RE_PY_SLEEP.is_match(&code_without_strings),
            SupportedLanguage::Java => RE_JAVA_SLEEP.is_match(&code_without_strings),
            SupportedLanguage::Rust => RE_RUST_SLEEP.is_match(&code_without_strings),
            SupportedLanguage::Unknown => {
                RE_TS_JS_SLEEP.is_match(&code_without_strings)
                    || RE_PY_SLEEP.is_match(&code_without_strings)
                    || RE_JAVA_SLEEP.is_match(&code_without_strings)
                    || RE_RUST_SLEEP.is_match(&code_without_strings)
            }
        };

        if is_sleep {
            let suggested = match language {
                SupportedLanguage::TypeScript | SupportedLanguage::JavaScript => {
                    "await expect(page.getByRole('...')).toBeVisible();".to_string()
                }
                SupportedLanguage::Python => {
                    "expect(page.get_by_role('...')).to_be_visible()".to_string()
                }
                SupportedLanguage::Java => {
                    "Awaitility.await().atMost(5, SECONDS).until(() -> element.isDisplayed());"
                        .to_string()
                }
                SupportedLanguage::Rust => {
                    "tokio::time::timeout(Duration::from_secs(5), async { ... }).await?;".to_string()
                }
                _ => "// Replace static sleep with dynamic state or event polling".to_string(),
            };

            violations.push(AstViolation {
                rule_id: "ANTI_PATTERN_HARDCODED_SLEEP".to_string(),
                severity: Severity::Error,
                file_path: file_path.to_string(),
                line_number: line_num,
                message: "Hardcoded sleep detected. Arbitrary timeouts introduce test flakiness, race conditions, and inflate pipeline runtime. Use event-driven dynamic auto-waiting or explicit state assertions instead.".to_string(),
                code_snippet: original_line.to_string(),
                suggested_fix: Some(suggested),
            });
        }
    }

    fn check_fragile_locators(
        file_path: &str,
        line_num: usize,
        original_line: &str,
        trimmed: &str,
        _language: SupportedLanguage,
        violations: &mut Vec<AstViolation>,
    ) {
        if RE_ABSOLUTE_XPATH.is_match(trimmed) {
            violations.push(AstViolation {
                rule_id: "ANTI_PATTERN_FRAGILE_LOCATOR_XPATH".to_string(),
                severity: Severity::Error,
                file_path: file_path.to_string(),
                line_number: line_num,
                message: "Deep absolute XPath locator detected. Any minor DOM refactoring or layout adjustment will break this locator. Use semantic role-based locators (getByRole), accessibility labels, or data-testid attributes.".to_string(),
                code_snippet: original_line.to_string(),
                suggested_fix: Some("page.getByRole('button', { name: 'Submit' })".to_string()),
            });
        } else if RE_DEEP_CSS_CHAIN.is_match(trimmed) {
            violations.push(AstViolation {
                rule_id: "ANTI_PATTERN_FRAGILE_LOCATOR_CSS".to_string(),
                severity: Severity::Warning,
                file_path: file_path.to_string(),
                line_number: line_num,
                message: "Deeply nested or structural CSS selector detected (e.g. nth-child chains). CSS structural selectors tightly couple tests to markup structure. Prefer resilient user-facing queries like getByText or getByTestId.".to_string(),
                code_snippet: original_line.to_string(),
                suggested_fix: Some("page.getByTestId('target-element')".to_string()),
            });
        } else if RE_AUTO_GENERATED_ID.is_match(trimmed) {
            violations.push(AstViolation {
                rule_id: "ANTI_PATTERN_FRAGILE_LOCATOR_AUTO_ID".to_string(),
                severity: Severity::Warning,
                file_path: file_path.to_string(),
                line_number: line_num,
                message: "Auto-generated or ephemeral framework ID selector detected. Dynamic IDs change between builds and server restarts, causing intermittent locator resolution failures.".to_string(),
                code_snippet: original_line.to_string(),
                suggested_fix: Some("page.getByLabel('Input Label')".to_string()),
            });
        }
    }

    fn check_unsafe_assertions(
        file_path: &str,
        line_num: usize,
        original_line: &str,
        trimmed: &str,
        language: SupportedLanguage,
        violations: &mut Vec<AstViolation>,
    ) {
        match language {
            SupportedLanguage::Rust => {
                if RE_RUST_UNWRAP.is_match(trimmed) && !trimmed.starts_with("//") {
                    violations.push(AstViolation {
                        rule_id: "ANTI_PATTERN_UNSAFE_UNWRAP".to_string(),
                        severity: Severity::Warning,
                        file_path: file_path.to_string(),
                        line_number: line_num,
                        message: "Raw unwrap()/expect() in test flow. A panic from unwrap hides structured diagnostic feedback. Prefer using Result propagation `?` or explicit assert assertions with failure explanations.".to_string(),
                        code_snippet: original_line.to_string(),
                        suggested_fix: Some("let value = result.expect(\"Meaningful error context\"); // Or use `?` in test fn returning Result<(), Box<dyn Error>>".to_string()),
                    });
                }
            }
            SupportedLanguage::TypeScript | SupportedLanguage::JavaScript => {
                if RE_TS_NON_NULL_OR_ANY.is_match(trimmed) {
                    violations.push(AstViolation {
                        rule_id: "ANTI_PATTERN_UNSAFE_TYPE_BYPASS".to_string(),
                        severity: Severity::Warning,
                        file_path: file_path.to_string(),
                        line_number: line_num,
                        message: "Unsafe type assertion (`as any` or non-null `!`) used in test. Bypassing type safety masks payload schema drift and null-pointer exceptions in test fixtures.".to_string(),
                        code_snippet: original_line.to_string(),
                        suggested_fix: Some("expect(response.body).toBeDefined(); // Use explicit runtime type guard".to_string()),
                    });
                }
            }
            _ => {}
        }
    }

    fn check_tautological_assertions(
        file_path: &str,
        line_num: usize,
        original_line: &str,
        trimmed: &str,
        _language: SupportedLanguage,
        violations: &mut Vec<AstViolation>,
    ) {
        if RE_TAUTOLOGY_ASSERTION.is_match(trimmed) {
            violations.push(AstViolation {
                rule_id: "ANTI_PATTERN_VACUOUS_ASSERTION".to_string(),
                severity: Severity::Error,
                file_path: file_path.to_string(),
                line_number: line_num,
                message: "Tautological / vacuous assertion detected. Asserting that true == true or 1 == 1 produces a false sense of test coverage while verifying zero actual system behavior.".to_string(),
                code_snippet: original_line.to_string(),
                suggested_fix: Some("expect(actualValue).toEqual(expectedValue);".to_string()),
            });
        }
    }

    fn check_floating_promises(
        file_path: &str,
        line_num: usize,
        original_line: &str,
        trimmed: &str,
        language: SupportedLanguage,
        violations: &mut Vec<AstViolation>,
    ) {
        if matches!(
            language,
            SupportedLanguage::TypeScript | SupportedLanguage::JavaScript
        ) {
            // Check if line calls async Playwright action without await and is not part of a return or chained promise
            let is_floating_action = RE_TS_FLOATING_PROMISE.is_match(trimmed)
                && !trimmed.starts_with("await ")
                && !trimmed.starts_with("return ")
                && !trimmed.starts_with("const ")
                && !trimmed.starts_with("let ")
                && !trimmed.starts_with("var ");

            let is_floating_expect = RE_TS_FLOATING_EXPECT.is_match(trimmed)
                && !trimmed.starts_with("await ")
                && !trimmed.starts_with("return ");

            if is_floating_action || is_floating_expect {
                let mut fix = original_line.trim_start().to_string();
                fix = format!("await {}", fix);
                violations.push(AstViolation {
                    rule_id: "ANTI_PATTERN_FLOATING_PROMISE".to_string(),
                    severity: Severity::Error,
                    file_path: file_path.to_string(),
                    line_number: line_num,
                    message: "Unawaited asynchronous action/matcher detected. Floating promises execute out of sequence, leading to phantom passes, unhandled promise rejections, and race conditions.".to_string(),
                    code_snippet: original_line.to_string(),
                    suggested_fix: Some(fix),
                });
            }
        }
    }

    fn check_hardcoded_secrets(
        file_path: &str,
        line_num: usize,
        original_line: &str,
        trimmed: &str,
        language: SupportedLanguage,
        violations: &mut Vec<AstViolation>,
    ) {
        let has_secret = RE_HARDCODED_SECRETS.is_match(trimmed) || RE_JWT_TOKEN.is_match(trimmed);
        // Exclude obvious mock placeholders like "test", "dummy", "placeholder", "xxx"
        let is_mock_value = trimmed.contains("\"test\"")
            || trimmed.contains("'test'")
            || trimmed.contains("\"dummy\"")
            || trimmed.contains("'dummy'")
            || trimmed.contains("\"example\"")
            || trimmed.contains("'example'");

        if has_secret && !is_mock_value {
            let suggested_fix = match language {
                SupportedLanguage::TypeScript | SupportedLanguage::JavaScript => {
                    "const password = process.env.TEST_PASSWORD || '';".to_string()
                }
                SupportedLanguage::Python => {
                    "password = os.getenv('TEST_PASSWORD', '')".to_string()
                }
                SupportedLanguage::Java => {
                    "String password = System.getenv(\"TEST_PASSWORD\");".to_string()
                }
                SupportedLanguage::Rust => {
                    "let password = std::env::var(\"TEST_PASSWORD\").unwrap_or_default();"
                        .to_string()
                }
                _ => "// Extract secret to environment variable or secure test vault".to_string(),
            };

            violations.push(AstViolation {
                rule_id: "ANTI_PATTERN_HARDCODED_SECRET".to_string(),
                severity: Severity::Error,
                file_path: file_path.to_string(),
                line_number: line_num,
                message: "Plaintext credential or API secret hardcoded in test code. Secrets in version control leak into test logs, artifacts, and CI reports. Inject test secrets via environment variables or secret managers.".to_string(),
                code_snippet: original_line.to_string(),
                suggested_fix: Some(suggested_fix),
            });
        }
    }

    fn check_missing_assertions_in_test_file(
        file_path: &str,
        content: &str,
        lines: &[&str],
        language: SupportedLanguage,
        violations: &mut Vec<AstViolation>,
    ) {
        // Only run for test files
        let is_test_file = file_path.contains("test")
            || file_path.contains("spec")
            || file_path.contains("exercise")
            || file_path.contains("drill");

        if !is_test_file || content.trim().is_empty() {
            return;
        }

        // Check whether the test file contains any assertion keywords
        let has_assertion = match language {
            SupportedLanguage::TypeScript | SupportedLanguage::JavaScript => {
                content.contains("expect(")
                    || content.contains("assert.")
                    || content.contains("assert(")
                    || content.contains("should.")
            }
            SupportedLanguage::Python => {
                content.contains("assert ")
                    || content.contains("self.assert")
                    || content.contains("expect(")
                    || content.contains("pytest.raises")
            }
            SupportedLanguage::Java => {
                content.contains("assert")
                    || content.contains("assertThat(")
                    || content.contains("assertEquals(")
                    || content.contains("assertTrue(")
                    || content.contains("assertFalse(")
                    || content.contains("then().assertThat()")
            }
            SupportedLanguage::Rust => {
                content.contains("assert!")
                    || content.contains("assert_eq!")
                    || content.contains("assert_ne!")
                    || content.contains("assert_matches!")
            }
            _ => {
                content.contains("assert") || content.contains("expect")
            }
        };

        if !has_assertion && lines.len() > 2 {
            violations.push(AstViolation {
                rule_id: "ANTI_PATTERN_MISSING_ASSERTION".to_string(),
                severity: Severity::Error,
                file_path: file_path.to_string(),
                line_number: 1,
                message: "No assertions found in test suite. Tests without assertions only verify that the target system didn't crash, missing all functional regressions.".to_string(),
                code_snippet: lines.first().unwrap_or(&"").to_string(),
                suggested_fix: Some("// Add explicit assertion validating response/DOM state\nexpect(result).toBeDefined();".to_string()),
            });
        }
    }
}

/// Applies automated AST rule fixes to the source code
pub fn apply_automated_fixes(content: &str, violations: &[AstViolation]) -> String {
    let mut lines: Vec<String> = content.lines().map(|s| s.to_string()).collect();
    if lines.is_empty() {
        return content.to_string();
    }

    // Apply fixes in reverse line order to prevent offset shifts
    let mut fixable: Vec<&AstViolation> = violations
        .iter()
        .filter(|v| v.suggested_fix.is_some() && v.line_number > 0 && v.line_number <= lines.len())
        .collect();

    fixable.sort_by(|a, b| b.line_number.cmp(&a.line_number));

    let mut applied_lines = HashSet::new();

    for violation in fixable {
        let line_idx = violation.line_number - 1;
        if applied_lines.contains(&line_idx) {
            continue;
        }

        if let Some(fix) = &violation.suggested_fix {
            // Preserve leading indentation of original line
            let original = &lines[line_idx];
            let indent_len = original.len() - original.trim_start().len();
            let indent = &original[..indent_len];

            // If suggested fix contains multiple lines, indent each
            let formatted_fix: Vec<String> = fix
                .lines()
                .map(|l| format!("{}{}", indent, l.trim_start()))
                .collect();

            lines[line_idx] = formatted_fix.join("\n");
            applied_lines.insert(line_idx);
        }
    }

    let trailing_newline = if content.ends_with('\n') { "\n" } else { "" };
    format!("{}{}", lines.join("\n"), trailing_newline)
}
