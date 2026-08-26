# Hints: Drill 02 - k6 vs JMeter

## Hint 1 (k6 strengths)
k6 writes tests as JavaScript files. They live in Git alongside your code. CI runs them with a single command. No GUI, no plugins, fast startup. Ideal for: DevOps teams, GitOps workflows, modern cloud-native stacks.

## Hint 2 (JMeter strengths)
JMeter has a visual GUI for recording HTTP sessions, a massive plugin ecosystem, and is the industry standard in banking, government, and enterprise. Ideal for: teams that need to record complex flows, legacy stacks, organizations with existing JMeter expertise.

## Hint 3 (Decision Rules)
startup + CI + Git = k6
enterprise + GUI + plugins = jmeter
version control needed = k6 (JS files vs binary JMX XML)
HTTP session recording = jmeter (has built-in recorder)
