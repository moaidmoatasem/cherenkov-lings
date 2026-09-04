import React from 'react';
import { Tick } from '../components/Primitives';
import { CURRENT_MODULE, LOOP_STEPS } from '../content';
import type { LearnerProgress } from '../useLearnerProgress';
import type { CurriculumModule, StepId, Track } from '../types';

interface TodayScreenProps {
  onOpenLab: () => void;
  onOpenModule: (step: StepId) => void;
  onOpenCatalogModule: (module: CurriculumModule, track: Track) => void;
  progress: LearnerProgress;
  tracks: Track[];
}

/**
 * The learner's day.
 *
 * What used to sit in the rail -- a 21-day streak grid, "620 of 900 points
 * earned this week", a three-block schedule with times and reminders, a recall
 * quiz -- described a learner the platform had never met, and none of it had
 * anything behind it: the Reschedule and Start recall buttons had no handlers,
 * and there is no scheduler or recall engine to give them. It is replaced by
 * what the record actually holds, plus the next drill to do, which the
 * curriculum can genuinely answer.
 */
export const TodayScreen: React.FC<TodayScreenProps> = ({
  onOpenLab,
  onOpenModule,
  onOpenCatalogModule,
  progress,
  tracks,
}) => {
  // The next unbuilt drill, in curriculum order -- the same rule the CLI
  // dashboard uses for "next recommended drill".
  let nextTrack: Track | undefined;
  let nextModule: CurriculumModule | undefined;
  for (const track of tracks) {
    const candidate = track.modules.find((m) => m.state !== 'done' && m.path);
    if (candidate) {
      nextTrack = track;
      nextModule = candidate;
      break;
    }
  }

  const pctBuilt = progress.modulesTotal
    ? Math.round((progress.modulesBuilt / progress.modulesTotal) * 100)
    : 0;

  return (
    <div className="l-split l-today">
      <div className="l-col l-today-main">
        {/* The guided walkthrough of one module, identical for everyone. */}
        <section className="l-card l-card-raised">
          <div className="l-continue-body">
            {/* Not "continue where you stopped": this is the guided module,
                the same one for everyone, and a new learner has stopped
                nowhere. The learner's real next drill is in "Next up" below. */}
            <span className="l-label" style={{ fontSize: 12 }}>
              How a module works
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
            <h3 className="l-section-title">Next up</h3>
            <span className="l-meta">the first drill you haven't built</span>
          </div>

          {nextModule && nextTrack ? (
            <div className="l-block" data-state="next">
              <span className="l-block-time">{nextTrack.done + 1}</span>
              <span className="l-block-rail" />
              <div className="l-block-body">
                <span className="l-block-title">{nextModule.title}</span>
                <span className="l-meta">
                  {nextTrack.name} · {nextTrack.done} of {nextTrack.total} built
                </span>
              </div>
              <button
                type="button"
                className="l-btn l-btn-ghost l-btn-sm"
                onClick={() => onOpenCatalogModule(nextModule, nextTrack)}
              >
                Open it
              </button>
            </div>
          ) : (
            <span className="l-meta" style={{ padding: '4px 0' }}>
              {progress.live
                ? 'Every drill in the manifest is built. Nothing left to queue.'
                : 'Reading the curriculum…'}
            </span>
          )}
        </section>
      </div>

      <div className="l-today-rail">
        <div className="l-card l-card-sm l-card-pad">
          <div className="l-row" style={{ alignItems: 'baseline', gap: 9 }}>
            <h3 className="l-section-title" style={{ fontSize: 18 }}>
              Kept it up
            </h3>
            <span className="l-spacer" />
            <span className="l-meta l-nowrap">
              {progress.streakDays} {progress.streakDays === 1 ? 'day' : 'days'}
            </span>
          </div>
          {/* One dot per day of the current streak, so the strip cannot show a
              history the learner does not have. */}
          <div className="l-dots">
            {Array.from({ length: Math.max(progress.streakDays, 1) }, (_, i) => (
              <span key={i} className="l-dot" data-kind={progress.streakDays ? 'recent' : 'rest'} />
            ))}
          </div>
          <span className="l-meta" style={{ lineHeight: 1.55 }}>
            {progress.streakDays > 0
              ? 'A day counts once a drill passes. Miss a day and it resets.'
              : 'Pass a drill today to start a streak.'}
          </span>
        </div>

        <div className="l-earned">
          <h3 className="l-section-title" style={{ fontSize: 18 }}>
            Points earned
          </h3>
          <div className="l-row" style={{ alignItems: 'baseline', gap: 7 }}>
            <span className="l-earned-value">{progress.points.toLocaleString('en-US')}</span>
            <span className="l-earned-of">{progress.levelName}</span>
          </div>
          <div className="l-bar">
            <div className="l-bar-fill" style={{ width: `${pctBuilt}%` }} />
          </div>
          <span className="l-earned-note">
            {progress.modulesBuilt} of {progress.modulesTotal || '—'} modules built,
            across {progress.tracksStarted} of {progress.tracksTotal || '—'} tracks.
          </span>
        </div>

        <div className="l-card l-card-sm l-card-pad" style={{ gap: 12 }}>
          <span className="l-label">How a module is counted</span>
          <span style={{ fontSize: 14.5, fontWeight: 500, lineHeight: 1.45 }}>
            Read it, then make the test hold up five runs in a row under injected
            latency. Watching alone doesn't count.
          </span>
        </div>
      </div>
    </div>
  );
};
