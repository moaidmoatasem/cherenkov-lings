"""Curriculum manifest loader.

`lings.toml` at the repository root is the single source of truth for tracks and
drills. It is read here rather than duplicated in Python, so adding a track or a
drill is a one-file change that the Rust engine and this backend both pick up.

The drill list comes from this manifest rather than a filesystem scan, so it is
correct even where the exercise tree is absent or partial. (Dockerfile.backend
ships both: `exercises/` is needed separately by GET /api/drill/theory, which
reads theory.md and hints.md off disk.)
"""

from __future__ import annotations

import tomllib
from functools import lru_cache
from pathlib import Path
from typing import Any

# crucible/backend/curriculum.py -> crucible/backend -> crucible -> repo root
_REPO_ROOT = Path(__file__).resolve().parents[2]
MANIFEST_PATH = _REPO_ROOT / "lings.toml"


class ManifestError(RuntimeError):
    """Raised when the curriculum manifest is missing or malformed."""


@lru_cache(maxsize=1)
def load_manifest() -> dict[str, Any]:
    """Parse and cache lings.toml.

    Cached because the manifest is immutable for the lifetime of the process;
    call `load_manifest.cache_clear()` in tests that rewrite it.
    """
    try:
        with MANIFEST_PATH.open("rb") as fh:
            return tomllib.load(fh)
    except FileNotFoundError as exc:
        raise ManifestError(f"Curriculum manifest not found at {MANIFEST_PATH}") from exc
    except tomllib.TOMLDecodeError as exc:
        raise ManifestError(f"Curriculum manifest at {MANIFEST_PATH} is not valid TOML: {exc}") from exc


def drill_root(track: dict[str, Any]) -> str:
    """Directory holding a track's drill sub-directories.

    Defaults to `exercise_dir`; the Maven-structured Java track overrides it.
    """
    return track.get("drill_root") or track["exercise_dir"]


def drill_path(track: dict[str, Any], drill_id: str) -> str:
    return f"{drill_root(track)}/{drill_id}"


def build_curriculum() -> list[dict[str, Any]]:
    """Project the manifest into the shape served by GET /api/curriculum."""
    manifest = load_manifest()
    tracks: list[dict[str, Any]] = []

    for track in manifest.get("tracks", []):
        tracks.append(
            {
                "id": track["id"],
                # `name` is the CLI/watcher label and carries the stack inline;
                # `display_name` is the shorter catalog heading. They are
                # separate pieces of curated copy, so the manifest holds both.
                "name": track.get("display_name") or track["name"],
                "stack": track.get("stack", ""),
                "tier": track.get("tier", ""),
                "description": track.get("description", ""),
                "drills": [
                    {
                        "id": drill["id"],
                        "name": drill["name"],
                        "path": drill_path(track, drill["id"]),
                    }
                    for drill in track.get("drills", [])
                ],
            }
        )

    return tracks


def total_drills() -> int:
    return sum(len(t["drills"]) for t in build_curriculum())
