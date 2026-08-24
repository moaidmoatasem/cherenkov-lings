#!/usr/bin/env node
/**
 * cherenkov-lings - Node.js IPC Worker
 *
 * Communicates with the Rust CLI runner over Stdio Line-Delimited JSON (NDJSON).
 * Executes Playwright test suites with injected chaos headers and returns structured JSON reports.
 */

const readline = require('readline');
const { spawn } = require('child_process');
const path = require('path');
const fs = require('fs');

const isWindows = process.platform === 'win32';
const npxCmd = isWindows ? 'npx.cmd' : 'npx';

/**
 * Strip ANSI color codes from error strings
 */
function stripAnsi(str) {
  if (!str || typeof str !== 'string') return '';
  return str.replace(/\u001b\[[0-9;]*[a-zA-Z]/g, '');
}

/**
 * Extract error message from Playwright JSON report
 */
function extractErrorMessage(parsedJson, stderr) {
  if (parsedJson) {
    if (Array.isArray(parsedJson.errors) && parsedJson.errors.length > 0) {
      const msgs = parsedJson.errors.map(e => stripAnsi(e.message || e.value || JSON.stringify(e))).filter(Boolean);
      if (msgs.length > 0) return msgs.join('\n');
    }

    if (Array.isArray(parsedJson.suites)) {
      const suiteErrors = [];
      function traverseSuite(suite) {
        if (suite.specs) {
          for (const spec of suite.specs) {
            if (spec.tests) {
              for (const test of spec.tests) {
                if (test.results) {
                  for (const res of test.results) {
                    if (res.error && res.error.message) {
                      suiteErrors.push(stripAnsi(res.error.message));
                    }
                    if (Array.isArray(res.errors)) {
                      for (const err of res.errors) {
                        if (err.message) suiteErrors.push(stripAnsi(err.message));
                      }
                    }
                  }
                }
              }
            }
          }
        }
        if (suite.suites) {
          for (const s of suite.suites) traverseSuite(s);
        }
      }
      for (const s of parsedJson.suites) traverseSuite(s);
      if (suiteErrors.length > 0) {
        return suiteErrors.join('\n');
      }
    }
  }

  if (stderr && stderr.trim()) {
    return stripAnsi(stderr.trim());
  }

  return 'Test failed with non-zero exit code';
}

/**
 * Run a single Playwright test iteration
 */
function runSingleTest(filePath, chaosHeader, timeoutMs) {
  return new Promise((resolve) => {
    const startTime = Date.now();
    const env = {
      ...process.env,
      PW_CHAOS_HEADER: chaosHeader || '',
      PW_CHAOS: chaosHeader || '',
      X_CHAOS: chaosHeader || '',
      FORCE_COLOR: '0',
    };

    const posixPath = path.relative(process.cwd(), filePath).replace(/\\/g, '/');
    const args = ['playwright', 'test', posixPath, '--reporter=json'];

    const child = spawn(npxCmd, args, {
      env,
      shell: isWindows,
      cwd: process.cwd(),
      stdio: ['ignore', 'pipe', 'pipe'],
    });

    let stdout = '';
    let stderr = '';
    let timedOut = false;

    const timer = setTimeout(() => {
      timedOut = true;
      try {
        if (isWindows && child.pid) {
          spawn('taskkill', ['/pid', child.pid.toString(), '/T', '/F']);
        } else {
          child.kill('SIGKILL');
        }
      } catch {
        // ignore kill error
      }
    }, timeoutMs);

    child.stdout.on('data', (data) => {
      stdout += data.toString();
    });

    child.stderr.on('data', (data) => {
      stderr += data.toString();
    });

    child.on('error', (err) => {
      clearTimeout(timer);
      const duration = Date.now() - startTime;
      resolve({
        passed: false,
        duration_ms: duration,
        error: `Failed to spawn playwright: ${err.message}`,
      });
    });

    child.on('close', (code) => {
      clearTimeout(timer);
      const duration = Date.now() - startTime;

      if (timedOut) {
        return resolve({
          passed: false,
          duration_ms: duration,
          error: `Test timed out after ${timeoutMs}ms`,
        });
      }

      let parsed = null;
      try {
        const jsonStart = stdout.indexOf('{');
        const jsonEnd = stdout.lastIndexOf('}');
        if (jsonStart !== -1 && jsonEnd !== -1 && jsonEnd >= jsonStart) {
          const jsonStr = stdout.slice(jsonStart, jsonEnd + 1);
          parsed = JSON.parse(jsonStr);
        }
      } catch {
        // JSON parsing failed
      }

      let passed = false;
      let error = null;

      if (parsed && parsed.stats) {
        const unexpected = parsed.stats.unexpected || 0;
        const expected = parsed.stats.expected || 0;
        if (code === 0 && unexpected === 0 && (expected > 0 || (parsed.suites && parsed.suites.length > 0))) {
          passed = true;
        } else {
          passed = false;
          error = extractErrorMessage(parsed, stderr);
        }
      } else {
        passed = (code === 0);
        if (!passed) {
          error = extractErrorMessage(null, stderr || stdout);
        }
      }

      resolve({
        passed,
        duration_ms: duration,
        error: passed ? null : error,
      });
    });
  });
}

/**
 * Handle a run_drill request
 */
async function handleRunDrill(req) {
  const { id, file, chaos, iterations = 1, timeout_ms = 30000 } = req;
  const resolvedPath = path.isAbsolute(file) ? file : path.resolve(process.cwd(), file);

  if (!fs.existsSync(resolvedPath)) {
    return {
      id,
      ok: false,
      passed: false,
      iterations: iterations || 1,
      passed_iterations: 0,
      failed_iterations: iterations || 1,
      total_duration_ms: 0,
      runs: [],
      error: `Exercise file does not exist: ${file}`,
    };
  }

  const numIterations = Math.max(1, parseInt(iterations, 10) || 1);
  const perIterationTimeout = Math.max(2000, Math.floor((timeout_ms || 30000) / numIterations));

  const totalStartTime = Date.now();
  const runs = [];
  let passedCount = 0;

  for (let i = 1; i <= numIterations; i++) {
    const result = await runSingleTest(resolvedPath, chaos, perIterationTimeout);
    runs.push({
      iteration: i,
      passed: result.passed,
      duration_ms: result.duration_ms,
      error: result.error,
    });
    if (result.passed) {
      passedCount++;
    }
  }

  const totalDuration = Date.now() - totalStartTime;
  const failedCount = numIterations - passedCount;
  const passed = passedCount === numIterations;

  return {
    id,
    ok: true,
    passed,
    iterations: numIterations,
    passed_iterations: passedCount,
    failed_iterations: failedCount,
    total_duration_ms: totalDuration,
    runs,
    error: null,
  };
}

/**
 * Main NDJSON message processing loop
 */
function main() {
  const rl = readline.createInterface({
    input: process.stdin,
    output: process.stdout,
    terminal: false,
  });

  // Write a single NDJSON line to stdout
  function sendResponse(obj) {
    process.stdout.write(JSON.stringify(obj) + '\n');
  }

  // Queue to handle requests serially
  let processingQueue = Promise.resolve();

  rl.on('line', (line) => {
    const trimmed = line.trim();
    if (!trimmed) return;

    processingQueue = processingQueue.then(async () => {
      let req;
      try {
        req = JSON.parse(trimmed);
        if (!req || typeof req !== 'object') {
          throw new Error('NDJSON payload must be a JSON object');
        }
      } catch (err) {
        sendResponse({
          id: 'unknown',
          ok: false,
          error: `Invalid NDJSON request: ${err.message}`,
        });
        return;
      }

      try {
        if (req.action === 'ping') {
          sendResponse({ id: req.id, ok: true, action: 'pong' });
        } else if (req.action === 'shutdown' || req.action === 'exit') {
          sendResponse({ id: req.id, ok: true, action: 'shutdown' });
          process.exit(0);
        } else if (req.action === 'run_drill') {
          const res = await handleRunDrill(req);
          sendResponse(res);
        } else {
          sendResponse({
            id: req.id,
            ok: false,
            error: `Unsupported action: ${req.action}`,
          });
        }
      } catch (err) {
        sendResponse({
          id: (req && req.id) || 'unknown',
          ok: false,
          passed: false,
          error: `Internal worker error: ${err.message || String(err)}`,
        });
      }
    });
  });

  rl.on('close', () => {
    process.exit(0);
  });
}

if (require.main === module) {
  main();
}
