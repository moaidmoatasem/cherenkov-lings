/**
 * PRODUCTION STORY:
 * Slack Real-Time Messaging Gateway Collapse (2019)
 * During a global outage, 100,000 enterprise desktop clients reconnected simultaneously to Server-Sent Event
 * gateways. Standard synthetic load tests treated SSE endpoints as simple one-shot HTTP GET requests,
 * missing connection pool exhaustion and stream truncation under high concurrent loads.
 */

import http from 'k6/http';
import { check, sleep } from 'k6';

// Anti-pattern: Treating Server-Sent Events (SSE) streaming endpoint as a standard one-shot HTTP GET
// TODO: Validate SSE stream headers (Content-Type: text/event-stream), stream event reception, and connection resilience

export const options = {
  vus: 10,
  duration: '5s',
};

export default function () {
  // Anti-pattern: Naive one-shot request ignoring stream nature and event headers
  const res = http.get('http://127.0.0.1:8081/events/stream');

  // Flawed check: Only checking status 200 without validating text/event-stream or event payloads
  check(res, {
    'status is 200': (r) => r.status === 200,
  });

  sleep(1);
}
