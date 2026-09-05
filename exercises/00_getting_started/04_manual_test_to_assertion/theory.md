# Theoretical Context: Translating a Manual Test Case Into Code

## Production Incident: Therac-25 (1985-1987)

The Therac-25 was a radiation therapy machine whose control software contained a race condition: if an operator, working quickly and confidently, entered treatment parameters and then used the edit key to correct a mistake within about eight seconds, a safety interlock that was supposed to prevent an overdose could be silently bypassed. Experienced operators at some facilities developed a rough sense that typing fast and correcting a field could occasionally produce a strange error code, and worked around it by re-entering data more slowly -- without knowing why, and without anyone recording it as a defect. Between 1985 and 1987, at least six patients received massive radiation overdoses, several fatally, before investigators traced the pattern to the race condition. The tragedy of the near-misses that came before the fatalities is specific: the knowledge of what to watch for existed, in the hands of people who had seen it, and it never became a check that ran automatically before the next release shipped.

## The Underlying Mechanism

A manual test case and an automated assertion are the same claim, expressed in two different languages:

```
[The same check, two languages]

  MANUAL (a person reads this and does it):
    Steps: open checkout, note subtotal, note tax, note total
    Expected result: total equals subtotal plus tax

  AUTOMATED (a machine reads this and does it):
    checkout = requests.get(".../checkout").json()
    assert checkout["total"] == checkout["subtotal"] + checkout["tax"]

  Both check the SAME claim.
  Only the manual version forgets everything the moment the tester logs off.
```

The translation skill is not "learn to write code" in the abstract -- it's the much narrower habit of reading a manual test case's *expected result* line, identifying which fields in a response, screen, or database row that sentence is actually comparing, and writing that comparison directly rather than a proxy for it. A common failure mode at this step is writing the assertion against a value observed once (`total == 160.92`) instead of the relationship the manual tester actually verified (`total == subtotal + tax`); the first only detects a wrong price today, and stops meaning anything the day a real price changes. The second keeps checking the actual claim indefinitely, with zero human attention required -- which is precisely the check the Therac-25 investigation found didn't exist anywhere written down.

You will now simulate this in the Crucible: read a manual test case's expected result, and write the one assertion that actually checks it.
