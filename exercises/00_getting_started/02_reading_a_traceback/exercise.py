"""
PRODUCTION STORY:
Ariane 5 Flight 501 (1996)
Thirty-seven seconds after launch, the rocket's inertial reference system
tried to convert a 64-bit velocity value into a 16-bit integer. The value
was too large. The conversion raised an unhandled exception, the flight
computer shut itself down, and the resulting bad guidance data steered the
rocket into a self-destruct. The bug had a precise, nameable cause -- an
unhandled exception at a specific line -- and nobody had read it before
launch, because the code path had never been tested under Flight 501's
actual flight profile.
"""

orders = [
    {"id": "ORD-1", "status": "COMPLETED"},
    {"id": "ORD-2", "status": "PENDING"},
    {"id": "ORD-3", "status": "PENDING"},
]


def test_status_summary_counts_pending_orders():
    # TODO: This raises before it ever reaches the assertion. Save the
    # file, read the traceback the watcher prints -- not just the last
    # line, the exception type and the exact line number above it -- and
    # fix the bug the traceback is pointing at. Do not touch the
    # assertion; it is already correct.
    pending_count = sum(1 for o in orders if o["stauts"] == "PENDING")
    assert pending_count == 2
