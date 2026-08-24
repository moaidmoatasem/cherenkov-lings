/**
 * PRODUCTION STORY:
 * Ticketmaster Taylor Swift Eras Tour Onsale Surge (2022)
 * 3.5 billion system requests overwhelmed API gateways. Tests monitoring only average response times
 * failed to catch extreme p99 tail latency degradation (jumping from 200ms to 45,000ms), locking out millions of fans.
 */

import http from 'k6/http';
import { Trend, Rate } from 'k6/metrics';
import { sleep, check } from 'k6';

// Anti-pattern: No metrics or thresholds — you can't tell if p99 degraded
// TODO: Define a Trend metric and a threshold that fails if p99 > 500ms
export const options = {
  stages: [
    { duration: '5s', target: 100 },  // Spike to 100 VUs instantly
  ],
  // Missing threshold — test never fails even under catastrophic latency
};

const searchDuration = new Trend('search_response_time', true);

export default function () {
  const res = http.get('http://127.0.0.1:8081/search?q=P');
  searchDuration.add(res.timings.duration);
  check(res, { 'status 200': (r) => r.status === 200 });
  sleep(0.05);
}
