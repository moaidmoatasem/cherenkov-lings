import http from 'k6/http';
import { Trend, Rate, Counter } from 'k6/metrics';
import { check, sleep } from 'k6';

/**
 * SDET Resilient Pattern: Grafana / InfluxDB Custom Observability & Strict SLOs
 * Emits custom tagged Trends, Rates, and Counters with strict percentile thresholds (p99, p95)
 * for real-time visualization in Grafana dashboards and automated CI quality gates.
 */

// Custom business metrics
const orderLatency = new Trend('order_processing_latency', true);
const failedOrdersRate = new Rate('failed_orders');
const successfulOrdersCount = new Counter('successful_orders');

export const options = {
  stages: [
    { duration: '3s', target: 10 },
    { duration: '5s', target: 25 },
    { duration: '2s', target: 0 },
  ],
  thresholds: {
    // SLO: 99th percentile response time must be under 250ms
    'http_req_duration{endpoint:checkout}': ['p(99)<250', 'p(95)<150'],
    'order_processing_latency': ['p(99)<250'],
    // SLO: Failure rate must remain strictly below 1%
    'failed_orders': ['rate<0.01'],
    'http_req_failed': ['rate<0.01'],
  },
};

export default function () {
  const params = {
    headers: { 'Content-Type': 'application/json' },
    tags: { endpoint: 'checkout' },
  };

  const payload = JSON.stringify({
    item_id: 'item-prod-101',
    quantity: 1,
  });

  const res = http.post('http://127.0.0.1:8081/checkout', payload, params);

  // Track latency trend
  orderLatency.add(res.timings.duration, { endpoint: 'checkout' });

  const passed = check(res, {
    'status is 200': (r) => r.status === 200,
    'has valid order_id or status': (r) => r.json('order_id') !== undefined || r.json('status') === 'success',
  });

  if (passed) {
    failedOrdersRate.add(0, { endpoint: 'checkout' });
    successfulOrdersCount.add(1);
  } else {
    failedOrdersRate.add(1, { endpoint: 'checkout' });
  }

  sleep(0.1);
}
