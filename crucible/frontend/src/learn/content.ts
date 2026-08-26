/**
 * Seeded curriculum and module content.
 *
 * All copy here is final product copy from the design handoff and is meant to be
 * used as-is. The tracks below are the four the design specifies; the full
 * eleven come from `/api/curriculum` (see `useLearnerProgress`), which is why
 * everything is shaped as the API would return it rather than inlined in JSX.
 */

import type {
  Badge,
  Certificate,
  Chapter,
  Checkpoint,
  CodeLine,
  DeviceCondition,
  FlowStep,
  Kpi,
  Learner,
  ModuleStep,
  PracticeAnswer,
  RunRecord,
  ScheduleBlock,
  Skill,
  TocEntry,
  Track,
} from './types';

export const LEARNER: Learner = {
  id: 'moaid',
  name: 'Moaid',
  points: 2520,
  streakDays: 6,
  motivation: '“Stop being the person whose tests everyone reruns.”',
  motivationWhen: 'your note, week 1',
  publicSlug: 'cherenkov.dev/moaid',
};

/** The module the Continue card resumes into. */
export const CURRENT_MODULE = {
  id: 'waiting-without-sleeping',
  trackName: 'Web Automation',
  index: 4,
  trackTotal: 10,
  title: 'Waiting without sleeping',
  lede:
    "You've read it and watched the trace. What's left is the part that sticks: make a real test survive a slow, jittery network.",
  minutesLeft: 'about 25 minutes left',
};

export const LOOP_STEPS: ModuleStep[] = [
  { id: 'read', label: 'Read it', detail: '6 min', state: 'done' },
  { id: 'watch', label: 'Watch the trace', detail: '9 min', state: 'done' },
  { id: 'practice', label: 'Answer five questions', detail: '3 of 5', state: 'now' },
  { id: 'build', label: 'Build it in the lab', detail: '25 min', state: 'todo' },
];

/** The step switcher at the top of a module. `build` routes to the lab. */
export const STEP_TABS: ModuleStep[] = [
  { id: 'read', label: 'Read', detail: '6 min', state: 'done' },
  { id: 'watch', label: 'Watch', detail: '9 min', state: 'done' },
  { id: 'practice', label: 'Practice', detail: '3 of 5', state: 'now' },
  { id: 'build', label: 'Build', detail: 'lab', state: 'todo' },
];

export const SCHEDULE: ScheduleBlock[] = [
  { time: '14:00', title: 'Recall, seven questions', meta: 'done in 4 minutes', kind: 'done', stateLabel: 'done' },
  {
    time: '18:30',
    title: 'Answer the last two questions, then build',
    meta: 'reminder set · 45 min',
    kind: 'next',
    stateLabel: 'next up',
  },
  {
    time: '20:00',
    title: "Thursday's reading, if you want it",
    meta: 'optional · 20 min',
    kind: 'optional',
    stateLabel: 'optional',
  },
];

/** Three weeks of sessions: index 0 is the oldest day. */
export const STREAK_DOTS: Array<'recent' | 'earlier' | 'rest' | 'missed'> = Array.from(
  { length: 21 },
  (_, i) => {
    if ([2, 6, 11].includes(i)) return 'rest';
    const kept = [0, 1, 3, 4, 5, 7, 8, 9, 10, 12, 13, 14, 15, 16, 17, 18, 19, 20].includes(i);
    if (!kept) return 'missed';
    return i > 13 ? 'recent' : 'earlier';
  }
);

export const WEEK_POINTS = { earned: 620, target: 900 };

// ─── Module: read ──────────────────────────────────────────────────────────

