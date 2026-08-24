# Hints: Drill 04 - Listener in Production Load Test

## Hint 1 (Concept)
JMeter Listeners like "View Results Tree" and "Summary Report" are designed for the GUI -- they capture every request/response in memory so you can inspect them visually. During a real load test with 1000 VUs and 60 minutes runtime, these listeners will consume all available RAM and crash the test.

## Hint 2 (Pattern)
Rule: DISABLE all Listeners before running a load test in non-GUI mode.
Use the -l flag to write raw results to a JTL file instead:
  jmeter -n -t plan.jmx -l results.jtl
Then generate the report after the test completes: jmeter -g results.jtl -o report/

## Hint 3 (Answer)
In JMeter GUI: right-click each Listener, select "Disable".
Or use the checkbox in the test plan tree to uncheck the Listener.
Never ship a JMX file with active Listeners to CI.
