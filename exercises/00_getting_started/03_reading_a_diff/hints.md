# Hints: Drill 03 - Reading a Diff

## Hint 1 (Architectural Nudge)
A diff is not "the new file" or "the old file" -- it is specifically the *difference* between them, and reading one is a distinct skill from reading either file alone. Every line `difflib.unified_diff` marks with a leading `+` was added in the patched version; every line marked `-` was removed. This test has already computed that diff for you into `added_lines` -- your job is to look at what's actually inside it, not to assume "a diff exists" is the same as "the right fix exists."

## Hint 2 (API Pattern)
`git diff`, code review tools, and `difflib.unified_diff` all produce the same shape of output: unchanged lines with no prefix, removed lines prefixed `-`, added lines prefixed `+`. The exercise already filters to just the `+` lines and strips the `+++` file-header line, so `added_lines` is exactly the new code that didn't exist before. Search it for the specific fix you expect -- a length comparison against the real payload -- rather than asserting only that the list is non-empty, which would pass for *any* one-line change, including one that fixed nothing.

## Hint 3 (Code Diff)
```diff
- assert False, "replace this with a real assertion against added_lines"
+ assert any("min(claimed_length, len(payload))" in line for line in added_lines), (
+     "expected the diff to add a bounds check comparing claimed_length "
+     "against the real payload length"
+ )
```
This checks for the specific safety comparison, not just that some line moved -- so it would still fail if a future "fix" changed something else in the function and left the bounds check out again.
