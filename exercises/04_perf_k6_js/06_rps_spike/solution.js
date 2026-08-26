/**
 * PRODUCTION STORY:
 * Robinhood Trading Outage (March 2020)
 * Closed-model load tests pass because the generator slows down with the
 * server. An open arrival-rate model keeps firing, and the queue tells the truth.
 */

import http from 'k6/http';
import { check } from 'k6';

// Open workload model: hold a request RATE, not a VU count. When /checkout
// degrades, k6 keeps firing and the queue grows, instead of quietly throttling
// itself the way a closed VU-based model would (coordinated omission).
export const options = {
  scenarios: {
    checkout_spike: {
      executor: 'ramping-arrival-rate',
      startRate: 10,
      timeUnit: '1s',
      preAllocatedVUs: 50,
      maxVUs: 500,
      stages: [
        { target: 10, duration: '10s' },  // baseline
        { target: 300, duration: '5s' },  // spike: 30x in five seconds
        { target: 300, duration: '30s' }, // sustain at peak arrival rate
        { target: 10, duration: '5s' },   // recovery
      ],
    },
  },
  thresholds: {
    http_req_failed: ['rate<0.01'],
    http_req_duration: ['p(99)<1500'],
    // If k6 cannot sustain the arrival rate even at maxVUs, the service fell
    // behind. A single dropped iteration fails the run.
    dropped_iterations: ['count<1'],
  },
};

export default function () {
  const res = http.get('http://localhost:8081/checkout');
  check(res, {
    'status is 200': (r) => r.status === 200,
  });
}
