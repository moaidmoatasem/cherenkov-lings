#!/usr/bin/env node
/**
 * k6 Runner Bootstrap Script for cherenkov-lings
 *
 * Executes k6 load testing scripts, manages summary export generation,
 * and reports threshold metrics and execution results.
 *
 * Usage:
 *   node k6_runner.js <script-path> [--summary-export=summary.json] [--iterations=N]
 */

const { spawn } = require('child_process');
const fs = require('fs');
const path = require('path');

function parseArgs() {
  const args = process.argv.slice(2);
  let scriptPath = null;
  let summaryExport = 'summary.json';
  let iterations = 1;
  const extraArgs = [];

  for (let i = 0; i < args.length; i++) {
    const arg = args[i];
    if (arg.startsWith('--summary-export=')) {
      summaryExport = arg.split('=')[1];
    } else if (arg === '--summary-export' && i + 1 < args.length) {
      summaryExport = args[++i];
    } else if (arg.startsWith('--iterations=')) {
      iterations = parseInt(arg.split('=')[1], 10) || 1;
    } else if (!arg.startsWith('-') && !scriptPath) {
      scriptPath = arg;
    } else {
      extraArgs.push(arg);
    }
  }

  return { scriptPath, summaryExport, iterations, extraArgs };
}

function runK6(scriptPath, summaryExport, extraArgs) {
  return new Promise((resolve, reject) => {
    const resolvedScript = path.resolve(process.cwd(), scriptPath);
    if (!fs.existsSync(resolvedScript)) {
      return reject(new Error(`k6 test script not found: ${resolvedScript}`));
    }

    const resolvedSummary = path.resolve(process.cwd(), summaryExport);
    const summaryDir = path.dirname(resolvedSummary);
    if (!fs.existsSync(summaryDir)) {
      fs.mkdirSync(summaryDir, { recursive: true });
    }

    const k6Binary = process.platform === 'win32' ? 'k6.exe' : 'k6';
    const k6Args = [
      'run',
      `--summary-export=${resolvedSummary}`,
      ...extraArgs,
      resolvedScript,
    ];

    console.log(`[k6_runner] Executing: k6 run --summary-export=${summaryExport} ${scriptPath}`);

    const env = {
      ...process.env,
      X_CHAOS: process.env.X_CHAOS || process.env.PW_CHAOS_HEADER || '',
      CHAOS_DIRECTIVES: process.env.CHAOS_DIRECTIVES || '',
    };

    const proc = spawn(k6Binary, k6Args, {
      stdio: 'inherit',
      env,
      shell: process.platform === 'win32',
    });

    proc.on('error', (err) => {
      if (err.code === 'ENOENT') {
        console.error(`[k6_runner] Error: 'k6' binary not found in PATH.`);
        console.error(`[k6_runner] Please install k6: https://grafana.com/docs/k6/latest/get-started/installation/`);
      }
      reject(err);
    });

    proc.on('close', (code) => {
      if (fs.existsSync(resolvedSummary)) {
        try {
          const rawSummary = fs.readFileSync(resolvedSummary, 'utf8');
          const summary = JSON.parse(rawSummary);
          const metrics = summary.metrics || {};
          let allPassed = true;
          const failedThresholds = [];

          for (const [metricName, metricData] of Object.entries(metrics)) {
            if (metricData.thresholds) {
              for (const [threshName, threshResult] of Object.entries(metricData.thresholds)) {
                if (!threshResult.ok) {
                  allPassed = false;
                  failedThresholds.push(`${metricName}: threshold '${threshName}' failed`);
                }
              }
            }
          }

          if (failedThresholds.length > 0) {
            console.log(`[k6_runner] Failed thresholds detected:`);
            failedThresholds.forEach((t) => console.log(`  - ${t}`));
          } else {
            console.log(`[k6_runner] All threshold assertions passed.`);
          }
        } catch (parseErr) {
          console.warn(`[k6_runner] Warning: Could not parse summary JSON: ${parseErr.message}`);
        }
      }

      resolve(code);
    });
  });
}

async function main() {
  const { scriptPath, summaryExport, iterations, extraArgs } = parseArgs();

  if (!scriptPath) {
    console.error('Usage: node k6_runner.js <script-path> [--summary-export=summary.json]');
    process.exit(1);
  }

  let lastCode = 0;
  for (let iter = 1; iter <= iterations; iter++) {
    if (iterations > 1) {
      console.log(`\n--- Iteration ${iter}/${iterations} ---`);
    }
    try {
      const exportFile = iterations > 1 ? summaryExport.replace(/\.json$/, `_${iter}.json`) : summaryExport;
      lastCode = await runK6(scriptPath, exportFile, extraArgs);
      if (lastCode !== 0) {
        break;
      }
    } catch (err) {
      console.error(`[k6_runner] Execution failed: ${err.message}`);
      process.exit(1);
    }
  }

  process.exit(lastCode);
}

if (require.main === module) {
  main();
}

module.exports = { parseArgs, runK6 };
