#!/bin/bash
# JMeter Automated HTML Report Dashboard Generation
# Step 1: Run headless test generating CSV JTL output
# Step 2: Auto-compile rich HTML performance dashboard

jmeter -n -t solution.jmx -l results.jtl -e -o html_dashboard/

echo "Dashboard generated at html_dashboard/index.html"
