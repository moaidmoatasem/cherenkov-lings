import React, { useState, useEffect, useMemo, useRef } from 'react';

export interface PipelineStage {
  id: string;
  type: 'trigger' | 'setup' | 'matrix' | 'chaos' | 'test' | 'allure' | 'artifact' | 'custom';
  title: string;
  enabled: boolean;
  config: {
    triggers?: string[];
    checkout?: boolean;
    nodeVersion?: string;
    osList?: string[];
    shards?: string[];
    chaosLatency?: string;
    chaosJitter?: string;
    testCommand?: string;
    reportPath?: string;
    artifactPath?: string;
    customCommand?: string;
  };
}

export interface RunnerJob {
  id: string;
  runnerName: string;
  os: string;
  shard: string;
  status: 'idle' | 'running' | 'success' | 'failed';
  currentStepIndex: number;
  steps: Array<{
    name: string;
    status: 'pending' | 'running' | 'success' | 'failed';
    durationMs: number;
  }>;
  logs: string[];
}

const DEFAULT_STAGES: PipelineStage[] = [
  {
    id: 'stage-trigger',
    type: 'trigger',
    title: 'Workflow Triggers',
    enabled: true,
    config: {
      triggers: ['push: main', 'pull_request: main', 'schedule: "0 2 * * *"']
    }
  },
  {
    id: 'stage-setup',
    type: 'setup',
    title: 'Environment Setup',
    enabled: true,
    config: {
      checkout: true,
      nodeVersion: '20.x'
    }
  },
  {
    id: 'stage-matrix',
    type: 'matrix',
    title: 'Parallel Matrix Strategy',
    enabled: true,
    config: {
      osList: ['ubuntu-latest', 'windows-latest'],
      shards: ['1/4', '2/4', '3/4', '4/4']
    }
  },
  {
    id: 'stage-chaos',
    type: 'chaos',
    title: 'L4/L7 Chaos Proxy',
    enabled: true,
    config: {
      chaosLatency: '200ms',
      chaosJitter: '50ms'
    }
  },
  {
    id: 'stage-test',
    type: 'test',
    title: 'Distributed Playwright Execution',
    enabled: true,
    config: {
      testCommand: 'npx playwright test --shard=${{ matrix.shard }} --reporter=line,allure-playwright'
    }
  },
  {
    id: 'stage-allure',
    type: 'allure',
    title: 'Generate Allure Telemetry',
    enabled: true,
    config: {
      reportPath: 'allure-results'
    }
  },
  {
    id: 'stage-artifact',
    type: 'artifact',
    title: 'Archive Test Traces & Reports',
    enabled: true,
    config: {
      artifactPath: 'allure-results, test-results/'
    }
  }
];

