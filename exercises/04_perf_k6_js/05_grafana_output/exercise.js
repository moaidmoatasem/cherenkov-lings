/**
 * PRODUCTION STORY:
 * DoorDash Super Bowl Performance Blindspot (2021)
 * High-volume food ordering during halftime created a massive tail latency spike (p99 > 14,000ms).
 * Engineering dashboards monitoring only average response times showed a green 120ms average,
 * masking the fact that 1% of customers (tens of thousands of orders) were timing out.
 */

import http from 'k6/http';
import { sleep } from 'k6';

// Anti-pattern: Default stdout metrics only, no custom business metrics, and no percentile SLO thresholds
// TODO: Define custom Trend, Rate metrics, and strict p95/p99 thresholds formatted for Grafana/InfluxDB

export const options = {
  vus: 20,
  duration: '10s',
  // Anti-pattern: Missing thresholds and custom tagged metrics for observability pipelines
};

export default function () {
  http.get('http://127.0.0.1:8081/checkout');
  sleep(0.5);
}
