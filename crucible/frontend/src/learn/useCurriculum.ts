import { useEffect, useState } from 'react';
import { apiUrl } from '../lib/api';
import { TRACKS } from './content';
import type { CurriculumModule, ProgressState, Track } from './types';

/**
 * The learner-facing catalog, backed by the curriculum manifest.
 *
 * `content.ts` carries hand-written copy for four tracks — the situation each
 * module teaches, its running time, whether it has a video. That copy does not
 * exist for the other eight tracks, and inventing it would be worse than
 * omitting it, so those are projected straight from `lings.toml` via
 * GET /api/curriculum: real names, real drill counts, real completion.
 *
 * The result is that every track the manifest declares is reachable here.
 * Curated tracks keep their designed presentation; the rest arrive as the
 * manifest describes them.
 */

interface ApiDrill {
  id: string;
  name: string;
  path: string;
}

interface ApiTrack {
  id: string;
  name: string;
  stack?: string;
  tier?: string;
  description?: string;
  drills?: ApiDrill[];
}

interface CompletedDrill {
  track_id?: string;
  drill_id?: string;
}

/** `completed_drills` is keyed `"<track>/<drill>"`; see gamification.rs. */
type CompletedMap = Record<string, CompletedDrill> | CompletedDrill[] | undefined;

const completedKeys = (completed: CompletedMap): Set<string> => {
  const keys = new Set<string>();
  const record = (entry: CompletedDrill | undefined, fallbackKey?: string) => {
    if (entry?.track_id && entry?.drill_id) keys.add(`${entry.track_id}/${entry.drill_id}`);
    else if (fallbackKey) keys.add(fallbackKey);
  };

  if (Array.isArray(completed)) completed.forEach((entry) => record(entry));
  else if (completed) {
    Object.entries(completed).forEach(([key, entry]) => record(entry, key));
  }
  return keys;
};

/** 'Python / Pytest' reads better as two chips than one. */
const skillsFromStack = (stack: string): string[] =>
  stack
    .split(/[/,]/)
    .map((part) => part.trim())
    .filter(Boolean);

const trackState = (done: number, total: number): Track['state'] => {
  if (total > 0 && done >= total) return 'finished';
  return done > 0 ? 'in progress' : 'not started';
};

const synthesise = (track: ApiTrack, completed: Set<string>): Track => {
  const drills = track.drills ?? [];
  let markedNow = false;

  const modules: CurriculumModule[] = drills.map((drill) => {
    const isDone = completed.has(`${track.id}/${drill.id}`);
    let state: ProgressState = 'todo';
    if (isDone) state = 'done';
    else if (!markedNow) {
      state = 'now';
      markedNow = true;
    }

    return {
      id: `${track.id}/${drill.id}`,
      title: drill.name,
      path: drill.path,
      // No hand-written situation exists for these drills. An empty line is
      // honest; a generated one would just restate the title.
      situation: '',
      duration: '',
      state,
      hasVideo: false,
      // 0 keeps them out of the "Under 20 min" filter, which is a claim about
      // a running time nobody has measured.
      minutes: 0,
    };
  });

  const done = modules.filter((m) => m.state === 'done').length;
  const stack = track.stack ?? '';

  return {
    id: track.id,
    name: track.name,
    meta: [stack, `${drills.length} modules`].filter(Boolean).join(' · '),
    done,
    total: drills.length,
    state: trackState(done, drills.length),
    skills: skillsFromStack(stack),
    modules,
  };
};

export function useCurriculumTracks(): Track[] {
  const [tracks, setTracks] = useState<Track[]>(TRACKS);

  useEffect(() => {
    const ctrl = new AbortController();

    const load = async () => {
      const [curriculum, progress] = await Promise.all([
        fetch(apiUrl('/api/curriculum'), { signal: ctrl.signal })
          .then((res) => (res.ok ? res.json() : null))
          .catch(() => null),
        fetch(apiUrl('/api/progress'), { signal: ctrl.signal })
          .then((res) => (res.ok ? res.json() : null))
          .catch(() => null),
      ]);

      const apiTracks: ApiTrack[] | undefined = curriculum?.tracks;
      if (!Array.isArray(apiTracks) || apiTracks.length === 0) return;

      const completed = completedKeys(progress?.completed_drills);
      const curated = new Map(TRACKS.map((t) => [t.id, t]));

      setTracks(
        apiTracks.map((apiTrack) => curated.get(apiTrack.id) ?? synthesise(apiTrack, completed))
      );
    };

    void load();
    return () => ctrl.abort();
  }, []);

  return tracks;
}
