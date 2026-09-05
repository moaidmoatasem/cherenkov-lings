import { apiUrl } from './api';

/**
 * Client for the CI/CD pipeline endpoints.
 *
 * The Pipeline Builder page generated real GitHub Actions YAML from a visual
 * stage canvas, but "running" it was a setInterval that invented step logs and
 * pass counts ("✓ 15 tests passed on Shard 2/4") with no backend call.  The
 * backend carries both a policy validator and a simulated matrix-runner whose
 * results — jobs, steps, durations, logs — are deterministic from the YAML
 * content and genuinely reflect the workflow's structure.
 *
 * Pattern follows reviewApi.ts / triageApi.ts exactly.
 */

// ---------------------------------------------------------------------------
// Backend response shapes (mirrors crucible/backend/models.py)
// ---------------------------------------------------------------------------

interface ApiStepRunResult {
  name: string;
  status: string;
  duration_ms: number;
  exit_code: number;
  output: string;
}

interface ApiJobRunResult {
  job_id: string;
  runner_name: string;
  matrix_combination: Record<string, string>;
  status: string;
  duration_ms: number;
  steps: ApiStepRunResult[];
}

interface ApiLogEntry {
  timestamp: number;
  runner: string;
  step: string;
  level: string;
  message: string;
}

interface ApiPipelineError {
  code: string;
  message: string;
  job: string | null;
  step: string | null;
  line: number | null;
  suggestion: string | null;
}

interface ApiPipelineWarning {
  code: string;
  message: string;
  job: string | null;
  step: string | null;
  suggestion: string | null;
}

interface ApiPipelineValidation {
  valid: boolean;
  sdet_score: number;
  matrix_detected: boolean;
  artifact_upload_detected: boolean;
  errors: ApiPipelineError[];
  warnings: ApiPipelineWarning[];
  summary: string;
}

interface ApiPipelineRunResult {
  workflow_name: string;
  jobs: ApiJobRunResult[];
  duration_ms: number;
  success: boolean;
  logs: ApiLogEntry[];
  validation: ApiPipelineValidation | null;
}

// ---------------------------------------------------------------------------
// Frontend-facing types (camelCase, narrower unions where useful)
// ---------------------------------------------------------------------------

export interface StepResult {
  name: string;
  status: 'pending' | 'running' | 'success' | 'failed';
  durationMs: number;
  exitCode: number;
  output: string;
}

export interface JobResult {
  jobId: string;
  runnerName: string;
  matrixCombination: Record<string, string>;
  status: 'idle' | 'running' | 'success' | 'failed';
  durationMs: number;
  steps: StepResult[];
}

export interface LogEntry {
  timestamp: number;
  runner: string;
  step: string;
  level: string;
  message: string;
}

export interface PipelineValidationResult {
  valid: boolean;
  sdetScore: number;
  matrixDetected: boolean;
  artifactUploadDetected: boolean;
  errors: Array<{
    code: string;
    message: string;
    job: string | null;
    step: string | null;
    line: number | null;
    suggestion: string | null;
  }>;
  warnings: Array<{
    code: string;
    message: string;
    job: string | null;
    step: string | null;
    suggestion: string | null;
  }>;
  summary: string;
}

export interface PipelineRunResult {
  workflowName: string;
  jobs: JobResult[];
  durationMs: number;
  success: boolean;
  logs: LogEntry[];
  validation: PipelineValidationResult | null;
}

// ---------------------------------------------------------------------------
// Mappers
// ---------------------------------------------------------------------------

function toStepResult(s: ApiStepRunResult): StepResult {
  const status =
    s.status === 'passed' || s.status === 'success'
      ? 'success'
      : s.status === 'failed'
        ? 'failed'
        : s.status === 'running'
          ? 'running'
          : 'pending';
  return {
    name: s.name,
    status: status as StepResult['status'],
    durationMs: s.duration_ms,
    exitCode: s.exit_code,
    output: s.output,
  };
}

