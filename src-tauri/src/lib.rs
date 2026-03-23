mod office_manager;
mod parser;
mod state_machine;
mod watcher;

use office_manager::{OfficeManager, OfficeState};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Instant;
use tauri::{Emitter, Listener, Manager};

/// Global office state protected by a mutex.
struct AppState {
    office: Mutex<OfficeManager>,
    last_tick: Mutex<Instant>,
}

/// Tauri command: get current office state for rendering.
#[tauri::command]
fn get_office_state(state: tauri::State<'_, AppState>) -> OfficeState {
    let mut office = state.office.lock().unwrap();
    let mut last_tick = state.last_tick.lock().unwrap();

    let now = Instant::now();
    let dt = now.duration_since(*last_tick).as_secs_f64();
    *last_tick = now;

    office.tick(dt)
}

/// Tauri command: end the current session (all agents leave).
#[tauri::command]
fn end_session(state: tauri::State<'_, AppState>) {
    let mut office = state.office.lock().unwrap();
    office.end_session();
}

/// Read terminal app name and pixel width from the geometry file written by the launch hook.
fn read_terminal_geometry() -> Option<(String, u32)> {
    let home = dirs::home_dir()?;
    let path = home.join(".claude").join(".terminal-geometry");
    let content = std::fs::read_to_string(path).ok()?;
    let mut parts = content.trim().splitn(2, ',');
    let app_name = parts.next()?.to_string();
    let width = parts.next()?.parse::<u32>().ok()?;
    // Sanity-check: reject implausible widths
    if (200..=5000).contains(&width) {
        Some((app_name, width))
    } else {
        None
    }
}

/// Query the pixel width of a terminal app's frontmost window via AppleScript.
fn query_terminal_width(app_name: &str) -> Option<u32> {
    let script = format!(
        r#"tell application "System Events"
    try
        return (item 1 of (size of first window of application process "{}")) as text
    end try
end tell"#,
        app_name
    );
    let output = std::process::Command::new("osascript")
        .arg("-e")
        .arg(&script)
        .output()
        .ok()?;
    let width_str = String::from_utf8(output.stdout).ok()?;
    width_str.trim().parse::<u32>().ok()
}

/// Resize a window's width while preserving its height.
fn set_window_width(window: &tauri::WebviewWindow, width: u32) {
    // Default height matches tauri.conf.json
    const DEFAULT_HEIGHT: f64 = 480.0;
    let height = window
        .inner_size()
        .ok()
        .map(|s| {
            let scale = window.scale_factor().unwrap_or(1.0);
            s.height as f64 / scale
        })
        .unwrap_or(DEFAULT_HEIGHT);
    let _ = window.set_size(tauri::Size::Logical(tauri::LogicalSize {
        width: width as f64,
        height,
    }));
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_window_state::Builder::new().build())
        .manage(AppState {
            office: Mutex::new(OfficeManager::new()),
            last_tick: Mutex::new(Instant::now()),
        })
        .setup(|app| {
            let handle = app.handle().clone();
            let shutdown = Arc::new(AtomicBool::new(false));
            let mut _thread_handles: Vec<std::thread::JoinHandle<()>> = Vec::new();

            // --- Terminal width matching ---
            // Read geometry captured by the launch hook and size our window to match.
            // Then poll the terminal app every 5s so we track live resizes.
            // On first launch (no saved window state), apply terminal width immediately.
            // On subsequent launches, the window-state plugin restores position/size,
            // so we skip the initial override and only do live polling.
            if let Some((term_app, term_width)) = read_terminal_geometry() {
                let state_dir = app.path().app_data_dir().ok();
                let has_saved_state = state_dir
                    .map(|d| d.join(".window-state.json").exists())
                    .unwrap_or(false);

                if !has_saved_state {
                    if let Some(window) = app.get_webview_window("main") {
                        set_window_width(&window, term_width);
                    }
                }

                let resize_handle = handle.clone();
                let shutdown_flag = shutdown.clone();
                _thread_handles.push(std::thread::spawn(move || {
                    let mut last_width = term_width;
                    while !shutdown_flag.load(Ordering::Relaxed) {
                        std::thread::sleep(std::time::Duration::from_secs(5));
                        if shutdown_flag.load(Ordering::Relaxed) { break; }
                        if let Some(new_width) = query_terminal_width(&term_app) {
                            if new_width != last_width && (200..=5000).contains(&new_width) {
                                last_width = new_width;
                                if let Some(win) = resize_handle.get_webview_window("main") {
                                    set_window_width(&win, new_width);
                                }
                            }
                        }
                    }
                }));
            }

            // Listen for transcript events and feed them to the office manager.
            // Register BEFORE starting the watcher so replay events aren't missed.
            let event_handle = handle.clone();
            app.listen("transcript_event", move |event: tauri::Event| {
                if let Ok(te) =
                    serde_json::from_str::<parser::TranscriptEvent>(event.payload())
                {
                    let state: tauri::State<'_, AppState> = event_handle.state();
                    let mut office = state.office.lock().unwrap();
                    office.handle_event(te);
                }
            });

            // Start the transcript watcher (replays recently active sessions on startup)
            let watcher_handle = handle.clone();
            let shutdown_watcher = shutdown.clone();
            _thread_handles.push(std::thread::spawn(move || {
                match watcher::TranscriptWatcher::start(watcher_handle) {
                    Ok(_watcher) => {
                        while !shutdown_watcher.load(Ordering::Relaxed) {
                            std::thread::sleep(std::time::Duration::from_secs(60));
                        }
                    }
                    Err(e) => {
                        eprintln!("Failed to start transcript watcher: {}", e);
                    }
                }
            }));

            // Start a tick loop that emits office state to the frontend at ~30fps
            let tick_handle = handle.clone();
            let shutdown_tick = shutdown.clone();
            _thread_handles.push(std::thread::spawn(move || {
                while !shutdown_tick.load(Ordering::Relaxed) {
                    std::thread::sleep(std::time::Duration::from_millis(33));

                    let state: tauri::State<'_, AppState> = tick_handle.state();
                    // Acquire both locks once, compute state, then release both before emitting
                    let office_state = {
                        let mut office = state.office.lock().unwrap();
                        let mut last_tick = state.last_tick.lock().unwrap();

                        let now = Instant::now();
                        let dt = now.duration_since(*last_tick).as_secs_f64();
                        *last_tick = now;

                        office.tick(dt)
                    }; // both locks released here

                    let _ = tick_handle.emit("office_state", &office_state);
                }
            }));

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![get_office_state, end_session])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
