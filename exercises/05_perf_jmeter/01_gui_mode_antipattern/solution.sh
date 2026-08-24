#!/bin/bash
# JMeter Non-GUI Execution -- the ONLY way to run load tests in CI
# -n          = non-GUI mode (no rendering overhead)
# -t          = test plan file
# -l          = results log file (JTL format)
# -e          = generate HTML dashboard report
# -o          = output directory for the report

jmeter -n -t exercise.jmx -l results.jtl -e -o report/

# After the run:
# - results.jtl contains raw timing data for every request
# - report/index.html is the visual dashboard
echo "Load test complete. Open report/index.html to view results."
