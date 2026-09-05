import React from 'react';
import type { BadgeDefinition, BadgeCompletionState } from './types';

export interface BadgeCardProps {
  badge: BadgeDefinition;
  state: BadgeCompletionState;
  onClick?: () => void;
}

export const BadgeCard: React.FC<BadgeCardProps> = ({ badge, state, onClick }) => {
  const { id, name, icon, desc, category, requiredPath } = badge;
  const { unlocked, unlockedAt, progressPercent } = state;
  const pct = Math.min(100, Math.max(0, progressPercent ?? 0));

  const formatUnlockedDate = (dateStr?: unknown) => {
    if (!dateStr || typeof dateStr !== 'string') return 'Active';
    try {
      const d = new Date(dateStr);
      return isNaN(d.getTime()) ? dateStr : d.toLocaleDateString();
    } catch {
      return dateStr;
    }
  };

  return (
    <div
      data-testid={`badge-${id}`}
      data-unlocked={unlocked ? 'true' : 'false'}
      data-category={category}
      className={`mastery-badge-card ${unlocked ? 'unlocked' : 'locked'}`}
      onClick={onClick}
      style={{
        background: unlocked ? 'rgba(56, 189, 248, 0.08)' : 'rgba(15, 23, 42, 0.4)',
        border: unlocked ? '1px solid var(--accent-cyan)' : '1px solid var(--border-color)',
        boxShadow: unlocked ? '0 0 14px rgba(56, 189, 248, 0.15)' : 'none',
        borderRadius: '8px',
        padding: '14px 16px',
        display: 'flex',
        flexDirection: 'column',
        gap: '10px',
        transition: 'all 0.2s ease',
        opacity: unlocked ? 1 : 0.7,
        cursor: onClick ? 'pointer' : 'default',
        position: 'relative',
      }}
    >
      <div style={{ display: 'flex', alignItems: 'center', justifyContent: 'space-between', gap: '8px' }}>
        <div style={{ display: 'flex', alignItems: 'center', gap: '12px' }}>
          <div style={{ fontSize: '28px', filter: unlocked ? 'none' : 'grayscale(80%)' }}>
            {icon}
          </div>
          <div>
            <div
              data-testid="badge-name"
              style={{
                fontWeight: 700,
                fontSize: '14px',
                color: unlocked ? 'var(--accent-cyan)' : 'var(--text-main)',
              }}
            >
              {name}
            </div>
            <div
              data-testid="badge-path-desc"
              style={{ fontSize: '11px', color: 'var(--text-muted)' }}
            >
              {requiredPath}
            </div>
          </div>
        </div>

        <span
          data-testid="badge-status-pill"
          className={`badge ${unlocked ? 'info' : 'warning'}`}
          style={{ fontSize: '10px', padding: '2px 8px', whiteSpace: 'nowrap' }}
        >
          {unlocked ? '✓ UNLOCKED' : '🔒 LOCKED'}
        </span>
      </div>

      <p
        data-testid="badge-desc"
        style={{ fontSize: '12px', color: 'var(--text-muted)', margin: 0, lineHeight: 1.4 }}
      >
        {desc}
      </p>

      {unlocked && (
        <div
          data-testid="badge-unlocked-at"
          style={{
            fontSize: '11px',
            color: 'var(--accent-green, #10b981)',
            fontFamily: 'var(--font-mono, monospace)',
            marginTop: '2px',
          }}
        >
          Unlocked: {formatUnlockedDate(unlockedAt)}
        </div>
      )}

      {!unlocked && (
        <div data-testid="badge-progress-container" style={{ marginTop: '4px' }}>
          <div
            style={{
              display: 'flex',
              justifyContent: 'space-between',
              fontSize: '10px',
              color: 'var(--text-muted)',
              marginBottom: '3px',
            }}
          >
            <span>Path Completion</span>
            <span data-testid="badge-progress-percent">{pct}%</span>
          </div>
          <div
            data-testid="badge-progress-bar"
            style={{
              width: '100%',
              height: '4px',
              background: '#1e293b',
              borderRadius: '2px',
              overflow: 'hidden',
            }}
          >
            <div
              style={{
                width: `${pct}%`,
                height: '100%',
                background: 'var(--accent-amber, #f59e0b)',
                transition: 'width 0.3s ease',
              }}
            />
          </div>
        </div>
      )}
    </div>
  );
};