function toJobResult(j: ApiJobRunResult): JobResult {
  const status =
    j.status === 'passed' || j.status === 'success'
      ? 'success'
      : j.status === 'failed'
        ? 'failed'
        : j.status === 'running'
          ? 'running'
          : 'idle';
  return {
    jobId: j.job_id,
    runnerName: j.runner_name,
    matrixCombination: j.matrix_combination,
    status: status as JobResult['status'],
    durationMs: j.duration_ms,
    steps: j.steps.map(toStepResult),
  };
}

function toValidation(v: ApiPipelineValidation): PipelineValidationResult {
  return {
    valid: v.valid,
    sdetScore: v.sdet_score,
    matrixDetected: v.matrix_detected,
    artifactUploadDetected: v.artifact_upload_detected,
    errors: v.errors,
    warnings: v.warnings,
    summary: v.summary,
  };
}

function toRunResult(r: ApiPipelineRunResult): PipelineRunResult {
  return {
    workflowName: r.workflow_name,
    jobs: r.jobs.map(toJobResult),
    durationMs: r.duration_ms,
    success: r.success,
    logs: r.logs,
    validation: r.validation ? toValidation(r.validation) : null,
  };
}

// ---------------------------------------------------------------------------
// Errors
// ---------------------------------------------------------------------------

/**
 * A response the backend actually sent — e.g. a 400 for YAML that fails to
 * parse or a matrix that exceeds the combination cap. Distinct from a plain
 * network failure so callers can tell "the backend rejected this run" from
 * "the backend is unreachable" instead of collapsing both into the same
 * fallback path and presenting a rejected run as a fake success.
 */
export class PipelineApiError extends Error {
  status: number;

  constructor(status: number, message: string) {
    super(message);
    this.name = 'PipelineApiError';
    this.status = status;
  }
}

async function throwApiError(res: Response, path: string): Promise<never> {
  let detail = `${path} -> ${res.status}`;
  try {
    const body = await res.json();
    if (typeof body?.detail === 'string' && body.detail.trim()) detail = body.detail;
  } catch {
    // Non-JSON error body — keep the status-based message.
  }
  throw new PipelineApiError(res.status, detail);
}

// ---------------------------------------------------------------------------
// Public API
// ---------------------------------------------------------------------------

/**
 * Validate a GitHub Actions workflow YAML against enterprise SDET policies.
 *
 * Maps to `POST /api/pipeline/validate`.
 */
export async function validatePipeline(
  yamlContent: string,
  strict = false,
  signal?: AbortSignal,
): Promise<PipelineValidationResult> {
  const res = await fetch(apiUrl('/api/pipeline/validate'), {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({ yaml_content: yamlContent, strict }),
    signal,
  });
  if (!res.ok) await throwApiError(res, 'POST /api/pipeline/validate');
  const data: ApiPipelineValidation = await res.json();
  return toValidation(data);
}

/**
 * Run a simulated CI/CD pipeline from the provided YAML.
 *
 * Maps to `POST /api/pipeline/run`. Returns the full execution outcome
 * including jobs, steps, logs, and an optional embedded validation.
 *
 * Throws `PipelineApiError` for a response the backend actually sent (bad
 * YAML, a matrix that exceeds the combination cap, etc.) so callers can
 * distinguish that from a genuine network failure.
 */
export async function runPipeline(
  yamlContent: string,
  opts: {
    parallel?: boolean;
    failFast?: boolean;
    strictValidation?: boolean;
    verbose?: boolean;
  } = {},
): Promise<PipelineRunResult> {
  const res = await fetch(apiUrl('/api/pipeline/run'), {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({
      yaml_content: yamlContent,
      parallel: opts.parallel ?? true,
      fail_fast: opts.failFast ?? false,
      strict_validation: opts.strictValidation ?? false,
      verbose: opts.verbose ?? true,
    }),
  });
  if (!res.ok) await throwApiError(res, 'POST /api/pipeline/run');
  const data: ApiPipelineRunResult = await res.json();
  return toRunResult(data);
}
