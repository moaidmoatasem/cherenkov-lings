"""Workspace-wide pytest configuration for exercise runs.

Besides putting the repository root on `sys.path`, this makes the chaos the
scorecard claims credit for actually happen.

The runner sets `CHAOS_DIRECTIVES` (e.g. `delay=200ms;jitter=75ms`) in the
environment of every iteration, and the Crucible applies chaos per request from
the `X-Chaos` header. Nothing connected the two: Python drills called the API
with no header, got an unperturbed server, and were then scored "5/5 passed
under chaos (200ms delay + 75ms jitter)" -- 35% of the total, awarded for
surviving a network nobody had degraded. Forwarding the directives as the
header makes the Flakiness Resistance dimension measure what it says it does.

Drills that set their own X-Chaos header keep it; this only fills the gap.
"""

import os
import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))


def _install_chaos_forwarding() -> None:
    directives = os.environ.get("CHAOS_DIRECTIVES", "").strip()
    if not directives:
        return

    try:
        import requests
    except ImportError:
        return

    if getattr(requests.Session, "_cherenkov_chaos_wrapped", False):
        return

    original_request = requests.Session.request

    def request_with_chaos(self, method, url, **kwargs):
        headers = kwargs.get("headers") or {}
        if not any(k.lower() == "x-chaos" for k in headers):
            headers = {**headers, "X-Chaos": directives}
            kwargs["headers"] = headers
        return original_request(self, method, url, **kwargs)

    requests.Session.request = request_with_chaos
    requests.Session._cherenkov_chaos_wrapped = True


_install_chaos_forwarding()
