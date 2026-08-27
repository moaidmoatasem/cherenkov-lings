"""Dummy OpenTelemetry tracing middleware for Micro-Crucible."""

import time
import uuid
import logging
from starlette.middleware.base import BaseHTTPMiddleware, RequestResponseEndpoint
from starlette.requests import Request
from starlette.responses import Response

# Configure a simple logger for our spans
logger = logging.getLogger("crucible.tracing")
logger.setLevel(logging.INFO)
if not logger.handlers:
    ch = logging.StreamHandler()
    formatter = logging.Formatter('%(asctime)s - %(name)s - %(levelname)s - %(message)s')
    ch.setFormatter(formatter)
    logger.addHandler(ch)

class TracingMiddleware(BaseHTTPMiddleware):
    """Dummy OpenTelemetry tracing middleware.
    Adds X-Trace-Id to responses and logs span events to stdout.
    """

    async def dispatch(
        self, request: Request, call_next: RequestResponseEndpoint
    ) -> Response:
        trace_id = str(uuid.uuid4())
        span_id = str(uuid.uuid4())[:8]
        start_time = time.time()
        
        logger.info(f"[SPAN START] trace_id={trace_id} span_id={span_id} method={request.method} url={request.url.path}")

        status_code = 500
        try:
            response = await call_next(request)
            status_code = response.status_code
            response.headers["X-Trace-Id"] = trace_id
            return response
        except Exception as exc:
            # An unhandled route error still closes its span, and the trace id
            # is recorded in the log so the 500 the client sees can be
            # correlated even though the header never made it onto a response.
            logger.info(
                f"[SPAN ERROR] trace_id={trace_id} span_id={span_id} "
                f"exception={type(exc).__name__}"
            )
            raise
        finally:
            duration_ms = (time.time() - start_time) * 1000
            logger.info(
                f"[SPAN END] trace_id={trace_id} span_id={span_id} "
                f"status_code={status_code} duration_ms={duration_ms:.2f}"
            )
