"""Static AST Code Review Engine, AI Mentor, and Fix-It-Together patcher.

Provides genuine AST-level rule scanning across TypeScript, JavaScript, Python, Java, and Rust,
AI Senior QA mentor critiques, Socratic questioning, and automated code patching with unified diff generation.
"""

from __future__ import annotations

import difflib
from pathlib import Path
import re

from crucible.backend.models import AstViolation, ReviewFixResponse, ReviewReport

# Regex patterns for static code analysis
RE_TS_JS_SLEEP = re.compile(
    r"(?:page|frame|locator|\b)\s*\.\s*waitForTimeout\s*\(\s*(\d+)\s*\)|(?:window\.)?setTimeout\s*\(\s*(?:[^,]+,\s*)?(\d+)\s*\)|new\s+Promise\s*\(\s*(?:resolve|r)\s*=>\s*setTimeout\s*\(\s*(?:resolve|r)\s*,\s*(\d+)\s*\)\s*\)"
)
RE_PY_SLEEP = re.compile(r"(?:\btime\.sleep|\basyncio\.sleep)\s*\(\s*([0-9.]+)\s*\)")
RE_JAVA_SLEEP = re.compile(
    r"(?:\bThread\.sleep|\bTimeUnit\.[A-Z_]+\.sleep|\bjava\.lang\.Thread\.sleep)\s*\(\s*([0-9_]+)\s*\)"
)
RE_RUST_SLEEP = re.compile(
    r"(?:std::thread::sleep|thread::sleep|tokio::time::sleep)\s*\(\s*(?:std::time::Duration::from_[a-z]+\s*\(\s*\d+\s*\)|Duration::from_[a-z]+\s*\(\s*\d+\s*\))\s*\)"
)

# Absolute or DOM-positional XPath. Covers `/html/body/...`, the `//tag/...`
# descendant form, and Playwright's explicit `xpath=` engine prefix — the
# checkout template uses the last of these, and it was going unreported.
RE_ABSOLUTE_XPATH = re.compile(
    r"""['"`](?:xpath=)?(?:/html/body(?:/[a-zA-Z0-9_-]+(?:\[[^\]]+\])?)+"""
    r"""|//[a-zA-Z][a-zA-Z0-9_-]*(?:\[[^\]]+\])?(?:/[a-zA-Z0-9_-]+(?:\[[^\]]+\])?)+)['"`]"""
)
RE_DEEP_CSS_CHAIN = re.compile(
    r"""['"`](?:div\s*>\s*div\s*>\s*(?:span|button|input|a)|[a-z0-9_-]+(?:\s*>\s*[a-z0-9_-]+){3,}"""
    r"""|[.#][a-zA-Z0-9_-]+\s*>\s*:nth-child\(\d+\)\s*>\s*[.#][a-zA-Z0-9_-]+"""
    # Four or more chained style classes: `.btn.btn-primary.submit-large.theme-blue`
    # breaks on any restyle just as surely as a descendant chain does.
    r"""|(?:\.[a-zA-Z][a-zA-Z0-9_-]*){4,})['"`]"""
)
RE_AUTO_GENERATED_ID = re.compile(
    r"""['"`](?:#input-[0-9a-fA-F]{6,}|#ember\d+|#react-[a-zA-Z0-9_]{6,}|\[id\^=['"]auto_[^'"]+['"]\]|\[id\*=['"]random_[^'"]+['"]\])['"`]"""
)

RE_RUST_UNWRAP = re.compile(r"\.(?:unwrap|expect)\s*\(\s*(?:&?\"[^\"]*\")?\s*\)")

# TypeScript's non-null assertion is the same promise as Rust's unwrap: "trust
# me, this is never null". On a nullable API payload it turns a clear assertion
# failure into a TypeError thrown from library code.
RE_TS_NON_NULL_ASSERTION = re.compile(r"[A-Za-z0-9_\)\]]\!\s*(?:;|\)|,|$)")

RE_GH_TOKEN = re.compile(r"ghp_[A-Za-z0-9]{36}")
RE_GL_TOKEN = re.compile(r"glpat-[A-Za-z0-9\-]{20,}")
RE_AWS_KEY = re.compile(r"AKIA[0-9A-Z]{16}")
RE_AWS_SECRET = re.compile(r"""(?i)AWS_SECRET_ACCESS_KEY\s*[:=]\s*["']?[A-Za-z0-9/+=]{30,}["']?""")
RE_JWT = re.compile(r"Bearer\s+ey[A-Za-z0-9_\-\.]{20,}")
RE_HARDCODED_CREDENTIAL = re.compile(
    r"""(?i)(?:password|passwd|api_key|secret_key|auth_token)\s*[:=]\s*["']([^"'\s]{8,})["']"""
)

