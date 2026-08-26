import http from 'k6/http';
import { check } from 'k6';

export const options = {
  stages: [
    { duration: '10s', target: 100 }, // fast ramp-up
    { duration: '30s', target: 100 }, // sustain
    { duration: '10s', target: 0 },   // ramp-down
  ],
};

export default function () {
  const res = http.get('http://localhost:8081/checkout');
  check(res, {
    'status is 200': (r) => r.status === 200,
  });
}
