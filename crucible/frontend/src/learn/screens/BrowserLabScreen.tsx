import React from 'react';
import { IterationDots } from '../components/Primitives';
import {
  FAILING_RUN,
  LAB_INTRO,
  PASSING_RUN,
  RESULT_WIDTHS,
  SPEC_CODE,
  SPEC_FILE,
  VERDICT_COPY,
} from '../content';
import type { RunRecord } from '../types';

interface BrowserLabScreenProps {
  /**
   * Which run to show. In the real app this comes from the run itself; the
   * switcher is a prototype affordance kept so both states stay reviewable.
   */
  run: 'pass' | 'fail';
  onRunChange: (run: 'pass' | 'fail') => void;
  /** Opens the module's read step — the two controls here that are navigation. */
  onOpenRead: () => void;
}

export const BrowserLabScreen: React.FC<BrowserLabScreenProps> = ({
  run,
  onRunChange,
  onOpenRead,
}) => {
  const record: RunRecord = run === 'pass' ? PASSING_RUN : FAILING_RUN;
  const settled = record.iterations.filter((it) => it.settled).length;
  const pct = Math.round((settled / record.iterations.length) * 100);

  return (
    <div className="l-col" style={{ gap: 20 }}>
      <div className="l-lab-intro">
        <p>{LAB_INTRO}</p>
        {/* Said plainly, once. The code below is not an editor and nothing on
            this screen executes: it is a worked example of a run. The loop it
            depicts is real, and runs from the watcher. */}
        <p className="l-meta" style={{ lineHeight: 1.6 }}>
          <strong style={{ color: 'var(--l-ink)' }}>A worked example.</strong> Nothing
          on this page runs. To do it for real, open the drill in your editor and
          start the watcher — it re-runs on save and scores the result:{' '}
          <code>cherenkov-lings watch --track=playwright-ts</code>
        </p>
        <div style={{ display: 'flex', gap: 7 }}>
          {(['pass', 'fail'] as const).map((id) => (
            <button
              key={id}
              type="button"
              className="l-run-tab"
              data-run={id}
              aria-pressed={run === id}
              onClick={() => onRunChange(id)}
            >
              {id === 'pass' ? 'passing run' : 'failing run'}
            </button>
          ))}
        </div>
      </div>

      <section className="l-panel">
        <div className="l-panel-bar">
          <span className="l-panel-file">{SPEC_FILE}</span>
          <span className="l-spacer" />
          <span className="l-panel-status">example run · 5 iterations</span>
        </div>

        <div className="l-lab-split">
          <div className="l-code">
            {SPEC_CODE.map((line) => (
              <div key={line.n} className="l-code-row" data-kind={line.kind}>
                <span className="l-code-n">{line.n}</span>
                <span className="l-code-text" data-kind={line.kind}>
                  {line.text}
                </span>
              </div>
            ))}
          </div>

          <div className="l-preview">
            <div className="l-preview-bar">
              <span className="l-preview-url">localhost:8080/search</span>
              <span className="l-preview-lat">+200 ms</span>
            </div>
            <div className="l-preview-note" style={{ fontSize: 11.5, color: 'var(--l-ink-muted)', padding: '6px 12px', borderBottom: '1px solid var(--l-border)', display: 'flex', gap: 6, alignItems: 'center' }}>
              <span style={{ fontWeight: 600, color: 'var(--l-ink)' }}>Static preview</span>
              <span>— teaching replica of</span>
              <a href="/search" style={{ color: 'var(--l-blue)', textDecoration: 'underline', fontWeight: 500 }}>live /search</a>
              <span>· real test runs in browser at +200 ms</span>
            </div>

            <div className="l-stage">
              <div className="l-row" style={{ gap: 10 }}>
                <div className="l-stage-search">
                  playwright
                  <span className="l-caret" />
                  <span className="l-annotation">getByRole('searchbox')</span>
                </div>
                <div className="l-stage-btn">Search</div>
              </div>

              <div className="l-stage-results">
                <span className="l-annotation" data-tone="moss">
                  3 results visible
                </span>
                {RESULT_WIDTHS.map((w) => (
                  <div key={w} className="l-result-row">
                    <div className="l-result-thumb" />
                    <div className="l-result-lines">
                      <div className="l-result-title" style={{ width: `${w}%` }} />
                      <div className="l-result-sub" style={{ width: `${w - 28}%` }} />
                    </div>
                  </div>
                ))}
              </div>
            </div>

            <div className="l-iter-bar">
              <span className="l-iter-label">
                run {settled} of {record.iterations.length}
              </span>
              <span className="l-iter-track">
                <span className="l-iter-fill" style={{ width: `${pct}%` }} />
              </span>
              <IterationDots iterations={record.iterations} />
            </div>
          </div>
        </div>
      </section>

      {run === 'pass' ? (
        <PassingVerdict record={record} onOpenRead={onOpenRead} />
      ) : (
        <FailingVerdict record={record} />
      )}
    </div>
  );
};