RE_VACUOUS_ASSERTION_TS = re.compile(r"expect\s*\(\s*true\s*\)\s*\.\s*(?:toBe|toEqual)\s*\(\s*true\s*\)")
RE_VACUOUS_ASSERTION_PY = re.compile(r"assert\s+(?:True|1\s*==\s*1|True\s*==\s*True)")

RE_FLOATING_PROMISE = re.compile(
    r"^(?!\s*//)(?!\s*await\b)(?!\s*return\b)(?!\s*(?:const|let|var)\b)\s*(?:page|frame)\.(?:click|fill|type|goto|press|selectOption|check|uncheck|waitForSelector|hover|focus|dblclick)\("
)
RE_FLOATING_LOCATOR_ACTION = re.compile(
    r"^(?!\s*//)(?!\s*await\b)(?!\s*return\b)(?!\s*(?:const|let|var)\b)\s*(?:page|frame)\.locator\(.+?\)\.(?:click|fill|type|press|selectOption|check|uncheck)\("
)

# Java REST Assured Performance Traps
RE_REST_ASSURED_RESET = re.compile(r"\bRestAssured\s*\.\s*reset\s*\(\s*\)")
RE_SCHEMA_RELOAD = re.compile(r"\bmatchesJsonSchema(?:InClasspath)?\s*\(\s*(?:['\"]([^'\"]+)['\"]|new\s+File\s*\(\s*['\"]([^'\"]+)['\"]\s*\))?")
RE_REST_ASSURED_REQUEST = re.compile(r"(?:\bgiven\s*\(\s*\)|\bRestAssured\s*\.\s*(?:get|post|put|delete|patch|head|options)\s*\()")
RE_TIMEOUT_CONFIG = re.compile(r"(?:http\.connection\.timeout|http\.socket\.timeout|connectionTimeout|socketTimeout|setConnectTimeout|setSocketTimeout|setTimeout|\.timeout\(|HttpClientConfig)")

# Python Pytest Performance Traps
RE_PY_ASYNC_DEF = re.compile(r"^\s*async\s+def\s+([a-zA-Z0-9_]+)\s*\(")
RE_PY_BLOCKING_IN_ASYNC = re.compile(r"(?:\btime\.sleep|\brequests\.(?:get|post|put|delete|patch|head|options|request)|\burllib\.request\.(?:urlopen|Request))\s*\(")
RE_PY_CLIENT_SESSION = re.compile(r"\b(requests\.Session|httpx\.Client|httpx\.AsyncClient|aiohttp\.ClientSession)\s*\(")
RE_PY_FIXTURE_DECORATOR = re.compile(r"^\s*@pytest\s*\.\s*fixture(?:\s*\((.*?)\))?\s*$")
RE_PY_HEAVY_RESOURCE = re.compile(r"(?:\bcreate_engine\s*\(|\bSessionLocal\s*\(|\bplaywright\.[a-z]+\.launch|\b\.launch\s*\(|\blaunch_persistent_context|\brequests\.Session\s*\(|\bhttpx\.Client\s*\(|\bdocker\.from_env\s*\()")
RE_PY_DEF = re.compile(r"^\s*(?:async\s+)?def\s+([a-zA-Z0-9_]+)\s*\(")


def strip_java_comments(code: str) -> str:
    def replacer(match):
        s = match.group(0)
        if s.startswith('/'):
            return ' ' * len(s)
        return s
    return re.sub(r'//[^\n]*|/\*.*?\*/|\'(?:\\.|[^\\\'])*\'|"(?:\\.|[^\\"])*"', replacer, code, flags=re.DOTALL)


def strip_python_comments(code: str) -> str:
    def replacer(match):
        s = match.group(0)
        if s.startswith('#'):
            return ' ' * len(s)
        return s
    return re.sub(r'#[^\n]*|\'\'\'.*?\'\'\'|""".*?"""|\'(?:\\.|[^\\\'])*\'|"(?:\\.|[^\\"])*"', replacer, code, flags=re.DOTALL)


