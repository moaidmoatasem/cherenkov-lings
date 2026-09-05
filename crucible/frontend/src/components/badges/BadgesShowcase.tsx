import React, { useState } from 'react';
import { MISSION_CONTROL_BADGES } from './types';
import { BadgeCard } from './BadgeCard';
import type { BadgesShowcaseProps, BadgeCompletionState } from './types';

export const BadgesShowcase: React.FC<BadgesShowcaseProps> = ({
  progress,
  completionOverrides,
  onBadgeClick,
}) => {
  const [filter, setFilter] = useState<'all' | 'chaos' | 'architecture' | 'core' | 'performance'>('all');

  const getBadgeState = (badgeId: string): BadgeCompletionState => {
    // 1. Check completionOverrides first (for test mocks or manual overrides)
    if (completionOverrides && completionOverrides[badgeId] !== undefined) {
      const isUnlocked = Boolean(completionOverrides[badgeId]);
      return {
        id: badgeId,
        unlocked: isUnlocked,
        unlockedAt: isUnlocked ? '2026-09-05T10:00:00Z' : undefined,
        progressPercent: isUnlocked ? 100 : 0,
      };
    }

    // 2. Check backend achievements list
    const achievement = Array.isArray(progress?.achievements)
      ? progress.achievements.find((a) => a.id === badgeId)
      : undefined;
    if (achievement) {
      return {
        id: badgeId,
        unlocked: true,
        unlockedAt: achievement.unlocked_at || '2026-09-05T10:00:00Z',
        progressPercent: 100,
      };
    }

    // 3. Fallback path evaluation based on completed_drills and metrics
    const drills = progress?.completed_drills || {};
    if (badgeId === 'chaos_survivor') {
      const hasChaosDrill = Object.entries(drills).some(
        ([key, drill]) =>
          Boolean(drill && ((drill.best_score ?? 0) > 0 || (drill.completion_count ?? 0) > 0)) &&
          (key.includes('k6') || key.includes('chaos') || key.includes('triage') || key.includes('04_perf_k6'))
      );
      const streakProgress = progress?.flakiness_100_streak
        ? Math.min(100, Math.round((progress.flakiness_100_streak / 5) * 100))
        : 0;
      return {
        id: badgeId,
        unlocked: hasChaosDrill,
        unlockedAt: hasChaosDrill ? '2026-09-05T10:00:00Z' : undefined,
        progressPercent: hasChaosDrill ? 100 : streakProgress,
      };
    }

    if (badgeId === 'the_architect') {
      const toolDecisionsDone = Object.entries(drills).filter(
        ([k, drill]) =>
          Boolean(drill && ((drill.best_score ?? 0) > 0 || (drill.completion_count ?? 0) > 0)) &&
          (k.includes('tool-decisions') || k.includes('decision') || k.includes('pipeline'))
      ).length;
      const isArchUnlocked = toolDecisionsDone >= 2;
      return {
        id: badgeId,
        unlocked: isArchUnlocked,
        unlockedAt: isArchUnlocked ? '2026-09-05T10:00:00Z' : undefined,
        progressPercent: isArchUnlocked ? 100 : Math.min(100, Math.round((toolDecisionsDone / 2) * 100)),
      };
    }

    if (badgeId === 'first_blood') {
      const hasAny = Object.values(drills).some(
        (drill) => Boolean(drill && ((drill.best_score ?? 0) > 0 || (drill.completion_count ?? 0) > 0))
      );
      return {
        id: badgeId,
        unlocked: hasAny,
        unlockedAt: hasAny ? '2026-09-05T10:00:00Z' : undefined,
        progressPercent: hasAny ? 100 : 0,
      };
    }

    if (badgeId === 'flakiness_slayer') {
      const streak = progress?.flakiness_100_streak || 0;
      const isUnlocked = streak >= 3;
      return {
        id: badgeId,
        unlocked: isUnlocked,
        unlockedAt: isUnlocked ? '2026-09-05T10:00:00Z' : undefined,
        progressPercent: Math.min(100, Math.round((streak / 3) * 100)),
      };
    }

    if (badgeId === 'perfect_locator') {
      const count = progress?.perfect_locator_count || 0;
      const isUnlocked = count >= 5;
      return {
        id: badgeId,
        unlocked: isUnlocked,
        unlockedAt: isUnlocked ? '2026-09-05T10:00:00Z' : undefined,
        progressPercent: Math.min(100, Math.round((count / 5) * 100)),
      };
    }

    return {
      id: badgeId,
      unlocked: false,
      progressPercent: 0,
    };
  };

  const badgeStates = MISSION_CONTROL_BADGES.map((b) => ({
    badge: b,
    state: getBadgeState(b.id),
  }));

  const unlockedCount = badgeStates.filter((b) => b.state.unlocked).length;

  const filtered = filter === 'all'
    ? badgeStates
    : badgeStates.filter((b) => b.badge.category === filter);

  return (
    <div className="card" data-testid="badges-showcase">
      <div
        style={{
          display: 'flex',
          justifyContent: 'space-between',
          alignItems: 'center',
          marginBottom: '16px',
          flexWrap: 'wrap',
          gap: '12px',
        }}
      >
        <div>
          <h2 className="card-title" style={{ display: 'flex', alignItems: 'center', gap: '8px', margin: 0 }}>
            <span>🏅</span> SDET Mastery Achievements
          </h2>
          <p style={{ color: 'var(--text-muted)', fontSize: '13px', marginTop: '4px' }}>
            Earn industry-recognized micro-credentials by proving fault tolerance, architecture design, and automation speed.
          </p>
        </div>
        <div style={{ display: 'flex', alignItems: 'center', gap: '8px' }}>
          <span
            data-testid="unlocked-counter"
            className="badge info"
            style={{ fontSize: '12px', padding: '6px 12px' }}
          >
            {unlockedCount} of {MISSION_CONTROL_BADGES.length} Unlocked
          </span>
        </div>
      </div>

      {/* Category Tabs */}
      <div style={{ display: 'flex', gap: '8px', marginBottom: '16px', flexWrap: 'wrap' }}>
        <button
          data-testid="filter-all"
          onClick={() => setFilter('all')}
          style={{
            padding: '6px 14px',
            fontSize: '12px',
            borderRadius: '6px',
            cursor: 'pointer',
            border: filter === 'all' ? '1px solid var(--accent-cyan)' : '1px solid var(--border-color)',
            background: filter === 'all' ? 'rgba(56, 189, 248, 0.15)' : 'rgba(15, 23, 42, 0.6)',
            color: filter === 'all' ? 'var(--accent-cyan)' : 'var(--text-main)',
            fontWeight: filter === 'all' ? 700 : 500,
            transition: 'all 0.15s ease',
          }}
        >
          All Badges
        </button>
        <button
          data-testid="filter-core"
          onClick={() => setFilter('core')}
          style={{
            padding: '6px 14px',
            fontSize: '12px',
            borderRadius: '6px',
            cursor: 'pointer',
            border: filter === 'core' ? '1px solid var(--accent-cyan)' : '1px solid var(--border-color)',
            background: filter === 'core' ? 'rgba(56, 189, 248, 0.15)' : 'rgba(15, 23, 42, 0.6)',
            color: filter === 'core' ? 'var(--accent-cyan)' : 'var(--text-main)',
            fontWeight: filter === 'core' ? 700 : 500,
            transition: 'all 0.15s ease',
          }}
        >
          Core
        </button>
        <button
          data-testid="filter-chaos"
          onClick={() => setFilter('chaos')}
          style={{
            padding: '6px 14px',
            fontSize: '12px',
            borderRadius: '6px',
            cursor: 'pointer',
            border: filter === 'chaos' ? '1px solid var(--accent-cyan)' : '1px solid var(--border-color)',
            background: filter === 'chaos' ? 'rgba(56, 189, 248, 0.15)' : 'rgba(15, 23, 42, 0.6)',
            color: filter === 'chaos' ? 'var(--accent-cyan)' : 'var(--text-main)',
            fontWeight: filter === 'chaos' ? 700 : 500,
            transition: 'all 0.15s ease',
          }}
        >
          Chaos
        </button>
        <button
          data-testid="filter-architecture"
          onClick={() => setFilter('architecture')}
          style={{
            padding: '6px 14px',
            fontSize: '12px',
            borderRadius: '6px',
            cursor: 'pointer',
            border: filter === 'architecture' ? '1px solid var(--accent-cyan)' : '1px solid var(--border-color)',
            background: filter === 'architecture' ? 'rgba(56, 189, 248, 0.15)' : 'rgba(15, 23, 42, 0.6)',
            color: filter === 'architecture' ? 'var(--accent-cyan)' : 'var(--text-main)',
            fontWeight: filter === 'architecture' ? 700 : 500,
            transition: 'all 0.15s ease',
          }}
        >
          Architecture
        </button>
        <button
          data-testid="filter-performance"
          onClick={() => setFilter('performance')}
          style={{
            padding: '6px 14px',
            fontSize: '12px',
            borderRadius: '6px',
            cursor: 'pointer',
            border: filter === 'performance' ? '1px solid var(--accent-cyan)' : '1px solid var(--border-color)',
            background: filter === 'performance' ? 'rgba(56, 189, 248, 0.15)' : 'rgba(15, 23, 42, 0.6)',
            color: filter === 'performance' ? 'var(--accent-cyan)' : 'var(--text-main)',
            fontWeight: filter === 'performance' ? 700 : 500,
            transition: 'all 0.15s ease',
          }}
        >
          Performance
        </button>
      </div>

      {/* Badges Grid */}
      <div
        data-testid="badges-grid"
        style={{
          display: 'grid',
          gridTemplateColumns: 'repeat(auto-fill, minmax(min(280px, 100%), 1fr))',
          gap: '14px',
        }}
      >
        {filtered.map(({ badge, state }) => (
          <BadgeCard
            key={badge.id}
            badge={badge}
            state={state}
            onClick={onBadgeClick ? () => onBadgeClick(badge) : undefined}
          />
        ))}
      </div>
    </div>
  );
};
