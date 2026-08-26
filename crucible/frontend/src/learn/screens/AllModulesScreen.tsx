import React, { useMemo, useState } from 'react';
import { ProgressRing, Tick } from '../components/Primitives';
import { FILTERS } from '../content';
import type { CurriculumModule, Track } from '../types';

interface AllModulesScreenProps {
  /** Every track lings.toml declares, not just the four with curated copy.
   *  Owned by LearnApp so the heading count above these rows and the rows
   *  themselves are computed from one fetch rather than two that can disagree. */
  tracks: Track[];
  /** The track is needed to name the drill's track on the drill screen. */
  onOpenModule: (module: CurriculumModule, track: Track) => void;
}

const matchesFilter = (module: CurriculumModule, filter: string): boolean => {
  switch (filter) {
    case 'Not started':
      return module.state === 'todo';
    case 'Has a video':
      return module.hasVideo;
    case 'Under 20 min':
      return module.minutes < 20;
    default:
      return true;
  }
};

export const AllModulesScreen: React.FC<AllModulesScreenProps> = ({
  tracks,
  onOpenModule,
}) => {
  const source = tracks;
  const [query, setQuery] = useState<string>('');
  const [filter, setFilter] = useState<string>(FILTERS[0]);

  const visible = useMemo(() => {
    const needle = query.trim().toLowerCase();
    return source
      .map((track) => ({
        ...track,
        modules: track.modules.filter((module) => {
          if (!matchesFilter(module, filter)) return false;
          if (!needle) return true;
          return (
            module.title.toLowerCase().includes(needle) ||
            module.situation.toLowerCase().includes(needle) ||
            track.name.toLowerCase().includes(needle)
          );
        }),
      }))
      .filter((track) => track.modules.length > 0);
  }, [source, query, filter]);

  return (
    <div className="l-col" style={{ gap: 20 }}>
      <div className="l-row l-wrap" style={{ gap: 9 }}>
        <input
          type="search"
          className="l-search-field"
          placeholder="Search modules, notes, error messages…"
          aria-label="Search modules"
          value={query}
          onChange={(e) => setQuery(e.target.value)}
        />
        {FILTERS.map((label) => (
          <button
            key={label}
            type="button"
            className="l-filter"
            aria-pressed={filter === label}
            onClick={() => setFilter(label)}
          >
            {label}
          </button>
        ))}
      </div>

      {visible.length === 0 && (
        <section className="l-card">
          <p className="l-empty">Nothing matches that yet. Try another filter, or clear the search.</p>
        </section>
      )}

      {visible.map((track) => (
        <section key={track.id} className="l-card">
          <div className="l-track-head">
            <ProgressRing
              size={46}
              inset={5}
              fraction={track.total > 0 ? track.done / track.total : 0}
              label={`${track.done}/${track.total}`}
              labelSize={11.5}
            />
            <div className="l-track-titles">
              <span className="l-track-name">{track.name}</span>
              <span className="l-track-meta">{track.meta}</span>
            </div>
            <span className="l-track-state" data-state={track.state}>
              {track.state}
            </span>
          </div>

          <div className="l-track-skills">
            {track.skills.map((skill) => (
              <span key={skill} className="l-chip">
                {skill}
              </span>
            ))}
          </div>

          {track.modules.map((module) => (
            <button
              key={module.id}
              type="button"
              className="l-mod-row"
              data-state={module.state}
              onClick={() => onOpenModule(module, track)}
            >
              <Tick state={module.state} hideNowGlyph />
              <span className="l-mod-body">
                <span className="l-mod-title">{module.title}</span>
                {/* Names the situation it teaches, not the API. Absent for
                    tracks projected straight from the manifest. */}
                {module.situation && <span className="l-mod-case">{module.situation}</span>}
              </span>
              {module.duration && <span className="l-mod-time">{module.duration}</span>}
            </button>
          ))}
        </section>
      ))}
    </div>
  );
};
