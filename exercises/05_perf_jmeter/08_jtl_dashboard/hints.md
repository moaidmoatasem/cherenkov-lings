# Hints: Drill 08 - JTL to HTML Dashboard

## Hint 1 (Concept)
A raw JTL file is a CSV with timestamps and response times. Without a dashboard, your load test produced data that nobody can read. The JMeter HTML Dashboard converts JTL into charts for: throughput over time, response time percentiles, error rates, and top slow transactions.

## Hint 2 (Pattern)
Generate the dashboard in two ways:
  During test:  jmeter -n -t plan.jmx -l results.jtl -e -o report/
  After test:   jmeter -g results.jtl -o report/
Then open report/index.html in a browser.

## Hint 3 (Answer)
The most important charts to review:
  1. Response Times Over Time -- shows degradation under load
  2. Percentiles Over Time -- shows p95/p99 tail latency
  3. Active Threads Over Time -- shows ramp-up worked correctly
  4. Transactions Per Second -- shows throughput achieved
Publish report/index.html as a CI artifact for every test run.