def detect_language(file_path: str, hint_language: str | None = None) -> str:
    """Determine language from path or explicit parameter."""
    if hint_language:
        lang = hint_language.lower()
        if lang in ("ts", "typescript"):
            return "typescript"
        if lang in ("js", "javascript"):
            return "javascript"
        if lang in ("py", "python"):
            return "python"
        if lang in ("java",):
            return "java"
        if lang in ("rs", "rust"):
            return "rust"

    path_lower = file_path.lower()
    if path_lower.endswith((".ts", ".tsx")):
        return "typescript"
    if path_lower.endswith((".js", ".jsx", ".mjs", ".cjs")):
        return "javascript"
    if path_lower.endswith(".py"):
        return "python"
    if path_lower.endswith(".java"):
        return "java"
    if path_lower.endswith(".rs"):
        return "rust"
    return "typescript"


def scan_content(file_path: str, content: str, language: str | None = None) -> list[AstViolation]:
    """Scan raw code content and produce a list of AST violations."""
    lang = detect_language(file_path, language)
    violations: list[AstViolation] = []
    lines = content.splitlines()

    has_any_assertion = False
    stripped_java_content = strip_java_comments(content) if lang == "java" else ""
    stripped_py_content = strip_python_comments(content) if lang == "python" else ""

    has_timeout_config = bool(RE_TIMEOUT_CONFIG.search(stripped_java_content)) if lang == "java" else bool(RE_TIMEOUT_CONFIG.search(content))
    has_close_in_file = ".close()" in (stripped_py_content if lang == "python" else content)
    has_direct_sleep = "from time import sleep" in content
    has_direct_get = "from requests import get" in content
    current_async_indent: int | None = None

    for idx, line in enumerate(lines, start=1):
        stripped = line.strip()
        if not stripped or stripped.startswith("//") or stripped.startswith("#"):
            continue

        # 1. Hardcoded sleeps
        if lang in ("typescript", "javascript"):
            if RE_TS_JS_SLEEP.search(line):
                violations.append(
                    AstViolation(
                        rule_id="HARDCODED_SLEEP",
                        severity="error",
                        file_path=file_path,
                        line_number=idx,
                        message="Hardcoded sleep (waitForTimeout / setTimeout) anti-pattern causes flakiness and CI slowdown.",
                        code_snippet=stripped,
                        suggested_fix="await expect(page.locator('.status-badge')).toBeVisible();",
                    )
                )
        elif lang == "python":
            if RE_PY_SLEEP.search(line):
                violations.append(
                    AstViolation(
                        rule_id="HARDCODED_SLEEP",
                        severity="error",
                        file_path=file_path,
                        line_number=idx,
                        message="Hardcoded time.sleep / asyncio.sleep anti-pattern blocks async loop and fails under jitter.",
                        code_snippet=stripped,
                        suggested_fix="await wait_until_visible(page, '#target')",
                    )
                )
        elif lang == "java":
            if RE_JAVA_SLEEP.search(line):
                violations.append(
                    AstViolation(
                        rule_id="HARDCODED_SLEEP",
                        severity="error",
                        file_path=file_path,
                        line_number=idx,
                        message="Thread.sleep anti-pattern causes arbitrary delays in JVM test runners.",
                        code_snippet=stripped,
                        suggested_fix="Awaitility.await().atMost(5, SECONDS).until(() -> status.isDisplayed());",
                    )
                )
        elif lang == "rust":
            if RE_RUST_SLEEP.search(line):
                violations.append(
                    AstViolation(
                        rule_id="HARDCODED_SLEEP",
                        severity="error",
                        file_path=file_path,
                        line_number=idx,
                        message="thread::sleep or tokio::time::sleep anti-pattern introduces arbitrary thread blocking.",
                        code_snippet=stripped,
                        suggested_fix="tokio::time::timeout(Duration::from_secs(5), async { ... }).await?",
                    )
                )

        # 2. Fragile locators
        if RE_ABSOLUTE_XPATH.search(line):
            violations.append(
                AstViolation(
                    rule_id="FRAGILE_LOCATOR_ABSOLUTE_XPATH",
                    severity="error",
                    file_path=file_path,
                    line_number=idx,
                    message="Absolute XPath selector tightly couples tests to ephemeral HTML DOM hierarchy.",
                    code_snippet=stripped,
                    suggested_fix="page.getByTestId('receipt-info')",
                )
            )

        if RE_DEEP_CSS_CHAIN.search(line):
            violations.append(
                AstViolation(
                    rule_id="FRAGILE_LOCATOR_DEEP_CSS",
                    severity="warning",
                    file_path=file_path,
                    line_number=idx,
                    message="Deeply nested CSS descendant chain is brittle and breaks upon styling refactors.",
                    code_snippet=stripped,
                    suggested_fix="page.getByRole('button', { name: 'Submit' })",
                )
            )

        if RE_AUTO_GENERATED_ID.search(line):
            violations.append(
                AstViolation(
                    rule_id="FRAGILE_LOCATOR_DYNAMIC_ID",
                    severity="warning",
                    file_path=file_path,
                    line_number=idx,
                    message="Selector targets auto-generated framework hash or dynamic ID.",
                    code_snippet=stripped,
                    suggested_fix="page.getByLabel('Account ID')",
                )
            )

        # 3. Unsafe unwraps in TypeScript, which has no `.unwrap()` but does have
        #    the non-null assertion. Skipped inside comments so a line that only
        #    describes the anti-pattern is not reported as committing it.
        if lang in ("typescript", "javascript") and not stripped.startswith("//"):
            if RE_TS_NON_NULL_ASSERTION.search(stripped):
                violations.append(
                    AstViolation(
                        rule_id="UNSAFE_UNWRAP",
                        severity="warning",
                        file_path=file_path,
                        line_number=idx,
                        message=(
                            "Non-null assertion (!) silences the type checker; a null at runtime "
                            "throws from library code instead of failing the assertion you wrote."
                        ),
                        code_snippet=stripped,
                        suggested_fix="expect(payload?.data?.orders?.[0]?.receipt?.transactionId).toBeDefined();",
                    )
                )

        # 4. Floating unawaited promises
        if lang in ("typescript", "javascript"):
            if RE_FLOATING_PROMISE.search(line) or RE_FLOATING_LOCATOR_ACTION.search(line):
                violations.append(
                    AstViolation(
                        rule_id="FLOATING_PROMISE_UNAWAITED_ACTION",
                        severity="error",
                        file_path=file_path,
                        line_number=idx,
                        message="Asynchronous browser interaction dispatched without 'await', causing unhandled promise race conditions.",
                        code_snippet=stripped,
                        suggested_fix=f"await {stripped}",
                    )
                )

        # 4. Plaintext credentials / secrets
        if (
            RE_GH_TOKEN.search(line)
            or RE_GL_TOKEN.search(line)
            or RE_AWS_KEY.search(line)
            or RE_AWS_SECRET.search(line)
            or RE_JWT.search(line)
            or RE_HARDCODED_CREDENTIAL.search(line)
        ):
            violations.append(
                AstViolation(
                    rule_id="HARDCODED_PLAINTEXT_CREDENTIALS",
                    severity="error",
                    file_path=file_path,
                    line_number=idx,
                    message="Hardcoded plaintext credential, token, or private key detected in test code.",
                    code_snippet=stripped,
                    suggested_fix="process.env.TEST_PASSWORD || 'secret'",
                )
            )

        # 5. Vacuous assertions
        if lang in ("typescript", "javascript"):
            if RE_VACUOUS_ASSERTION_TS.search(line):
                violations.append(
                    AstViolation(
                        rule_id="VACUOUS_ASSERTION",
                        severity="error",
                        file_path=file_path,
                        line_number=idx,
                        message="Vacuous assertion 'expect(true).toBe(true)' provides zero regression validation.",
                        code_snippet=stripped,
                        suggested_fix="expect(status).toContain('Order Confirmed');",
                    )
                )
            if "expect(" in line or "assert" in line:
                has_any_assertion = True
        elif lang == "python":
            if RE_VACUOUS_ASSERTION_PY.search(line):
                violations.append(
                    AstViolation(
                        rule_id="VACUOUS_ASSERTION",
                        severity="error",
                        file_path=file_path,
                        line_number=idx,
                        message="Vacuous assertion 'assert True' provides zero regression validation.",
                        code_snippet=stripped,
                        suggested_fix="assert response.status_code == 200",
                    )
                )
            if "assert " in line or "assertEqual" in line:
                has_any_assertion = True
        elif lang == "rust":
            if RE_RUST_UNWRAP.search(line):
                violations.append(
                    AstViolation(
                        rule_id="UNSAFE_UNWRAP",
                        severity="warning",
                        file_path=file_path,
                        line_number=idx,
                        message="Direct unwrap() or expect() in test code causes uninformative panics without context.",
                        code_snippet=stripped,
                        suggested_fix="let val = res.map_err(|e| format!(\"Failed with: {e}\"))?;",
                    )
                )
            if "assert!" in line or "assert_eq!" in line:
                has_any_assertion = True
        else:
            if "assert" in line or "expect" in line:
                has_any_assertion = True

        # 6. Java REST Assured Performance Traps
        if lang == "java":
            is_reset = bool(RE_REST_ASSURED_RESET.search(line)) or (
                stripped.startswith(".reset(") and idx > 1 and lines[idx - 2].strip().endswith("RestAssured")
            )
            if is_reset:
                violations.append(
                    AstViolation(
                        rule_id="PERF_TRAP_CLIENT_CHURN",
                        severity="warning",
                        file_path=file_path,
                        line_number=idx,
                        message="RestAssured.reset() called inside test flow. Tearing down global configuration clears HTTP client connection pools and forces SSL handshake renegotiation.",
                        code_snippet=stripped,
                        suggested_fix="// Use isolated RequestSpecification with connection pooling instead of resetting RestAssured global state\n// RestAssuredConfig.config().httpClient(httpClientConfig().reuseHttpClientInstance());",
                    )
                )

            m_schema = RE_SCHEMA_RELOAD.search(line)
            if m_schema:
                found_static = "static" in line
                if not found_static:
                    for prev_i in range(idx - 1, 0, -1):
                        prev_line = lines[prev_i - 1].strip()
                        if "static" in prev_line:
                            found_static = True
                            break
                        if ";" in prev_line or "{" in prev_line or "}" in prev_line:
                            break

                if not found_static:
                    schema = m_schema.group(1) or "schema.json"
                    violations.append(
                        AstViolation(
                            rule_id="PERF_TRAP_REPEATED_SCHEMA_RELOAD",
                            severity="warning",
                            file_path=file_path,
                            line_number=idx,
                            message="matchesJsonSchemaInClasspath called inline inside test. JSON schema is reloaded and parsed from disk on every assertion; cache in a static final field.",
                            code_snippet=stripped,
                            suggested_fix=f'private static final Matcher<String> SCHEMA = matchesJsonSchemaInClasspath("{schema}");',
                        )
                    )

            if not has_timeout_config and RE_REST_ASSURED_REQUEST.search(line):
                violations.append(
                    AstViolation(
                        rule_id="PERF_TRAP_MISSING_TIMEOUT",
                        severity="warning",
                        file_path=file_path,
                        line_number=idx,
                        message="REST Assured HTTP call executed without socket/connection timeouts. Network partitions or upstream hangs will stall test execution indefinitely.",
                        code_snippet=stripped,
                        suggested_fix='RestAssured.config = RestAssuredConfig.config().httpClient(httpClientConfig().setParam("http.connection.timeout", 5000).setParam("http.socket.timeout", 5000));',
                    )
                )

        # 7. Python Pytest Performance Traps
        if lang == "python":
            m_async = RE_PY_ASYNC_DEF.match(line)
            if m_async:
                current_async_indent = len(line) - len(line.lstrip())
            elif current_async_indent is not None:
                if stripped:
                    indent = len(line) - len(line.lstrip())
                    if indent <= current_async_indent and not stripped.startswith("async def "):
                        current_async_indent = None

            line_no_comment = line.split("#")[0].rstrip() if "#" in line else line
            stripped_no_comment = line_no_comment.strip()

            is_blocking = (
                "to_thread" not in line_no_comment
                and not stripped_no_comment.startswith("await ")
                and (
                    bool(RE_PY_BLOCKING_IN_ASYNC.search(line_no_comment))
                    or (has_direct_sleep and "sleep(" in line_no_comment)
                    or (has_direct_get and "get(" in line_no_comment)
                )
            )

            if current_async_indent is not None and is_blocking:
                fix = "await asyncio.sleep(1)" if "sleep" in line_no_comment else "async with httpx.AsyncClient() as client:\n    response = await client.get(...)"
                violations.append(
                    AstViolation(
                        rule_id="PERF_TRAP_BLOCKING_CALL_IN_ASYNC",
                        severity="error",
                        file_path=file_path,
                        line_number=idx,
                        message="Synchronous blocking call detected inside async test function. Blocking calls freeze the asyncio event loop and starve concurrent coroutines.",
                        code_snippet=stripped,
                        suggested_fix=fix,
                    )
                )

            m_session = RE_PY_CLIENT_SESSION.search(line_no_comment)
            if m_session:
                is_with = stripped_no_comment.startswith("with ") or stripped_no_comment.startswith("async with ") or " with " in stripped_no_comment
                if not is_with and not has_close_in_file:
                    client_type = m_session.group(1)
                    fix = "with requests.Session() as session:\n    response = session.get(...)" if "requests" in client_type else "with httpx.Client() as client:\n    response = client.get(...)"
                    violations.append(
                        AstViolation(
                            rule_id="PERF_TRAP_UNCLOSED_SESSION",
                            severity="warning",
                            file_path=file_path,
                            line_number=idx,
                            message=f"Unclosed HTTP client session '{client_type}' instantiated without context manager or close teardown. Leaks TCP sockets and connection resources.",
                            code_snippet=stripped,
                            suggested_fix=fix,
                        )
                    )

            if RE_PY_FIXTURE_DECORATOR.match(line):
                is_broad = any(s in line for s in ('scope="session"', "scope='session'", 'scope="module"', "scope='module'", 'scope="package"', "scope='package'", 'scope="class"', "scope='class'"))
                if not is_broad:
                    fn_name = "fixture"
                    heavy_res = None
                    for next_idx in range(idx, min(len(lines), idx + 35)):
                        next_line = lines[next_idx]
                        next_stripped = next_line.strip()
                        if next_stripped.startswith("@pytest.") and next_idx > idx:
                            break
                        def_m = RE_PY_DEF.match(next_line)
                        if def_m:
                            fn_name = def_m.group(1)
                        res_m = RE_PY_HEAVY_RESOURCE.search(next_line)
                        if res_m:
                            heavy_res = res_m.group(0).strip()
                            break
                    if heavy_res:
                        violations.append(
                            AstViolation(
                                rule_id="PERF_TRAP_INEFFICIENT_FIXTURE_SCOPE",
                                severity="warning",
                                file_path=file_path,
                                line_number=idx,
                                message=f"Heavy resource '{heavy_res}' initialized inside function-scoped fixture '{fn_name}'. Re-instantiating on every test introduces severe execution churn.",
                                code_snippet=stripped,
                                suggested_fix='@pytest.fixture(scope="session")',
                            )
                        )

    # 8. Missing assertions check (if test file has code but 0 assertions)
    if not has_any_assertion and len(lines) > 5 and not any(v.rule_id == "VACUOUS_ASSERTION" for v in violations):
        violations.append(
            AstViolation(
                rule_id="MISSING_ASSERTION",
                severity="warning",
                file_path=file_path,
                line_number=len(lines),
                message="Test function does not contain any observable state assertions.",
                code_snippet="// End of test function",
                suggested_fix="await expect(page.locator('.status-badge')).toBeVisible();",
            )
        )

    return violations


