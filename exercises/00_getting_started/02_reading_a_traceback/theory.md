# Theoretical Context: Tracebacks Are Not Random Noise

## Production Incident: Ariane 5 Flight 501 (1996)

Thirty-seven seconds after the Ariane 5's maiden flight, its inertial reference system tried to convert a 64-bit floating-point velocity value into a 16-bit signed integer. The rocket's horizontal velocity at that point in the flight profile was larger than the previous Ariane 4 had ever produced, and the conversion overflowed. The software had no handler for that exception, so the processor halted -- and, by design, handed control to a backup unit running the identical software, which failed the identical way half a second later. With both inertial units down, the flight computer received garbage guidance data, steered hard to correct for a deviation that never happened, and the rocket broke apart under aerodynamic stress. The post-flight inquiry's report is, at its core, a very long, very expensive traceback: it names the exact operation, the exact variable, and the exact line where an unhandled exception became a lost rocket.

## The Underlying Mechanism

An assertion failure and an unhandled exception are different signals, and conflating them wastes time:

```
[Two different failures, read differently]

  Assertion failure:              Traceback:
  code ran to completion    ✓      code stopped partway through   ✗
  the RESULT was wrong             something RAISED before result existed
  fix: check your expected value   fix: read bottom-up, find what raised

  Traceback anatomy (read bottom -> top):
  ┌────────────────────────────────────────────┐
  │ KeyError: 'stauts'          <- what & why   │  read this FIRST
  ├────────────────────────────────────────────┤
  │ File "exercise.py", line 19, in <genexpr>   │  where it happened
  ├────────────────────────────────────────────┤
  │ File "exercise.py", line 19, in test_...    │  how you got there
  └────────────────────────────────────────────┘
```

Python (and most languages) print the exception type and message last, then walk the call stack upward from there. The bottom line is the destination; everything above it is the route you took to arrive. For a `KeyError`, that bottom line already names the missing key verbatim -- there is rarely a need to guess when the interpreter has already told you exactly what it was looking for.

The habit this drill builds -- read the exception type and message first, then use the stack only if the message alone isn't enough -- is the same habit that turns a 45-minute "why is this failing" session into a 45-second one, on a rocket guidance system or a five-line pytest file.

You will now simulate this in the Crucible: fix a bug by reading what the traceback actually names, not by rewriting the line you assume is broken.
