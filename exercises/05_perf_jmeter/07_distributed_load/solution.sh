#!/bin/bash
# JMeter Distributed Load Execution Pattern
# -n : Non-GUI headless execution
# -t : Target JMX test plan
# -r : Run test on all remote servers defined in remote_hosts of jmeter.properties
# -l : Write aggregated raw results to JTL log
# -e -o : Generate comprehensive HTML dashboard report

jmeter -n -t solution.jmx -r -l results.jtl -e -o distributed_report/