export const PipelineBuilderPage: React.FC = () => {
  const [stages, setStages] = useState<PipelineStage[]>(DEFAULT_STAGES);
  const [yamlCode, setYamlCode] = useState<string>('');
  const [activeTab, setActiveTab] = useState<'canvas' | 'yaml' | 'simulation'>('canvas');
  const [draggedStageIndex, setDraggedStageIndex] = useState<number | null>(null);
  const [selectedStageId, setSelectedStageId] = useState<string | null>('stage-matrix');
  const [toastMessage, setToastMessage] = useState<string | null>(null);

  // Simulation State
  const [isSimulating, setIsSimulating] = useState<boolean>(false);
  const [simProgress, setSimProgress] = useState<number>(0);
  const [runnerJobs, setRunnerJobs] = useState<RunnerJob[]>([]);
  const [selectedRunnerId, setSelectedRunnerId] = useState<string | null>(null);
  const simulationTimerRef = useRef<number | null>(null);

  // Generate GitHub Actions YAML from Stages
  const generateYamlFromStages = (currentStages: PipelineStage[]): string => {
    const triggerStage = currentStages.find((s) => s.type === 'trigger' && s.enabled);
    const matrixStage = currentStages.find((s) => s.type === 'matrix' && s.enabled);
    const setupStage = currentStages.find((s) => s.type === 'setup' && s.enabled);
    const chaosStage = currentStages.find((s) => s.type === 'chaos' && s.enabled);
    const testStage = currentStages.find((s) => s.type === 'test' && s.enabled);
    const allureStage = currentStages.find((s) => s.type === 'allure' && s.enabled);
    const artifactStage = currentStages.find((s) => s.type === 'artifact' && s.enabled);

    let lines: string[] = [];
    lines.push('name: Enterprise SDET CI/CD Matrix Pipeline');
    lines.push('');
    lines.push('on:');
    if (triggerStage?.config.triggers && triggerStage.config.triggers.length > 0) {
      triggerStage.config.triggers.forEach((trig) => {
        if (trig.includes('push')) lines.push('  push:\n    branches: [main, master]');
        else if (trig.includes('pull_request')) lines.push('  pull_request:\n    branches: [main, master]');
        else if (trig.includes('schedule')) lines.push('  schedule:\n    - cron: "0 2 * * *"');
      });
    } else {
      lines.push('  push:\n    branches: [main]');
    }

    lines.push('');
    lines.push('jobs:');
    lines.push('  sdet-chaos-matrix-test:');
    lines.push('    name: Test Shard (${{ matrix.os }} - ${{ matrix.shard }})');
    lines.push('    runs-on: ${{ matrix.os || \'ubuntu-latest\' }}');

    if (matrixStage) {
      lines.push('    strategy:');
      lines.push('      fail-fast: false');
      lines.push('      matrix:');
      const osList = matrixStage.config.osList || ['ubuntu-latest'];
      const shards = matrixStage.config.shards || ['1/4', '2/4', '3/4', '4/4'];
      lines.push(`        os: [${osList.join(', ')}]`);
      lines.push(`        shard: [${shards.join(', ')}]`);
    }

    lines.push('    steps:');
    if (setupStage) {
      lines.push('      - name: Checkout Code Repository');
      lines.push('        uses: actions/checkout@v4');
      lines.push('');
      lines.push('      - name: Setup Node.js & Dependencies');
      lines.push('        uses: actions/setup-node@v4');
      lines.push('        with:');
      lines.push(`          node-version: ${setupStage.config.nodeVersion || '20.x'}`);
      lines.push('          cache: npm');
      lines.push('');
      lines.push('      - name: Install Project Dependencies');
      lines.push('        run: npm ci');
      lines.push('');
      lines.push('      - name: Install Playwright Browsers & OS Dependencies');
      lines.push('        run: npx playwright install --with-deps chromium');
    }

    if (chaosStage) {
      lines.push('');
      lines.push('      - name: Launch Cherenkov L4/L7 Chaos Fault Proxy');
      lines.push(
        `        run: cherenkov-lings proxy start --latency=${chaosStage.config.chaosLatency || '200ms'} --jitter=${chaosStage.config.chaosJitter || '50ms'} --daemon`
      );
    }

    if (testStage) {
      lines.push('');
      lines.push('      - name: Execute Parallel Sharded Playwright Suite');
      lines.push(`        run: ${testStage.config.testCommand || 'npx playwright test --shard=${{ matrix.shard }}'}`);
      lines.push('        env:');
      lines.push('          CI: true');
      lines.push('          CHAOS_PROXY_URL: http://localhost:8086');
    }

    if (allureStage) {
      lines.push('');
      lines.push('      - name: Aggregate Allure Telemetry Results');
      lines.push('        if: always()');
      lines.push('        run: npx allure generate allure-results --clean -o allure-report');
    }

    if (artifactStage) {
      lines.push('');
      lines.push('      - name: Upload Test Traces & Allure Artifacts');
      lines.push('        if: always()');
      lines.push('        uses: actions/upload-artifact@v4');
      lines.push('        with:');
      lines.push('          name: test-artifacts-${{ matrix.os }}-${{ matrix.shard }}');
      lines.push(`          path: ${artifactStage.config.artifactPath || 'allure-results'}`);
      lines.push('          retention-days: 14');
    }

    return lines.join('\n');
  };

  // Sync YAML whenever stages change
  useEffect(() => {
    setYamlCode(generateYamlFromStages(stages));
  }, [stages]);

  // Parse YAML string back into Stages
  const handleYamlChange = (newYaml: string) => {
    setYamlCode(newYaml);
    // Real parser detecting core SDET blocks
    const hasMatrix = newYaml.includes('matrix:') || newYaml.includes('strategy:');
    const hasArtifact = newYaml.includes('upload-artifact') || newYaml.includes('upload-artifact@v4');
    const hasChaos = newYaml.includes('proxy start') || newYaml.includes('chaos');
    const hasAllure = newYaml.includes('allure') || newYaml.includes('allure-report');
    const hasCheckout = newYaml.includes('actions/checkout');

    setStages((prev) =>
      prev.map((s) => {
        if (s.type === 'matrix') return { ...s, enabled: hasMatrix };
        if (s.type === 'artifact') return { ...s, enabled: hasArtifact };
        if (s.type === 'chaos') return { ...s, enabled: hasChaos };
        if (s.type === 'allure') return { ...s, enabled: hasAllure };
        if (s.type === 'setup') return { ...s, enabled: hasCheckout };
        return s;
      })
    );
  };

  // Enterprise SDET Rule Validator
  const validationResults = useMemo(() => {
    const matrixStage = stages.find((s) => s.type === 'matrix' && s.enabled);
    const artifactStage = stages.find((s) => s.type === 'artifact' && s.enabled);
    const chaosStage = stages.find((s) => s.type === 'chaos' && s.enabled);
    const testStage = stages.find((s) => s.type === 'test' && s.enabled);
    const setupStage = stages.find((s) => s.type === 'setup' && s.enabled);

    const issues: Array<{ id: string; type: 'error' | 'warning' | 'info'; title: string; desc: string; fixAction?: () => void }> = [];

    if (!matrixStage) {
      issues.push({
        id: 'val-no-matrix',
        type: 'error',
        title: 'SDET-R01: Missing Matrix Parallelism (Sharding)',
        desc: 'Single-runner pipeline takes 45+ minutes. Enterprise SDET standards mandate strategy.matrix sharding across runners.',
        fixAction: () => {
          setStages((prev) => prev.map((s) => (s.type === 'matrix' ? { ...s, enabled: true } : s)));
          showToast('Enabled Matrix Strategy with 4x Parallel Shards!');
        }
      });
    }

    if (!artifactStage) {
      issues.push({
        id: 'val-no-artifact',
        type: 'error',
        title: 'SDET-R02: Missing Test Artifact Archival',
        desc: 'Failing tests without trace/video artifacts cannot be triaged post-mortem. Add actions/upload-artifact@v4.',
        fixAction: () => {
          setStages((prev) => prev.map((s) => (s.type === 'artifact' ? { ...s, enabled: true } : s)));
          showToast('Added actions/upload-artifact@v4 step!');
        }
      });
    }

    if (!chaosStage) {
      issues.push({
        id: 'val-no-chaos',
        type: 'warning',
        title: 'SDET-R03: No Chaos Fault Injection Step',
        desc: 'Tests running against a static localhost without chaos latency risk false green approvals before production load.',
        fixAction: () => {
          setStages((prev) => prev.map((s) => (s.type === 'chaos' ? { ...s, enabled: true } : s)));
          showToast('Enabled Cherenkov L4/L7 Chaos Proxy stage!');
        }
      });
    }

    if (!setupStage) {
      issues.push({
        id: 'val-no-setup',
        type: 'error',
        title: 'SDET-R04: Missing Repository Checkout Step',
        desc: 'Pipeline does not clone source repository (actions/checkout@v4 missing).'
      });
    }

    if (!testStage) {
      issues.push({
        id: 'val-no-test',
        type: 'error',
        title: 'SDET-R05: Missing Test Execution Stage',
        desc: 'Pipeline does not execute test runner or playwright suite.',
        fixAction: () => {
          setStages((prev) => prev.map((s) => (s.type === 'test' ? { ...s, enabled: true } : s)));
          showToast('Enabled Distributed Playwright Execution stage!');
        }
      });
    }

    const isCompliant = issues.filter((i) => i.type === 'error').length === 0;

    return {
      isCompliant,
      issues,
      errorCount: issues.filter((i) => i.type === 'error').length,
      warningCount: issues.filter((i) => i.type === 'warning').length
    };
  }, [stages]);

  const showToast = (msg: string) => {
    setToastMessage(msg);
    setTimeout(() => setToastMessage(null), 3500);
  };

  // Drag and Drop Handlers for Visual Stages
  const handleDragStart = (index: number) => {
    setDraggedStageIndex(index);
  };

  const handleDragOver = (e: React.DragEvent, targetIndex: number) => {
    e.preventDefault();
    if (draggedStageIndex === null || draggedStageIndex === targetIndex) return;

    const newStages = [...stages];
    const draggedItem = newStages.splice(draggedStageIndex, 1)[0];
    newStages.splice(targetIndex, 0, draggedItem);
    setDraggedStageIndex(targetIndex);
    setStages(newStages);
  };

  const handleDragEnd = () => {
    setDraggedStageIndex(null);
  };

  // Move stage up/down
  const moveStage = (index: number, direction: 'up' | 'down') => {
    const target = direction === 'up' ? index - 1 : index + 1;
    if (target < 0 || target >= stages.length) return;
    const newStages = [...stages];
    const temp = newStages[index];
    newStages[index] = newStages[target];
    newStages[target] = temp;
    setStages(newStages);
  };

  const toggleStage = (id: string) => {
    setStages((prev) => prev.map((s) => (s.id === id ? { ...s, enabled: !s.enabled } : s)));
  };

  // Run CI Pipeline Simulation
  const handleStartSimulation = () => {
    if (isSimulating) return;
    setActiveTab('simulation');
    setIsSimulating(true);
    setSimProgress(0);

    const matrixStage = stages.find((s) => s.type === 'matrix' && s.enabled);
    const osList = matrixStage?.config.osList || ['ubuntu-latest'];
    const shards = matrixStage?.config.shards || ['1/2', '2/2'];

    // Generate Runners based on matrix dimensions
    const runners: RunnerJob[] = [];
    let runnerCount = 1;
    osList.forEach((os) => {
      shards.forEach((shard) => {
        runners.push({
          id: `runner-${runnerCount++}`,
          runnerName: `Matrix Runner #${runnerCount - 1}`,
          os,
          shard,
          status: 'running',
          currentStepIndex: 0,
          steps: [
            { name: 'actions/checkout@v4', status: 'pending', durationMs: 450 },
            { name: 'actions/setup-node@v4 (npm ci)', status: 'pending', durationMs: 1200 },
            { name: 'L4/L7 Chaos Proxy Daemon (200ms)', status: 'pending', durationMs: 300 },
            { name: `Playwright Test Shard (${shard})`, status: 'pending', durationMs: 2400 },
            { name: 'Generate Allure Telemetry', status: 'pending', durationMs: 600 },
            { name: 'Upload Artifacts (traces/report)', status: 'pending', durationMs: 500 }
          ],
          logs: [
            `[${new Date().toISOString()}] Initializing container runner on ${os}...`,
            `[${new Date().toISOString()}] Provisioning virtual environment (Shard ${shard})...`
          ]
        });
      });
    });

    setRunnerJobs(runners);
    setSelectedRunnerId(runners[0].id);

    let progressVal = 0;
    let stepIndex = 0;

    const interval = window.setInterval(() => {
      progressVal += 16.6;
      setSimProgress(Math.min(100, Math.round(progressVal)));

      setRunnerJobs((prevRunners) => {
        return prevRunners.map((runner) => {
          const updatedSteps = runner.steps.map((step, idx) => {
            if (idx < stepIndex) return { ...step, status: 'success' as const };
            if (idx === stepIndex) return { ...step, status: 'running' as const };
            return step;
          });

          const currentStep = runner.steps[stepIndex];
          const newLogs = [...runner.logs];
          if (currentStep) {
            newLogs.push(`[${new Date().toISOString()}] Step: ${currentStep.name} -> EXECUTION SUCCESS (${currentStep.durationMs}ms)`);
            if (currentStep.name.includes('Playwright')) {
              newLogs.push(`[${new Date().toISOString()}]   ✓ 15 tests passed on Shard ${runner.shard} against Chaos Proxy`);
            }
          }

          const isFinal = stepIndex >= runner.steps.length - 1;

          return {
            ...runner,
            steps: updatedSteps,
            currentStepIndex: stepIndex,
            status: isFinal ? ('success' as const) : ('running' as const),
            logs: newLogs
          };
        });
      });

      stepIndex++;

      if (stepIndex >= 6) {
        window.clearInterval(interval);
        setIsSimulating(false);
        setSimProgress(100);
        showToast('🎉 All Parallel Matrix Runners Finished Successfully (0 Failures)');
      }
    }, 1100);

    simulationTimerRef.current = interval;
  };

  const selectedRunner = useMemo(() => {
    return runnerJobs.find((r) => r.id === selectedRunnerId) || runnerJobs[0];
  }, [runnerJobs, selectedRunnerId]);

  return (
    <div className="page-container pipeline-builder-page">
      {/* Toast */}
      {toastMessage && (
        <div className="pipeline-toast" role="alert">
          <span className="toast-icon">🚀</span>
          <span>{toastMessage}</span>
        </div>
      )}

      {/* Header Banner */}
      <div className="pipeline-header">
        <div className="header-left">
          <div className="badge-row">
            <span className="badge info">R2: CI/CD Pipeline Simulator</span>
            <span className="badge purple">Bidirectional YAML Sync</span>
            <span className={`badge ${validationResults.isCompliant ? 'green' : 'danger'}`}>
              {validationResults.isCompliant ? 'SDET Compliant' : `${validationResults.errorCount} SDET Violations`}
            </span>
          </div>
          <h1 className="page-title">CI/CD Pipeline Simulator & Workflow Builder</h1>
          <p className="page-description">
            Design parallel test matrix pipelines, enforce enterprise SDET validation rules, and simulate high-throughput
            virtual GitHub Actions runners.
          </p>
        </div>

        <div className="header-actions">
          <button className="primary-btn run-sim-btn" onClick={handleStartSimulation} disabled={isSimulating}>
            <span className="btn-icon">▶</span>
            <span>{isSimulating ? 'Simulating Matrix...' : 'Run Simulation'}</span>
          </button>
        </div>
      </div>

      {/* SDET Validation Alert Banner */}
      {validationResults.issues.length > 0 && (
        <div className={`validation-alert-banner ${validationResults.isCompliant ? 'warning' : 'danger'}`}>
          <div className="alert-header">
            <span className="alert-icon">{validationResults.isCompliant ? '⚠️' : '🚫'}</span>
            <div className="alert-title-wrap">
              <h4>Enterprise SDET Architecture Validation</h4>
              <p>
                {validationResults.isCompliant
                  ? 'Pipeline is valid, but optional enterprise enhancements were identified:'
                  : 'Pipeline fails strict SDET enterprise gates. Resolve the following violations to unblock CI:'}
              </p>
            </div>
          </div>

          <div className="validation-issues-list">
            {validationResults.issues.map((issue) => (
              <div key={issue.id} className={`issue-card ${issue.type}`}>
                <div className="issue-content">
                  <span className="issue-title">{issue.title}</span>
                  <p className="issue-desc">{issue.desc}</p>
                </div>
                {issue.fixAction && (
                  <button className="secondary-btn fix-issue-btn" onClick={issue.fixAction}>
                    ⚡ Auto-Fix
                  </button>
                )}
              </div>
            ))}
          </div>
        </div>
      )}

      {/* Mode Navigation Tabs */}
      <div className="pipeline-tabs-nav">
        <button
          className={`tab-btn ${activeTab === 'canvas' ? 'active' : ''}`}
          onClick={() => setActiveTab('canvas')}
        >
          <span className="tab-icon">🎨</span>
          <span>Visual Workflow Canvas ({stages.filter((s) => s.enabled).length} Active Stages)</span>
        </button>
        <button className={`tab-btn ${activeTab === 'yaml' ? 'active' : ''}`} onClick={() => setActiveTab('yaml')}>
          <span className="tab-icon">📄</span>
          <span>Live GitHub Actions YAML Editor</span>
        </button>
        <button
          className={`tab-btn ${activeTab === 'simulation' ? 'active' : ''}`}
          onClick={() => setActiveTab('simulation')}
        >
          <span className="tab-icon">⚡</span>
          <span>Virtual Runner Simulation {isSimulating && <span className="sim-pulse-dot"></span>}</span>
        </button>
      </div>

      {/* TAB 1: VISUAL DRAG-AND-DROP WORKFLOW CANVAS */}
      {activeTab === 'canvas' && (
        <div className="canvas-tab-view">
          <div className="canvas-main-grid">
            {/* Left: Stages Sequence Canvas */}
            <div className="canvas-stages-column">
              <div className="stages-column-header">
                <h3>Pipeline Execution Stages</h3>
                <span className="stages-subtitle">Drag to reorder stages or toggle them on/off</span>
              </div>

              <div className="stages-drag-list">
                {stages.map((stage, idx) => (
                  <div
                    key={stage.id}
                    className={`stage-card ${stage.enabled ? 'enabled' : 'disabled'} ${
                      selectedStageId === stage.id ? 'selected' : ''
                    }`}
                    draggable
                    onDragStart={() => handleDragStart(idx)}
                    onDragOver={(e) => handleDragOver(e, idx)}
                    onDragEnd={handleDragEnd}
                    onClick={() => setSelectedStageId(stage.id)}
                  >
                    <div className="stage-card-left">
                      <span className="drag-handle" title="Drag to reorder">
                        ⋮⋮
                      </span>
                      <span className="stage-index">{idx + 1}</span>
                      <div className="stage-type-icon">
                        {stage.type === 'trigger' && '⚡'}
                        {stage.type === 'setup' && '⚙️'}
                        {stage.type === 'matrix' && '🔀'}
                        {stage.type === 'chaos' && '🌀'}
                        {stage.type === 'test' && '🧪'}
                        {stage.type === 'allure' && '📊'}
                        {stage.type === 'artifact' && '📦'}
                      </div>
                      <div className="stage-info">
                        <span className="stage-title">{stage.title}</span>
                        <span className="stage-type-tag">{stage.type.toUpperCase()}</span>
                      </div>
                    </div>

                    <div className="stage-card-right">
                      <div className="reorder-btns">
                        <button
                          className="reorder-btn"
                          disabled={idx === 0}
                          onClick={(e) => {
                            e.stopPropagation();
                            moveStage(idx, 'up');
                          }}
                          title="Move up"
                        >
                          ▲
                        </button>
                        <button
                          className="reorder-btn"
                          disabled={idx === stages.length - 1}
                          onClick={(e) => {
                            e.stopPropagation();
                            moveStage(idx, 'down');
                          }}
                          title="Move down"
                        >
                          ▼
                        </button>
                      </div>
                      <label
                        className="stage-toggle-switch"
                        onClick={(e) => e.stopPropagation()}
                        title="Enable/Disable Stage"
                      >
                        <input
                          type="checkbox"
                          checked={stage.enabled}
                          onChange={() => toggleStage(stage.id)}
                        />
                        <span className="toggle-slider"></span>
                      </label>
                    </div>
                  </div>
                ))}
              </div>
            </div>

            {/* Right: Stage Configuration Inspector */}
            <div className="canvas-inspector-column">
              <div className="inspector-card">
                <div className="inspector-header">
                  <span className="inspector-icon">⚙️</span>
                  <h3>Stage Inspector & SDET Tuning</h3>
                </div>

                {selectedStageId ? (
                  (() => {
                    const stage = stages.find((s) => s.id === selectedStageId);
                    if (!stage) return null;

                    return (
                      <div className="inspector-form">
                        <div className="form-group">
                          <label className="form-label">Stage Name:</label>
                          <input
                            type="text"
                            className="text-input"
                            value={stage.title}
                            onChange={(e) => {
                              const val = e.target.value;
                              setStages((prev) =>
                                prev.map((s) => (s.id === stage.id ? { ...s, title: val } : s))
                              );
                            }}
                          />
                        </div>

                        {/* Stage Specific Controls */}
                        {stage.type === 'matrix' && (
                          <div className="stage-specific-config">
                            <h4 className="config-subheading">Parallel Matrix Configuration</h4>
                            <div className="config-box">
                              <label className="form-label">Operating Systems Matrix:</label>
                              <div className="checkbox-pills">
                                {['ubuntu-latest', 'windows-latest', 'macos-latest'].map((os) => {
                                  const currentOs = stage.config.osList || [];
                                  const isChecked = currentOs.includes(os);
                                  return (
                                    <label key={os} className={`pill-check ${isChecked ? 'active' : ''}`}>
                                      <input
                                        type="checkbox"
                                        checked={isChecked}
                                        onChange={() => {
                                          const next = isChecked
                                            ? currentOs.filter((o) => o !== os)
                                            : [...currentOs, os];
                                          setStages((prev) =>
                                            prev.map((s) =>
                                              s.id === stage.id
                                                ? { ...s, config: { ...s.config, osList: next } }
                                                : s
                                            )
                                          );
                                        }}
                                      />
                                      <span>{os}</span>
                                    </label>
                                  );
                                })}
                              </div>

                              <label className="form-label" style={{ marginTop: '16px' }}>
                                Parallel Shards Strategy:
                              </label>
                              <div className="radio-pills">
                                {['2 Shards (1/2, 2/2)', '4 Shards (1/4 .. 4/4)', '8 Shards (1/8 .. 8/8)'].map(
                                  (opt, optIdx) => {
                                    const shardsArr =
                                      optIdx === 0
                                        ? ['1/2', '2/2']
                                        : optIdx === 1
                                        ? ['1/4', '2/4', '3/4', '4/4']
                                        : ['1/8', '2/8', '3/8', '4/8', '5/8', '6/8', '7/8', '8/8'];
                                    const isSelected = stage.config.shards?.length === shardsArr.length;

                                    return (
                                      <button
                                        key={opt}
                                        type="button"
                                        className={`strategy-opt-btn ${isSelected ? 'active' : ''}`}
                                        onClick={() => {
                                          setStages((prev) =>
                                            prev.map((s) =>
                                              s.id === stage.id
                                                ? { ...s, config: { ...s.config, shards: shardsArr } }
                                                : s
                                            )
                                          );
                                        }}
                                      >
                                        <span>{opt}</span>
                                      </button>
                                    );
                                  }
                                )}
                              </div>
                            </div>
                          </div>
                        )}

                        {stage.type === 'chaos' && (
                          <div className="stage-specific-config">
                            <h4 className="config-subheading">Chaos Proxy Parameters</h4>
                            <div className="form-row-2">
                              <div>
                                <label className="form-label">Injected Latency:</label>
                                <input
                                  type="text"
                                  className="text-input"
                                  value={stage.config.chaosLatency || '200ms'}
                                  onChange={(e) => {
                                    const val = e.target.value;
                                    setStages((prev) =>
                                      prev.map((s) =>
                                        s.id === stage.id
                                          ? { ...s, config: { ...s.config, chaosLatency: val } }
                                          : s
                                      )
                                    );
                                  }}
                                />
                              </div>
                              <div>
                                <label className="form-label">Latency Jitter:</label>
                                <input
                                  type="text"
                                  className="text-input"
                                  value={stage.config.chaosJitter || '50ms'}
                                  onChange={(e) => {
                                    const val = e.target.value;
                                    setStages((prev) =>
                                      prev.map((s) =>
                                        s.id === stage.id
                                          ? { ...s, config: { ...s.config, chaosJitter: val } }
                                          : s
                                      )
                                    );
                                  }}
                                />
                              </div>
                            </div>
                          </div>
                        )}

                        {stage.type === 'test' && (
                          <div className="stage-specific-config">
                            <label className="form-label">Playwright Test Command:</label>
                            <input
                              type="text"
                              className="text-input"
                              value={stage.config.testCommand || ''}
                              onChange={(e) => {
                                const val = e.target.value;
                                setStages((prev) =>
                                  prev.map((s) =>
                                    s.id === stage.id
                                      ? { ...s, config: { ...s.config, testCommand: val } }
                                      : s
                                  )
                                );
                              }}
                            />
                          </div>
                        )}

                        {stage.type === 'artifact' && (
                          <div className="stage-specific-config">
                            <label className="form-label">Artifact Archival Paths:</label>
                            <input
                              type="text"
                              className="text-input"
                              value={stage.config.artifactPath || ''}
                              onChange={(e) => {
                                const val = e.target.value;
                                setStages((prev) =>
                                  prev.map((s) =>
                                    s.id === stage.id
                                      ? { ...s, config: { ...s.config, artifactPath: val } }
                                      : s
                                  )
                                );
                              }}
                            />
                          </div>
                        )}

                        <div className="inspector-actions">
                          <button
                            className="secondary-btn toggle-btn"
                            onClick={() => toggleStage(stage.id)}
                          >
                            {stage.enabled ? '⏸ Disable Stage' : '▶ Enable Stage'}
                          </button>
                        </div>
                      </div>
                    );
                  })()
                ) : (
                  <div className="no-selection-prompt">
                    <p>Select any stage from the sequence list on the left to configure its properties.</p>
                  </div>
                )}
              </div>
            </div>
          </div>
        </div>
      )}

      {/* TAB 2: LIVE BIDIRECTIONAL YAML CODE EDITOR */}
      {activeTab === 'yaml' && (
        <div className="yaml-tab-view">
          <div className="yaml-editor-card">
            <div className="yaml-topbar">
              <div className="yaml-meta">
                <span className="yaml-file-name">.github/workflows/sdet-matrix-pipeline.yml</span>
                <span className="yaml-sync-badge">⚡ Bidirectional Sync Active</span>
              </div>
              <div className="yaml-actions">
                <button
                  className="secondary-btn"
                  onClick={() => {
                    navigator.clipboard.writeText(yamlCode);
                    showToast('YAML copied to clipboard!');
                  }}
                >
                  📋 Copy YAML
                </button>
                <button
                  className="secondary-btn"
                  onClick={() => setYamlCode(generateYamlFromStages(stages))}
                >
                  ↺ Regenerate
                </button>
              </div>
            </div>

            <div className="yaml-editor-body">
              <textarea
                className="yaml-textarea"
                value={yamlCode}
                onChange={(e) => handleYamlChange(e.target.value)}
                spellCheck={false}
                aria-label="GitHub Actions YAML Editor"
              />
            </div>
          </div>
        </div>
      )}

      {/* TAB 3: RUN SIMULATION PANEL */}
      {activeTab === 'simulation' && (
        <div className="simulation-tab-view">
          {/* Progress Overview Bar */}
          <div className="sim-progress-card">
            <div className="sim-progress-header">
              <div className="sim-status-title">
                <h3>
                  {isSimulating
                    ? '⚡ Parallel Matrix Simulation in Progress...'
                    : runnerJobs.length > 0
                    ? '✅ Simulation Run Completed'
                    : 'Ready to Simulate Matrix Execution'}
                </h3>
                <span className="sim-sub">
                  {runnerJobs.length > 0
                    ? `Executing across ${runnerJobs.length} parallel matrix runner containers`
                    : 'Click "Start Simulation" to simulate parallel runners and step telemetry.'}
                </span>
              </div>

              {!isSimulating && (
                <button className="primary-btn restart-sim-btn" onClick={handleStartSimulation}>
                  {runnerJobs.length > 0 ? '↺ Rerun Simulation' : '▶ Start Simulation'}
                </button>
              )}
            </div>

            <div className="sim-progress-track">
              <div
                className="sim-progress-fill"
                style={{
                  width: `${simProgress}%`,
                  background: isSimulating
                    ? 'linear-gradient(90deg, #38bdf8, #0284c7)'
                    : 'linear-gradient(90deg, #4ade80, #22c55e)'
                }}
              ></div>
            </div>
            <div className="sim-progress-meta">
              <span>Progress: {simProgress}%</span>
              <span>Estimated Speedup: 3.8x faster than sequential single runner</span>
            </div>
          </div>

          {/* Runners Grid and Logs Panel */}
          {runnerJobs.length > 0 && (
            <div className="sim-runner-grid">
              {/* Left: Runner Cards List */}
              <div className="runner-cards-column">
                <h4 className="column-title">Parallel Matrix Runners</h4>
                <div className="runner-cards-list">
                  {runnerJobs.map((runner) => {
                    const isSelected = selectedRunnerId === runner.id;
                    return (
                      <div
                        key={runner.id}
                        className={`runner-card ${runner.status} ${isSelected ? 'active' : ''}`}
                        onClick={() => setSelectedRunnerId(runner.id)}
                      >
                        <div className="runner-card-header">
                          <span className="runner-name">{runner.runnerName}</span>
                          <span className={`runner-status-tag ${runner.status}`}>
                            {runner.status.toUpperCase()}
                          </span>
                        </div>

                        <div className="runner-badges-row">
                          <span className="os-badge">{runner.os}</span>
                          <span className="shard-badge">Shard {runner.shard}</span>
                        </div>

                        <div className="runner-step-mini-indicators">
                          {runner.steps.map((st, sIdx) => (
                            <span
                              key={sIdx}
                              className={`step-dot ${st.status}`}
                              title={`${st.name} (${st.status})`}
                            ></span>
                          ))}
                        </div>
                      </div>
                    );
                  })}
                </div>
              </div>

              {/* Right: Live Terminal Logs for Selected Runner */}
              <div className="runner-logs-column">
                <div className="terminal-card">
                  <div className="terminal-header">
                    <div className="terminal-dots">
                      <span className="dot red"></span>
                      <span className="dot yellow"></span>
                      <span className="dot green"></span>
                    </div>
                    <span className="terminal-title">
                      Logs: {selectedRunner?.runnerName} ({selectedRunner?.os} - Shard {selectedRunner?.shard})
                    </span>
                    <span className="terminal-status">{selectedRunner?.status.toUpperCase()}</span>
                  </div>

                  {/* Step Breakdown Table */}
                  <div className="steps-progress-table">
                    {selectedRunner?.steps.map((step, idx) => (
                      <div key={idx} className={`step-row ${step.status}`}>
                        <div className="step-name-wrap">
                          <span className="step-icon">
                            {step.status === 'success' && '✓'}
                            {step.status === 'running' && '⏳'}
                            {step.status === 'pending' && '○'}
                            {step.status === 'failed' && '✗'}
                          </span>
                          <span className="step-label">{step.name}</span>
                        </div>
                        <span className="step-duration">
                          {step.status === 'success' ? `${step.durationMs}ms` : step.status === 'running' ? 'running...' : 'queued'}
                        </span>
                      </div>
                    ))}
                  </div>

                  {/* Monospace Console Logs */}
                  <div className="terminal-log-output">
                    <pre>
                      <code>{selectedRunner?.logs.join('\n')}</code>
                    </pre>
                  </div>
                </div>
              </div>
            </div>
          )}
        </div>
      )}
    </div>
  );
};
