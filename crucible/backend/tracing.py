"""Minimal in-memory tracing for Micro-Crucible.

Real enough to teach W3C Trace Context propagation, not a full OpenTelemetry
SDK: every request is recorded as one span, correlated to an incoming
`traceparent` header when the client sends one. There is no fan-out to child
spans inside a single handler -- one HTTP request in, one span out.
"""

import logging
import re
import time
import uuid
from collections import deque
from typing import Any

from starlette.middleware.base import BaseHTTPMiddleware, RequestResponseEndpoint
from starlette.requests import Request
from starlette.responses import Response

logger = logging.getLogger("crucible.tracing")
logger.setLevel(logging.INFO)
if not logger.handlers:
    ch = logging.StreamHandler()
    formatter = logging.Formatter("%(asctime)s - %(name)s - %(levelname)s - %(message)s")
    ch.setFormatter(formatter)
    logger.addHandler(ch)

# W3C Trace Context (https://www.w3.org/TR/trace-context/): version-trace_id-parent_id-flags
_TRACEPARENT_RE = re.compile(
    r"^([0-9a-f]{2})-([0-9a-f]{32})-([0-9a-f]{16})-([0-9a-f]{2})$", re.IGNORECASE
)

# Bounded so a long-running dev server doesn't grow this unboundedly; recent
# spans are what a drill or a learner debugging a single request cares about.
_MAX_SPANS = 2000
_spans: deque[dict[str, Any]] = deque(maxlen=_MAX_SPANS)


def _parse_traceparent(header: str | None) -> tuple[str, str | None]:
    """Return (trace_id, parent_span_id). Generates a fresh root trace_id when
    the header is absent or malformed, so a span is still recorded either way
    -- the correlation just won't be findable under a trace_id the client
    never actually sent."""
    if header:
        match = _TRACEPARENT_RE.match(header.strip())
        if match:
            _version, trace_id, parent_span_id, _flags = match.groups()
            return trace_id.lower(), parent_span_id.lower()
    return uuid.uuid4().hex, None


def get_spans(trace_id: str) -> list[dict[str, Any]]:
    """Spans recorded under this trace_id, oldest first."""
    return [s for s in _spans if s["trace_id"] == trace_id]


def reset_spans() -> None:
    _spans.clear()


class TracingMiddleware(BaseHTTPMiddleware):
    """Records one span per request, correlated to an incoming W3C
    `traceparent` header when present. Adds `X-Trace-Id` to the response and
    logs span start/end -- and, unlike a request-scoped log line, the span is
    queryable afterward via `GET /api/telemetry/spans?trace_id=...`."""

    async def dispatch(
        self, request: Request, call_next: RequestResponseEndpoint
    ) -> Response:
        trace_id, parent_span_id = _parse_traceparent(request.headers.get("traceparent"))
        span_id = uuid.uuid4().hex[:16]
        start_time = time.time()

        logger.info(
            f"[SPAN START] trace_id={trace_id} span_id={span_id} "
            f"parent_span_id={parent_span_id} method={request.method} url={request.url.path}"
        )

        status_code = 500
        try:
            response = await call_next(request)
            status_code = response.status_code
            response.headers["X-Trace-Id"] = trace_id
            return response
        except Exception as exc:
            logger.info(
                f"[SPAN ERROR] trace_id={trace_id} span_id={span_id} "
                f"exception={type(exc).__name__}"
            )
            raise
        finally:
            duration_ms = (time.time() - start_time) * 1000
            _spans.append(
                {
                    "trace_id": trace_id,
                    "span_id": span_id,
                    "parent_span_id": parent_span_id,
                    "method": request.method,
                    "path": request.url.path,
                    "status_code": status_code,
                    "duration_ms": round(duration_ms, 2),
                    "timestamp": start_time,
                }
            )
            logger.info(
                f"[SPAN END] trace_id={trace_id} span_id={span_id} "
                f"status_code={status_code} duration_ms={duration_ms:.2f}"
            )
