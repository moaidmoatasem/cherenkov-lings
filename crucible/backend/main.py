"""Entrypoint module for Micro-Crucible FastAPI Server.

Exposes the FastAPI `app` instance for uvicorn, test runners, and CLI integration.
"""

from __future__ import annotations

import uvicorn
from crucible.backend.app import app

__all__ = ["app"]

if __name__ == "__main__":
    uvicorn.run("crucible.backend.main:app", host="127.0.0.1", port=8080, reload=True)
