# Hints: Drill 01 - GUI Mode Anti-Pattern

## Hint 1 (Concept)
JMeter has two modes: GUI (the visual application) and non-GUI (command line). The GUI is like Postman -- great for designing your test plan. But when you run a load test in GUI mode, JMeter renders charts in real-time which uses 20-30% of your CPU and memory, making your performance numbers unreliable.

## Hint 2 (Pattern)
Rule: Design in GUI, run in CLI.
The non-GUI command is: jmeter -n -t <plan.jmx> -l <results.jtl>
Add -e -o report/ to automatically generate an HTML dashboard.

## Hint 3 (Answer)
The correct command to run this plan non-interactively:
  jmeter -n -t exercise.jmx -l results.jtl -e -o report/
