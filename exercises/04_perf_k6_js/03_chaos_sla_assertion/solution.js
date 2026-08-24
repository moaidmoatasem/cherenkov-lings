import http from 'k6/http';
import { Rate } from 'k6/metrics';
import { sleep, check } from 'k6';

// Solution: Rate metric + threshold asserts error rate stays under 5% under chaos
const errorRate = new Rate('chaos_errors');

export const options = {
  vus: 20,
  duration: '15s',
  thresholds: {
    // Under X-Chaos: delay=500 the backend adds latency but still responds 200
    // Error rate must stay under 5% to pass SLA
    'chaos_errors': ['rate<0.05'],
    'http_req_duration': ['p(95)<3000'], // 95th pct under 3s (500ms delay + headroom)
  },
};

export default function () {
  const res = http.get('http://127.0.0.1:8086/checkout', {
    headers: { 'X-Chaos': 'delay=500' },
    timeout: '10s',
  });
  // Only count non-2xx responses as errors; timeout or 5xx counts as a failure
  errorRate.add(res.status < 200 || res.status >= 300 ? 1 : 0);
  check(res, { 'status 200': (r) => r.status === 200 });
  sleep(0.5);
}
