import React, { useEffect, useState } from 'react';
import { apiUrl } from '../../lib/api';
import { Markdown } from '../components/Markdown';
import type { SelectedDrill } from '../types';

interface DrillTheory {
  drill_id: string;
  title: string;
  theory_markdown: string;
  hints_markdown: string;
  has_theory: boolean;
  has_hints: boolean;
}

type Pane = 'theory' | 'hints';

/**
 * A drill's own material, read from the repository through
 * GET /api/drill/theory — the same theory.md and hints.md the CLI shows.
 *
 * Every module row used to open one hardcoded module regardless of which drill
 * was clicked. This screen is what the other 61 drills open instead.
 */
export const DrillScreen: React.FC<{ drill: SelectedDrill; onBack: () => void }> = ({
  drill,
  onBack,
}) => {
  const [data, setData] = useState<DrillTheory | null>(null);
  const [pane, setPane] = useState<Pane>('theory');
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    const ctrl = new AbortController();
    setData(null);
    setError(null);
    setPane('theory');

    fetch(apiUrl(`/api/drill/theory?path=${encodeURIComponent(drill.path)}`), {
      signal: ctrl.signal,
    })
      .then((res) => (res.ok ? res.json() : Promise.reject(new Error(String(res.status)))))
      .then(setData)
      .catch((err: Error) => {
        if (err.name !== 'AbortError') setError(err.message);
      });

    return () => ctrl.abort();
  }, [drill.path]);

  // The drill is done in the editor with the watcher running; saying so beats
  // offering a Run button this screen cannot honour.
  const command = `cherenkov-lings watch --track=${drill.trackId}`;

  return (
    <div className="l-col" style={{ gap: 18 }} data-testid="drill-screen">
      <div className="l-row l-wrap" style={{ gap: 10, alignItems: 'center' }}>
        <button type="button" className="l-btn l-btn-ghost l-btn-sm" onClick={onBack}>
          ← All modules
        </button>
        <span className="l-meta">{drill.trackName}</span>
      </div>

      <div className="l-row l-wrap" style={{ gap: 7 }}>
        {(['theory', 'hints'] as const).map((id) => (
          <button
            key={id}
            type="button"
            className="l-filter"
            aria-pressed={pane === id}
            onClick={() => setPane(id)}
          >
            {id === 'theory' ? 'Read' : 'Hints'}
          </button>
        ))}
      </div>

      <section className="l-card l-card-pad">
        {error && (
          <p className="l-empty">
            Could not load this drill ({error}). The material is on disk at{' '}
            <code>{drill.path}</code>.
          </p>
        )}
        {!error && !data && <p className="l-empty">Loading {drill.title}…</p>}
        {data && pane === 'theory' && <Markdown source={data.theory_markdown} />}
        {data && pane === 'hints' && <Markdown source={data.hints_markdown} />}
      </section>

      <section className="l-card l-card-sm l-card-pad" style={{ gap: 10 }}>
        <span className="l-label">Do the drill</span>
        <span className="l-aside-text">
          Open <code>{drill.path}</code> in your editor and run the watcher:
        </span>
        <pre className="l-md-code">
          <code>{command}</code>
        </pre>
      </section>
    </div>
  );
};
