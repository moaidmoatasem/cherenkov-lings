import http from 'k6/http';
import { Trend, Rate } from 'k6/metrics';
import { sleep, check } from 'k6';

// Solution: Custom Trend metric + p99 threshold correctly catches latency regression
const searchDuration = new Trend('search_response_time', true);
const errorRate = new Rate('search_errors');

export const options = {
  stages: [
    { duration: '3s', target: 20 },   // Ramp up
    { duration: '5s', target: 100 },  // Spike to 100 VUs
    { duration: '3s', target: 0 },    // Drain
  ],
  thresholds: {
    // p99 search duration must stay under 5000ms
    // (short query '/search?q=P' has 800ms backend sleep — this threshold is set
    //  at a realistic level that the solution meets but blind spiking breaks)
    'search_response_time': ['p(99)<5000'],
    'search_errors':        ['rate<0.05'],
  },
};

export default function () {
  const res = http.get('http://127.0.0.1:8081/search?q=Playwright');
  searchDuration.add(res.timings.duration);
  errorRate.add(res.status !== 200 ? 1 : 0);
  check(res, { 'status 200': (r) => r.status === 200 });
  sleep(0.05);
}
