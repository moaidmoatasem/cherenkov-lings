import React, { useState } from 'react';
import { Header, Sidebar, TabRow } from './components/Shell';
import { TodayScreen } from './screens/TodayScreen';
import { ModuleScreen } from './screens/ModuleScreen';
import { BrowserLabScreen } from './screens/BrowserLabScreen';
import { DeviceLabScreen } from './screens/DeviceLabScreen';
import { AllModulesScreen } from './screens/AllModulesScreen';
import { RecordScreen } from './screens/RecordScreen';
import { HomePage } from '../pages/HomePage';
import { useLearnerProgress } from './useLearnerProgress';
import type { ScreenId, StepId } from './types';
import './learn.css';

/** Breadcrumb, title and the contextual note in the header, per screen. */
const HEADS: Record<ScreenId, [string, string, string]> = {
  today: ['Monday, week 3', 'Good evening, Moaid', 'one session left today'],
  module: ['Web Automation · module 4 of 10', 'Waiting without sleeping', 'read · watch · practice · build'],
  lab: ['Web Automation · module 4 · build', 'The lab', 'your code, a real browser, a bad network'],
  device: ['Mobile UI · module 1 · build', 'The device lab', 'a real handset, in a bad mood'],
  tracks: ['Everything there is', '11 tracks, 60 modules', '14 built so far'],
  progress: ['Your record', 'What you can prove', 'evidence, not badges'],
  sandbox: ['Sandbox Environment', 'Micro-Crucible Sandbox', 'broken apps for automation testing'],
};

export const LearnApp: React.FC<{ initialScreen?: ScreenId; onExit?: () => void }> = ({ initialScreen = 'today', onExit }) => {
  const [screen, setScreen] = useState<ScreenId>(initialScreen);
  const [step, setStep] = useState<StepId>('practice');
  const [run, setRun] = useState<'pass' | 'fail'>('pass');
  const [bigText, setBigText] = useState<boolean>(false);
  const [easyFace, setEasyFace] = useState<boolean>(false);

  const progress = useLearnerProgress();
  const [crumb, heading, note] = HEADS[screen];

  const openModule = (nextStep: StepId) => {
    setStep(nextStep);
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

          {screen === 'module' && (
            <ModuleScreen step={step} onStep={setStep} onOpenLab={() => setScreen('lab')} />
          )}

          {screen === 'lab' && <BrowserLabScreen run={run} onRunChange={setRun} />}

          {screen === 'device' && <DeviceLabScreen />}

          {screen === 'tracks' && (
            <AllModulesScreen
              onOpenModule={(moduleId) =>
                openModule(moduleId === 'waiting-without-sleeping' ? 'practice' : 'read')
              }
            />
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
