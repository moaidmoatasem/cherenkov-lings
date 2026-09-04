import { apiUrl } from './api';
import type { AstViolation } from '../pages/CodeReviewPage';

/**
 * Client for the code-review endpoints.
 *
 * The review page scanned code with its own five regex rules and a local score.
 * The backend engine carries a superset — the three fragile-locator variants,
 * unsafe unwraps and vacuous assertions on top of what the page knew — plus the
 * mentor critique, the Socratic questions and a real patcher. Both existed; only
 * the weaker one was reachable.
 */

interface ApiViolation {
  rule_id: string;
  severity: 'error' | 'warning' | 'info';
  file_path: string;
  line_number: number;
  message: string;
  code_snippet: string;
  suggested_fix?: string | null;
}

interface ApiReviewReport {
  exercise_name: string;
  score: number;
  passed: boolean;
  violations: ApiViolation[];
  mentor_critique: string;
  socratic_questions: string[];
  suggested_diff?: string | null;
}

export interface ReviewResult {
  score: number;
  passed: boolean;
  violations: AstViolation[];
  mentorCritique: string;
  socraticQuestions: string[];
  suggestedDiff: string;
}

/**
 * Presentation for each rule the engine can raise. The API returns a rule id and
 * a message; the page's UI also wants a human name and a button label. Keyed by
 * rule id so a rule the table has not met yet still renders, rather than
 * silently disappearing from the list.
 */
const RULE_META: Record<string, { name: string; fixLabel: string; asks: RegExp }> = {
  HARDCODED_SLEEP: {
    name: 'Hardcoded Sleep',
    fixLabel: 'Replace with a web-first assertion',
    asks: /sleep|auto-wait|polling/i,
  },
  FRAGILE_LOCATOR_ABSOLUTE_XPATH: {
    name: 'Fragile Locator — Absolute XPath',
    fixLabel: 'Replace with a role or test-id locator',
    asks: /refactor|css grid|<div>/i,
  },
  FRAGILE_LOCATOR_DEEP_CSS: {
    name: 'Fragile Locator — Deep CSS Chain',
    fixLabel: 'Replace with a role or test-id locator',
    asks: /refactor|css grid|<div>/i,
  },
  FRAGILE_LOCATOR_DYNAMIC_ID: {
    name: 'Fragile Locator — Generated Id',
    fixLabel: 'Replace with a stable locator',
    asks: /refactor|css grid|<div>/i,
  },
  FLOATING_PROMISE_UNAWAITED_ACTION: {
    name: 'Floating Promise',
    fixLabel: 'Await the action',
    asks: /promise|await|rejection/i,
  },
  UNSAFE_UNWRAP: {
    name: 'Unsafe Unwrap',
    fixLabel: 'Handle the error case',
    asks: /unwrap|panic|error case/i,
  },
  HARDCODED_PLAINTEXT_CREDENTIALS: {
    name: 'Hardcoded Credentials',
    fixLabel: 'Read the secret from the environment',
    asks: /credential|secret|externalize/i,
  },
  MISSING_ASSERTION: {
    name: 'Missing Assertions',
    fixLabel: 'Assert the business outcome',
    asks: /business state|assert|prove/i,
  },
  VACUOUS_ASSERTION: {
    name: 'Vacuous Assertion',
    fixLabel: 'Assert something that can fail',
    asks: /business state|assert|prove/i,
  },
};

/** Turn HARDCODED_SLEEP into "Hardcoded Sleep" for a rule the table lacks. */
const humanise = (ruleId: string): string =>
  ruleId
    .toLowerCase()
    .split('_')
    .map((word) => word.charAt(0).toUpperCase() + word.slice(1))
    .join(' ');

const toViolation = (item: ApiViolation, index: number, questions: string[]): AstViolation => {
  const meta = RULE_META[item.rule_id];
  // Pick the question that belongs to this rule rather than the one at the same
  // index: the engine returns questions per rule family, not per violation.
  const question = (meta && questions.find((q) => meta.asks.test(q))) ?? questions[index] ?? '';

  return {
    id: `${item.rule_id}-${item.line_number}-${index}`,
    rule_id: item.rule_id,
    rule_name: meta?.name ?? humanise(item.rule_id),
    severity: item.severity,
    line_number: item.line_number,
    message: item.message,
    code_snippet: item.code_snippet,
    suggested_fix: item.suggested_fix ?? undefined,
    socratic_prompt: question,
    // The engine explains itself through the critique and the suggested fix; it
    // does not ship a written answer per question, and inventing one here would
    // be putting words in the mentor's mouth.
    socratic_answer: item.suggested_fix
      ? `Consider: ${item.suggested_fix}`
      : item.message,
    fix_label: meta?.fixLabel ?? 'Apply the suggested fix',
    replacement_code: item.suggested_fix ?? item.code_snippet,
  };
};

export async function runReview(code: string, filePath: string): Promise<ReviewResult> {
  const res = await fetch(apiUrl('/api/review'), {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({ code, file_path: filePath }),
  });
  if (!res.ok) throw new Error(`POST /api/review -> ${res.status}`);
  const report: ApiReviewReport = await res.json();

  return {
    score: report.score,
    passed: report.passed,
    violations: report.violations.map((v, i) => toViolation(v, i, report.socratic_questions ?? [])),
    mentorCritique: report.mentor_critique ?? '',
    socraticQuestions: report.socratic_questions ?? [],
    suggestedDiff: report.suggested_diff ?? '',
  };
}

export interface ReviewFixResult {
  patchedCode: string;
  appliedFixes: string[];
  diff: string;
  success: boolean;
}

/** No filePath argument on purpose — see the body. */
export async function applyReviewFix(code: string, fixId: string): Promise<ReviewFixResult> {
  const res = await fetch(apiUrl('/api/review/fix'), {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    // No file_path in the body on purpose: supplying one makes the endpoint
    // write the patch to that file on the server. The editor holds the code, so
    // the patch belongs in the response, not on somebody's disk.
    body: JSON.stringify({ code, fix_id: fixId }),
  });
  if (!res.ok) throw new Error(`POST /api/review/fix -> ${res.status}`);
  const data = await res.json();

  return {
    patchedCode: data.patched_code ?? code,
    appliedFixes: data.applied_fixes ?? [],
    diff: data.diff ?? '',
    success: Boolean(data.success),
  };
}
