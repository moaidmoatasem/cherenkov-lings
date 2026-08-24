import http from 'k6/http';
import { Rate } from 'k6/metrics';
import { sleep } from 'k6';

// Anti-pattern: Hits the Chaos Proxy with no error rate assertion
// TODO: Add a Rate metric + threshold to assert error rate stays under 5%
export const options = {
  vus: 20,
  duration: '15s',
  // No threshold — the test passes even if 50% of requests fail under chaos
};

export default function () {
  // Route through the Chaos Proxy with a delay injected
  const res = http.get('http://127.0.0.1:8086/checkout', {
    headers: { 'X-Chaos': 'delay=500' },
  });
  sleep(0.5);
}
