/**
 * Domain model for the learning platform.
 *
 * Shapes follow the handoff's state table so that swapping the seeded content in
 * `content.ts` for live API data is a mechanical change. Two rules from the
 * handoff are encoded here rather than left to convention:
 *
 *  - The run is the unit of truth. Points, skill stages and certificate progress
 *    are derived from runs; nothing stores them independently.
 *  - A run is five iterations under chaos, so `RunRecord` keeps per-iteration
 *    results rather than a single pass flag.
 */

export type ScreenId = 'today' | 'module' | 'lab' | 'device' | 'tracks' | 'progress';

export type StepId = 'read' | 'watch' | 'practice' | 'build';

/** Every tick circle in the UI is one of these three states. */
export type ProgressState = 'done' | 'now' | 'todo';

export interface Learner {
  id: string;
  name: string;
  points: number;
  streakDays: number;
  /** The learner's own reason for being here, shown on the sticky note. */
  motivation: string;
  motivationWhen: string;
  publicSlug: string;
}

export interface ModuleStep {
  id: StepId;
  label: string;
  /** Right-aligned detail: a duration, or progress through the questions. */
  detail: string;
  state: ProgressState;
}

export interface CurriculumModule {
  id: string;
  title: string;
  /** Names the situation it teaches, never the API. The most important content rule. */
  situation: string;
  duration: string;
  state: ProgressState;
  hasVideo: boolean;
  minutes: number;
}

export interface Track {
  id: string;
  name: string;
  meta: string;
  done: number;
  total: number;
  state: 'in progress' | 'finished' | 'not started';
  skills: string[];
  modules: CurriculumModule[];
}

export interface ScheduleBlock {
  time: string;
  title: string;
  meta: string;
  kind: 'done' | 'next' | 'optional';
  stateLabel: string;
}

export interface TocEntry {
  label: string;
}

export interface Chapter {
  time: string;
  label: string;
  state: ProgressState;
}

export interface PracticeAnswer {
  key: string;
  label: string;
  correct: boolean;
}

export interface Checkpoint {
  label: string;
  state: ProgressState;
}

/** A syntax-highlighted source line in a lab panel. */
export interface CodeLine {
  n: number;
  text: string;
  kind: 'imp' | 'fn' | 'c' | 'p' | 'hl' | 'good' | 'k' | 'v' | 'run';
  /** Right-aligned annotation chip, device lab only. */
  tag?: string;
}

export interface RunIteration {
  index: number;
  passed: boolean;
  /** False while this iteration is still in flight. */
  settled: boolean;
}

export interface RunOutcome {
  label: string;
  state: string;
  pct: number;
}

export interface RunFailure {
  rank: string;
  title: string;
  detail: string;
  tag: string;
}

export interface RunRecord {
  moduleId: string;
  iterations: RunIteration[];
  outcomes: RunOutcome[];
  failures: RunFailure[];
  passed: boolean;
  points: number;
  /** Kept even when the run passes: the certificate links the failures too. */
  traceRef: string;
}

export interface DeviceCondition {
  label: string;
  value: string;
  on: boolean;
}

export interface FlowStep {
  label: string;
  state: ProgressState;
}

/** 0 = not yet, 1 = read about it, 2 = answered questions, 3 = built with it. */
export type SkillStage = 0 | 1 | 2 | 3;

export interface Skill {
  label: string;
  level: SkillStage;
  stage: string;
}

export interface Kpi {
  label: string;
  value: string;
  sub: string;
}

export interface Badge {
  icon: string;
  name: string;
  meta: string;
  tone: 'moss' | 'blue' | 'idle';
}

export interface Certificate {
  trackName: string;
  title: string;
  copy: string;
  modulesBuilt: number;
  modulesTotal: number;
  projectedOn: string;
}
