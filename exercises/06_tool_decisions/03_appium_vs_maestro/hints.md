# Hints: Drill 03 - Appium vs Maestro

## Hint 1 (Decision Framework)
Ask: "What is the mobile application architecture and team velocity requirement?"
- Modern React Native / Flutter apps with fast CI needs -> Maestro
- Legacy hybrid apps with embedded browser contexts or cross-browser device grids -> Appium

## Hint 2 (Pattern)
- Maestro eliminates WebDriver server architecture, inspecting the accessibility hierarchy directly with built-in auto-retry and declarative YAML.
- Appium adheres to the W3C WebDriver standard, enabling multi-language bindings and deep driver plugin customization.

## Hint 3 (Code Diff)
Set values:
- `react_native_fast_pr_feedback_pipeline`: `"maestro"`
- `legacy_hybrid_app_with_heavy_embedded_webviews`: `"appium"`
- `declarative_yaml_black_box_mobile_flows`: `"maestro"`
- `custom_w3c_multitouch_gestures_on_legacy_device_clouds`: `"appium"`
