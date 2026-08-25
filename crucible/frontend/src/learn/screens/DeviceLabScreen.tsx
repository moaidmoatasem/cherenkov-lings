import React, { useState } from 'react';
import { IterationDots, Tick } from '../components/Primitives';
import {
  DEVICE_CONDITIONS,
  DEVICE_COPY,
  DEVICE_FILE,
  DEVICE_FLOW,
  DEVICE_INTRO,
  DEVICE_YAML,
  PASSING_RUN,
} from '../content';

/** Same grammar as the browser lab, for Maestro flows. */
export const DeviceLabScreen: React.FC = () => {
  const [conditions, setConditions] = useState(DEVICE_CONDITIONS);

  const toggle = (label: string) =>
    setConditions((prev) =>
      prev.map((c) => (c.label === label ? { ...c, on: !c.on } : c))
    );

  const doneSteps = DEVICE_FLOW.filter((s) => s.state === 'done').length + 1;

  return (
    <div className="l-split l-device">
      <div className="l-col" style={{ gap: 18 }}>
        <p style={{ fontSize: 14.5, color: 'var(--l-ink-body)', lineHeight: 1.65, maxWidth: '64ch' }}>
          {DEVICE_INTRO}
        </p>

        <section className="l-panel">
          <div className="l-panel-bar">
            <span className="l-panel-file">{DEVICE_FILE}</span>
            <span className="l-spacer" />
            <span className="l-panel-status">{DEVICE_COPY.hardware}</span>
            <button type="button" className="l-panel-run">
              {DEVICE_COPY.play}
            </button>
          </div>
          <div className="l-code" style={{ padding: '14px 0', borderRight: 'none' }}>
            {DEVICE_YAML.map((line) => (
              <div key={line.n} className="l-code-row" data-kind={line.kind}>
                <span className="l-code-n" style={{ width: 38, paddingRight: 14 }}>
                  {line.n}
                </span>
                <span className="l-code-text" data-kind={line.kind}>
                  {line.text}
                </span>
                {line.tag && <span className="l-code-tag">{line.tag}</span>}
              </div>
            ))}
          </div>
        </section>

        <section className="l-card" style={{ padding: '20px 22px', display: 'flex', flexDirection: 'column', gap: 14 }}>
          <div className="l-row l-wrap" style={{ alignItems: 'baseline', gap: 11 }}>
            <h3 className="l-section-title">{DEVICE_COPY.harderTitle}</h3>
            <span className="l-meta">{DEVICE_COPY.harderHint}</span>
          </div>
          <div className="l-device-conds">
            {conditions.map((cond) => (
              <button
                key={cond.label}
                type="button"
                className="l-cond"
                aria-pressed={cond.on}
                onClick={() => toggle(cond.label)}
              >
                <span className="l-cond-label">{cond.label}</span>
                <span className="l-cond-value">{cond.value}</span>
              </button>
            ))}
          </div>
          <span style={{ fontSize: 13, color: 'var(--l-ink-body)', lineHeight: 1.6 }}>
            {DEVICE_COPY.harderNote}
          </span>
        </section>
      </div>

      <div className="l-device-rail l-sticky">
        <div className="l-bezel">
          <div className="l-screen">
            <div className="l-screen-status">
              <span>9:41</span>
              <span className="l-spacer" />
              <span>3G</span>
              <span>▮▮</span>
            </div>
            <div className="l-screen-head">
              <span className="l-screen-app">{DEVICE_COPY.appName}</span>
              <span className="l-screen-title">{DEVICE_COPY.screenTitle}</span>
            </div>
            <div className="l-screen-body">
              <div className="l-screen-glyph" aria-hidden="true">
                ◍
              </div>
              <span className="l-screen-msg">{DEVICE_COPY.screenMsg}</span>
              <div className="l-screen-cta">
                {DEVICE_COPY.screenCta}
                <span className="l-screen-tag">{DEVICE_COPY.screenTag}</span>
              </div>
            </div>
            <div className="l-screen-dots">
              <IterationDots iterations={PASSING_RUN.iterations} />
            </div>
          </div>
        </div>

        <div className="l-card l-card-sm">
          <div className="l-row" style={{ padding: '15px 17px 11px', alignItems: 'baseline', gap: 9 }}>
            <span className="l-label">The flow</span>
            <span className="l-spacer" />
            <span style={{ fontSize: 12, color: 'var(--l-ink-muted)' }}>
              {doneSteps} of {DEVICE_FLOW.length}
            </span>
          </div>
          {DEVICE_FLOW.map((step) => (
            <div key={step.label} className="l-flow-step" data-state={step.state}>
              <Tick state={step.state} hideNowGlyph />
              <span className="l-flow-label">{step.label}</span>
            </div>
          ))}
        </div>
      </div>
    </div>
  );
};
