use notify::{Config, RecommendedWatcher, RecursiveMode, Watcher};
use std::path::Path;
use std::sync::mpsc::channel;
use std::time::{Duration, Instant};
use tokio::sync::mpsc;

/// Determines whether a changed file path should be ignored by the watcher.
/// Ignores target/ directory, .class files, temporary files, editor swaps, and hidden dirs.
pub fn should_ignore_path(path: &Path) -> bool {
    let path_str = path.to_string_lossy();
    let normalized = path_str.replace('\\', "/");

    // Ignore build output and target directories
    if normalized.contains("/target/")
        || normalized.starts_with("target/")
        || normalized.ends_with("/target")
        || normalized == "target"
    {
        return true;
    }

    // Ignore compiled bytecode
    if normalized.ends_with(".class") {
        return true;
    }

    // Ignore git metadata
    if normalized.contains("/.git/")
        || normalized.starts_with(".git/")
        || normalized.ends_with("/.git")
    {
        return true;
    }

    // Ignore temporary files and editor swap files
    if normalized.ends_with(".tmp")
        || normalized.ends_with(".temp")
        || normalized.ends_with('~')
        || normalized.ends_with(".swp")
        || normalized.ends_with(".swx")
        || normalized.ends_with(".DS_Store")
        || normalized.ends_with(".bak")
    {
        return true;
    }

    false
}

/// Watch the exercise directory for file saves with a 50ms sliding-window debounce.
/// When a save is detected, sends the changed file path over the tokio channel.
pub async fn watch_exercises(
    exercise_dir: &Path,
    tx: mpsc::Sender<String>,
) -> Result<(), Box<dyn std::error::Error>> {
    let (std_tx, std_rx) = channel();

    // notify requires a sync std channel internally
    let mut watcher = RecommendedWatcher::new(std_tx, Config::default())?;
    watcher.watch(exercise_dir, RecursiveMode::Recursive)?;

    println!("👁  Watching: {:?}", exercise_dir);
    println!("   Save any exercise file to trigger the feedback loop.\n");

    // The debounce loop runs in a blocking thread so we don't starve the tokio executor
    let tx_clone = tx.clone();
    tokio::task::spawn_blocking(move || {
        // Keep watcher alive in this thread
        let _watcher = watcher;
        let mut last_event_time: Option<Instant> = None;
        let mut last_path: Option<String> = None;
        let debounce_window = Duration::from_millis(50);

        loop {
            match std_rx.recv_timeout(debounce_window) {
                Ok(Ok(event)) => {
                    if (event.kind.is_modify() || event.kind.is_create())
                        && let Some(path) = event.paths.first()
                        && !should_ignore_path(path)
                    {
                        let path_str = path.to_string_lossy().to_string();
                        last_event_time = Some(Instant::now());
                        last_path = Some(path_str);
                    }
                }
                // Timeout — check if the debounce window has passed since the last event
                Ok(Err(e)) => eprintln!("Watch error: {:?}", e),
                Err(_timeout) => {
                    // Check if we have a pending event that's past the debounce window
                    if let (Some(t), Some(path)) = (last_event_time, &last_path)
                        && t.elapsed() >= debounce_window
                    {
                        // Debounce window passed — fire the event
                        if tx_clone.blocking_send(path.clone()).is_err() {
                            // Receiver dropped, exit cleanly
                            break;
                        }
                        last_event_time = None;
                        last_path = None;
                    }
                }
            }
        }
    });

    // Wait for Ctrl+C to exit gracefully
    tokio::signal::ctrl_c().await?;
    println!("\n👋  Cherenkov-lings exiting. Keep learning!");

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_should_ignore_path_filters() {
        // Target directory
        assert!(should_ignore_path(Path::new(
            "exercises/02_api_restassured_java/target/classes/App.class"
        )));
        assert!(should_ignore_path(Path::new(
            "target/surefire-reports/TEST-Exercise.xml"
        )));
        assert!(should_ignore_path(Path::new(
            r"exercises\02_api_restassured_java\target\test-classes\Exercise.class"
        )));

        // Bytecode
        assert!(should_ignore_path(Path::new("Exercise.class")));
        assert!(should_ignore_path(Path::new(
            "com/cherenkov/Solution.class"
        )));

        // Temp and editor swap files
        assert!(should_ignore_path(Path::new("Exercise.java.tmp")));
        assert!(should_ignore_path(Path::new("Exercise.java~")));
        assert!(should_ignore_path(Path::new(".Exercise.java.swp")));
        assert!(should_ignore_path(Path::new(".DS_Store")));
        assert!(should_ignore_path(Path::new(".git/HEAD")));

        // Valid source files must NOT be ignored
        assert!(!should_ignore_path(Path::new(
            "exercises/02_api_restassured_java/src/test/java/com/cherenkov/drill01_idempotency/Exercise.java"
        )));
        assert!(!should_ignore_path(Path::new(
            "exercises/02_api_restassured_java/src/test/java/com/cherenkov/drill01_idempotency/Solution.java"
        )));
        assert!(!should_ignore_path(Path::new(
            "exercises/01_web_playwright_ts/01_hydration/exercise.ts"
        )));
    }
}
