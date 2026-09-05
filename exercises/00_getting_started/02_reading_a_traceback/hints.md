# Hints: Drill 02 - Reading a Traceback

## Hint 1 (Architectural Nudge)
An assertion failure and a traceback are not the same event. An assertion failure means your code ran to completion and the *result* was wrong -- that's drill 01. A traceback means your code never got that far; something raised an exception partway through, and execution stopped right there. Here, the crash happens on the line that builds `pending_count`, before the `assert` line ever runs. Whatever the assertion says is irrelevant until the line above it stops crashing.

## Hint 2 (API Pattern)
Read a Python traceback from the **bottom up**. The last line names the exception type and the specific message -- for a dictionary, a `KeyError` names the exact key it went looking for and didn't find. The lines above it, read upward, are the call stack: which line in which function called which other line, working backward from the crash to where the bad value entered. You don't need to read the whole stack here -- the bottom line alone tells you which key is missing, and a quick look at the dictionaries above shows you what the key is actually spelled.

## Hint 3 (Code Diff)
```diff
- pending_count = sum(1 for o in orders if o["stauts"] == "PENDING")
+ pending_count = sum(1 for o in orders if o["status"] == "PENDING")
```
`"stauts"` is not a key in any of these dictionaries -- it's a transposed-letter typo for `"status"`. The traceback's `KeyError: 'stauts'` was naming the exact bug the whole time.
