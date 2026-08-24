import http from 'k6/http';
import { Trend, Rate, Counter } from 'k6/metrics';
import { check, sleep } from 'k6';

/**
 * SDET Resilient Pattern: Server-Sent Events (SSE) Streaming Validation
 * Verifies stream content type ('text/event-stream'), asserts presence of event frames
 * ('event: message', 'data:'), and tracks stream latency trends under concurrent load.
 */

const sseConnectionDuration = new Trend('sse_connection_duration', true);
const sseStreamErrors = new Rate('sse_stream_errors');
const sseEventsReceived = new Counter('sse_events_received');

export const options = {
  vus: 10,
  duration: '10s',
  thresholds: {
    'sse_stream_errors': ['rate<0.05'], // Under 5% stream errors
    'http_req_duration': ['p(95)<2000'],
  },
};

export default function () {
  const params = {
    headers: {
      'Accept': 'text/event-stream',
      'Cache-Control': 'no-cache',
    },
    timeout: '10s',
  };

  const res = http.get('http://127.0.0.1:8081/events/stream', params);
  sseConnectionDuration.add(res.timings.duration);

  const isSuccess = check(res, {
    'status is 200': (r) => r.status === 200,
    'content-type is text/event-stream': (r) => r.headers['Content-Type'] && r.headers['Content-Type'].includes('text/event-stream'),
    'body contains event framing': (r) => r.body && (r.body.includes('event:') || r.body.includes('data:')),
  });

  if (!isSuccess) {
    sseStreamErrors.add(1);
  } else {
    sseStreamErrors.add(0);
    sseEventsReceived.add(1);
  }

  sleep(1);
}
