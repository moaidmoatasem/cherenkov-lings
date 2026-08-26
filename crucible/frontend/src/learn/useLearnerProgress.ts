import { useEffect, useState } from 'react';
import { apiUrl } from '../lib/api';
import { KPIS, LEARNER } from './content';
import type { Kpi } from './types';

/**
 * Live figures from `/api/progress`, falling back to the seeded content when the
 * backend isn't running — the platform is local-first and the UI must still read
 * correctly offline.
 *
 * Only the numbers the API actually reports are overridden. Everything else
 * stays as the design specifies it, rather than being invented from thin data.
 */

/** Used only until GET /api/curriculum answers; the manifest is the authority. */
const SEEDED_TOTAL_MODULES = 65;
const SEEDED_TOTAL_TRACKS = 12;

interface CurriculumResponse {
  tracks?: unknown[];
  total_drills?: number;
}

interface ProgressResponse {
  total_xp?: number;
  streak_days?: number;
  /** The on-disk file uses an object; the API default uses a list. Accept both. */
  completed_drills?: Record<string, unknown> | unknown[];
}

export interface LearnerProgress {
  points: number;
  streakDays: number;
  modulesBuilt: number;
  /** Drills declared in lings.toml, via GET /api/curriculum. */
  modulesTotal: number;
  /** Tracks declared in lings.toml, via GET /api/curriculum. */
  tracksTotal: number;
  kpis: Kpi[];
  live: boolean;
}

const countDrills = (drills: ProgressResponse['completed_drills']): number => {
  if (Array.isArray(drills)) return drills.length;
  if (drills && typeof drills === 'object') return Object.keys(drills).length;
  return 0;
};

const SEEDED: LearnerProgress = {
  points: LEARNER.points,
  streakDays: LEARNER.streakDays,
  modulesBuilt: 14,
  modulesTotal: SEEDED_TOTAL_MODULES,
  tracksTotal: SEEDED_TOTAL_TRACKS,
  kpis: KPIS,
  live: false,
};

export function useLearnerProgress(): LearnerProgress {
  const [progress, setProgress] = useState<LearnerProgress>(SEEDED);

  useEffect(() => {
    const ctrl = new AbortController();

    fetch(apiUrl('/api/progress'), { signal: ctrl.signal })
      .then((res) => (res.ok ? res.json() : null))
      .then((data: ProgressResponse | null) => {
        if (!data) return;

        const built = countDrills(data.completed_drills);
        const points = typeof data.total_xp === 'number' ? data.total_xp : SEEDED.points;
        const streak =
          typeof data.streak_days === 'number' ? data.streak_days : SEEDED.streakDays;

        setProgress((prev) => ({
          ...prev,
          points,
          streakDays: streak,
          modulesBuilt: built,
          kpis: KPIS.map((kpi) => {
            if (kpi.label === 'Points') {
              return { ...kpi, value: points.toLocaleString('en-US') };
            }
            if (kpi.label === 'Modules built') {
              return { ...kpi, value: String(built) };
            }
            return kpi;
          }),
          live: true,
        }));
      })
      .catch(() => {});

    // The curriculum size comes from the manifest, not from a literal in this
    // file -- a hardcoded total goes stale the moment a track is added.
    fetch(apiUrl('/api/curriculum'), { signal: ctrl.signal })
      .then((res) => (res.ok ? res.json() : null))
      .then((data: CurriculumResponse | null) => {
        if (!data) return;
        setProgress((prev) => ({
          ...prev,
          modulesTotal:
            typeof data.total_drills === 'number' && data.total_drills > 0
              ? data.total_drills
              : prev.modulesTotal,
          tracksTotal: Array.isArray(data.tracks) ? data.tracks.length : prev.tracksTotal,
        }));
      })
      .catch(() => {});

    return () => ctrl.abort();
  }, []);

  return progress;
}
