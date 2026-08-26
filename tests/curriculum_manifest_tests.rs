//! Guards the invariant that `lings.toml` is the single source of truth for the
//! curriculum.
//!
//! The curriculum used to be declared in four places — `lings.toml`, a hardcoded
//! Rust registry, a hardcoded Rust fallback drill list, and a hardcoded Python
//! dict in the FastAPI backend — which silently drifted (the Rust fallback
//! listed 2 GenAI drills when 5 existed on disk). These tests fail if the
//! manifest and the repository ever diverge again.

use cherenkov_lings::config::{self, Config, TrackConfig};
use cherenkov_lings::gamification::{default_curriculum_tracks, discover_track_drills};
use std::collections::HashSet;
use std::path::Path;

fn manifest() -> Config {
    config::load_config("lings.toml").expect("lings.toml must parse")
}

#[test]
fn manifest_parses_and_is_non_empty() {
    let cfg = manifest();
    assert!(
        !cfg.tracks.is_empty(),
        "curriculum manifest declares no tracks"
    );
    for track in &cfg.tracks {
        assert!(
            !track.drills.is_empty(),
            "track '{}' declares no drills in lings.toml",
            track.id
        );
    }
}

#[test]
fn embedded_manifest_matches_on_disk_manifest() {
    let on_disk = manifest();
    let embedded = default_curriculum_tracks();

    assert_eq!(
        on_disk.tracks.len(),
        embedded.len(),
        "compile-time-embedded manifest is stale; rebuild after editing lings.toml"
    );

    for (disk, emb) in on_disk.tracks.iter().zip(embedded.iter()) {
        assert_eq!(
            disk.id, emb.id,
            "track order differs between disk and embedded manifest"
        );
        assert_eq!(
            disk.drills.len(),
            emb.drills.len(),
            "track '{}' drill count differs between disk and embedded manifest",
            disk.id
        );
    }
}

#[test]
fn track_and_drill_ids_are_unique() {
    let cfg = manifest();

    let mut seen_tracks = HashSet::new();
    for track in &cfg.tracks {
        assert!(
            seen_tracks.insert(track.id.clone()),
            "duplicate track id '{}' in lings.toml",
            track.id
        );

        let mut seen_drills = HashSet::new();
        for drill in &track.drills {
            assert!(
                seen_drills.insert(drill.id.clone()),
                "duplicate drill id '{}' in track '{}'",
                drill.id,
                track.id
            );
        }
    }
}

#[test]
fn every_manifest_drill_exists_on_disk() {
    let cfg = manifest();
    let mut missing = Vec::new();

    for track in &cfg.tracks {
        for drill in &track.drills {
            let path = track.drill_path(&drill.id);
            if !Path::new(&path).is_dir() {
                missing.push(format!("{} -> {}", track.id, path));
            }
        }
    }

    assert!(
        missing.is_empty(),
        "manifest declares drills that do not exist on disk:\n  {}",
        missing.join("\n  ")
    );
}

#[test]
fn every_on_disk_drill_is_declared_in_manifest() {
    let cfg = manifest();
    let mut undeclared = Vec::new();

    for track in &cfg.tracks {
        let declared: HashSet<&str> = track.drills.iter().map(|d| d.id.as_str()).collect();
        for found in discover_track_drills(&track.id, &track.exercise_dir) {
            if !declared.contains(found.as_str()) {
                undeclared.push(format!("{} -> {}", track.id, found));
            }
        }
    }

    assert!(
        undeclared.is_empty(),
        "drills exist on disk but are absent from lings.toml (add them to the manifest):\n  {}",
        undeclared.join("\n  ")
    );
}

#[test]
fn every_drill_satisfies_the_four_file_contract() {
    let cfg = manifest();
    let mut violations = Vec::new();

    for track in &cfg.tracks {
        for drill in &track.drills {
            let dir = Path::new(&track.drill_path(&drill.id)).to_path_buf();
            for required in [
                track.exercise_file(),
                track.solution_file(),
                "hints.md".to_string(),
                "theory.md".to_string(),
            ] {
                // The JMeter track ships an executable solution.sh alongside the
                // .jmx plan rather than a solution.jmx.
                let alt_ok = required.ends_with(".jmx") && dir.join("solution.sh").exists();
                if !dir.join(&required).exists() && !alt_ok {
                    violations.push(format!("{}/{} missing {}", track.id, drill.id, required));
                }
            }
        }
    }

    assert!(
        violations.is_empty(),
        "drills violating the 4-file contract:\n  {}",
        violations.join("\n  ")
    );
}

#[test]
fn theory_documents_meet_the_minimum_depth_bar() {
    const MIN_WORDS: usize = 150;
    let cfg = manifest();
    let mut thin = Vec::new();

    for track in &cfg.tracks {
        for drill in &track.drills {
            let theory = Path::new(&track.drill_path(&drill.id)).join("theory.md");
            if let Ok(content) = std::fs::read_to_string(&theory) {
                let words = content.split_whitespace().count();
                if words < MIN_WORDS {
                    thin.push(format!("{}/{}: {} words", track.id, drill.id, words));
                }
            }
        }
    }

    assert!(
        thin.is_empty(),
        "theory.md files below the {MIN_WORDS}-word bar:\n  {}",
        thin.join("\n  ")
    );
}

#[test]
fn drill_root_defaults_to_exercise_dir_and_can_be_overridden() {
    let cfg = manifest();

    let java: &TrackConfig = cfg
        .tracks
        .iter()
        .find(|t| t.id == "restassured-java")
        .expect("restassured-java track must exist");
    assert_ne!(
        java.drill_root(),
        java.exercise_dir,
        "the Maven-layout Java track must override drill_root"
    );
    assert!(Path::new(java.drill_root()).is_dir());

    for track in cfg.tracks.iter().filter(|t| t.drill_root.is_none()) {
        assert_eq!(
            track.drill_root(),
            track.exercise_dir,
            "track '{}' should default drill_root to exercise_dir",
            track.id
        );
    }
}

#[test]
fn every_track_declares_catalog_metadata() {
    let cfg = manifest();
    for track in &cfg.tracks {
        for (field, value) in [
            ("stack", &track.stack),
            ("tier", &track.tier),
            ("description", &track.description),
        ] {
            assert!(
                !value.trim().is_empty(),
                "track '{}' is missing '{}' — GET /api/curriculum renders it",
                track.id,
                field
            );
        }
    }
}
