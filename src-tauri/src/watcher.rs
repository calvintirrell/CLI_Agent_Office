use notify::{Config, Event, EventKind, RecommendedWatcher, RecursiveMode, Watcher};
use std::collections::HashMap;
use std::fs;
use std::io::{BufRead, BufReader, Seek, SeekFrom};
use std::path::{Path, PathBuf};
use std::sync::mpsc;
use std::time::{Duration, Instant};
use tauri::{AppHandle, Emitter};

use crate::parser;

/// Tracks read position for each JSONL file so we only process new lines.
struct FileTracker {
    path: PathBuf,
    offset: u64,
    agent_id: String,
    /// The main session agent ID this file belongs to (for parent tracking).
    parent_agent_id: Option<String>,
}

/// Manages watching Claude Code transcript directories for changes.
pub struct TranscriptWatcher {
    _watcher: RecommendedWatcher,
}

impl TranscriptWatcher {
    pub fn start(app_handle: AppHandle) -> Result<Self, Box<dyn std::error::Error>> {
        let claude_dir = get_claude_projects_dir()?;

        let (tx, rx) = mpsc::channel();

        let mut watcher = RecommendedWatcher::new(
            move |res: Result<Event, notify::Error>| {
                if let Ok(event) = res {
                    let _ = tx.send(event);
                }
            },
            Config::default().with_poll_interval(Duration::from_secs(1)),
        )?;

        if claude_dir.exists() {
            watcher.watch(&claude_dir, RecursiveMode::Recursive)?;
        }

        let app = app_handle.clone();
        std::thread::spawn(move || {
            let mut trackers: HashMap<PathBuf, FileTracker> = HashMap::new();

            // Initial scan: register existing files (offset set to EOF).
            // For recently active files, emit synthetic events to populate the office.
            if claude_dir.exists() {
                scan_and_replay_active(&claude_dir, &mut trackers, &app);
            }

            // Periodic cleanup: prune trackers for deleted files every 60 seconds
            let mut last_prune = Instant::now();
            const PRUNE_INTERVAL_SECS: f64 = 60.0;

            loop {
                match rx.recv_timeout(Duration::from_secs(2)) {
                    Ok(event) => {
                        if matches!(
                            event.kind,
                            EventKind::Modify(_) | EventKind::Create(_)
                        ) {
                            for path in &event.paths {
                                if path.extension().map_or(false, |e| e == "jsonl") {
                                    if !trackers.contains_key(path) {
                                        let agent_id = derive_agent_id(path);
                                        let parent_id = derive_parent_agent_id(path);
                                        trackers.insert(
                                            path.clone(),
                                            FileTracker {
                                                path: path.clone(),
                                                offset: 0, // new file: read from start
                                                agent_id,
                                                parent_agent_id: parent_id,
                                            },
                                        );
                                    }
                                    if let Some(tracker) = trackers.get_mut(path) {
                                        process_new_lines(tracker, &app);
                                    }
                                }
                            }
                        }
                    }
                    Err(mpsc::RecvTimeoutError::Timeout) => {
                        let _ = app.emit("idle_check", ());
                    }
                    Err(mpsc::RecvTimeoutError::Disconnected) => break,
                }

                // Prune trackers for files that no longer exist
                if last_prune.elapsed().as_secs_f64() > PRUNE_INTERVAL_SECS {
                    trackers.retain(|path, _| path.exists());
                    last_prune = Instant::now();
                }
            }
        });

        Ok(TranscriptWatcher { _watcher: watcher })
    }
}

fn get_claude_projects_dir() -> Result<PathBuf, Box<dyn std::error::Error>> {
    let home = dirs::home_dir().ok_or("Could not find home directory")?;
    Ok(home.join(".claude").join("projects"))
}

/// Scan for existing JSONL files and register them with offset set to EOF.
/// This ensures we only process new lines written after app launch.
/// No synthetic events are emitted — only the default resident agent should
/// be visible on startup. Live agents appear when new transcript activity occurs.
fn scan_and_replay_active(
    dir: &Path,
    trackers: &mut HashMap<PathBuf, FileTracker>,
    _app: &AppHandle,
) {
    collect_jsonl_files(dir, trackers);
}

/// Recursively collect all JSONL files, setting offset to EOF.
fn collect_jsonl_files(dir: &Path, trackers: &mut HashMap<PathBuf, FileTracker>) {
    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                collect_jsonl_files(&path, trackers);
            } else if path.extension().map_or(false, |e| e == "jsonl") {
                if !trackers.contains_key(&path) {
                    let agent_id = derive_agent_id(&path);
                    let parent_id = derive_parent_agent_id(&path);
                    let offset = fs::metadata(&path).map(|m| m.len()).unwrap_or(0);
                    trackers.insert(
                        path.clone(),
                        FileTracker {
                            path,
                            offset,
                            agent_id,
                            parent_agent_id: parent_id,
                        },
                    );
                }
            }
        }
    }
}

/// Derive agent ID from file path.
fn derive_agent_id(path: &Path) -> String {
    let file_stem = path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("unknown");

    let is_subagent = path
        .parent()
        .and_then(|p| p.file_name())
        .and_then(|n| n.to_str())
        .map_or(false, |n| n == "subagents");

    if is_subagent {
        file_stem.to_string()
    } else {
        format!("main-{}", file_stem)
    }
}

/// Derive the parent agent ID from the directory structure.
/// Subagent files live at: <project>/<session-uuid>/subagents/agent-xxx.jsonl
/// Their parent is: main-<session-uuid>
fn derive_parent_agent_id(path: &Path) -> Option<String> {
    let is_subagent = path
        .parent()
        .and_then(|p| p.file_name())
        .and_then(|n| n.to_str())
        .map_or(false, |n| n == "subagents");

    if is_subagent {
        // Go up from subagents/ to session directory, get its name
        path.parent() // subagents/
            .and_then(|p| p.parent()) // <session-uuid>/
            .and_then(|p| p.file_name())
            .and_then(|n| n.to_str())
            .map(|session_uuid| format!("main-{}", session_uuid))
    } else {
        None
    }
}

/// Read new lines from a JSONL file and emit parsed events.
/// Also emits parent_agent_id info for sub-agents.
fn process_new_lines(tracker: &mut FileTracker, app: &AppHandle) {
    let file = match fs::File::open(&tracker.path) {
        Ok(f) => f,
        Err(_) => return,
    };

    let metadata = match file.metadata() {
        Ok(m) => m,
        Err(_) => return,
    };

    if metadata.len() <= tracker.offset {
        return;
    }

    let mut reader = BufReader::new(file);
    if reader.seek(SeekFrom::Start(tracker.offset)).is_err() {
        return;
    }

    let mut line = String::new();
    let mut first_line = tracker.offset == 0; // is this a brand new file?

    loop {
        line.clear();
        match reader.read_line(&mut line) {
            Ok(0) => break,
            Ok(n) => {
                tracker.offset += n as u64;
                let trimmed = line.trim();
                if !trimmed.is_empty() {
                    let mut events =
                        parser::parse_line(trimmed, &tracker.agent_id);

                    // If this is the first line of a subagent file, inject parent info
                    if first_line {
                        if let Some(ref parent_id) = tracker.parent_agent_id {
                            events.push(parser::TranscriptEvent::SubAgentStarted {
                                agent_id: tracker.agent_id.clone(),
                                parent_agent_id: parent_id.clone(),
                                timestamp: String::new(),
                            });
                        }
                        first_line = false;
                    }

                    for event in events {
                        let _ = app.emit("transcript_event", &event);
                    }
                }
            }
            Err(_) => break,
        }
    }
}
