import { apiUrl } from './api';
import type { FailureCategory, TestCaseResult, TestStatus } from '../pages/AllureTriagePage';

/**
 * Client for the triage endpoints.
 *
 * The Allure/Triage page used to carry six hardcoded failures and score
 * submissions with a keyword heuristic of its own. The backend already owns a
 * 70-case chaos dataset and the scoring model — and, unlike the page, it
 * persists awarded XP and unlocked badges to `.cherenkov-progress.json`, which
 * is what Mission Control and the Learn record screen read. Scoring in the page
 * meant every point a learner earned here was discarded on reload.
 */

interface ApiChaosEvent {
  layer?: string;
  event_type?: string;
  latency_ms?: number;
  jitter_ms?: number;
  packet_loss_rate?: number;
  proxy_log?: string;
  correlated_timestamp?: string;
  retry_attempts?: number;
  injection_target?: string;
}

interface ApiStep {
  name: string;
  status: string;
  duration_ms: number;
  error?: string | null;
}

interface ApiTestCase {
  test_id: string;
  name: string;
  suite: string;
  track_id: string;
  status: string;
  category: string;
  duration_ms: number;
  error_message?: string | null;
  stack_trace?: string | null;
  chaos_event?: ApiChaosEvent | null;
  flakiness_metrics?: { iterations?: number; failed_iterations?: number } | null;
  steps?: ApiStep[];
  labels?: Record<string, string>;
  root_cause_hint?: string | null;
}

export interface TriageVerdict {
  score: number;
  isCorrectCategory: boolean;
  feedback: string;
  xpEarned: number;
  badgeUnlocked?: string;
  /** Running total the backend persisted, so the header stops guessing. */
  totalXp?: number;
  detailedReasons: string[];
}

/** The API taxonomy is snake_case; the page's is PascalCase. */
const CATEGORY_FROM_API: Record<string, FailureCategory> = {
  real_bug: 'ProductBug',
  flaky_infra: 'FlakyInfra',
  anti_pattern: 'AntiPattern',
};

const CATEGORY_TO_API: Record<FailureCategory, string> = {
  ProductBug: 'real_bug',
  FlakyInfra: 'flaky_infra',
  AntiPattern: 'anti_pattern',
  Passed: 'none',
};

const STATUSES: TestStatus[] = ['passed', 'failed', 'broken', 'flaky'];

const asStatus = (status: string): TestStatus =>
  (STATUSES as string[]).includes(status) ? (status as TestStatus) : 'failed';

/**
 * The page renders a flat log list; the API gives structured telemetry. Compose
 * one from the other rather than dropping the detail on the floor.
 */
const composeChaosLogs = (item: ApiTestCase): string[] => {
  const logs: string[] = [];
  const event = item.chaos_event;

  if (event?.proxy_log) {
    const stamp = event.correlated_timestamp ?? '';
    logs.push(`[${stamp}] [${event.layer ?? 'proxy'}] ${event.proxy_log}`);
  }
  if (event && (event.latency_ms || event.jitter_ms || event.packet_loss_rate)) {
    logs.push(
      `[telemetry] latency ${event.latency_ms ?? 0}ms · jitter ${event.jitter_ms ?? 0}ms · ` +
        `packet loss ${((event.packet_loss_rate ?? 0) * 100).toFixed(1)}%`
    );
  }
  for (const step of item.steps ?? []) {
    const detail = step.error ? ` — ${step.error}` : '';
    logs.push(`[step] ${step.name}: ${step.status} (${step.duration_ms}ms)${detail}`);
  }
  if (logs.length === 0) logs.push('[telemetry] no chaos event recorded for this run');
  return logs;
};

export const toTestCase = (item: ApiTestCase): TestCaseResult => ({
  id: item.test_id,
  name: item.name,
  track: item.track_id,
  suite: item.suite,
  status: asStatus(item.status),
  durationMs: item.duration_ms,
  category: CATEGORY_FROM_API[item.category] ?? 'ProductBug',
  errorMessage: item.error_message ?? undefined,
  stackTrace: item.stack_trace ?? undefined,
  chaosLogs: composeChaosLogs(item),
  os: item.labels?.runner ?? item.labels?.tier ?? 'ubuntu-latest',
  shard: item.labels?.shard ?? '1/1',
  retries: item.chaos_event?.retry_attempts ?? item.flakiness_metrics?.failed_iterations ?? 0,
  // The backend is the authority on the verdict now, so these are only shown
  // after a submission and come back with it. root_cause_hint is what it has.
  groundTruthExplanation: item.root_cause_hint ?? '',
  groundTruthRemediation: '',
});

export async function fetchTriageTests(signal?: AbortSignal): Promise<TestCaseResult[]> {
  const res = await fetch(apiUrl('/api/triage/tests'), { signal });
  if (!res.ok) throw new Error(`GET /api/triage/tests -> ${res.status}`);
  const items: ApiTestCase[] = await res.json();
  return items.map(toTestCase);
}

export async function submitTriage(args: {
  testId: string;
  category: FailureCategory;
  explanation: string;
  fix: string;
}): Promise<TriageVerdict> {
  const res = await fetch(apiUrl('/api/triage/submit'), {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({
      test_id: args.testId,
      learner_category: CATEGORY_TO_API[args.category],
      root_cause_explanation: args.explanation,
      suggested_fix: args.fix,
    }),
  });
  if (!res.ok) throw new Error(`POST /api/triage/submit -> ${res.status}`);
  const data = await res.json();

  return {
    score: data.score_awarded ?? 0,
    isCorrectCategory: Boolean(data.correct),
    feedback: data.feedback ?? '',
    xpEarned: data.score_awarded ?? 0,
    badgeUnlocked: data.badge_unlocked ?? undefined,
    totalXp: data.updated_progress?.total_xp,
    detailedReasons: data.detailed_reasons ?? [],
  };
}
