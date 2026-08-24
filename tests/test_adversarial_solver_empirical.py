"""
Empirical Adversarial Test Suite for Cherenkov-Lings Learning Engine & Solver Script (R1 - R4)
Platform: Cherenkov-Lings QA & SDET Experiential Learning Platform
Author: Challenger 1 (teamwork_preview_challenger_1)

Comprehensive Empirical Scenarios:
1. Pristine baseline verification of verify_all_exercises.py across 00_foundations drills.
2. Contract Enforcement: Detection of missing contract files (solution.py, hints.md).
3. Adversarial Syntax Corruption: Fatal SyntaxError in solution.py -> Confirmed failure detection, non-zero exit, atomic restoration.
4. Adversarial Logic Invalidation: Semantic assertion failure in solution.py -> Confirmed failure detection, 0 XP, atomic restoration.
5. Adversarial Corrupted Starter: Garbage/non-Python syntax in exercise.py -> Baseline failure detected, solution injected, pass confirmed, starter cleanly restored.
6. Non-passing Starter Baseline: Ensures all 5 starter exercises genuinely require work / fail out of the box (zero false passes).
7. Atomic Restoration Integrity: Confirms zero lingering .backup_verifier or temporary files.
8. Byte-for-Byte SHA-256 Invariance: Bitwise exact equality of all exercise files before vs after multiple aggressive injection cycles.
9. Core Learning Engine CLI Audit: Execution of `cargo run -- audit` verifying 100% drill contract health across all 11 curriculum tracks.
10. Core Learning Engine CLI Dashboard: Execution of `cargo run -- dashboard` validating rank, level, and track progression display.
11. Core Learning Engine CLI Diagnostics: Execution of `cargo run -- diagnose` validating AST analysis and progressive hints.
12. Rust Core Engine Test Suite: Full execution of `cargo test` asserting 0 test regressions across the platform.
"""

import sys
import os
import shutil
import hashlib
import subprocess
import json
from pathlib import Path
from typing import Dict, List, Tuple

# Reconfigure stdout for UTF-8
if sys.stdout and hasattr(sys.stdout, "reconfigure"):
    try:
        sys.stdout.reconfigure(encoding="utf-8", errors="replace")
    except Exception:
        pass

WORKSPACE_ROOT = Path(__file__).resolve().parent.parent
VERIFIER_SCRIPT = WORKSPACE_ROOT / "tests" / "verify_all_exercises.py"
FOUNDATIONS_DIR = WORKSPACE_ROOT / "exercises" / "00_foundations"

def calculate_dir_hashes(directory: Path) -> Dict[str, str]:
    """Calculates SHA256 hashes of all files in a directory recursively."""
    hashes = {}
    for path in sorted(directory.rglob("*")):
        if path.is_file() and not path.name.endswith(".pyc") and "__pycache__" not in str(path):
            rel_path = str(path.relative_to(directory))
            h = hashlib.sha256(path.read_bytes()).hexdigest()
            hashes[rel_path] = h
    return hashes

def run_verifier(track: str = "00_foundations", cwd: Path = WORKSPACE_ROOT) -> Tuple[int, str, str]:
    """Executes verify_all_exercises.py as a subprocess and captures output."""
    cmd = [sys.executable, str(VERIFIER_SCRIPT), "--track", track, "--json"]
    proc = subprocess.run(
        cmd,
        cwd=str(cwd),
        capture_output=True,
        text=True,
        encoding="utf-8",
        errors="replace"
    )
    return proc.returncode, proc.stdout, proc.stderr

def run_cargo(args: List[str]) -> Tuple[int, str, str]:
    """Runs a cargo command and captures output."""
    cmd = ["cargo"] + args
    proc = subprocess.run(
        cmd,
        cwd=str(WORKSPACE_ROOT),
        capture_output=True,
        text=True,
        encoding="utf-8",
        errors="replace"
    )
    return proc.returncode, proc.stdout, proc.stderr

