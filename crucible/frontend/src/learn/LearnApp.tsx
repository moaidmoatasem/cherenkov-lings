import React, { useState } from 'react';
import { Header, Sidebar, TabRow } from './components/Shell';
import { TodayScreen } from './screens/TodayScreen';
import { ModuleScreen } from './screens/ModuleScreen';
import { BrowserLabScreen } from './screens/BrowserLabScreen';
import { DeviceLabScreen } from './screens/DeviceLabScreen';
import { AllModulesScreen } from './screens/AllModulesScreen';
import { RecordScreen } from './screens/RecordScreen';
import { DrillScreen } from './screens/DrillScreen';
import { HomePage } from '../pages/HomePage';
import { useLearnerProgress } from './useLearnerProgress';
import type { CurriculumModule, ScreenId, SelectedDrill, StepId, Track } from './types';
import './learn.css';

/** Breadcrumb, title and the contextual note in the header, per screen. */
const HEADS: Record<ScreenId, [string, string, string]> = {
  today: ['Monday, week 3', 'Good evening, Moaid', 'one session left today'],
  module: ['Web Automation · module 4 of 10', 'Waiting without sleeping', 'read · watch · practice · build'],
  lab: ['Web Automation · module 4 · build', 'The lab', 'your code, a real browser, a bad network'],
  device: ['Mobile UI · module 1 · build', 'The device lab', 'a real handset, in a bad mood'],
  tracks: ['Everything there is', '', ''],
  progress: ['Your record', 'What you can prove', 'evidence, not badges'],
  sandbox: ['Sandbox Environment', 'Micro-Crucible Sandbox', 'broken apps for automation testing'],
};

export const LearnApp: React.FC<{ initialScreen?: ScreenId; onExit?: () => void }> = ({ initialScreen = 'today', onExit }) => {
  const [screen, setScreen] = useState<ScreenId>(initialScreen);
  const [step, setStep] = useState<StepId>('practice');
  const [run, setRun] = useState<'pass' | 'fail'>('pass');
  const [bigText, setBigText] = useState<boolean>(false);
  const [easyFace, setEasyFace] = useState<boolean>(false);
  /** Non-null when the learner opened a manifest-backed drill from the catalog. */
  const [drill, setDrill] = useState<SelectedDrill | null>(null);

  const progress = useLearnerProgress();
  const [crumbRaw, headingRaw, noteRaw] = HEADS[screen];

  // The catalog heading counts what the manifest actually declares rather than
  // a literal that drifts every time a track lands.
  let crumb = crumbRaw;
  let heading = headingRaw;
  if (screen === 'tracks') {
    heading = `${progress.tracksTotal} tracks, ${progress.modulesTotal} modules`;
  } else if (screen === 'module' && drill) {
    crumb = drill.trackName;
    heading = drill.title;
  }
  let note = noteRaw;
  if (screen === 'tracks') note = `${progress.modulesBuilt} built so far`;
  else if (screen === 'module' && drill) note = 'theory and hints, straight from the repository';

  const openModule = (nextStep: StepId) => {
    setDrill(null);
    setStep(nextStep);
    setScreen('module');
  };

  /**
   * A module row opens the drill it names. Only the four curated tracks have a
   * hand-written module screen; everything else has real theory and hints on
   * disk, which the drill screen reads through the API.
   */
  const openCatalogModule = (module: CurriculumModule, track: Track) => {
    if (!module.path) {
      openModule(module.id === 'waiting-without-sleeping' ? 'practice' : 'read');
      return;
    }
    setDrill({
      trackId: track.id,
      trackName: track.name,
      title: module.title,
      path: module.path,
    });
    setStep('read');
    setScreen('module');
  };

  return (
    <div
      className="learn-root"
      data-type={bigText ? 'lg' : 'md'}
      data-dys={easyFace ? 'on' : 'off'}
    >
      <Sidebar
        screen={screen}
        onNavigate={setScreen}
        modulesBuilt={progress.modulesBuilt}
        modulesTotal={progress.modulesTotal}
        onExit={onExit}
      />

      <main className="l-main">
        <Header
          crumb={crumb}
          heading={heading}
          note={note}
          bigText={bigText}
          easyFace={easyFace}
          onToggleBigText={() => setBigText((v) => !v)}
          onToggleEasyFace={() => setEasyFace((v) => !v)}
        />

        <TabRow screen={screen} onNavigate={setScreen} />

        {/* Keyed so screen content remounts and the rise animation replays. */}
        <div className="l-page" key={screen}>
          {screen === 'today' && (
            <TodayScreen onOpenLab={() => setScreen('lab')} onOpenModule={openModule} />
          )}

          {screen === 'module' &&
            (drill ? (
              <DrillScreen drill={drill} onBack={() => setScreen('tracks')} />
            ) : (
              <ModuleScreen step={step} onStep={setStep} onOpenLab={() => setScreen('lab')} />
            ))}

          {screen === 'lab' && (
            <BrowserLabScreen
              run={run}
              onRunChange={setRun}
              onOpenRead={() => openModule('read')}
            />
          )}

          {screen === 'device' && <DeviceLabScreen />}

          {screen === 'tracks' && (
            <AllModulesScreen onOpenModule={openCatalogModule} />
          )}

          {screen === 'progress' && <RecordScreen kpis={progress.kpis} />}

          {screen === 'sandbox' && (
            <div style={{ padding: '24px' }}>
              <HomePage onNavigate={(path) => {
                if (onExit) onExit();
                window.history.pushState({}, '', path);
                window.dispatchEvent(new PopStateEvent('popstate'));
              }} />
            </div>
          )}
        </div>
      </main>
    </div>
  );
};
