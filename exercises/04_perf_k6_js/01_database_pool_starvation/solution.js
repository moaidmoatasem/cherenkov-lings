import http from 'k6/http';
import { sleep } from 'k6';

// Solution: Staged ramp-up prevents database pool exhaustion
// The server has time to warm up connection pools before full load
export const options = {
  stages: [
    { duration: '5s',  target: 10  },  // Warm up: ramp to 10 VUs
    { duration: '5s',  target: 30  },  // Steady state: ramp to 30 VUs
    { duration: '5s',  target: 10  },  // Cool down: ramp back down
    { duration: '3s',  target: 0   },  // Drain: wind down to zero
  ],
  thresholds: {
    http_req_failed: ['rate<0.01'],     // <1% errors — achievable with gradual ramp
    http_req_duration: ['p(95)<2000'],  // 95th pct under 2s
  },
};

export default function () {
  const res = http.post('http://127.0.0.1:8081/checkout', JSON.stringify({ item_id: 'item-1' }), {
    headers: { 'Content-Type': 'application/json' },
  });
  sleep(0.2);
}
