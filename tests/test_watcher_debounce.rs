use notify::{Config, RecommendedWatcher, RecursiveMode, Watcher};
use std::fs;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::channel;
use std::time::{Duration, Instant};
use tokio::sync::mpsc;

/// The debounce window this test drives its own loop with.
///
/// Deliberately wider than the 50ms `src/watcher.rs` ships. The loop polls on a
/// 20ms `recv_timeout`, so at 50ms a single descheduled poll -- routine on a
/// shared CI runner -- pushes `t.elapsed()` past the window mid-burst and the
/// debouncer emits twice. The test would then be reporting on the scheduler
/// rather than on coalescing. The property under test is that a burst well
/// inside the window collapses to one event; the width is what buys the margin
/// to measure it. Five writes 8ms apart span 40ms, comfortably inside 300ms.
const DEBOUNCE_WINDOW: Duration = Duration::from_millis(300);

/// How long to wait for the debouncer to emit: the full window plus margin.
const SETTLE: Duration = Duration::from_millis(600);

/// A scratch path unique to this test-binary run.
///
/// These tests previously shared fixed paths under the system temp directory and
/// cleared them with `remove_dir_all` immediately before writing. On Windows that
/// call can return before the directory is actually released, so a run could race
/// the leftovers of the previous one and fail to create the report tree.
fn unique_temp_path(label: &str) -> std::path::PathBuf {
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0);
    std::env::temp_dir().join(format!("{}_{}_{}", label, std::process::id(), nanos))
}

/// Stops the debounce loop when the test scope unwinds.
///
/// `spawn_blocking` tasks cannot be cancelled, and dropping a runtime waits for
/// the ones already running. So a failed assertion below used to hang the whole
/// test binary rather than report: the panic skipped the line that clears the
/// flag, the loop kept spinning, and the runtime's drop waited on it forever.
/// That is exactly what happened on ubuntu, where this assertion is timing
/// sensitive -- CI sat on one test for five hours and reported nothing.
struct StopOnDrop(Arc<AtomicBool>);

impl Drop for StopOnDrop {
    fn drop(&mut self) {
        self.0.store(false, Ordering::Relaxed);
    }
}

#[tokio::test]
async fn test_watcher_debouncing_coalesces_rapid_events() {
    let temp_dir = unique_temp_path("cherenkov_watcher_test");
    let _ = fs::remove_dir_all(&temp_dir);
    fs::create_dir_all(&temp_dir).unwrap();

    let test_file = temp_dir.join("exercise.ts");
    fs::write(&test_file, "// initial").unwrap();

    let (tx, mut rx) = mpsc::channel::<String>(100);
    let (std_tx, std_rx) = channel();

    let mut watcher = RecommendedWatcher::new(std_tx, Config::default()).unwrap();
    watcher.watch(&temp_dir, RecursiveMode::Recursive).unwrap();

    let running = Arc::new(AtomicBool::new(true));
    let running_clone = running.clone();
    // Covers the panic path only; the happy path still stops the loop explicitly
    // before awaiting the handle, because this guard drops after that await.
    let _stop_on_panic = StopOnDrop(running.clone());

    let tx_clone = tx.clone();
    let thread_handle = tokio::task::spawn_blocking(move || {
        let _watcher = watcher;
        let mut last_event_time: Option<Instant> = None;
        let mut last_path: Option<String> = None;
        let debounce_window = DEBOUNCE_WINDOW;
        // Backstop in case the flag is ever lost: an unbounded loop in a
        // blocking task is what turns a test failure into a hung CI job.
        let deadline = Instant::now() + Duration::from_secs(30);

        while running_clone.load(Ordering::Relaxed) && Instant::now() < deadline {
            match std_rx.recv_timeout(Duration::from_millis(20)) {
                Ok(Ok(event)) => {
                    if (event.kind.is_modify() || event.kind.is_create())
                        && let Some(path) = event.paths.first()
                    {
                        let path_str = path.to_string_lossy().to_string();
                        last_event_time = Some(Instant::now());
                        last_path = Some(path_str);
                    }
                }
                Ok(Err(e)) => eprintln!("Watch error: {:?}", e),
                Err(_) => {
                    if let (Some(t), Some(path)) = (last_event_time, &last_path)
                        && t.elapsed() >= debounce_window
                    {
                        if tx_clone.blocking_send(path.clone()).is_err() {
                            break;
                        }
                        last_event_time = None;
                        last_path = None;
                    }
                }
            }
        }
    });

    // Allow watcher to initialize
    tokio::time::sleep(Duration::from_millis(200)).await;

    // Rapidly write to the file 5 times within 40ms (each 8ms apart)
    for i in 1..=5 {
        fs::write(&test_file, format!("// edit {}", i)).unwrap();
        tokio::time::sleep(Duration::from_millis(8)).await;
    }

    // Wait for the debounce window to close, plus margin
    tokio::time::sleep(SETTLE).await;

    // Collect all events received
    let mut received_events = Vec::new();
    while let Ok(path) = rx.try_recv() {
        received_events.push(path);
    }

    println!(
        "Received events count after 5 rapid writes: {}",
        received_events.len()
    );
    for (idx, ev) in received_events.iter().enumerate() {
        println!("Event {}: {}", idx, ev);
    }

    // Exactly 1 coalesced event should be received
    assert_eq!(
        received_events.len(),
        1,
        "Expected exactly 1 coalesced event from 5 rapid saves inside one window, got {}",
        received_events.len()
    );

    // Now do a second write after the debounce window has closed
    fs::write(&test_file, "// second edit after pause").unwrap();
    tokio::time::sleep(SETTLE).await;

    let mut second_events = Vec::new();
    while let Ok(path) = rx.try_recv() {
        second_events.push(path);
    }

    assert_eq!(
        second_events.len(),
        1,
        "Expected 1 event for isolated save after window, got {}",
        second_events.len()
    );

    running.store(false, Ordering::Relaxed);
    let _ = thread_handle.await;
    let _ = fs::remove_dir_all(&temp_dir);
}