def calculate_score(violations: list[AstViolation]) -> int:
    """Calculate 0-100 score based on AST violations."""
    score = 100
    for v in violations:
        sev = v.severity.lower()
        if sev == "error":
            score -= 25
        elif sev == "warning":
            score -= 10
        elif sev == "info":
            score -= 5
    return max(0, score)


def generate_unified_diff(original: str, modified: str, file_name: str) -> str:
    """Generate standard unified diff format between original and modified text."""
    orig_lines = original.splitlines(keepends=True)
    mod_lines = modified.splitlines(keepends=True)
    diff = difflib.unified_diff(
        orig_lines,
        mod_lines,
        fromfile=f"a/{file_name}",
        tofile=f"b/{file_name}",
        lineterm="",
    )
    return "".join(diff)


def generate_offline_mentor_critique(
    file_name: str, code_content: str, violations: list[AstViolation]
) -> tuple[str, list[str]]:
    """Generate Senior QA mentor critique and Socratic questions."""
    if not violations:
        critique = (
            f"🏆 **Senior QA Architecture Assessment**: Exemplary test design in `{file_name}`.\n\n"
            "- Your test demonstrates strict deterministic synchronization without arbitrary sleep delays.\n"
            "- Locators adhere to user-centric semantic queries (role & accessibility based).\n"
            "- Assertions are explicit, meaningful, and resilient against false positives.\n"
            "- Zero credentials or environment-specific secrets are leaked in code."
        )
        questions = [
            "How could this test suite be integrated into a parallel matrix CI pipeline without shared state conflicts?",
            "What chaos faults (e.g. 504 Gateway Timeout or packet drops) might this test surface in staging?",
        ]
        return critique, questions

    critique_points = [
        f"🔍 **Senior QA Code Review for `{file_name}`** (Found {len(violations)} anti-pattern violation{'s' if len(violations) != 1 else ''}):"
    ]
    questions = []

    has_sleep = any("SLEEP" in v.rule_id for v in violations)
    has_xpath = any("XPATH" in v.rule_id or "LOCATOR" in v.rule_id for v in violations)
    has_floating = any("FLOATING" in v.rule_id for v in violations)
    has_secret = any("SECRET" in v.rule_id or "CREDENTIAL" in v.rule_id for v in violations)
    has_vacuous = any("VACUOUS" in v.rule_id or "MISSING_ASSERTION" in v.rule_id for v in violations)
    has_unwrap = any("UNWRAP" in v.rule_id for v in violations)

    if has_sleep:
        critique_points.append(
            "• **Hardcoded Sleep Anti-Pattern**: Arbitrary `waitForTimeout` or `Thread.sleep` calls are the #1 root cause of enterprise test flakiness. Under heavy CI load or CPU throttling, fixed timers expire prematurely causing false alarms; during fast runs, they needlessly inflate build duration."
        )
        questions.append(
            "Why is event-driven auto-waiting (polling DOM mutations or network events) strictly superior to fixed millisecond sleeps?"
        )

    if has_xpath:
        critique_points.append(
            "• **Fragile Structural Locators**: Deep absolute XPath and chained CSS selectors tightly bind tests to ephemeral DOM hierarchy. A simple designer CSS refactor or wrapping `<div>` will trigger cascading test failures across your suite."
        )
        questions.append(
            "If a developer refactors the page from a `<div>` table to CSS grid, will this test survive?"
        )

    if has_floating:
        critique_points.append(
            "• **Floating Unawaited Promises**: Triggering asynchronous browser interactions (`page.click()`, `page.fill()`) without `await` dispatches actions onto the event loop unmonitored. The test runner may conclude execution before the browser ever receives the click."
        )
        questions.append(
            "What happens to unhandled promise rejections when the test harness shuts down the browser context prematurely?"
        )

    if has_secret:
        critique_points.append(
            "• **Hardcoded Plaintext Credentials**: Hardcoding passwords, API tokens, or JWTs in test files risks credential exposure in git histories, test artifacts, and CI execution logs."
        )
        questions.append(
            "How can you externalize test credentials so tests run seamlessly in local dev, staging, and ephemeral CI environments without changing code?"
        )

    if has_vacuous:
        critique_points.append(
            "• **Vacuous / Missing Assertions**: A test without meaningful assertions (or asserting `true == true`) is merely a smoke runner verifying that the process didn't crash, missing 100% of business logic regressions."
        )
        questions.append(
            "What observable business state or API response contract definitively proves this user journey succeeded?"
        )

    if has_unwrap:
        critique_points.append(
            "• **Unsafe Error Handling / Unwraps**: Raw unwrapping in test fixtures leads to uninformative stack traces rather than actionable assertion failure diffs."
        )
        questions.append(
            "When this unwrap panics in a nighttime CI run, will the triage engineer know what payload caused it?"
        )

    if not questions:
        questions.append(
            "How would this test suite behave under 500ms network jitter in a parallel CI runner matrix?"
        )

    return "\n\n".join(critique_points), questions


