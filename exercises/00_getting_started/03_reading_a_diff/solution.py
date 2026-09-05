"""
PRODUCTION STORY:
Heartbleed (2014)
A missing bounds check in OpenSSL's heartbeat extension let a client claim
its payload was longer than the data it actually sent, leaking adjacent
memory for over two years. The entire fix was a two-line diff.
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

    assert any("min(claimed_length, len(payload))" in line for line in added_lines), (
        "expected the diff to add a bounds check comparing claimed_length "
        "against the real payload length"
    )
