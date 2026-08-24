"""
PRODUCTION STORY:
Monzo Modern Kubernetes Load Migration (2020)
Legacy heavy JVM load test suites required dedicated infrastructure clusters and GUI-driven maintenance.
Migrating to code-first, developer-friendly k6 scripts enabled version-controlled load testing directly inside PR pipelines.
"""

# Drill 02: k6 vs JMeter -- Which Performance Tool to Choose?
#
# Your team needs to run load tests in CI. Read each scenario and
# select the right tool by setting the answer variable.
# The test will pass when you have the correct answers.
#
# Scoring criteria:
#   k6: Code-first, version-controlled, Git-friendly, lightweight, fast CI startup
#   JMeter: GUI-based, enterprise standard, plugin ecosystem, familiar to legacy teams

scenarios = {
    "startup_with_git_workflow_and_modern_ci": None,       # TODO: "k6" or "jmeter"
    "enterprise_bank_team_familiar_with_gui_tools": None,  # TODO: "k6" or "jmeter"
    "test_plan_needs_version_control_in_github": None,     # TODO: "k6" or "jmeter"
    "team_needs_gui_to_record_http_sessions": None,        # TODO: "k6" or "jmeter"
}

def test_tool_selection_scenarios():
    assert scenarios["startup_with_git_workflow_and_modern_ci"] == "k6", "k6 is code-first and Git-friendly"
    assert scenarios["enterprise_bank_team_familiar_with_gui_tools"] == "jmeter", "JMeter is the enterprise standard"
    assert scenarios["test_plan_needs_version_control_in_github"] == "k6", "k6 scripts are .js files, easy to version"
    assert scenarios["team_needs_gui_to_record_http_sessions"] == "jmeter", "JMeter has a built-in HTTP recorder"