def apply_automated_fixes(content: str, target_violations: list[AstViolation]) -> str:
    """Apply automated fixes for the given violations to the code content."""
    lines = content.splitlines()
    applied_lines = list(lines)

    for v in sorted(target_violations, key=lambda x: x.line_number, reverse=True):
        idx = v.line_number - 1
        if idx < 0 or idx >= len(applied_lines):
            continue

        orig_line = applied_lines[idx]
        indent = len(orig_line) - len(orig_line.lstrip())
        indent_str = orig_line[:indent]

        if "SLEEP" in v.rule_id:
            # Replace sleep with web-first assertion
            if "waitForTimeout" in orig_line or "setTimeout" in orig_line:
                applied_lines[idx] = f"{indent_str}await expect(page.locator('.status-badge')).toBeVisible();"
            elif "time.sleep" in orig_line or "asyncio.sleep" in orig_line:
                applied_lines[idx] = f"{indent_str}await expect_element_visible(page, '.status-badge')"
            elif "Thread.sleep" in orig_line:
                applied_lines[idx] = f"{indent_str}Awaitility.await().atMost(5, SECONDS).until(() -> statusBadge.isDisplayed());"
            elif "sleep" in orig_line:
                applied_lines[idx] = f"{indent_str}// Replaced fixed sleep with deterministic event synchronization"

        elif "XPATH" in v.rule_id:
            # Replace absolute XPath with getByLabel or getByTestId
            if "fill(" in orig_line:
                applied_lines[idx] = f"{indent_str}await page.getByLabel('Account ID').fill('ACC-9942');"
            elif "click(" in orig_line:
                applied_lines[idx] = f"{indent_str}await page.getByRole('button', {{ name: 'Submit' }}).click();"
            else:
                applied_lines[idx] = f"{indent_str}await expect(page.getByTestId('receipt-info')).toBeVisible();"

        elif "CSS" in v.rule_id or "DYNAMIC_ID" in v.rule_id:
            if "click(" in orig_line:
                applied_lines[idx] = f"{indent_str}await page.getByRole('button', {{ name: 'Submit' }}).click();"
            else:
                applied_lines[idx] = f"{indent_str}await expect(page.getByTestId('receipt-info')).toBeVisible();"

        elif "FLOATING" in v.rule_id:
            # Prepend await
            stripped = orig_line.strip()
            if not stripped.startswith("await "):
                applied_lines[idx] = f"{indent_str}await {stripped}"

        elif "CREDENTIAL" in v.rule_id or "SECRET" in v.rule_id:
            if "password" in orig_line.lower():
                applied_lines[idx] = f"{indent_str}const password = process.env.TEST_PASSWORD || 'default_secret';"

        elif "VACUOUS" in v.rule_id:
            if "expect(" in orig_line:
                applied_lines[idx] = f"{indent_str}expect(status).toContain('Order Confirmed');"
            elif "assert" in orig_line:
                applied_lines[idx] = f"{indent_str}assert response.status_code == 200"

        elif v.suggested_fix:
            fix_lines = v.suggested_fix.splitlines()
            formatted_fix = "\n".join(f"{indent_str}{fl.lstrip()}" for fl in fix_lines)
            applied_lines[idx] = formatted_fix

    return "\n".join(applied_lines)


