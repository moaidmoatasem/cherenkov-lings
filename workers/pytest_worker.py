#!/usr/bin/env python
"""
Pytest Worker for Cherenkov-Lings QA & SDET Platform
Executes Python/pytest exercises using built-in pytest hooks (zero third-party plugins).
Outputs structured DrillResponse JSON to stdout.
"""

import sys
import os
import json
import time
import argparse
import io
from pathlib import Path
from contextlib import redirect_stdout, redirect_stderr

class CherenkovJsonPlugin:
    """In-process pytest hook plugin to capture test metrics without external packages."""
    def __init__(self):
        self.passed = 0
        self.failed = 0
        self.skipped = 0
        self.errors = 0
        self.test_reports = []
        self.start_time = 0.0
        self.duration_ms = 0.0

    def pytest_sessionstart(self, session):
        self.start_time = time.perf_counter()

    def pytest_runtest_logreport(self, report):
        if report.when == "call":
            if report.passed:
                self.passed += 1
            elif report.failed:
                self.failed += 1
                self.test_reports.append({
                    "nodeid": report.nodeid,
                    "outcome": "failed",
                    "duration_ms": report.duration * 1000.0,
                    "message": str(report.longrepr)
                })
            elif report.skipped:
                self.skipped += 1
        elif report.when == "setup" and report.failed:
            self.errors += 1
            self.test_reports.append({
                "nodeid": report.nodeid,
                "outcome": "error",
                "duration_ms": report.duration * 1000.0,
                "message": str(report.longrepr)
            })
        elif report.when == "teardown" and report.failed:
            self.errors += 1
            self.test_reports.append({
                "nodeid": report.nodeid,
                "outcome": "error",
                "duration_ms": report.duration * 1000.0,
                "message": str(report.longrepr)
            })

    def pytest_sessionfinish(self, session, exitstatus):
        self.duration_ms = (time.perf_counter() - self.start_time) * 1000.0

# Backward compatibility alias
PytestCollectorPlugin = CherenkovJsonPlugin


def run_single_iteration(test_file: str, chaos_header: str = "", iteration: int = 1) -> dict:
    """Runs a single iteration of pytest on the target file."""
    import pytest

    if chaos_header:
        os.environ["CHAOS_DIRECTIVES"] = chaos_header
        os.environ["PW_CHAOS_HEADER"] = chaos_header

    # Ensure workspace root is in sys.path
    workspace_root = str(Path(__file__).resolve().parent.parent)
    if workspace_root not in sys.path:
        sys.path.insert(0, workspace_root)

    plugin = CherenkovJsonPlugin()
    args = [
        "-q",
        "--tb=short",
        "-o", "cache_dir=.pytest_cache_tmp",
        test_file
    ]

    captured_stdout = io.StringIO()
    captured_stderr = io.StringIO()

    with redirect_stdout(captured_stdout), redirect_stderr(captured_stderr):
        exit_code = pytest.main(args, plugins=[plugin])

    passed = (exit_code == 0 and plugin.failed == 0 and plugin.errors == 0 and plugin.passed > 0)
    
    error_msg = None
    if not passed:
        if plugin.test_reports:
            error_msg = "\n".join(f"{r['nodeid']}: {r['message']}" for r in plugin.test_reports)
        else:
            err = captured_stderr.getvalue().strip() or captured_stdout.getvalue().strip()
            error_msg = err if err else f"Pytest failed with exit status {exit_code}"

    return {
        "iteration": iteration,
        "passed": passed,
        "duration_ms": max(1, int(round(plugin.duration_ms))),
        "error": error_msg,
        "passed_count": plugin.passed,
        "failed_count": plugin.failed,
        "error_count": plugin.errors
    }


def run_drill(test_file: str, chaos_header: str = "", iterations: int = 1, timeout_ms: int = 30000) -> dict:
    """Executes multi-iteration drill evaluation and builds standard DrillResponse."""
    file_path = Path(test_file)
    if not file_path.exists():
        return {
            "id": "pytest-run",
            "ok": False,
            "passed": False,
            "iterations": iterations,
            "passed_iterations": 0,
            "failed_iterations": iterations,
            "total_duration_ms": 0,
            "runs": [],
            "error": f"Exercise file does not exist: {test_file}"
        }

    iterations = max(1, iterations)
    runs = []
    passed_iterations = 0
    total_start = time.perf_counter()
    first_error = None

    for i in range(1, iterations + 1):
        res = run_single_iteration(str(file_path), chaos_header, iteration=i)
        runs.append({
            "iteration": i,
            "passed": res["passed"],
            "duration_ms": res["duration_ms"],
            "error": res["error"]
        })
        if res["passed"]:
            passed_iterations += 1
        elif first_error is None:
            first_error = res["error"]

    total_duration_ms = int(round((time.perf_counter() - total_start) * 1000.0))
    failed_iterations = iterations - passed_iterations
    all_passed = (passed_iterations == iterations and iterations > 0)

    return {
        "id": "pytest-run",
        "ok": True,
        "passed": all_passed,
        "iterations": iterations,
        "passed_iterations": passed_iterations,
        "failed_iterations": failed_iterations,
        "total_duration_ms": total_duration_ms,
        "runs": runs,
        "error": first_error
    }


def main():
    parser = argparse.ArgumentParser(description="Pytest Worker for Cherenkov-Lings")
    parser.add_argument("file", help="Path to exercise/solution python test file")
    parser.add_argument("--iterations", "-n", type=int, default=1, help="Number of test iterations (default: 1)")
    parser.add_argument("--chaos", default="", help="Chaos directive header string")
    parser.add_argument("--timeout", type=int, default=30000, help="Total execution timeout in ms")
    args = parser.parse_args()

    response = run_drill(args.file, args.chaos, args.iterations, args.timeout)
    print(json.dumps(response))
    sys.exit(0 if response["passed"] else 1)


if __name__ == "__main__":
    main()
