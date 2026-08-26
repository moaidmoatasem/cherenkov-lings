"""CI/CD Pipeline YAML Validator and Parallel Matrix Simulator.

Implements strict enterprise SDET validation policies (matrix parallelism, artifact uploads,
secret scanning, concurrency cancellation, and timeouts) and simulated matrix execution.
"""

from __future__ import annotations

import itertools
import re
import time
import yaml

from crucible.backend.models import (
    JobRunResult,
    LogEntry,
    PipelineError,
    PipelineRunResult,
    PipelineValidation,
    PipelineWarning,
    StepRunResult,
)

# Secret scanning regexes
RE_GH_TOKEN = re.compile(r"ghp_[A-Za-z0-9]{36}")
RE_GL_TOKEN = re.compile(r"glpat-[A-Za-z0-9\-]{20,}")
RE_AWS_KEY = re.compile(r"AKIA[0-9A-Z]{16}")
RE_AWS_SECRET = re.compile(r"""(?i)AWS_SECRET_ACCESS_KEY\s*[:=]\s*["']?[A-Za-z0-9/+=]{30,}["']?""")
RE_JWT = re.compile(r"Bearer\s+ey[A-Za-z0-9_\-\.]{20,}")
RE_PRIVATE_KEY = re.compile(r"-----BEGIN\s+(?:RSA|OPENSSH|EC|DSA|PGP)?\s*PRIVATE KEY-----")
RE_HARDCODED_CREDENTIAL = re.compile(
    r"""(?i)(?:password|passwd|api_key|secret_key|auth_token)\s*[:=]\s*["']([^"'\s]{8,})["']"""
)


