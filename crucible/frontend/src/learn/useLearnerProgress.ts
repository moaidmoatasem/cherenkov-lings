import { useEffect, useState } from 'react';
import { apiUrl } from '../lib/api';
import type { Kpi } from './types';

/**
 * Everything the learner's record actually knows, from `/api/progress` and
 * `/api/curriculum`.
 *
 * The rule here is that a figure shown to the learner has to have been earned.
 * This screen used to blend live numbers with seeded ones, which put "0 of 68
 * modules" in the sidebar next to "620 of 900 points earned this week" and a
 * six-day streak on an account that had completed nothing. Anything the
 * platform does not measure -- hours spent, sessions kept, a weekly point
 * target -- is no longer displayed rather than invented.
 *
 * Before the API answers, the figures are zero and `live` is false, so the UI
 * can say it is still reading rather than show a number nobody earned.
 */

/** Achievements defined by the Rust engine (`ALL_ACHIEVEMENTS`). */
export const TOTAL_ACHIEVEMENTS = 8;

interface ApiAchievement {
  id?: string;
  name?: string;
  description?: string;
  unlocked_at?: string;
}

interface ApiDrillRecord {
  track_id?: string;
  drill_id?: string;
  best_score?: number;
  completion_count?: number;
  last_completed_at?: string;
}

interface ProgressResponse {
  total_xp?: number;
  level_name?: string;
  streak_days?: number;
  last_active_date?: string | null;
  achievements?: ApiAchievement[];
  /** Keyed `"<track>/<drill>"` by the engine; tolerate a list too. */
  completed_drills?: Record<string, ApiDrillRecord> | ApiDrillRecord[];
}

interface CurriculumResponse {
  tracks?: unknown[];
  total_drills?: number;
}

export interface EarnedBadge {
  id: string;
  name: string;
  description: string;
  unlockedOn: string;
}

export interface LearnerProgress {
  points: number;
  levelName: string;
  streakDays: number;
  modulesBuilt: number;
  /** Drills declared in lings.toml, via GET /api/curriculum. */
  modulesTotal: number;
  /** Tracks declared in lings.toml, via GET /api/curriculum. */
  tracksTotal: number;
  /** Distinct tracks the learner has completed at least one drill in. */
  tracksStarted: number;
  badges: EarnedBadge[];
  /** `"<track>/<drill>"` keys, for marking the catalog. */
  completedKeys: string[];
  /** Completed drill count per track id. */
  builtByTrack: Record<string, number>;
  kpis: Kpi[];
  /** True once `/api/progress` has answered. */
  live: boolean;
}

const EMPTY_BASE: Omit<LearnerProgress, 'kpis'> = {
  points: 0,
  levelName: 'Trainee',
  streakDays: 0,
  modulesBuilt: 0,
  modulesTotal: 0,
  tracksTotal: 0,
  tracksStarted: 0,
  badges: [],
  completedKeys: [],
  builtByTrack: {},
  live: false,
};

const toRecords = (
  drills: ProgressResponse['completed_drills']
): Array<[string, ApiDrillRecord]> => {
  if (Array.isArray(drills)) {
    return drills.map((d) => [`${d.track_id ?? ''}/${d.drill_id ?? ''}`, d]);
  }
  if (drills && typeof drills === 'object') return Object.entries(drills);
  return [];
};

/** '2026-08-27T00:53:01Z' -> 'August 27'. Falls back to the raw string. */
const shortDate = (iso: string | undefined): string => {
  if (!iso) return '';
  const at = new Date(iso);
  if (Number.isNaN(at.getTime())) return iso;
  return at.toLocaleDateString('en-US', { month: 'long', day: 'numeric' });
};

const buildKpis = (p: Omit<LearnerProgress, 'kpis'>): Kpi[] => [
  {
    label: 'Modules built',
    value: String(p.modulesBuilt),
    sub: p.modulesTotal
      ? `of ${p.modulesTotal}, across ${p.tracksTotal} tracks`
      : 'reading the manifest',
  },
  {
    label: 'Tracks started',
    value: String(p.tracksStarted),
    sub: p.tracksTotal ? `of ${p.tracksTotal} available` : '',
  },
  {
    label: 'Day streak',
    value: String(p.streakDays),
    sub: p.streakDays === 1 ? 'day in a row' : 'days in a row',
  },
  {
    label: 'Points',
    value: p.points.toLocaleString('en-US'),
    sub: p.levelName,
  },
];

/** The record before anything has loaded: the cards exist, all at zero. */
const EMPTY: LearnerProgress = { ...EMPTY_BASE, kpis: buildKpis(EMPTY_BASE) };

export function useLearnerProgress(): LearnerProgress {
  const [progress, setProgress] = useState<LearnerProgress>(EMPTY);

  useEffect(() => {
    const ctrl = new AbortController();

    fetch(apiUrl('/api/progress'), { signal: ctrl.signal })
      .then((res) => (res.ok ? res.json() : null))
      .then((data: ProgressResponse | null) => {
        if (!data) return;

        const records = toRecords(data.completed_drills);
        const builtByTrack: Record<string, number> = {};
        records.forEach(([key, record]) => {
          const trackId = record.track_id || key.split('/')[0];
          if (trackId) builtByTrack[trackId] = (builtByTrack[trackId] ?? 0) + 1;
        });

        setProgress((prev) => {
          const next = {
            ...prev,
            points: typeof data.total_xp === 'number' ? data.total_xp : 0,
            levelName: data.level_name || 'Trainee',
            streakDays: typeof data.streak_days === 'number' ? data.streak_days : 0,
            modulesBuilt: records.length,
            tracksStarted: Object.keys(builtByTrack).length,
            completedKeys: records.map(([key]) => key),
            builtByTrack,
            badges: (data.achievements ?? []).map((a) => ({
              id: a.id ?? '',
              name: a.name ?? a.id ?? 'Badge',
              description: a.description ?? '',
              unlockedOn: shortDate(a.unlocked_at),
            })),
            live: true,
          };
          return { ...next, kpis: buildKpis(next) };
        });
      })
      .catch(() => {});

    // The curriculum size comes from the manifest, not from a literal in this
    // file -- a hardcoded total goes stale the moment a track is added.
    fetch(apiUrl('/api/curriculum'), { signal: ctrl.signal })
      .then((res) => (res.ok ? res.json() : null))
      .then((data: CurriculumResponse | null) => {
        if (!data) return;
        setProgress((prev) => {
          const next = {
            ...prev,
            modulesTotal:
              typeof data.total_drills === 'number' && data.total_drills > 0
                ? data.total_drills
                : prev.modulesTotal,
            tracksTotal: Array.isArray(data.tracks) ? data.tracks.length : prev.tracksTotal,
          };
          return { ...next, kpis: buildKpis(next) };
        });
      })
      .catch(() => {});

    return () => ctrl.abort();
  }, []);

  return progress;
}
