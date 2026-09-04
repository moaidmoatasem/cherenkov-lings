"""Test-suite isolation for learner state.

POST /api/triage/submit awards XP and unlocks badges, and it persists them to
the learner's progress file. Without redirecting that path, running this suite
credited whoever ran it with hundreds of XP and a `first_triage` badge they had
not earned -- the project's own CI silently forging the record the product
calls "evidence, not badges".
"""

import pytest


@pytest.fixture(autouse=True)
def isolated_progress_file(tmp_path, monkeypatch):
    """Point every test at a throwaway progress file."""
    monkeypatch.setenv(
        "CHERENKOV_PROGRESS_FILE", str(tmp_path / ".cherenkov-progress.json")
    )
    yield