def validate_workflow_yaml(yaml_content: str, strict: bool = False) -> PipelineValidation:
    """Validate workflow YAML content against enterprise SDET policies."""
    errors: list[PipelineError] = []
    warnings: list[PipelineWarning] = []
    matrix_detected = False
    artifact_upload_detected = False

    # 1. Parse YAML
    try:
        data = yaml.safe_load(yaml_content)
    except Exception as e:
        return PipelineValidation(
            valid=False,
            sdet_score=0,
            matrix_detected=False,
            artifact_upload_detected=False,
            errors=[
                PipelineError(
                    code="YAML_SYNTAX_ERROR",
                    message=f"Failed to parse workflow YAML: {e}",
                    suggestion="Ensure YAML indentation and syntax follow GitHub Actions specifications.",
                )
            ],
            warnings=[],
            summary=f"YAML Syntax Error: {e}",
        )

    if not isinstance(data, dict):
        return PipelineValidation(
            valid=False,
            sdet_score=0,
            matrix_detected=False,
            artifact_upload_detected=False,
            errors=[
                PipelineError(
                    code="INVALID_STRUCTURE",
                    message="Workflow root must be a YAML mapping/dictionary.",
                    suggestion="Define top-level keys like 'name', 'on', and 'jobs'.",
                )
            ],
            warnings=[],
            summary="Invalid workflow structure: Root is not a dictionary.",
        )

    # 2. Concurrency validation
    triggers = data.get("on")
    has_push_or_pr = False
    if isinstance(triggers, list):
        has_push_or_pr = any(t in ("push", "pull_request") for t in triggers)
    elif isinstance(triggers, dict):
        has_push_or_pr = any(k in ("push", "pull_request") for k in triggers.keys())
    elif isinstance(triggers, str):
        has_push_or_pr = triggers in ("push", "pull_request")

    concurrency = data.get("concurrency")
    if has_push_or_pr:
        if concurrency is None:
            warnings.append(
                PipelineWarning(
                    code="MISSING_CONCURRENCY",
                    message="Workflow is triggered on push/pull_request but lacks top-level 'concurrency' configuration with 'cancel-in-progress: true'.",
                    suggestion="Add 'concurrency: { group: ${{ github.workflow }}-${{ github.ref }}, cancel-in-progress: true }'.",
                )
            )
        elif isinstance(concurrency, dict):
            if not concurrency.get("cancel-in-progress", False):
                warnings.append(
                    PipelineWarning(
                        code="CONCURRENCY_CANCEL_DISABLED",
                        message="Concurrency group does not set 'cancel-in-progress: true'. Stale branch runs will not be cancelled on rapid commits.",
                        suggestion="Add 'cancel-in-progress: true' to the concurrency block.",
                    )
                )

    # 3. Secret scanning on entire raw YAML
    for line_idx, line in enumerate(yaml_content.splitlines(), start=1):
        if (
            RE_GH_TOKEN.search(line)
            or RE_GL_TOKEN.search(line)
            or RE_AWS_KEY.search(line)
            or RE_AWS_SECRET.search(line)
            or RE_JWT.search(line)
            or RE_PRIVATE_KEY.search(line)
            or RE_HARDCODED_CREDENTIAL.search(line)
        ):
            errors.append(
                PipelineError(
                    code="HARDCODED_SECRET_DETECTED",
                    message="Hardcoded token, secret access key, or credential pattern detected in workflow file.",
                    line=line_idx,
                    suggestion="Store secrets in GitHub Actions Repository Secrets and reference via ${{ secrets.SECRET_NAME }}.",
                )
            )

    # 4. Jobs validation
    jobs = data.get("jobs", {})
    if not isinstance(jobs, dict) or not jobs:
        errors.append(
            PipelineError(
                code="NO_JOBS_DEFINED",
                message="Workflow definition contains no jobs.",
                suggestion="Define at least one job under the 'jobs' mapping.",
            )
        )

    for job_id, job_def in jobs.items():
        if not isinstance(job_def, dict):
            continue

        # Check job timeout
        if "timeout-minutes" not in job_def:
            warnings.append(
                PipelineWarning(
                    code="MISSING_JOB_TIMEOUT",
                    message=f"Job '{job_id}' lacks explicit 'timeout-minutes'. Unbounded jobs can hang indefinitely in CI runners.",
                    job=job_id,
                    suggestion="Add 'timeout-minutes: 15' (or appropriate budget) to avoid stuck runners.",
                )
            )

        # Determine if this is a test job
        steps = job_def.get("steps", [])
        is_test_job = "test" in str(job_id).lower() or any(
            isinstance(s, dict)
            and (
                "test" in str(s.get("name", "")).lower()
                or "test" in str(s.get("run", "")).lower()
                or "pytest" in str(s.get("run", "")).lower()
                or "playwright" in str(s.get("run", "")).lower()
                or "k6" in str(s.get("run", "")).lower()
                or "mvn" in str(s.get("run", "")).lower()
            )
            for s in steps
            if isinstance(s, dict)
        )

        # Check matrix strategy
        strategy = job_def.get("strategy")
        if isinstance(strategy, dict) and "matrix" in strategy and isinstance(strategy["matrix"], dict):
            matrix_detected = True
        elif is_test_job:
            errors.append(
                PipelineError(
                    code="MISSING_MATRIX_STRATEGY",
                    message=f"Test job '{job_id}' does not define a parallel execution matrix ('strategy.matrix'). Enterprise SDET standards require multi-version or sharded parallel matrix testing.",
                    job=job_id,
                    suggestion="Add 'strategy: matrix: { os: [ubuntu-latest, windows-latest], shard: [1/2, 2/2] }'.",
                )
            )

        # Check artifact uploads
        has_artifact_upload = any(
            isinstance(s, dict)
            and (
                "actions/upload-artifact" in str(s.get("uses", ""))
                or "upload-artifact" in str(s.get("name", "")).lower()
            )
            for s in steps
            if isinstance(s, dict)
        )

        if has_artifact_upload:
            artifact_upload_detected = True
        elif is_test_job:
            errors.append(
                PipelineError(
                    code="MISSING_ARTIFACT_UPLOAD",
                    message=f"Test job '{job_id}' does not upload test execution artifacts or reports. CI test runs must publish test logs, traces, or Allure results via 'actions/upload-artifact'.",
                    job=job_id,
                    suggestion="Add a step with 'uses: actions/upload-artifact@v4' to preserve 'allure-results' or test reports.",
                )
            )

    # 5. Compute SDET score
    score = 100
    if not matrix_detected:
        score -= 30
    if not artifact_upload_detected:
        score -= 30
    if any(e.code == "HARDCODED_SECRET_DETECTED" for e in errors):
        score -= 40
    if any(w.code.startswith("MISSING_JOB_TIMEOUT") for w in warnings):
        score -= 10
    if any("CONCURRENCY" in w.code for w in warnings):
        score -= 10
    if any(e.code in ("YAML_SYNTAX_ERROR", "NO_JOBS_DEFINED", "INVALID_STRUCTURE") for e in errors):
        score = 0

    score = max(0, min(100, score))
    valid = len(errors) == 0

    summary = f"Enterprise SDET Policy Validation: Score {score}/100 — {'VALID' if valid else 'POLICY VIOLATIONS FOUND'}"

    return PipelineValidation(
        valid=valid,
        sdet_score=score,
        matrix_detected=matrix_detected,
        artifact_upload_detected=artifact_upload_detected,
        errors=errors,
        warnings=warnings,
        summary=summary,
    )


MAX_MATRIX_COMBINATIONS = 32

