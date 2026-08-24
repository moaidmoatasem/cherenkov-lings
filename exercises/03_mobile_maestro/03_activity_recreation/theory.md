# Theoretical Context: Activity Recreation & Configuration Changes

## Production Incident: WhatsApp Android Orientation Memory Leak & State Loss (2021)

In 2021, WhatsApp addressed a critical bug on Android devices where rotating the screen during long video composition or media caption entry resulted in lost user drafts, flickering camera freezes, and occasional `OutOfMemoryError` crashes. When users rotated their device between portrait and landscape modes, the Android OS destroyed and recreated the foreground Activity. Because the application had retained uncollected references to previous Activity instances inside static background event buses and failed to serialize draft text into `savedInstanceState` bundles, the newly recreated activity initialized with empty input fields while orphaned activities leaked hundreds of megabytes of bitmap memory.

## The Underlying Mechanism

Android and iOS applications undergo runtime configuration changes during normal device usage:

1. **Configuration Change Triggers**:
   - Screen orientation changes (Portrait $\leftrightarrow$ Landscape)
   - Dark mode toggle
   - Locale / language switching
   - Multi-window split-screen resizing
2. **The Android Activity Destruction Lifecycle**: By default, Android handles configuration changes by destroying the existing `Activity` (`onPause()` $\rightarrow$ `onStop()` $\rightarrow$ `onDestroy()`) and instantiating a completely new `Activity` (`onCreate()` $\rightarrow$ `onStart()` $\rightarrow$ `onResume()`).
3. **State Preservation Invariants**: Any user state not persisted in a `ViewModel`, database, or `onSaveInstanceState(Bundle)` is permanently lost.
4. **Maestro Orientation Testing**: Maestro enables declarative orientation changes (`setOrientation: LANDSCAPE`), allowing automated flows to verify that user input, form state, and UI scroll positions survive configuration recreation cycles.

```
[Activity Recreation Flow During Device Rotation]
User Enters Text: "Important Order Note"
               │
               ▼ (Device Rotated to Landscape)
+─────────────────────────────────────────────+
| Activity onDestroy() [Destroys Old UI View] |
+─────────────────────────────────────────────+
               │
       ┌───────┴──────────────────────────────┐
       ▼                                      ▼
[State NOT Saved]                    [State in ViewModel / Bundle]
New Activity onCreate()              New Activity onCreate()
Input Field is EMPTY! ❌             Restores "Important Order Note" ✅
```

Automating orientation and recreation tests guarantees that mobile apps preserve user data through real-world device rotations and low-memory process kills.

You will now simulate this in the Crucible: test screen rotation and assert state persistence across activity recreation using Maestro.
