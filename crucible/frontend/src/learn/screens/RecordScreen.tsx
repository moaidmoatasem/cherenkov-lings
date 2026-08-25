import React, { useState } from 'react';
import { BADGES, CERTIFICATE, KPIS, LEARNER, SKILLS } from '../content';
import type { Kpi } from '../types';

interface RecordScreenProps {
  kpis?: Kpi[];
}

export const RecordScreen: React.FC<RecordScreenProps> = ({ kpis = KPIS }) => {
  const [copied, setCopied] = useState<boolean>(false);

  const copySlug = async () => {
    try {
      await navigator.clipboard.writeText(LEARNER.publicSlug);
      setCopied(true);
      window.setTimeout(() => setCopied(false), 1600);
    } catch {
      // fallback: ignore in insecure context
    }
  };

  return (
    <div className="l-col" style={{ gap: 24 }}>
      <section className="l-kpis">
        {kpis.map((kpi) => (
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
            read about it → answered questions → made it work under chaos
          </span>
        </div>
        <div className="l-prove-list">
          {SKILLS.map((skill) => (
            <div key={skill.label} className="l-skill">
              <span className="l-skill-label">{skill.label}</span>
              <div className="l-skill-segments">
                {[1, 2, 3].map((stage) => (
                  <div
                    key={stage}
                    className="l-skill-seg"
                    data-stage={stage}
                    data-on={skill.level >= stage}
                  />
                ))}
              </div>
              <span className="l-skill-stage" data-level={skill.level}>
                {skill.stage}
              </span>
            </div>
          ))}
        </div>
      </section>

      <section className="l-cert-split">
        <div className="l-cert">
          <div className="l-cert-inset" />
          <div className="l-cert-body">
            <span className="l-label">Certificate · in progress</span>
            <h3 className="l-cert-title">{CERTIFICATE.title}</h3>
            <p className="l-cert-copy">{CERTIFICATE.copy}</p>
            <div className="l-cert-pips">
              {Array.from({ length: CERTIFICATE.modulesTotal }, (_, i) => (
                <span key={i} className="l-cert-pip" data-on={i < CERTIFICATE.modulesBuilt} />
              ))}
            </div>
            <div className="l-cert-actions">
              <span>{CERTIFICATE.projectedOn}</span>
              <button type="button" className="l-btn-outline">
                Preview it
              </button>
              <button type="button" className="l-btn l-btn-md" style={{ padding: '0 18px' }}>
                Share settings
              </button>
            </div>
          </div>
        </div>

        <div className="l-card l-card-sm" style={{ padding: 22, display: 'flex', flexDirection: 'column', gap: 15 }}>
          <span className="l-label">Already yours</span>
          {BADGES.map((badge) => (
            <div key={badge.name} className="l-badge-row">
              <span className="l-badge-icon" data-tone={badge.tone} aria-hidden="true">
                {badge.icon}
              </span>
              <div className="l-badge-body">
                <span className="l-badge-name">{badge.name}</span>
                <span className="l-badge-meta">{badge.meta}</span>
              </div>
            </div>
          ))}

          <div className="l-record-foot">
            <span className="l-label">Your page</span>
            <div className="l-page-url">
              <span>{LEARNER.publicSlug}</span>
              <span className="l-spacer" />
              <button type="button" className="l-btn l-btn-ghost l-btn-sm" onClick={copySlug}>
                {copied ? 'Copied' : 'Copy'}
              </button>
            </div>
          </div>
        </div>
      </section>
    </div>
  );
};
