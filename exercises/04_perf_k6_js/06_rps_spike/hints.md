# Hints: Drill 06 - Checkout RPS Spike (Open Workload Model)

## Hint 1 (Architectural Nudge)
The starter script has an empty `options` object, which means k6 falls back to one virtual user executing one iteration — a functional smoke test wearing a load testing tool's clothes. Before you reach for `stages`, decide which workload model you actually want. VU-based stages are a *closed* model: each VU waits for its response before issuing the next request, so when the server slows down, your offered load automatically slows down with it. Real checkout traffic does not do that. Shoppers keep pressing the button whether or not your backend is keeping up.

## Hint 2 (API Pattern)
k6 exposes the *open* model through the `constant-arrival-rate` and `ramping-arrival-rate` executors, configured under `options.scenarios`. These hold a request rate rather than a concurrency level: you declare `rate` and `timeUnit` (e.g. 200 iterations per `1s`), plus `preAllocatedVUs` and `maxVUs` for the worker pool k6 draws from. If the system under test degrades, k6 recruits more VUs to sustain the rate and reports `dropped_iterations` when even `maxVUs` cannot keep up — that metric is the signal that your service fell behind the arrival rate. Pair the scenario with a `thresholds` block so the run exits non-zero on breach, and use `check()` for per-response correctness, remembering that a failed `check` records the failure but does not fail the run on its own.

## Hint 3 (Code Diff)
Replace the empty options with a ramping arrival rate and real thresholds:

    export const options = {
      scenarios: {
        checkout_spike: {
          executor: 'ramping-arrival-rate',
          startRate: 10,
          timeUnit: '1s',
          preAllocatedVUs: 50,
          maxVUs: 500,
          stages: [
            { target: 10,  duration: '10s' },
            { target: 300, duration: '5s'  },
            { target: 300, duration: '30s' },
            { target: 10,  duration: '5s'  },
          ],
        },
      },
      thresholds: {
        http_req_failed: ['rate<0.01'],
        http_req_duration: ['p(99)<1500'],
        dropped_iterations: ['count<1'],
      },
    };

And assert the response inside the default function:

    check(res, { 'status is 200': (r) => r.status === 200 });
