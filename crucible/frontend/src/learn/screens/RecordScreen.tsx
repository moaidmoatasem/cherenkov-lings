import React from 'react';
import { CERTIFICATE } from '../content';
import { TOTAL_ACHIEVEMENTS, type LearnerProgress } from '../useLearnerProgress';
import type { Track } from '../types';

interface RecordScreenProps {
  progress: LearnerProgress;
  tracks: Track[];
}

/**
 * The learner's record, and nothing else.
 *
 * The screen's own promise is "evidence, not badges", so every figure on it now
 * comes from `/api/progress` and the curriculum manifest. It previously mixed
 * two live numbers in among invented ones -- 86% kept sessions, 9h 40m spent, a
 * certificate four modules along, a "Foundations, all five built · August 18"
 * award -- which contradicted the real count sitting beside them and survived
 * unchanged after a real drill was completed.
 */
export const RecordScreen: React.FC<RecordScreenProps> = ({ progress, tracks }) => {
  // The track the learner is furthest into is the one worth showing a
  // certificate for. Ties break toward the larger track.
  const leadTrack = [...tracks]
    .filter((t) => t.total > 0)
    .sort((a, b) => b.done - a.done || b.total - a.total)[0];

  const started = tracks.filter((t) => t.done > 0);

  return (
    <div className="l-col" style={{ gap: 24 }}>
      <section className="l-kpis">
        {progress.kpis.map((kpi) => (
          <div key={kpi.label} className="l-kpi">
            <span className="l-label l-nowrap">{kpi.label}</span>
            <span className="l-kpi-value">{kpi.value}</span>
            <span className="l-kpi-sub">{kpi.sub}</span>
          </div>
        ))}
      </section>

      {/* The evidence model, and the screen's main idea. */}
      <section className="l-prove">
        <div className="l-prove-head">
          <h3>What you can prove</h3>
          <span className="l-meta">
            a drill counts once it has passed five runs under chaos
          </span>
        </div>
        <div className="l-prove-list">
          {started.length === 0 && (
            <p className="l-meta" style={{ padding: '6px 0' }}>
              Nothing yet. Build a drill and it shows up here, by track.
            </p>
          )}
          {started.map((track) => {
            const pct = Math.round((track.done / track.total) * 100);
            return (
              <div key={track.id} className="l-skill">
                <span className="l-skill-label">{track.name}</span>
                <div className="l-skill-segments" aria-hidden="true">
                  {[1, 2, 3].map((stage) => (
                    <div
                      key={stage}
                      className="l-skill-seg"
                      data-stage={stage}
                      data-on={pct >= stage * 33}
                    />
                  ))}
                </div>
                <span className="l-skill-stage" data-level={Math.ceil(pct / 34)}>
                  {track.done} of {track.total} built
                </span>
              </div>
            );
          })}
        </div>
      </section>

      <section className="l-cert-split">
        <div className="l-cert">
          <div className="l-cert-inset" />
          <div className="l-cert-body">
            <span className="l-label">Certificate · in progress</span>
            <h3 className="l-cert-title">
              {leadTrack ? `${leadTrack.name}, proven under chaos` : CERTIFICATE.title}
            </h3>
            <p className="l-cert-copy">{CERTIFICATE.copy}</p>
            <div className="l-cert-pips">
              {Array.from({ length: leadTrack?.total ?? 0 }, (_, i) => (
                <span key={i} className="l-cert-pip" data-on={i < (leadTrack?.done ?? 0)} />
              ))}
            </div>
            <div className="l-cert-actions">
              {/* No projected date: the platform records what was built, not
                  how fast the learner is expected to go. */}
              <span>
                {leadTrack
                  ? `${leadTrack.done} of ${leadTrack.total} built`
                  : 'Nothing built yet'}
              </span>
            </div>
          </div>
        </div>

        <div
          className="l-card l-card-sm"
          style={{ padding: 22, display: 'flex', flexDirection: 'column', gap: 15 }}
        >
          <div className="l-row" style={{ alignItems: 'baseline', gap: 9 }}>
            <span className="l-label">Already yours</span>
            <span className="l-spacer" />
            <span className="l-meta l-nowrap">
              {progress.badges.length} of {TOTAL_ACHIEVEMENTS}
            </span>
          </div>

          {progress.badges.length === 0 && (
            <span className="l-meta">
              No badges yet. Completing your first drill earns First Blood.
            </span>
          )}

          {progress.badges.map((badge) => (
            <div key={badge.id || badge.name} className="l-badge-row">
              <span className="l-badge-icon" data-tone="moss" aria-hidden="true">
                ✓
              </span>
              <div className="l-badge-body">
                <span className="l-badge-name">{badge.name}</span>
                <span className="l-badge-meta">
                  {badge.unlockedOn ? `${badge.description} · ${badge.unlockedOn}` : badge.description}
                </span>
              </div>
            </div>
          ))}
        </div>
      </section>
    </div>
  );
};
