# Hints: Drill 07 - Distributed Load Generation

## Hint 1 (Concept)
A single laptop or VM can generate roughly 200-500 concurrent users before network saturation, CPU limits, or JVM heap limits make the results unreliable. For realistic internet-scale load (10,000+ users), you need distributed testing with a Controller and multiple Agent nodes.

## Hint 2 (Pattern)
JMeter Distributed Testing:
  1. On each Agent machine: start jmeter-server
  2. On the Controller: add agent IPs to jmeter.properties remote_hosts=192.168.1.10,192.168.1.11
  3. Run: jmeter -n -t plan.jmx -r -l results.jtl
  The -r flag triggers all remote agents simultaneously.

## Hint 3 (Answer)
Key config in jmeter.properties:
  remote_hosts=agent1_ip:1099,agent2_ip:1099
  server.rmi.ssl.disable=true  (for internal networks)
Each agent multiplies your thread capacity. 5 agents x 200 threads = 1000 concurrent users.
