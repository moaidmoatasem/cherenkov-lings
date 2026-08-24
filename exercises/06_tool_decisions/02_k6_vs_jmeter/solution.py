scenarios = {
    "startup_with_git_workflow_and_modern_ci": "k6",
    "enterprise_bank_team_familiar_with_gui_tools": "jmeter",
    "test_plan_needs_version_control_in_github": "k6",
    "team_needs_gui_to_record_http_sessions": "jmeter",
}

def test_tool_selection_scenarios():
    assert scenarios["startup_with_git_workflow_and_modern_ci"] == "k6"
    assert scenarios["enterprise_bank_team_familiar_with_gui_tools"] == "jmeter"
    assert scenarios["test_plan_needs_version_control_in_github"] == "k6"
    assert scenarios["team_needs_gui_to_record_http_sessions"] == "jmeter"
