import React from 'react';
import { Tick } from '../components/Primitives';
import {
  CURRENT_MODULE,
  LEARNER,
  LOOP_STEPS,
  PACE_NOTE,
  SCHEDULE,
  STREAK_DOTS,
  WEEK_POINTS,
} from '../content';
import type { StepId } from '../types';

interface TodayScreenProps {
  onOpenLab: () => void;
  onOpenModule: (step: StepId) => void;
}

export const TodayScreen: React.FC<TodayScreenProps> = ({ onOpenLab, onOpenModule }) => {
  const pct = Math.round((WEEK_POINTS.earned / WEEK_POINTS.target) * 100);

  return (
    <div className="l-split l-today">
      <div className="l-col l-today-main">
        {/* The single resume point: it deep-links to the exact unfinished step. */}
        <section className="l-card l-card-raised">
          <div className="l-continue-body">
            <span className="l-label" style={{ fontSize: 12 }}>
              Continue where you stopped
            </span>
            <h2 className="l-continue-title">{CURRENT_MODULE.title}</h2>
            <p className="l-continue-lede">{CURRENT_MODULE.lede}</p>

            <div className="l-loop">
              {LOOP_STEPS.map((step) => (
                <div key={step.id} className="l-loop-row" data-state={step.state}>
                  <Tick state={step.state} />
                  <span className="l-loop-label">{step.label}</span>
                  <span className="l-spacer" />
                  <span className="l-loop-detail">{step.detail}</span>
                </div>
              ))}
            </div>

            <div className="l-continue-actions">
              <button type="button" className="l-btn" onClick={onOpenLab}>
                Open the lab and build it
              </button>
              <button
                type="button"
                className="l-btn l-btn-ghost"
                onClick={() => onOpenModule('read')}
              >
                Re-read the page first
              </button>
            </div>
          </div>
          <div className="l-continue-foot">
            <span className="l-continue-foot-note">
              Runs in your browser. Nothing to install, and your place is kept if you stop.
            </span>
            <span className="l-continue-foot-left">{CURRENT_MODULE.minutesLeft}</span>
          </div>
        </section>

        <section className="l-card">
          <div className="l-section-head">
            <h3 className="l-section-title">Your day</h3>
            <span className="l-meta">two blocks, both movable</span>
            <span className="l-spacer" />
            <button type="button" className="l-btn l-btn-ghost l-btn-sm">
              Reschedule
            </button>
          </div>
          {SCHEDULE.map((block) => (
            <div key={block.time} className="l-block" data-state={block.kind}>
              <span className="l-block-time">{block.time}</span>
              <span className="l-block-rail" />
              <div className="l-block-body">
                <span className="l-block-title">{block.title}</span>
                <span className="l-meta">{block.meta}</span>
              </div>
              <span className="l-block-state">{block.stateLabel}</span>
            </div>
          ))}
        </section>
      </div>

      <div className="l-today-rail">
        {/* The learner's own reason for being here. Keep the rotation. */}
        <div className="l-sticky-note">
          <span className="l-label">Why you're here</span>
          <p className="l-sticky-note-quote">{LEARNER.motivation}</p>
          <span className="l-sticky-note-by">{LEARNER.motivationWhen}</span>
        </div>

        <div className="l-card l-card-sm l-card-pad">
          <div className="l-row" style={{ alignItems: 'baseline', gap: 9 }}>
            <h3 className="l-section-title" style={{ fontSize: 18 }}>
              Kept it up
            </h3>
            <span className="l-spacer" />
            <span className="l-meta l-nowrap">{LEARNER.streakDays} days</span>
          </div>
          <div className="l-dots">
            {STREAK_DOTS.map((kind, i) => (
              <span key={i} className="l-dot" data-kind={kind} />
            ))}
          </div>
          <span className="l-meta" style={{ lineHeight: 1.55 }}>
            {PACE_NOTE.weeks} <span style={{ color: 'var(--l-ink)' }}>{PACE_NOTE.finishDate}</span>.
          </span>
        </div>

        <div className="l-earned">
          <h3 className="l-section-title" style={{ fontSize: 18 }}>
            Earned this week
          </h3>
          <div className="l-row" style={{ alignItems: 'baseline', gap: 7 }}>
            <span className="l-earned-value">{WEEK_POINTS.earned}</span>
            <span className="l-earned-of">of {WEEK_POINTS.target} points</span>
          </div>
          <div className="l-bar">
            <div className="l-bar-fill" style={{ width: `${pct}%` }} />
          </div>
          <span className="l-earned-note">{PACE_NOTE.unlock}</span>
        </div>

        <div className="l-card l-card-sm l-card-pad" style={{ gap: 12 }}>
          <span className="l-label">{PACE_NOTE.recallKicker}</span>
          <span style={{ fontSize: 14.5, fontWeight: 500, lineHeight: 1.45 }}>
            {PACE_NOTE.recall}
          </span>
          <button type="button" className="l-btn l-btn-ghost l-btn-md l-btn-full">
            Start recall
          </button>
        </div>
      </div>
    </div>
  );
};
