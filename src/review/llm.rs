use crate::review::rules::AstViolation;
use serde::{Deserialize, Serialize};
use std::time::Duration;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MentorReview {
    pub critique: String,
    pub socratic_questions: Vec<String>,
    pub architectural_advice: String,
}

#[derive(Debug, Clone)]
pub struct AiMentorClient {
    pub endpoint: String,
    pub model: String,
    pub offline_fallback: bool,
    pub timeout: Duration,
}

impl Default for AiMentorClient {
    fn default() -> Self {
        let endpoint = std::env::var("CHERENKOV_LLM_URL")
            .unwrap_or_else(|_| "http://localhost:11434/api/generate".to_string());
        let model = std::env::var("CHERENKOV_LLM_MODEL")
            .unwrap_or_else(|_| "llama3".to_string());

        Self {
            endpoint,
            model,
            offline_fallback: true,
            timeout: Duration::from_millis(2500),
        }
    }
}

#[derive(Serialize)]
struct OllamaRequest<'a> {
    model: &'a str,
    prompt: &'a str,
    stream: bool,
}

#[derive(Deserialize)]
struct OllamaResponse {
    response: Option<String>,
}

impl AiMentorClient {
    pub fn new(endpoint: Option<&str>, model: Option<&str>, offline_fallback: bool) -> Self {
        let default_client = Self::default();
        Self {
            endpoint: endpoint
                .map(|s| {
                    if !s.contains("/api/generate") && !s.contains("/v1") {
                        format!("{}/api/generate", s.trim_end_matches('/'))
                    } else {
                        s.to_string()
                    }
                })
                .unwrap_or(default_client.endpoint),
            model: model
                .map(|s| s.to_string())
                .unwrap_or(default_client.model),
            offline_fallback,
            timeout: Duration::from_millis(2500),
        }
    }

    pub async fn review_code(
        &self,
        file_name: &str,
        code_content: &str,
        violations: &[AstViolation],
    ) -> MentorReview {
        // If not in offline-only mode, try calling the remote LLM
        if !self.endpoint.is_empty() {
            if let Ok(review) = self.call_llm(file_name, code_content, violations).await {
                return review;
            }
        }

        // Fallback to deterministic offline Senior QA Mentor
        self.generate_offline_mentor_review(file_name, code_content, violations)
    }

    async fn call_llm(
        &self,
        file_name: &str,
        code_content: &str,
        violations: &[AstViolation],
    ) -> Result<MentorReview, Box<dyn std::error::Error + Send + Sync>> {
        let client = reqwest::Client::builder()
            .timeout(self.timeout)
            .build()?;

        let violations_summary = violations
            .iter()
            .map(|v| format!("- [Line {} | {}] {}: {}", v.line_number, v.severity, v.rule_id, v.message))
            .collect::<Vec<_>>()
            .join("\n");

        let prompt = format!(
            "You are a Senior Principal Quality Engineering (SDET) Architect conducting a code review.\n\
            File: {}\n\n\
            Static AST Rule Violations Detected:\n{}\n\n\
            Code Under Review:\n```\n{}\n```\n\n\
            Provide:\n\
            1. An incisive Senior QA architectural critique explaining why these patterns cause test flakiness, maintenance drag, or security vulnerabilities in enterprise CI/CD.\n\
            2. 2-3 Socratic questions that guide the engineer to reflect on root causes.\n\
            3. Concrete architectural recommendations.",
            file_name, violations_summary, code_content
        );

        let req_body = OllamaRequest {
            model: &self.model,
            prompt: &prompt,
            stream: false,
        };

        let res = client
            .post(&self.endpoint)
            .json(&req_body)
            .send()
            .await?;

        if !res.status().is_success() {
            return Err(format!("LLM returned status {}", res.status()).into());
        }

        let resp_body: OllamaResponse = res.json().await?;
        if let Some(text) = resp_body.response {
            if !text.trim().is_empty() {
                return Ok(MentorReview {
                    critique: text.clone(),
                    socratic_questions: vec![
                        "How would this test behave under 500ms network jitter in a parallel CI runner matrix?".to_string(),
                        "What contract boundary is this test truly validating versus implementation details?".to_string(),
                    ],
                    architectural_advice: "Adopt resilient user-facing locators, deterministic event-driven waiting, and isolated test secrets.".to_string(),
                });
            }
        }

        Err("Empty LLM response".into())
    }