export const ARTICLE = {
  kicker: 'Reading · 6 minutes',
  savedNote: 'saved for offline',
  title: 'Why a sleep is never a wait',
  /** Segments rather than an HTML string, so the emphasis stays real markup. */
  paragraphs: [
    [
      { text: 'A fixed sleep says ' },
      { text: 'the app will be ready in one second', em: true },
      {
        text:
          '. On a real network that is a guess — and a guess that is wrong one run in five is what everyone else on your team calls a flaky test.',
      },
    ],
    [
      {
        text:
          "The fix isn't a longer sleep. It's asking the browser something it can answer honestly: is this element visible yet?",
      },
    ],
  ] as Array<Array<{ text: string; em?: boolean }>>,
  pullquote: { kicker: 'Keep this', text: 'Assert on state, never on time.' },
  diff: {
    removed: 'await page.waitForTimeout(1000);',
    added: 'await expect(results).toBeVisible();',
  },
  closing:
    'In the lab your test runs five times behind a proxy that adds 200 ms of delay and 75 ms of jitter. A test that only passes on a quiet machine isn’t finished.',
  comesBackIn: '4 modules, and 2 recall questions',
};

export const TOC: TocEntry[] = [
  { label: 'The guess you make' },
  { label: 'What the browser can answer' },
  { label: 'Before and after' },
  { label: 'Why five runs' },
];

// ─── Module: watch ─────────────────────────────────────────────────────────

export const VIDEO = {
  title: 'Watch the sleep run out, in a real trace',
  body:
    'This is your own failing run from yesterday. We open the trace and find the moment the response arrived — 240 ms after the wait had already given up.',
  position: '3:12 / 9:24',
  progressPct: 34,
  offer: {
    kicker: 'Short on time',
    text: 'Watch only 3:12–4:40 and answer one question.',
    action: 'Play the 90-second cut',
  },
  note: { at: 'Note at 3:12', text: 'the response came back after the wait had already failed' },
};

export const CHAPTERS: Chapter[] = [
  { time: '0:00', label: 'The failing run', state: 'done' },
  { time: '1:48', label: 'Opening the trace', state: 'done' },
  { time: '3:12', label: 'The wait gives up early', state: 'now' },
  { time: '5:30', label: 'Rewriting with expect', state: 'todo' },
  { time: '7:40', label: 'Running it under jitter', state: 'todo' },
];

// ─── Module: practice ──────────────────────────────────────────────────────

export const PRACTICE = {
  kicker: 'Question 3 of 5 · nothing is graded',
  question: 'This passes on your laptop and fails one CI run in five. What actually fixes it?',
  snippet: [
    { text: "await input.fill('playwright');", bad: false },
    { text: 'await page.waitForTimeout(1000);', bad: true },
    { text: 'expect(await results.count()).toBe(3);', bad: false },
  ],
  explanation: {
    kicker: "That's the one",
    text:
      'A web-first assertion keeps re-checking until the condition is true, so it adapts to whatever the network does that run. A bigger sleep just moves the line you’ll cross next month.',
    action: 'Now do it in the lab',
  },
  noPenalty: 'Anything you miss comes back tomorrow as a short question. No penalty.',
};

export const ANSWERS: PracticeAnswer[] = [
  { key: 'A', label: 'Raise the wait to three seconds', correct: false },
  { key: 'B', label: 'Assert that the results are visible, and drop the wait', correct: true },
  { key: 'C', label: 'Retry the test twice in CI', correct: false },
  { key: 'D', label: 'Run the suite one test at a time', correct: false },
];

export const CHECKPOINTS: Checkpoint[] = [
  { label: 'What the browser waits for', state: 'done' },
  { label: 'One match, or none', state: 'done' },
  { label: 'Sleep against assertion', state: 'now' },
  { label: 'Timeouts worth setting', state: 'todo' },
  { label: 'Reading a flaky trace', state: 'todo' },
];

// ─── Browser lab ───────────────────────────────────────────────────────────

export const LAB_INTRO =
  'Make the search test hold up five times in a row with 200 ms of delay and 75 ms of jitter. Change the code, hit run, watch it happen.';

export const SPEC_FILE = 'search.spec.ts';

