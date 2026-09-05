export interface BadgeDefinition {
  id: string;
  name: string;
  icon: string;
  desc: string;
  category: 'chaos' | 'architecture' | 'core' | 'performance';
  requiredPath: string;
  criteria: string[];
}

export interface BadgeCompletionState {
  id: string;
  unlocked: boolean;
  unlockedAt?: string;
  progressPercent?: number;
  completedSteps?: number;
  totalSteps?: number;
}

export interface AchievementItem {
  id: string;
  name: string;
  description: string;
  unlocked_at: string;
}

export interface DrillRecord {
  track_id: string;
  drill_id: string;
  best_score: number;
  completion_count: number;
  first_completed_at: string;
  last_completed_at: string;
}

export interface ProgressData {
  total_xp: number;
  completed_drills?: Record<string, DrillRecord>;
  achievements?: AchievementItem[];
  streak_days?: number;
  flakiness_100_streak?: number;
  perfect_locator_count?: number;
  level_name?: string;
}

export interface BadgesShowcaseProps {
  progress?: ProgressData | null;
  /** Explicit completion state overrides for testing or client-side overrides */
  completionOverrides?: {
    chaos_survivor?: boolean;
    the_architect?: boolean;
    [badgeId: string]: boolean | undefined;
  };
  onBadgeClick?: (badge: BadgeDefinition) => void;
}

export const MISSION_CONTROL_BADGES: BadgeDefinition[] = [
  {
    id: 'first_blood',
    name: 'First Blood',
    icon: '🩸',
    desc: 'Complete your first drill ever',
    category: 'core',
    requiredPath: 'foundations',
    criteria: ['Complete 1 drill in any track'],
  },
  {
    id: 'flakiness_slayer',
    name: 'Flakiness Slayer',
    icon: '🗡️',
    desc: 'Score 100/100 on Flakiness dimension 3 times in a row',
    category: 'chaos',
    requiredPath: 'resilience',
    criteria: ['3 consecutive 100% flakiness resilience runs'],
  },
  {
    id: 'chaos_survivor',
    name: 'Chaos Survivor',
    icon: '🌀',
    desc: 'Pass 5/5 k6 load test iterations under chaos',
    category: 'chaos',
    requiredPath: 'k6-js / chaos-proxy / allure-triage',
    criteria: [
      'Pass 5/5 k6 load test iterations under chaos',
      'Diagnose chaotic root cause in Allure Triage',
      'Survive L4/L7 chaos latency injection',
    ],
  },
  {
    id: 'tool_polyglot',
    name: 'Tool Polyglot',
    icon: '🧰',
    desc: 'Complete drills across 4 different tech stacks',
    category: 'core',
    requiredPath: 'polyglot',
    criteria: ['Complete drills in at least 4 distinct tracks'],
  },
  {
    id: 'the_architect',
    name: 'The Architect',
    icon: '🏗️',
    desc: 'Master all Cross-Tool Decision drills',
    category: 'architecture',
    requiredPath: 'tool-decisions / pipeline-builder',
    criteria: [
      'Complete all Cross-Tool Decision drills',
      'Configure CI/CD matrix with L4/L7 chaos stage',
      'Validate distributed telemetry and tracing assertions',
    ],
  },
  {
    id: 'perfect_locator',
    name: 'Perfect Locator',
    icon: '🎯',
    desc: 'Score 100/100 on Locator Quality 5 times',
    category: 'core',
    requiredPath: 'playwright-ts',
    criteria: ['5 drills with semantic locators and zero test-id fallbacks'],
  },
  {
    id: 'speed_demon',
    name: 'Speed Demon',
    icon: '⚡',
    desc: 'Beat execution speed baseline by 40%+',
    category: 'performance',
    requiredPath: 'optimization',
    criteria: ['Beat speed baseline by 40% on any drill'],
  },
  {
    id: 'sdet_master',
    name: 'SDET Master',
    icon: '👑',
    desc: 'Complete every drill across every track',
    category: 'core',
    requiredPath: 'all-tracks',
    criteria: ['Complete 100% of curriculum drills'],
  },
];
