# Theoretical Context: SQL Injection, Time-Based Exfiltration & Parameterization

## Real-World Incident Case Study
In the infamous 2011 Sony Pictures data breach, an attacker group extracted over one million unencrypted user passwords, email addresses, and home addresses using automated SQL injection tools (sqlmap). The vulnerability stemmed from legacy custom PHP/Python scripts that concatenated user query parameters directly into raw SQL strings without parameter binding or ORM encapsulation.

## Protocol & Runtime Mechanism
When an application constructs SQL commands via direct string concatenation, the database engine parser cannot distinguish between control instructions authored by the developer and adversarial syntax provided in untrusted request buffers:

$$\text{Executed Query} = \text{"SELECT * FROM accounts WHERE id = '"} + \text{user\_input} + \text{"'"}$$

If an attacker injects `' OR SLEEP(5)--`, the SQL AST is fundamentally restructured:
1. The `'` closes the intended string literal.
2. The `OR` operator introduces an additional truth condition.
3. The `SLEEP(5)` instruction commands the database engine thread to sleep for 5,000 milliseconds before returning.
4. The `--` comment delimiter instructs the parser to discard remaining trailing SQL code.

```
  Vulnerable (String Interpolation):
  [ SELECT * FROM users WHERE id = ' ] + [ 1' OR SLEEP(5)-- ] 
                   │
                   ?
  Parsed SQL AST Mutated -> Executes Database Sleep Command

  Secure (Parameterized Prepared Statement):
  [ SELECT * FROM users WHERE id = ? ]  <--- Bound Value: "1' OR SLEEP(5)--"
                   │
                   ?
  Database treats entire input payload as literal string value
```

Parameterized queries solve this by pre-compiling the SQL query template on the database server before variable values are transmitted across the wire. When variables arrive, the database runtime treats them exclusively as literal scalar values, completely neutralizing syntax injection.

## You will now simulate this in the Crucible
Run `cherenkov-lings watch --track=devsecops-python` and verify that parameterized user lookups reject SQL injection payloads and return deterministic responses.