def simulate_pipeline_run(
    yaml_content: str,
    parallel: bool = True,
    fail_fast: bool = False,
    strict_validation: bool = False,
    verbose: bool = True,
) -> PipelineRunResult:
    """Simulate parallel matrix execution of a workflow."""
    validation = validate_workflow_yaml(yaml_content, strict=strict_validation)

    yaml_error: str | None = None
    try:
        data = yaml.safe_load(yaml_content) or {}
    except Exception as exc:
        # Keep the reason. Discarding it reports a failed run with no diagnostic
        # in the logs, which leaves the learner nothing to act on.
        yaml_error = str(exc)
        data = {}

    if not isinstance(data, dict):
        # Scalar or sequence at the root: no jobs to run, and `.get` would raise.
        data = {}

    workflow_name = data.get("name", "Enterprise CI/CD Workflow")
    jobs_dict = data.get("jobs", {})

    job_results: list[JobRunResult] = []
    logs: list[LogEntry] = []
    start_ts = int(time.time() * 1000)

    if yaml_error is not None:
        logs.append(
            LogEntry(
                timestamp=start_ts,
                runner="orchestrator",
                step="parse_workflow",
                level="error",
                message=f"YAML parsing error: {yaml_error}",
            )
        )

    runner_counter = 1
    total_pipeline_duration = 0

    for job_id, job_def in jobs_dict.items():
        if not isinstance(job_def, dict):
            continue

        strategy = job_def.get("strategy", {})
        matrix = strategy.get("matrix", {}) if isinstance(strategy, dict) else {}

        matrix_keys = list(matrix.keys()) if isinstance(matrix, dict) else []
        matrix_values = [matrix[k] if isinstance(matrix[k], list) else [matrix[k]] for k in matrix_keys]
        if matrix_keys:
            total = 1
            for vals in matrix_values:
                total *= len(vals)
                if total > MAX_MATRIX_COMBINATIONS:
                    raise ValueError(f"Matrix explosion: {total} combinations exceeds cap {MAX_MATRIX_COMBINATIONS}")
            combinations = [dict(zip(matrix_keys, [str(v) for v in combo])) for combo in itertools.product(*matrix_values)]
        else:
            combinations = [{}]

        steps_raw = job_def.get("steps", [])

        for combo in combinations:
            combo_suffix = ", ".join(f"{k}: {v}" for k, v in combo.items())
            full_job_id = f"{job_id} ({combo_suffix})" if combo_suffix else job_id
            runner_name = f"GitHub Runner #{runner_counter}"
            runner_counter += 1

            step_results: list[StepRunResult] = []
            job_duration = 0
            job_status = "passed"

            logs.append(
                LogEntry(
                    timestamp=start_ts + job_duration,
                    runner=runner_name,
                    step="job_init",
                    level="info",
                    message=f"Starting job '{full_job_id}' on {combo.get('os', 'ubuntu-latest')}",
                )
            )

            for step_def in steps_raw:
                if not isinstance(step_def, dict):
                    continue

                step_name = step_def.get("name") or step_def.get("uses") or step_def.get("run", "Step")
                step_name = str(step_name).splitlines()[0][:60]

                # Determine duration and output based on step type
                step_lower = step_name.lower()
                if "checkout" in step_lower:
                    duration = 380
                    output = "Repository checked out at HEAD (commit 9e4f2b1)"
                elif "setup" in step_lower or "install" in step_lower:
                    duration = 540
                    output = "Environment and package dependencies resolved successfully"
                elif "test" in step_lower or "playwright" in step_lower or "pytest" in step_lower:
                    duration = 1250
                    output = f"Executing test suite with matrix parameters [{combo_suffix}]...\nPassed: 18, Failed: 0, Duration: 1.25s"
                elif "upload" in step_lower or "artifact" in step_lower:
                    duration = 310
                    output = "Archiving 'allure-results' and test traces -> artifact id: 89124 (1.4 MB)"
                else:
                    duration = 200
                    output = f"Executed: {step_name}"

                job_duration += duration
                step_results.append(
                    StepRunResult(
                        name=step_name,
                        status="passed",
                        duration_ms=duration,
                        exit_code=0,
                        output=output,
                    )
                )

                logs.append(
                    LogEntry(
                        timestamp=start_ts + job_duration,
                        runner=runner_name,
                        step=step_name,
                        level="info",
                        message=f"[{step_name}] {output}",
                    )
                )

            total_pipeline_duration = max(total_pipeline_duration, job_duration)

            job_results.append(
                JobRunResult(
                    job_id=full_job_id,
                    runner_name=runner_name,
                    matrix_combination=combo,
                    status=job_status,
                    duration_ms=job_duration,
                    steps=step_results,
                )
            )

            logs.append(
                LogEntry(
                    timestamp=start_ts + job_duration,
                    runner=runner_name,
                    step="job_finish",
                    level="info",
                    message=f"Job '{full_job_id}' completed with status: PASSED ({job_duration}ms)",
                )
            )

    success = (not strict_validation or validation.valid) and len(job_results) > 0

    return PipelineRunResult(
        workflow_name=workflow_name,
        jobs=job_results,
        duration_ms=total_pipeline_duration,
        success=success,
        logs=logs,
        validation=validation,
    )
