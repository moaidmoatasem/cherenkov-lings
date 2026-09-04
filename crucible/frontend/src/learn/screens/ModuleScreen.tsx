import React, { useState } from 'react';
import { Tick } from '../components/Primitives';
import {
  ANSWERS,
  ARTICLE,
  CHAPTERS,
  CHECKPOINTS,
  PRACTICE,
  STEP_TABS,
  TOC,
  VIDEO,
} from '../content';
import type { StepId } from '../types';

interface ModuleScreenProps {
  step: StepId;
  onStep: (step: StepId) => void;
  onOpenLab: () => void;
}

export const ModuleScreen: React.FC<ModuleScreenProps> = ({ step, onStep, onOpenLab }) => (
  <div className="l-col" style={{ gap: 22 }}>
    {/* Said plainly, once: this is a fixed illustration of one module, not the
        learner's own place in it. The Read/Watch/Practice content here does
        not change per learner or per drill -- for a drill with real theory
        and hints behind it, open it from the catalog instead. */}
    <p className="l-meta" style={{ lineHeight: 1.6 }}>
      <strong style={{ color: 'var(--l-ink)' }}>A worked example.</strong> One module,
      shown end to end, so the shape of read → watch → practice → build is clear
      before you start. A drill opened from All modules shows its own theory and
      hints instead.
    </p>

    <div className="l-steps" role="tablist" aria-label="Module steps">
      {STEP_TABS.map((tab) => (
        <button
          key={tab.id}
          type="button"
          role="tab"
          className="l-step"
          aria-current={step === tab.id ? 'step' : undefined}
          aria-selected={step === tab.id}
          onClick={() => (tab.id === 'build' ? onOpenLab() : onStep(tab.id))}
        >
          <Tick state={tab.state} hideNowGlyph />
          <span>{tab.label}</span>
          <span className="l-step-meta">{tab.detail}</span>
        </button>
      ))}
    </div>

    {step === 'read' && <ReadStep onNext={() => onStep('watch')} />}
    {step === 'watch' && <WatchStep />}
    {step === 'practice' && <PracticeStep onOpenLab={onOpenLab} />}
  </div>
);

// ─── Read ──────────────────────────────────────────────────────────────────

const ReadStep: React.FC<{ onNext: () => void }> = ({ onNext }) => {
  const [activeSection, setActiveSection] = useState<number>(1);

  return (
    <div className="l-split l-read">
      <article className="l-article">
        <div className="l-article-kicker">
          <span>{ARTICLE.kicker}</span>
          <span className="l-article-dot" />
          <span>{ARTICLE.savedNote}</span>
        </div>
        <h2>{ARTICLE.title}</h2>

        {ARTICLE.paragraphs.map((segments, i) => (
          <p key={i}>
            {segments.map((seg, j) => (seg.em ? <em key={j}>{seg.text}</em> : <React.Fragment key={j}>{seg.text}</React.Fragment>))}
          </p>
        ))}

        <div className="l-pullquote">
          <span className="l-label">{ARTICLE.pullquote.kicker}</span>
          <span className="l-pullquote-text">{ARTICLE.pullquote.text}</span>
        </div>

        <div className="l-diff">
          <div className="l-diff-head">BEFORE / AFTER</div>
          <div className="l-diff-line" data-kind="removed">
            <span className="l-diff-sign">−</span>
            <span>{ARTICLE.diff.removed}</span>
          </div>
          <div className="l-diff-line" data-kind="added">
            <span className="l-diff-sign">+</span>
            <span>{ARTICLE.diff.added}</span>
          </div>
        </div>

        <p>{ARTICLE.closing}</p>

        <div className="l-row l-wrap" style={{ gap: 12, paddingTop: 6 }}>
          <button type="button" className="l-btn" style={{ fontSize: 14 }} onClick={onNext}>
            Next · watch the trace
          </button>
          <button type="button" className="l-btn l-btn-ghost" style={{ fontSize: 13.5, padding: '0 16px' }}>
            Save a note
          </button>
        </div>
      </article>

      <div className="l-toc l-sticky">
        <span className="l-label">In this page</span>
        {TOC.map((entry, i) => (
          <button
            key={entry.label}
            type="button"
            className="l-toc-item"
            aria-current={i === activeSection}
            onClick={() => setActiveSection(i)}
          >
            {entry.label}
          </button>
        ))}
        <div className="l-toc-foot">
          <span className="l-label">Comes back in</span>
          <span style={{ fontSize: 12.5, color: 'var(--l-ink-body)', lineHeight: 1.55 }}>
            {ARTICLE.comesBackIn}
          </span>
        </div>
      </div>
    </div>
  );
};

// ─── Watch ─────────────────────────────────────────────────────────────────

