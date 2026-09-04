import React from 'react';
import { ProgressRing } from './Primitives';
import { LEARNER } from '../content';
import type { ScreenId } from '../types';

export interface NavEntry {
  id: ScreenId;
  label: string;
  meta: string;
}

export const NAV: NavEntry[] = [
  // No counts here that the platform cannot measure. "2 left" and "3 of 4"
  // were fixed strings that stayed put no matter what the learner did.
  { id: 'today', label: 'Today', meta: '' },
  { id: 'module', label: 'This module', meta: '' },
  { id: 'lab', label: 'Browser lab', meta: '' },
  { id: 'device', label: 'Device lab', meta: '' },
  { id: 'tracks', label: 'All modules', meta: '' },
  { id: 'progress', label: 'My record', meta: '' },
];

interface SidebarProps {
  screen: ScreenId;
  onNavigate: (id: ScreenId) => void;
  modulesBuilt: number;
  modulesTotal: number;
  /** Rank from the learner's record, e.g. "Trainee". */
  levelName: string;
  streakDays: number;
  /** Leaves the learning environment for the sandbox app on the same origin. */
  onExit?: () => void;
}

export const Sidebar: React.FC<SidebarProps> = ({
  screen,
  onNavigate,
  modulesBuilt,
  modulesTotal,
  levelName,
  streakDays,
  onExit,
}) => {
  const fraction = modulesTotal > 0 ? modulesBuilt / modulesTotal : 0;
  const percent = Math.round(fraction * 100);

  return (
    <aside className="l-side">
      <button
        type="button"
        className="l-wordmark"
        onClick={onExit}
        title="Back to the sandbox"
        aria-label="Cherenkov — back to the sandbox"
      >
        <div className="l-wordmark-badge" aria-hidden="true">
          c
        </div>
        <div className="l-wordmark-text">
          <span className="l-wordmark-name">Cherenkov</span>
          <span className="l-wordmark-sub">Learn testing by doing</span>
        </div>
      </button>

      <div className="l-side-progress">
        <ProgressRing size={52} inset={6} fraction={fraction} label={`${percent}%`} serif />
        <div className="l-side-progress-text">
          <span className="l-side-progress-name">{LEARNER.name}</span>
          <span className="l-side-progress-meta">
            {modulesBuilt} of {modulesTotal} modules
            <br />
            {levelName}
          </span>
        </div>
      </div>

      <nav className="l-nav" aria-label="Sections">
        {NAV.map((item) => (
          <button
            key={item.id}
            type="button"
            className="l-nav-item"
            aria-current={screen === item.id ? 'page' : undefined}
            onClick={() => onNavigate(item.id)}
          >
            <span className="l-nav-item-dot" />
            <span className="l-nav-item-label">{item.label}</span>
            <span className="l-nav-item-meta">
              {item.id === 'tracks' ? String(modulesTotal) : item.meta}
            </span>
          </button>
        ))}
      </nav>

      <div className="l-side-foot">
        <div className="l-next-session">
          <span className="l-label">Streak</span>
          <span className="l-next-session-when">
            {streakDays} {streakDays === 1 ? 'day' : 'days'}
          </span>
          <span className="l-next-session-note">
            A day counts once a drill passes under chaos.
          </span>
        </div>
      </div>
    </aside>
  );
};

interface HeaderProps {
  crumb: string;
  heading: string;
  note: string;
  bigText: boolean;
  easyFace: boolean;
  onToggleBigText: () => void;
  onToggleEasyFace: () => void;
}

export const Header: React.FC<HeaderProps> = ({
  crumb,
  heading,
  note,
  bigText,
  easyFace,
  onToggleBigText,
  onToggleEasyFace,
}) => (
  <header className="l-header">
    <div className="l-header-titles">
      <span className="l-crumb">{crumb}</span>
      <h1 className="l-h1">{heading}</h1>
    </div>
    <span className="l-header-note">{note}</span>
    <div className="l-a11y">
      <button
        type="button"
        className="l-a11y-btn"
        title="Bigger text"
        aria-label="Bigger text"
        aria-pressed={bigText}
        onClick={onToggleBigText}
      >
        A⁺
      </button>
      <button
        type="button"
        className="l-a11y-btn"
        title="Easier-reading typeface"
        aria-label="Easier-reading typeface"
        aria-pressed={easyFace}
        onClick={onToggleEasyFace}
      >
        Dx
      </button>
    </div>
  </header>
);

/** Below 1080px the sidebar hides and this chip row takes over navigation. */
export const TabRow: React.FC<{ screen: ScreenId; onNavigate: (id: ScreenId) => void }> = ({
  screen,
  onNavigate,
}) => (
  <div className="l-tabs">
    {NAV.map((item) => (
      <button
        key={item.id}
        type="button"
        className="l-tab"
        aria-current={screen === item.id ? 'page' : undefined}
        onClick={() => onNavigate(item.id)}
      >
        {item.label}
      </button>
    ))}
  </div>
);
