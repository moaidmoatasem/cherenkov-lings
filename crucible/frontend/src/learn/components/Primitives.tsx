import React from 'react';
import type { ProgressState } from '../types';

/**
 * Progress ring. A conic-gradient turn for the completed portion with an inset
 * hole punched out of the middle — no SVG, per the design's asset notes.
 */
export const ProgressRing: React.FC<{
  size: number;
  inset: number;
  fraction: number;
  label: string;
  labelSize?: number;
  serif?: boolean;
}> = ({ size, inset, fraction, label, labelSize = 15, serif = false }) => {
  const turn = fraction.toFixed(3);
  return (
    <div className="l-ring" style={{ width: size, height: size }}>
      <div
        className="l-ring-track"
        style={{
          background: `conic-gradient(${
            fraction > 0 ? 'var(--l-blue)' : 'var(--l-border)'
          } 0turn ${turn}turn, var(--l-border) ${turn}turn 1turn)`,
        }}
      />
      <div className="l-ring-hole" style={{ inset }}>
        <span
          style={{
            fontFamily: serif ? 'var(--l-serif)' : undefined,
            fontSize: labelSize,
            fontWeight: serif ? 500 : undefined,
            color: serif ? undefined : 'var(--l-ink-body)',
          }}
        >
          {label}
        </span>
      </div>
    </div>
  );
};

const GLYPH: Record<ProgressState, string> = { done: '✓', now: '●', todo: '' };

/** The read/watch/practice/build state marker. Size comes from the parent rule. */
export const Tick: React.FC<{ state: ProgressState; hideNowGlyph?: boolean }> = ({
  state,
  hideNowGlyph = false,
}) => (
  <span className="l-tick" data-state={state} aria-hidden="true">
    {hideNowGlyph && state === 'now' ? '' : GLYPH[state]}
  </span>
);

/** Five iteration dots. The in-flight one breathes rather than spinning. */
export const IterationDots: React.FC<{
  iterations: Array<{ index: number; passed: boolean; settled: boolean }>;
}> = ({ iterations }) => (
  <>
    {iterations.map((it) => (
      <span
        key={it.index}
        className="l-iter-dot"
        data-done={it.settled && it.passed}
        data-pending={!it.settled}
      />
    ))}
  </>
);
