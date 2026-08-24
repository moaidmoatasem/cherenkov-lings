import json
import os
import sys
from pathlib import Path
import subprocess

def run_pytest(test_file):
    print(f"Running pytest for {test_file}")
    res = subprocess.run(["python", "-m", "pytest", test_file, "--json-report", "--json-report-file=report.json"], capture_output=True)
    
    if os.path.exists("report.json"):
        with open("report.json", "r") as f:
            data = json.load(f)
            passed = data.get("summary", {}).get("passed", 0) > 0
            failed = data.get("summary", {}).get("failed", 0) > 0
            print(json.dumps({ "passed": passed and not failed }))
    else:
        print(json.dumps({ "passed": False }))

if __name__ == '__main__':
    run_pytest(sys.argv[1])
