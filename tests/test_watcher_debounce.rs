use notify::{Config, RecommendedWatcher, RecursiveMode, Watcher};
use std::fs;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::channel;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::mpsc;

#[tokio::test]
async fn test_watcher_debouncing_coalesces_rapid_events() {
    let temp_dir = std::env::temp_dir().join("cherenkov_watcher_test");
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

    let tx_clone = tx.clone();
    let thread_handle = tokio::task::spawn_blocking(move || {
        let _watcher = watcher;
        let mut last_event_time: Option<Instant> = None;
        let mut last_path: Option<String> = None;
        let debounce_window = Duration::from_millis(50);

        while running_clone.load(Ordering::Relaxed) {
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
    tokio::time::sleep(Duration::from_millis(100)).await;

    // Rapidly write to the file 5 times within 40ms (each 8ms apart)
    for i in 1..=5 {
        fs::write(&test_file, format!("// edit {}", i)).unwrap();
        tokio::time::sleep(Duration::from_millis(8)).await;
    }

    // Wait for the 50ms debounce window + margin
    tokio::time::sleep(Duration::from_millis(150)).await;

    // Collect all events received
    let mut received_events = Vec::new();
    while let Ok(path) = rx.try_recv() {
        received_events.push(path);
    }

    println!("Received events count after 5 rapid writes: {}", received_events.len());
    for (idx, ev) in received_events.iter().enumerate() {
        println!("Event {}: {}", idx, ev);
    }

    // Exactly 1 coalesced event should be received
    assert_eq!(
        received_events.len(),
        1,
        "Expected exactly 1 coalesced event from 5 rapid saves within 50ms, got {}",
        received_events.len()
    );

    // Now do a second write after the debounce window has closed
    fs::write(&test_file, "// second edit after pause").unwrap();
    tokio::time::sleep(Duration::from_millis(150)).await;

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