const WatchStep: React.FC = () => (
  <div className="l-split l-watch">
    <div className="l-col" style={{ gap: 16 }}>
      <div className="l-player-frame">
        <div className="l-player">
          <button type="button" className="l-player-play" aria-label="Play the module video">
            ▶
          </button>
          <div className="l-player-controls">
            <div className="l-scrub">
              <div className="l-scrub-fill" style={{ width: `${VIDEO.progressPct}%` }} />
            </div>
            <div className="l-player-meta">
              <span>{VIDEO.position}</span>
              <span className="l-spacer" />
              <span className="l-player-chip">1.25×</span>
              <span className="l-player-chip">CC</span>
              <span className="l-player-chip">transcript</span>
            </div>
          </div>
        </div>
      </div>

      <h3 className="l-watch-title">{VIDEO.title}</h3>
      <p className="l-watch-body">{VIDEO.body}</p>

      <div className="l-offer">
        <span className="l-offer-kicker">{VIDEO.offer.kicker}</span>
        <span className="l-offer-text">{VIDEO.offer.text}</span>
        <button type="button" className="l-offer-btn">
          {VIDEO.offer.action}
        </button>
      </div>
    </div>

    <div className="l-col" style={{ gap: 16 }}>
      <div className="l-card l-card-sm">
        <div className="l-label" style={{ padding: '14px 17px 11px' }}>
          Chapters
        </div>
        {CHAPTERS.map((chapter) => (
          <button key={chapter.time} type="button" className="l-chapter" data-state={chapter.state}>
            <span className="l-chapter-time">{chapter.time}</span>
            <span className="l-chapter-label">{chapter.label}</span>
            <span className="l-chapter-mark">
              {chapter.state === 'done' ? '✓' : chapter.state === 'now' ? '▶' : ''}
            </span>
          </button>
        ))}
      </div>

      <div className="l-timestamp-note">
        <span className="l-label">{VIDEO.note.at}</span>
        <span className="l-timestamp-note-text">{VIDEO.note.text}</span>
      </div>
    </div>
  </div>
);

// ─── Practice ──────────────────────────────────────────────────────────────

const PracticeStep: React.FC<{ onOpenLab: () => void }> = ({ onOpenLab }) => {
  // Nothing is graded, so a pick just reveals whether it was the one.
  const [picked, setPicked] = useState<string | null>('B');
  const answered = picked !== null;
  const correct = ANSWERS.find((a) => a.key === picked)?.correct ?? false;

  return (
    <div className="l-split l-practice">
      <div className="l-col" style={{ gap: 18 }}>
        <div className="l-question-card">
          <span className="l-label" style={{ fontSize: 12 }}>
            {PRACTICE.kicker}
          </span>
          <h3 className="l-question">{PRACTICE.question}</h3>

          <div className="l-snippet">
            {PRACTICE.snippet.map((line) => (
              <div key={line.text} className="l-snippet-line" data-kind={line.bad ? 'bad' : 'ok'}>
                {line.text}
              </div>
            ))}
          </div>

          <div className="l-answers">
            {ANSWERS.map((answer) => {
              const isPicked = picked === answer.key;
              const kind = !isPicked ? 'idle' : answer.correct ? 'right' : 'wrong';
              return (
                <button
                  key={answer.key}
                  type="button"
                  className="l-answer"
                  data-kind={kind}
                  aria-pressed={isPicked}
                  onClick={() => setPicked(answer.key)}
                >
                  <span className="l-answer-key">{answer.key}</span>
                  <span className="l-answer-label">{answer.label}</span>
                  <span className="l-answer-mark">
                    {kind === 'right' ? '✓' : kind === 'wrong' ? '×' : ''}
                  </span>
                </button>
              );
            })}
          </div>

          {answered && correct && (
            <div className="l-explain">
              <span className="l-label">{PRACTICE.explanation.kicker}</span>
              <span className="l-explain-text">{PRACTICE.explanation.text}</span>
              <button type="button" className="l-btn l-btn-moss" onClick={onOpenLab}>
                {PRACTICE.explanation.action}
              </button>
            </div>
          )}
        </div>
      </div>

      <div className="l-card l-card-sm l-card-pad" style={{ padding: 19 }}>
        <span className="l-label">The five</span>
        {CHECKPOINTS.map((checkpoint) => (
          <div key={checkpoint.label} className="l-checkpoint">
            <Tick state={checkpoint.state} hideNowGlyph />
            <span className="l-checkpoint-label">{checkpoint.label}</span>
          </div>
        ))}
        <span className="l-meta" style={{ lineHeight: 1.6, paddingTop: 4 }}>
          {PRACTICE.noPenalty}
        </span>
      </div>
    </div>
  );
};