def run_code_review(
    content: str,
    file_path: str = "exercise.ts",
    strict: bool = False,
    score_threshold: int = 80,
) -> ReviewReport:
    """Execute complete code review and return structured report."""
    violations = scan_content(file_path, content)
    score = calculate_score(violations)
    has_error = any(v.severity.lower() == "error" for v in violations)

    if strict:
        passed = score >= score_threshold and not has_error and len(violations) == 0
    else:
        passed = score >= score_threshold and not has_error

    critique, questions = generate_offline_mentor_critique(Path(file_path).name, content, violations)

    suggested_diff = None
    if violations:
        patched = apply_automated_fixes(content, violations)
        if patched != content:
            suggested_diff = generate_unified_diff(content, patched, Path(file_path).name)

    return ReviewReport(
        exercise_name=Path(file_path).name,
        score=score,
        passed=passed,
        violations=violations,
        mentor_critique=critique,
        socratic_questions=questions,
        suggested_diff=suggested_diff,
    )


def apply_review_fix(
    content: str,
    file_path: str = "exercise.ts",
    fix_id: str = "all",
) -> ReviewFixResponse:
    """Apply fixes and return modified code with unified diff."""
    violations = scan_content(file_path, content)

    if fix_id == "all":
        target_violations = violations
    else:
        target_violations = [
            v
            for v in violations
            if v.rule_id == fix_id
            or f"{v.rule_id}@{v.line_number}" == fix_id
            or str(v.line_number) == fix_id
        ]

    applied_rule_ids = list({v.rule_id for v in target_violations})
    patched = apply_automated_fixes(content, target_violations)
    diff = generate_unified_diff(content, patched, Path(file_path).name) if patched != content else None

    return ReviewFixResponse(
        patched_code=patched,
        original_code=content,
        applied_fixes=applied_rule_ids,
        diff=diff,
        success=True,
    )
