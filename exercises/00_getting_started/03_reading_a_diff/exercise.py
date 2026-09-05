"""
PRODUCTION STORY:
Heartbleed (2014)
A missing bounds check in OpenSSL's heartbeat extension let a client claim
its payload was longer than the data it actually sent. The server echoed
back whatever was sitting in adjacent memory -- private keys, session
cookies, passwords -- for over two years before anyone noticed. The entire
fix was a two-line diff: one length comparison that had never been added.
"""

import difflib
from pathlib import Path

FIXTURE_DIR = Path(__file__).parent


def test_patch_adds_the_missing_bounds_check():
    before = FIXTURE_DIR.joinpath("vulnerable_before.py").read_text().splitlines()
    after = FIXTURE_DIR.joinpath("patched_after.py").read_text().splitlines()
    diff = list(difflib.unified_diff(before, after, lineterm=""))
    added_lines = [
        line for line in diff if line.startswith("+") and not line.startswith("+++")
    ]

    # TODO: `added_lines` is the real diff between the vulnerable and patched
    # versions -- print it if you want to see it. A diff tells you exactly
    # what changed and nothing else. Replace the line below with an
    # assertion that the fix actually present in `added_lines` is a real
    # bounds check against `len(payload)` -- not just that *some* line
    # changed, which would pass even if the "fix" changed something
    # unrelated.
    assert False, "replace this with a real assertion against added_lines"