export const SPEC_CODE: CodeLine[] = [
  { n: 1, text: "import { test, expect } from '@playwright/test';", kind: 'imp' },
  { n: 2, text: '', kind: 'p' },
  { n: 3, text: "test('search survives a slow network', async ({ page }) => {", kind: 'fn' },
  { n: 4, text: "  await page.goto('http://localhost:8080/search');", kind: 'p' },
  { n: 5, text: '', kind: 'p' },
  { n: 6, text: "  const input = page.getByRole('searchbox', { name: 'Query' });", kind: 'hl' },
  { n: 7, text: "  const results = page.getByTestId('search-results');", kind: 'hl' },
  { n: 8, text: '', kind: 'p' },
  { n: 9, text: "  await input.fill('playwright');", kind: 'p' },
  { n: 10, text: '', kind: 'p' },
  { n: 11, text: '  // no sleep — ask about the state you care about', kind: 'c' },
  { n: 12, text: '  await expect(results).toBeVisible();', kind: 'good' },
  { n: 13, text: "  await expect(results.getByRole('listitem')).toHaveCount(3);", kind: 'good' },
  { n: 14, text: '});', kind: 'fn' },
];

/** Widths of the placeholder result blocks on the rendered sandbox page. */
export const RESULT_WIDTHS = [82, 64, 73];

export const PASSING_RUN: RunRecord = {
  moduleId: CURRENT_MODULE.id,
  iterations: [
    { index: 1, passed: true, settled: true },
    { index: 2, passed: true, settled: true },
    { index: 3, passed: true, settled: true },
    { index: 4, passed: true, settled: true },
    { index: 5, passed: false, settled: false },
  ],
  outcomes: [
    { label: 'Survived five runs', state: 'yes', pct: 100 },
    { label: 'Locators', state: 'role and test id', pct: 100 },
    { label: 'Waiting', state: 'no fixed sleeps', pct: 100 },
    { label: 'Runtime', state: '1.4 s', pct: 78 },
  ],
  failures: [],
  passed: true,
  points: 180,
  traceRef: 'trace/2026-08-25-run-3.zip',
};

export const FAILING_RUN: RunRecord = {
  moduleId: CURRENT_MODULE.id,
  iterations: PASSING_RUN.iterations,
  outcomes: [],
  failures: [
    {
      rank: '1',
      title: 'Your wait is shorter than the response',
      detail:
        'Run 3 stopped waiting at 1 000 ms; the results arrived at 1 240 ms. Replace the wait with an assertion.',
      tag: 'fix first',
    },
    {
      rank: '2',
      title: 'The locator matches three things',
      detail: 'Once the list renders, strict mode will fail. Scope it inside the results container.',
      tag: 'fix next',
    },
    {
      rank: '3',
      title: 'Runtime is 2.4 s against a 2.0 s budget',
      detail: 'Almost all of it is the sleep. Fixing the first one likely fixes this too.',
      tag: 'follows',
    },
  ],
  passed: false,
  points: 0,
  traceRef: 'trace/2026-08-25-run-3.zip',
};

export const VERDICT_COPY = {
  passTitle: "Five runs, five passes. That's the skill.",
  passFoot: "Yesterday's failing trace stays attached, so the certificate shows what you fixed.",
  passNext: 'Next module · Locators',
  whatChanged:
    'You removed one sleep and added two assertions. Runtime dropped from 2.4 s to 1.4 s as a side effect.',
  failTitle: 'Two of five runs failed — start at the top',
  failLede: 'Nothing is lost. Retrying is free, and this is the useful part.',
  whyItBroke:
    'Run 3 got its response 240 ms after your one-second wait had given up. With ±75 ms of jitter, the same code passes and fails on the same machine.',
  hintsNote: "Hints don't cost points. They're recorded so your certificate stays honest.",
};

// ─── Device lab ────────────────────────────────────────────────────────────

export const DEVICE_INTRO =
  "Face ID isn't available after a restart. Write the flow so it notices, falls back to the passcode, and still ends up at the balance — on a slow connection, after Android has killed the app once.";

export const DEVICE_FILE = 'biometric_fallback.yaml';

export const DEVICE_YAML: CodeLine[] = [
  { n: 1, text: 'appId: dev.cherenkov.bank', kind: 'k' },
  { n: 2, text: '---', kind: 'p' },
  { n: 3, text: '- launchApp:', kind: 'k' },
  { n: 4, text: '    clearState: true', kind: 'v', tag: 'cold start' },
  { n: 5, text: '- assertVisible: "Unlock to continue"', kind: 'k' },
  { n: 6, text: '- tapOn: "Use biometrics"', kind: 'k' },
  { n: 7, text: '- assertVisible:', kind: 'k' },
  { n: 8, text: '    text: "Biometric unavailable.*"', kind: 'v', tag: 'a pattern, not exact text' },
  { n: 9, text: '- tapOn: "Use passcode instead"', kind: 'run', tag: 'running now' },
  { n: 10, text: '- inputText: "482913"', kind: 'p' },
  { n: 11, text: '- assertVisible: "Balance"', kind: 'p' },
];

