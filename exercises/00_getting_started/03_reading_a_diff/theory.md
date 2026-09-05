# Theoretical Context: A Diff Is the Whole Story, If You Read It

## Production Incident: Heartbleed (2014)

OpenSSL's heartbeat extension let one side of a TLS connection send a small payload and ask the other side to echo it back, along with a length the sender claimed the payload was. The implementation trusted that claimed length completely: it allocated a reply buffer sized to the claim and copied that many bytes starting from the payload, regardless of how much payload had actually been sent. Send one byte and claim sixty-four kilobytes, and the server obligingly copied sixty-four kilobytes of whatever else happened to be sitting in adjacent memory -- private keys, session tokens, plaintext passwords -- back to you. The bug existed in shipped code for over two years before it was found. When the fix landed, it was two lines: a comparison ensuring the reply never exceeds the payload actually received. Security researchers who later read the original commit that introduced the bug pointed out that the entire vulnerability was visible in that one diff, to anyone who had looked for what a length check should have included and didn't.

## The Underlying Mechanism

A diff isolates exactly what changed and discards everything that didn't -- which makes it a more precise verification target than "read the whole file again":

```
[Diff anatomy]

  --- vulnerable_before.py
  +++ patched_after.py
  @@ function body @@
    def read_heartbeat_payload(payload, claimed_length):
  -     return payload[:claimed_length]
  +     safe_length = min(claimed_length, len(payload))
  +     return payload[:safe_length]

    '-' lines: removed   -- gone in the new version
    '+' lines: added     -- new in the new version
    unmarked:  unchanged -- present in both, not the point of this diff
```

A diff answers one question precisely: *what is different, and in which direction*. That precision is what makes it a regression check, not just documentation -- a test that asserts a diff contains a specific added line will fail the moment someone reintroduces the old behavior, even if the surrounding code has been rewritten since. Asserting only that "a diff exists" or "the file changed" verifies nothing about *which* change happened; the Heartbleed fix and a purely cosmetic rename would both satisfy that bar.

The same reading skill applies whether the diff comes from `difflib`, `git diff`, or a pull request review: find the `+` lines, decide whether the *specific* change you expect is actually among them, and don't accept "something changed" as a substitute for "the right thing changed."

You will now simulate this in the Crucible: turn a computed diff into an assertion that checks for the specific fix, not just for change.
