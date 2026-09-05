"""
PRODUCTION STORY:
Ariane 5 Flight 501 (1996)
An unhandled exception from a 64-bit-to-16-bit conversion shut down the
flight computer 37 seconds after launch, and the rocket destroyed itself.
The bug had a precise, nameable cause that nobody had read before launch.
"""

orders = [
    {"id": "ORD-1", "status": "COMPLETED"},
    {"id": "ORD-2", "status": "PENDING"},
    {"id": "ORD-3", "status": "PENDING"},
]


def test_status_summary_counts_pending_orders():
    pending_count = sum(1 for o in orders if o["status"] == "PENDING")
    assert pending_count == 2