    pub fn generate_offline_mentor_review(
        &self,
        file_name: &str,
        _code_content: &str,
        violations: &[AstViolation],
    ) -> MentorReview {
        if violations.is_empty() {
            return MentorReview {
                critique: format!(
                    "🏆 **Senior QA Architecture Assessment**: Exemplary test design in `{}`.\n\
                    - Your test demonstrates strict deterministic synchronization without arbitrary sleep delays.\n\
                    - Locators adhere to user-centric semantic queries (role & accessibility based).\n\
                    - Assertions are explicit, meaningful, and resilient against false positives.\n\
                    - Zero credentials or environment-specific secrets are leaked in code.",
                    file_name
                ),
                socratic_questions: vec![
                    "How could this test suite be integrated into a parallel matrix CI pipeline without shared state conflicts?".to_string(),
                    "What chaos faults (e.g. 504 Gateway Timeout or packet drops) might this test surface in staging?".to_string(),
                ],
                architectural_advice: "Your test suite meets Enterprise SDET Tier-1 standards. Ready for pipeline integration and automated regression suites.".to_string(),
            };
        }

        let mut critique_points = Vec::new();
        let mut questions = Vec::new();
        let mut advice_points = Vec::new();

        let has_sleep = violations.iter().any(|v| v.rule_id.contains("SLEEP"));
        let has_xpath = violations.iter().any(|v| v.rule_id.contains("XPATH") || v.rule_id.contains("LOCATOR"));
        let has_secret = violations.iter().any(|v| v.rule_id.contains("SECRET"));
        let has_floating = violations.iter().any(|v| v.rule_id.contains("FLOATING_PROMISE"));
        let has_vacuous = violations.iter().any(|v| v.rule_id.contains("VACUOUS_ASSERTION") || v.rule_id.contains("MISSING_ASSERTION"));
        let has_unwrap = violations.iter().any(|v| v.rule_id.contains("UNWRAP") || v.rule_id.contains("UNSAFE"));

        critique_points.push(format!(
            "🔍 **Senior QA Code Review for `{}`** (Found {} anti-pattern violation{}):",
            file_name,
            violations.len(),
            if violations.len() == 1 { "" } else { "s" }
        ));

        if has_sleep {
            critique_points.push(
                "• **Hardcoded Sleep Anti-Pattern**: Arbitrary `waitForTimeout` or `Thread.sleep` calls are the #1 root cause of enterprise test flakiness. Under heavy CI load or CPU throttling, fixed timers expire prematurely causing false alarms; during fast runs, they needlessly inflate build duration.".to_string()
            );
            questions.push(
                "Why is event-driven auto-waiting (polling DOM mutations or network events) strictly superior to fixed millisecond sleeps?".to_string()
            );
            advice_points.push(
                "Replace static sleeps with dynamic state assertions (e.g. `expect(locator).toBeVisible()` or `Awaitility.await().until(...)`).".to_string()
            );
        }

        if has_xpath {
            critique_points.push(
                "• **Fragile Structural Locators**: Deep absolute XPath and chained CSS selectors tightly bind tests to ephemeral DOM hierarchy. A simple designer CSS refactor or wrapping `<div>` will trigger cascading test failures across your suite.".to_string()
            );
            questions.push(
                "If a developer refactors the page from a `<div>` table to CSS grid, will this test survive?".to_string()
            );
            advice_points.push(
                "Query elements using accessible user roles (`getByRole`), labels (`getByLabel`), or dedicated test attributes (`getByTestId`).".to_string()
            );
        }

        if has_floating {
            critique_points.push(
                "• **Floating Unawaited Promises**: Triggering asynchronous browser interactions (`page.click()`, `page.fill()`) without `await` dispatches actions onto the event loop unmonitored. The test runner may conclude execution before the browser ever receives the click.".to_string()
            );
            questions.push(
                "What happens to unhandled promise rejections when the test harness shuts down the browser context prematurely?".to_string()
            );
            advice_points.push(
                "Always prepend `await` to asynchronous Playwright/Puppeteer actions and async matchers.".to_string()
            );
        }

        if has_secret {
            critique_points.push(
                "• **Hardcoded Plaintext Credentials**: Hardcoding passwords, API tokens, or JWTs in test files risks credential exposure in git histories, test artifacts, and CI execution logs.".to_string()
            );
            questions.push(
                "How can you externalize test credentials so tests run seamlessly in local dev, staging, and ephemeral CI environments without changing code?".to_string()
            );
            advice_points.push(
                "Inject test secrets via environment variables (`process.env.TEST_PASSWORD`, `System.getenv`) or vault secrets.".to_string()
            );
        }

        if has_vacuous {
            critique_points.push(
                "• **Vacuous / Missing Assertions**: A test without meaningful assertions (or asserting `true == true`) is merely a smoke runner verifying that the process didn't crash, missing 100% of business logic regressions.".to_string()
            );
            questions.push(
                "What observable business state or API response contract definitively proves this user journey succeeded?".to_string()
            );
            advice_points.push(
                "Write explicit assertions against response payloads, URL transitions, or verified DOM state changes.".to_string()
            );
        }

        if has_unwrap {
            critique_points.push(
                "• **Unsafe Error Handling / Unwraps**: Raw unwrapping in test fixtures leads to uninformative stack traces rather than actionable assertion failure diffs.".to_string()
            );
            questions.push(
                "When this unwrap panics in a nighttime CI run, will the triage engineer know what payload caused it?".to_string()
            );
            advice_points.push(
                "Use idiomatic error assertions with explicit error messages and structured context.".to_string()
            );
        }

        if questions.is_empty() {
            questions.push("How would this test behave in a heavily throttled containerized test matrix?".to_string());
        }

        MentorReview {
            critique: critique_points.join("\n\n"),
            socratic_questions: questions,
            architectural_advice: advice_points.join(" "),
        }
    }
}
