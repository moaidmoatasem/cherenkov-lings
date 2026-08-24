# Drill 03: Solution -- Appium vs Maestro Mobile Automation Decision Framework
# Rule of thumb:
# Use Maestro for: Modern Native, React Native, Flutter, declarative CI flows, fast PR validation.
# Use Appium for: Complex hybrid/webview apps, legacy Selenium grids, fine-grained custom driver plugins.

scenarios = {
    "react_native_fast_pr_feedback_pipeline": "maestro",
    "legacy_hybrid_app_with_heavy_embedded_webviews": "appium",
    "declarative_yaml_black_box_mobile_flows": "maestro",
    "custom_w3c_multitouch_gestures_on_legacy_device_clouds": "appium",
}

def test_mobile_tool_decisions():
    assert scenarios["react_native_fast_pr_feedback_pipeline"] == "maestro", (
        "Maestro runs 10x faster with built-in tolerance for React Native / Flutter apps"
    )
    assert scenarios["legacy_hybrid_app_with_heavy_embedded_webviews"] == "appium", (
        "Appium excels at switching between NATIVE_APP and WEBVIEW contexts"
    )
    assert scenarios["declarative_yaml_black_box_mobile_flows"] == "maestro", (
        "Maestro uses declarative YAML with zero driver setup"
    )
    assert scenarios["custom_w3c_multitouch_gestures_on_legacy_device_clouds"] == "appium", (
        "Appium supports fine-grained W3C Actions and broad device cloud vendor ecosystems"
    )