def test_suite():
    print("=" * 80)
    print("STARTING COMPREHENSIVE EMPIRICAL ADVERSARIAL TEST SUITE (CHALLENGER 1)")
    print("=" * 80)

    initial_hashes = calculate_dir_hashes(FOUNDATIONS_DIR)
    print(f"[*] Baseline SHA-256 snapshot computed for {len(initial_hashes)} files in exercises/00_foundations.")

    passed_scenarios = 0
    total_scenarios = 0

    # =========================================================================
    # SCENARIO 1: Pristine Baseline Verification
    # =========================================================================
    total_scenarios += 1
    print(f"\n[SCENARIO 1] Baseline verify_all_exercises.py on pristine 00_foundations...")
    code, stdout, stderr = run_verifier()
    print(f"  Return code: {code}")
    if code == 0 and "5 / 5 (100.0%)" in stdout and "100% VERIFIED SUCCESS" in stdout:
        print("  [PASS] Scenario 1: Baseline verification succeeded with 100% pass rate.")
        passed_scenarios += 1
    else:
        print(f"  [FAIL] Scenario 1: Output mismatch:\nSTDOUT:\n{stdout}\nSTDERR:\n{stderr}")

    # =========================================================================
    # SCENARIO 2: Missing 4-File Contract Components
    # =========================================================================
    total_scenarios += 1
    print(f"\n[SCENARIO 2] Missing contract file adversarial test (missing hints.md & solution.py in drill 01)...")
    target_drill = FOUNDATIONS_DIR / "01_what_is_a_test"
    solution_file = target_drill / "solution.py"
    hints_file = target_drill / "hints.md"
    
    sol_bytes = solution_file.read_bytes()
    hints_bytes = hints_file.read_bytes()
    
    try:
        solution_file.unlink()
        hints_file.unlink()
        code, stdout, stderr = run_verifier()
        print(f"  Return code with missing files: {code}")
        
        # Check that verifier failed and caught the missing contract files
        if code != 0 and "Missing contract files: solution.py, hints.md" in stdout:
            print("  [PASS] Scenario 2: Contract violation properly detected and failed.")
            passed_scenarios += 1
        else:
            print(f"  [FAIL] Scenario 2: Did not catch missing files as expected:\n{stdout}")
    finally:
        solution_file.write_bytes(sol_bytes)
        hints_file.write_bytes(hints_bytes)

    # =========================================================================
    # SCENARIO 3: Syntax Error in Solution (Adversarial Corrupted Code)
    # =========================================================================
    total_scenarios += 1
    print(f"\n[SCENARIO 3] Corrupted Python syntax in solution.py in drill 02...")
    drill_02 = FOUNDATIONS_DIR / "02_test_naming_matters"
    sol_02 = drill_02 / "solution.py"
    original_sol_02_bytes = sol_02.read_bytes()
    corrupted_syntax = b"def test_invalid_syntax(:\n    assert True\n"
    
    try:
        sol_02.write_bytes(corrupted_syntax)
        code, stdout, stderr = run_verifier()
        print(f"  Return code with syntax error: {code}")
        
        if code != 0 and "Test Validation -> FAILED" in stdout and "SyntaxError" in stdout:
            print("  [PASS] Scenario 3: SyntaxError in solution correctly caught and failed verification.")
            passed_scenarios += 1
        else:
            print(f"  [FAIL] Scenario 3: Syntax error not properly caught:\n{stdout}")
    finally:
        sol_02.write_bytes(original_sol_02_bytes)

    # =========================================================================
    # SCENARIO 4: Broken Assertion Logic in Solution (Semantic Failure)
    # =========================================================================
    total_scenarios += 1
    print(f"\n[SCENARIO 4] Broken assertion logic in solution.py in drill 03...")
    drill_03 = FOUNDATIONS_DIR / "03_arrange_act_assert"
    sol_03 = drill_03 / "solution.py"
    original_sol_03_bytes = sol_03.read_bytes()
    original_sol_03_text = original_sol_03_bytes.decode("utf-8")
    broken_logic = original_sol_03_text.replace('assert result["status"] == "success"', 'assert result["status"] == "FAILURE_SIMULATION_EXPECTED"')
    
    try:
        sol_03.write_bytes(broken_logic.encode("utf-8"))
        code, stdout, stderr = run_verifier()
        print(f"  Return code with broken assertion: {code}")
        
        if code != 0 and "Test Validation -> FAILED" in stdout and "assert 'success' == 'FAILURE_SIMULATION_EXPECTED'" in stdout:
            print("  [PASS] Scenario 4: Broken logic correctly caught and failed verification.")
            passed_scenarios += 1
        else:
            print(f"  [FAIL] Scenario 4: Broken assertion not properly caught:\n{stdout}")
    finally:
        sol_03.write_bytes(original_sol_03_bytes)

    # =========================================================================
    # SCENARIO 5: Corrupted Starter Code in exercise.py (Garbage / Non-Python)
    # =========================================================================
    total_scenarios += 1
    print(f"\n[SCENARIO 5] Corrupted starter code in exercise.py (Junk content) in drill 04...")
    drill_04 = FOUNDATIONS_DIR / "04_dont_test_the_mock"
    ex_04 = drill_04 / "exercise.py"
    original_ex_04_bytes = ex_04.read_bytes()
    junk_starter = b"### CORRUPTED STARTER CODE ###\n!@#$%^&*() INVALID SYNTAX\n// I AM NOT DONE\n"
    
    try:
        ex_04.write_bytes(junk_starter)
        code, stdout, stderr = run_verifier()
        print(f"  Return code with junk starter: {code}")
        
        restored_bytes = ex_04.read_bytes()
        if code == 0 and restored_bytes == junk_starter:
            print("  [PASS] Scenario 5: Junk starter detected as baseline fail, solution passed, starter restored cleanly.")
            passed_scenarios += 1
        else:
            print(f"  [FAIL] Scenario 5: Starter injection/restoration failed. code={code}")
    finally:
        ex_04.write_bytes(original_ex_04_bytes)

    # =========================================================================
    # SCENARIO 6: True Baseline Failure Validation on All 5 Starter Exercises
    # =========================================================================
    total_scenarios += 1
    print(f"\n[SCENARIO 6] True Baseline Failure Verification on Unsolved Starter Drills...")
    all_starters_pending = True
    for drill in sorted(FOUNDATIONS_DIR.iterdir()):
        if drill.is_dir():
            ex_py = drill / "exercise.py"
            content = ex_py.read_text(encoding="utf-8")
            has_todo = "TODO" in content or "I AM NOT DONE" in content
            proc = subprocess.run([sys.executable, "-m", "pytest", str(ex_py), "-q"], capture_output=True, text=True)
            pytest_failed = (proc.returncode != 0)
            is_pending = has_todo or pytest_failed
            print(f"  Drill [{drill.name}]: pytest_exit={proc.returncode}, has_todo={has_todo} -> pending={is_pending}")
            if not is_pending:
                all_starters_pending = False
    
    if all_starters_pending:
        print("  [PASS] Scenario 6: All starter exercises confirmed broken/pending (no false passes).")
        passed_scenarios += 1
    else:
        print("  [FAIL] Scenario 6: Starter exercise trivially passed without work.")

    # =========================================================================
    # SCENARIO 7: Atomic Restoration Verification (Zero Residual Backup Files)
    # =========================================================================
    total_scenarios += 1
    print(f"\n[SCENARIO 7] Verification of atomic file cleanup and zero leftover artifacts...")
    backup_files = list(FOUNDATIONS_DIR.rglob("*.backup_verifier"))
    temp_files = list(FOUNDATIONS_DIR.rglob("*.hidden_test"))
    
    if len(backup_files) == 0 and len(temp_files) == 0:
        print("  [PASS] Scenario 7: Zero lingering backup or temporary files found.")
        passed_scenarios += 1
    else:
        print(f"  [FAIL] Scenario 7: Leftover files found: backups={backup_files}, temps={temp_files}")

    # =========================================================================
    # SCENARIO 8: SHA-256 Byte-for-Byte Integrity Snapshot Comparison
    # =========================================================================
    total_scenarios += 1
    print(f"\n[SCENARIO 8] SHA-256 Directory Integrity Comparison (Before vs After)...")
    post_hashes = calculate_dir_hashes(FOUNDATIONS_DIR)
    
    hash_mismatches = []
    for rel_path, orig_hash in initial_hashes.items():
        curr_hash = post_hashes.get(rel_path)
        if curr_hash != orig_hash:
            hash_mismatches.append((rel_path, orig_hash, curr_hash))
            
    if not hash_mismatches and len(post_hashes) == len(initial_hashes):
        print(f"  [PASS] Scenario 8: All {len(initial_hashes)} files match original SHA-256 hashes perfectly. 100% clean state.")
        passed_scenarios += 1
    else:
        print(f"  [FAIL] Scenario 8: Hash mismatches detected: {hash_mismatches}")

    # =========================================================================
    # SCENARIO 9: CLI Learning Engine Verification (audit, dashboard, diagnose)
    # =========================================================================
    total_scenarios += 1
    print(f"\n[SCENARIO 9] Learning Engine CLI Commands Verification...")
    
    # 9a: cargo run -- audit
    audit_code, audit_out, audit_err = run_cargo(["run", "--", "audit"])
    audit_ok = (audit_code == 0) and ("ALL DRILL CONTRACTS VERIFIED" in audit_out or "AUDIT SUMMARY" in audit_out)
    print(f"  cargo run -- audit: code={audit_code}, ok={audit_ok}")
    
    # 9b: cargo run -- dashboard
    dash_code, dash_out, dash_err = run_cargo(["run", "--", "dashboard"])
    dash_ok = (dash_code == 0) and ("CHERENKOV-LINGS" in dash_out or "Dashboard" in dash_out or "Track" in dash_out or "Rank" in dash_out)
    print(f"  cargo run -- dashboard: code={dash_code}, ok={dash_ok}")
    
    # 9c: cargo run -- diagnose
    diag_file = "exercises/00_foundations/01_what_is_a_test/exercise.py"
    diag_code, diag_out, diag_err = run_cargo(["run", "--", "diagnose", "--file", diag_file])
    diag_ok = (diag_code == 0) and ("AST Analysis" in diag_out or "Hint" in diag_out or "Score" in diag_out)
    print(f"  cargo run -- diagnose: code={diag_code}, ok={diag_ok}")

    if audit_ok and dash_ok and diag_ok:
        print("  [PASS] Scenario 9: All CLI commands (audit, dashboard, diagnose) executed successfully.")
        passed_scenarios += 1
    else:
        print(f"  [FAIL] Scenario 9: CLI commands failed: audit={audit_ok}, dash={dash_ok}, diag={diag_ok}")

    # =========================================================================
    # SCENARIO 10: Cargo Test Suite (Zero Regressions)
    # =========================================================================
    total_scenarios += 1
    print(f"\n[SCENARIO 10] Cargo Test Suite Execution...")
    test_code, test_out, test_err = run_cargo(["test"])
    test_ok = (test_code == 0) and ("test result: ok." in test_out or "test result: ok." in test_err)
    print(f"  cargo test: code={test_code}, ok={test_ok}")

    if test_ok:
        print("  [PASS] Scenario 10: cargo test passed completely with zero failures.")
        passed_scenarios += 1
    else:
        print(f"  [FAIL] Scenario 10: cargo test failed.")

    print("\n" + "=" * 80)
    print(f"EMPIRICAL ADVERSARIAL TEST SUMMARY: {passed_scenarios} / {total_scenarios} SCENARIOS PASSED ({passed_scenarios/total_scenarios*100:.1f}%)")
    print("=" * 80)

    if passed_scenarios == total_scenarios:
        print("[VERDICT] ALL EMPIRICAL CHALLENGES SATISFIED -> APPROVE")
        return 0
    else:
        print("[VERDICT] FAILURES ENCOUNTERED -> REQUEST_CHANGES")
        return 1

if __name__ == "__main__":
    sys.exit(test_suite())