export const DEVICE_CONDITIONS: DeviceCondition[] = [
  { label: 'Network', value: '3G · 340 ms', on: true },
  { label: 'Process', value: 'killed once', on: true },
  { label: 'Locale', value: 'low-memory device', on: true },
  { label: 'Orientation', value: 'portrait', on: false },
];

export const DEVICE_FLOW: FlowStep[] = [
  { label: 'Launch cold', state: 'done' },
  { label: 'See the lock screen', state: 'done' },
  { label: 'Tap biometrics', state: 'done' },
  { label: 'Handle the refusal', state: 'now' },
  { label: 'Enter the passcode', state: 'todo' },
  { label: 'See the balance', state: 'todo' },
];

export const DEVICE_COPY = {
  hardware: 'Pixel 7 · Android 14',
  play: 'Play on device',
  harderTitle: 'Make it harder',
  harderHint: 'tap any condition and the flow replays against it',
  harderNote:
    'Cold start after the system reclaims your app is where most mobile flows quietly break.',
  appName: 'Crucible Bank',
  screenTitle: 'Unlock to continue',
  screenMsg: 'Biometrics unavailable after restart',
  screenCta: 'Use passcode instead',
  screenTag: 'tapping now',
};

// ─── All modules ───────────────────────────────────────────────────────────

export const FILTERS = ['Everything', 'Not started', 'Has a video', 'Under 20 min'];

export const TRACKS: Track[] = [
  {
    id: 'foundations',
    name: 'Foundations',
    meta: 'Python · 5 modules · about 2 hours',
    done: 5,
    total: 5,
    state: 'finished',
    skills: ['Pytest', 'Assertions', 'Test design'],
    modules: [
      { id: 'f1', title: 'What a test really is', situation: 'You inherit a suite nobody trusts', duration: '24m', state: 'done', hasVideo: true, minutes: 24 },
      { id: 'f2', title: 'Naming that survives review', situation: 'A failure report no one can read', duration: '22m', state: 'done', hasVideo: false, minutes: 22 },
      { id: 'f3', title: 'Arrange, act, assert', situation: 'Three tests tangled into one', duration: '26m', state: 'done', hasVideo: true, minutes: 26 },
      { id: 'f4', title: "Don't test the mock", situation: 'Green tests, broken product', duration: '28m', state: 'done', hasVideo: false, minutes: 28 },
      { id: 'f5', title: 'One thing per test', situation: 'A failure with five possible causes', duration: '20m', state: 'done', hasVideo: true, minutes: 20 },
    ],
  },
  {
    id: 'playwright-ts',
    name: 'Modern Web Automation',
    meta: 'Playwright · TypeScript · 10 modules · about 6 hours',
    done: 4,
    total: 10,
    state: 'in progress',
    skills: ['Playwright', 'Locators', 'Auto-waiting', 'Traces'],
    modules: [
      { id: 'w1', title: 'Hydration timing', situation: 'The button exists but does nothing yet', duration: '40m', state: 'done', hasVideo: true, minutes: 40 },
      { id: 'w2', title: 'Shadow DOM', situation: 'A component your selector cannot see', duration: '38m', state: 'done', hasVideo: true, minutes: 38 },
      { id: 'w3', title: 'Debounced search', situation: 'The race you only lose in CI', duration: '42m', state: 'done', hasVideo: true, minutes: 42 },
      { id: 'waiting-without-sleeping', title: 'Waiting without sleeping', situation: 'One run in five fails, nobody knows why', duration: '45m', state: 'now', hasVideo: true, minutes: 45 },
      { id: 'w5', title: 'Locator hierarchy', situation: 'A refactor breaks forty tests', duration: '35m', state: 'todo', hasVideo: true, minutes: 35 },
      { id: 'w6', title: 'Page objects, lightly', situation: 'Abstraction that hides the failure', duration: '32m', state: 'todo', hasVideo: false, minutes: 32 },
    ],
  },
  {
    id: 'maestro-mobile',
    name: 'Mobile UI Automation',
    meta: 'Maestro · 5 modules · about 3 hours',
    done: 1,
    total: 5,
    state: 'in progress',
    skills: ['Maestro', 'Device state', 'Deep links'],
    modules: [
      { id: 'm1', title: 'Biometric fallback', situation: 'Face ID unavailable after a restart', duration: '34m', state: 'now', hasVideo: true, minutes: 34 },
      { id: 'm2', title: 'Deep link, cold start', situation: 'The link works only when the app is warm', duration: '30m', state: 'todo', hasVideo: false, minutes: 30 },
      { id: 'm3', title: 'Activity recreation', situation: 'Android kills your app mid-flow', duration: '32m', state: 'todo', hasVideo: true, minutes: 32 },
    ],
  },
  {
    id: 'restassured-java',
    name: 'API Resilience',
    meta: 'REST Assured · Java · 7 modules · about 3 hours',
    done: 0,
    total: 7,
    state: 'not started',
    skills: ['REST Assured', 'Idempotency', 'Schemas'],
    modules: [
      { id: 'a1', title: 'Idempotency keys', situation: 'A retry charges the customer twice', duration: '36m', state: 'todo', hasVideo: true, minutes: 36 },
      { id: 'a2', title: 'Tokens and elevation', situation: 'A standard user reaches an admin route', duration: '40m', state: 'todo', hasVideo: false, minutes: 40 },
      { id: 'a3', title: 'Consumer lag', situation: 'The event arrives, eventually', duration: '34m', state: 'todo', hasVideo: true, minutes: 34 },
    ],
  },
];