const PassingVerdict: React.FC<{ record: RunRecord; onOpenRead: () => void }> = ({
  record,
  onOpenRead,
}) => (
  <section className="l-verdict">
    <div className="l-verdict-pass">
      <div className="l-verdict-head">
        <span className="l-verdict-tick" aria-hidden="true">
          ✓
        </span>
        <h3 className="l-verdict-title">{VERDICT_COPY.passTitle}</h3>
        <span className="l-points">+{record.points} points</span>
      </div>

      <div className="l-outcomes">
        {record.outcomes.map((outcome) => (
          <div key={outcome.label} className="l-outcome">
            <div className="l-outcome-head">
              <span className="l-outcome-label">{outcome.label}</span>
              <span className="l-outcome-state">{outcome.state}</span>
            </div>
            <div className="l-outcome-bar">
              <div className="l-outcome-fill" style={{ width: `${outcome.pct}%` }} />
            </div>
          </div>
        ))}
      </div>

      <div className="l-verdict-foot">
        <span>{VERDICT_COPY.passFoot}</span>
        <button type="button" className="l-btn l-btn-moss" onClick={onOpenRead}>
          {VERDICT_COPY.passNext}
        </button>
      </div>
    </div>

    <div className="l-card l-card-sm l-card-pad" style={{ gap: 12 }}>
      <span className="l-label">What changed</span>
      <span className="l-aside-text">{VERDICT_COPY.whatChanged}</span>
      <a
        href="#read"
        style={{ fontSize: 13 }}
        onClick={(e) => {
          e.preventDefault();
          onOpenRead();
        }}
      >
        Read: auto-waiting, in depth
      </a>
    </div>
  </section>
);

/** Not a punishment, and not an alert: a first-class content state. */
const FailingVerdict: React.FC<{ record: RunRecord }> = ({ record }) => (
  <section className="l-verdict">
    <div className="l-verdict-fail">
      <div className="l-verdict-fail-head">
        <h3>{VERDICT_COPY.failTitle}</h3>
        <span>{VERDICT_COPY.failLede}</span>
      </div>
      {record.failures.map((failure) => (
        <div key={failure.rank} className="l-failure">
          <span className="l-failure-rank">{failure.rank}</span>
          <div className="l-failure-body">
            <span className="l-failure-title">{failure.title}</span>
            <span className="l-failure-detail">{failure.detail}</span>
          </div>
          <span className="l-failure-tag">{failure.tag}</span>
        </div>
      ))}
    </div>

    <div className="l-col" style={{ gap: 16 }}>
      <div className="l-card l-card-sm l-card-pad" style={{ gap: 11 }}>
        <span className="l-label">Why it broke</span>
        <span className="l-aside-text">{VERDICT_COPY.whyItBroke}</span>
      </div>
      <div className="l-card l-card-sm l-card-pad" style={{ gap: 11 }}>
        <span className="l-label">If you're stuck</span>
        <button type="button" className="l-btn l-btn-blue-soft l-btn-md l-btn-full">
          Show me the line to change
        </button>
        <button type="button" className="l-btn l-btn-ghost l-btn-md l-btn-full">
          Run it again, 5×
        </button>
        <span style={{ fontSize: 12, color: 'var(--l-ink-muted)', lineHeight: 1.55 }}>
          {VERDICT_COPY.hintsNote}
        </span>
      </div>
    </div>
  </section>
);
