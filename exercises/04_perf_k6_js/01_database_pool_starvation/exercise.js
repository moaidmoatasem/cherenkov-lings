import http from 'k6/http';
import { sleep } from 'k6';

// Anti-pattern: 50 VUs hit the server instantly — no ramp-up
// TODO: Replace with staged ramp-up to avoid database pool exhaustion
export const options = {
  vus: 50,
  duration: '10s',
  thresholds: {
    http_req_failed: ['rate<0.01'], // <1% errors — will FAIL under 50 VUs cold start
  },
};

export default function () {
  // Hammer the checkout endpoint with no warm-up
  http.post('http://127.0.0.1:8081/checkout', JSON.stringify({ item_id: 'item-1' }), {
    headers: { 'Content-Type': 'application/json' },
  });
  sleep(0.1);
}
