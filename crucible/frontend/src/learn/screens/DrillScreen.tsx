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

interface HintSection {
  title: string;
  body: string;
}

/**
 * hints.md is written as an H1 title followed by "Hint N (...)" sections —
 * `##` in most tracks, `###` in the REST Assured Java ones. Splitting on
 * either lets every drill's hints gate the same way regardless of track.
 */
const HINT_HEADING = /^#{2,3}\s+(Hint\b.*)$/i;

function splitHints(markdown: string): { preamble: string; sections: HintSection[] } {
  const lines = markdown.replace(/\r\n/g, '\n').split('\n');
  const preambleLines: string[] = [];
  const sections: HintSection[] = [];
  let current: { title: string; bodyLines: string[] } | null = null;

  for (const line of lines) {
    const match = HINT_HEADING.exec(line);
    if (match) {
      if (current) sections.push({ title: current.title, body: current.bodyLines.join('\n').trim() });
      current = { title: match[1].trim(), bodyLines: [] };
      continue;
    }
    if (current) current.bodyLines.push(line);
    else preambleLines.push(line);
  }
  if (current) sections.push({ title: current.title, body: current.bodyLines.join('\n').trim() });

  return { preamble: preambleLines.join('\n').trim(), sections };
}

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
  // How many hint sections the learner has chosen to reveal, in order. Hints
  // escalate toward the exact fix, so dumping all three the moment the tab
  // opens skips the struggle the rest of this platform is built around.
  const [revealed, setRevealed] = useState(0);

  useEffect(() => {
    const ctrl = new AbortController();
    setData(null);
    setError(null);
    setPane('theory');
    setRevealed(0);

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
        {data && pane === 'hints' && (() => {
          const { preamble, sections } = splitHints(data.hints_markdown);
          if (sections.length === 0) {
            return <Markdown source={data.hints_markdown} />;
          }
          return (
            <div className="l-col" style={{ gap: 14 }}>
              {preamble && <Markdown source={preamble} />}
              <p className="l-aside-text">
                Each hint gets you closer to the fix. Reveal them one at a time — the earlier
                ones are worth sitting with before you ask for the next.
              </p>
              {sections.map((section, i) =>
                i < revealed ? (
                  <div key={i} className="l-card l-card-sm l-card-pad">
                    <span className="l-label">{section.title}</span>
                    <Markdown source={section.body} />
                  </div>
                ) : i === revealed ? (
                  <button
                    key={i}
                    type="button"
                    className="l-btn l-btn-outline l-btn-md"
                    onClick={() => setRevealed(i + 1)}
                  >
                    {i === sections.length - 1
                      ? `Show hint ${i + 1} of ${sections.length} — the full fix`
                      : `Show hint ${i + 1} of ${sections.length}`}
                  </button>
                ) : null
              )}
            </div>
          );
        })()}
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