// ─── My record ─────────────────────────────────────────────────────────────

export const KPIS: Kpi[] = [
  { label: 'Modules built', value: '14', sub: 'of 60, across four tracks' },
  { label: 'Kept sessions', value: '86%', sub: 'six of seven this week' },
  { label: 'Time spent', value: '9h 40m', sub: '42 minutes a day' },
  { label: 'Points', value: '2,520', sub: 'two more modules to the certificate' },
];

export const SKILLS: Skill[] = [
  { label: 'Waiting and assertions', level: 3, stage: 'built with it' },
  { label: 'Locator strategy', level: 3, stage: 'built with it' },
  { label: 'Diagnosing flakiness', level: 2, stage: 'answered questions' },
  { label: 'Mobile flows', level: 2, stage: 'answered questions' },
  { label: 'Contract testing', level: 1, stage: 'read about it' },
  { label: 'Load and concurrency', level: 0, stage: 'not yet' },
];

export const CERTIFICATE: Certificate = {
  trackName: 'Modern Web Automation',
  title: 'Modern Web Automation, proven under chaos',
  copy:
    "Issued when all ten modules have been built and held up — not when they've been watched. It links to the runs, including the ones that failed first, so anyone can check it.",
  modulesBuilt: 4,
  modulesTotal: 10,
  projectedOn: '4 of 10 built · around Sep 6',
};

export const BADGES: Badge[] = [
  { icon: '✓', name: 'Foundations', meta: 'all five built · August 18', tone: 'moss' },
  { icon: '◆', name: 'Steady three weeks', meta: '21 days, 18 of them kept', tone: 'blue' },
  { icon: '◇', name: 'Mobile UI Automation', meta: 'one of five built', tone: 'idle' },
];

export const PACE_NOTE = {
  weeks: 'Three weeks in, 42 minutes a day on average. At this rate Web Automation is done by',
  finishDate: 'Sep 6',
  unlock: 'Two more built modules and the Web Automation certificate is yours.',
  recall: '7 questions from mistakes you actually made',
  recallKicker: 'Four minutes, if you have them',
};

export const NEXT_SESSION = {
  when: 'Today, 18:30 · 45 min',
  note: 'Reminder 10 minutes before, on your phone.',
};
