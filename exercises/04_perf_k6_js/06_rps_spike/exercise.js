/**
 * PRODUCTION STORY:
 * Robinhood Trading Outage (March 2020)
 * A full-session outage as markets opened on record volume, followed by a
 * second failure the next day and a $70 million FINRA penalty. The traffic
 * was a step function, not a ramp — and a closed VU-based load model quietly
 * throttles itself when the server slows, hiding exactly that failure.
 */

import http from 'k6/http';
import { check } from 'k6';

export const options = {
  // TODO: Configure an OPEN-model spike against /checkout.
  //   - Use a `scenarios` block with the `ramping-arrival-rate` executor so the
  //     offered load is a request rate, not a VU count.
  //   - Ramp from a baseline of ~10 iterations/s to ~300, sustain, then recover.
  //   - Add `thresholds` for http_req_failed, http_req_duration p(99),
  //     and dropped_iterations.
};

export default function () {
  const res = http.get('http://localhost:8081/checkout');
  // TODO: Check that the response status is 200
}
