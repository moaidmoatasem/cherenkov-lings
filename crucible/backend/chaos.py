"""Chaos injection engine and middleware for Micro-Crucible."""

import asyncio
import random
import re
from typing import Any

from starlette.middleware.base import BaseHTTPMiddleware, RequestResponseEndpoint
from starlette.requests import Request
from starlette.responses import Response


def parse_duration_ms(val_str: str) -> float:
    """Parse duration strings like '500ms', '1.5s', or '500' into milliseconds."""
    val_str = val_str.strip().lower()
    if val_str.endswith("ms"):
        return float(val_str[:-2].strip())
    elif val_str.endswith("s"):
        return float(val_str[:-1].strip()) * 1000.0
    return float(val_str)


def parse_chaos_header(header_val: str | None) -> dict[str, Any]:
    """Parse compound X-Chaos header into structured directives dict.

    Supported directives:
      - delay=<ms|s>: artificial latency (e.g. '500ms', '1s')
      - jitter=<ms|s>: latency variance (e.g. '75ms')
      - stale_dom=true: sets stale dom header and response flag
      - token_expire=immediate: causes JWT expiration immediately
      - kafka_lag=<ms|s>: async settlement delay (e.g. '1500ms')
      - idempotency_conflict=true: triggers 409 Conflict on checkout
      - drop_partial=true: triggers partial upload failure (HTTP 400)
      - drop_after=<n>: drops SSE stream after N emitted events
    """
    if not header_val:
        return {}

    directives: dict[str, Any] = {}
    tokens = re.split(r"[;,]", header_val)
    for token in tokens:
        token = token.strip()
        if not token:
            continue
        if "=" in token:
            k, v = token.split("=", 1)
            k = k.strip().lower()
            v = v.strip()
            if k in ("delay", "jitter", "kafka_lag"):
                try:
                    directives[k] = parse_duration_ms(v)
                except ValueError:
                    directives[k] = 0.0
            elif k in ("stale_dom", "idempotency_conflict", "drop_partial", "db_timeout", "dast_xss"):
                directives[k] = v.lower() in ("true", "1", "yes")
            elif k == "drop_after":
                try:
                    directives[k] = int(v)
                except ValueError:
                    directives[k] = 0
            elif k == "token_expire":
                directives[k] = v.lower()
            else:
                directives[k] = v
        else:
            directives[token.lower()] = True

    return directives


class ChaosMiddleware(BaseHTTPMiddleware):
    """Middleware injecting artificial latency, jitter, and response mutations."""

    async def dispatch(
        self, request: Request, call_next: RequestResponseEndpoint
    ) -> Response:
        chaos_header = request.headers.get("x-chaos")
        chaos = parse_chaos_header(chaos_header)
        request.state.chaos = chaos

        delay_ms = float(chaos.get("delay", 0.0))
        jitter_ms = float(chaos.get("jitter", 0.0))

        if delay_ms > 0.0 or jitter_ms > 0.0:
            actual_delay_ms = delay_ms
            if jitter_ms > 0.0:
                actual_delay_ms += random.uniform(-jitter_ms, jitter_ms)
            actual_delay_ms = max(0.0, actual_delay_ms)
            await asyncio.sleep(actual_delay_ms / 1000.0)

        response = await call_next(request)

        if chaos.get("stale_dom"):
            response.headers["X-Chaos-Stale-DOM"] = "true"

        return response
