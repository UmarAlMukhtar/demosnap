mod capture;
mod input;

use std::sync::Mutex;
use tauri::State;
use tauri_plugin_opener::init as opener_init;

#[tauri::command]
fn test_click_log() -> String {
    let log = input::new_click_log();

    input::push_test_click(&log, 100, 200);
    input::push_test_click(&log, 350, 420);
    input::push_test_click(&log, 960, 540);

    let events = log.lock().unwrap();
    format!(
        "Logged {} clicks. First click at ({}, {})",
        events.len(),
        events[0].x,
        events[0].y
    )
}

#[tauri::command]
fn record_click(log: State<'_, input::ClickLog>, x: i32, y: i32) -> Vec<input::ClickEvent> {
    input::push_test_click(log.inner(), x, y);

    log.lock().unwrap().clone()
}

#[tauri::command]
fn get_click_log(log: State<'_, input::ClickLog>) -> Vec<input::ClickEvent> {
    log.lock().unwrap().clone()
}

#[tauri::command]
fn start_recording(
    app: tauri::AppHandle,
    recording: State<'_, capture::RecordingState>,
    click_log: State<'_, input::ClickLog>,
    capture_region: Option<capture::RecordingRegion>,
) -> Result<String, String> {
    let mut active_session = recording.lock().unwrap();

    if active_session.is_some() {
        return Err("A recording is already in progress.".to_string());
    }

    let session = capture::start(&app, capture_region)?;
    click_log.lock().unwrap().clear();
    input::set_active_click_log(Some(click_log.inner().clone()));
    let project_dir = session.project_dir.to_string_lossy().to_string();
    *active_session = Some(session);

    log::info!("Recording started, project dir: {}", project_dir);
    Ok(project_dir)
}

#[tauri::command]
fn stop_recording(
    recording: State<'_, capture::RecordingState>,
    click_log: State<'_, input::ClickLog>,
) -> Result<String, String> {
    let mut active_session = recording.lock().unwrap();
    let session = active_session
        .take()
        .ok_or_else(|| "No active recording session.".to_string())?;

    let clicks = click_log.lock().unwrap().clone();
    input::set_active_click_log(None);
    capture::stop(session, &clicks)
}

#[tauri::command]
fn pause_recording(
    recording: State<'_, capture::RecordingState>,
) -> Result<(), String> {
    let mut active_session = recording.lock().unwrap();
    let session = active_session
        .as_mut()
        .ok_or_else(|| "No active recording session.".to_string())?;

    capture::pause(session)
}

#[tauri::command]
fn resume_recording(
    recording: State<'_, capture::RecordingState>,
) -> Result<(), String> {
    let mut active_session = recording.lock().unwrap();
    let session = active_session
        .as_mut()
        .ok_or_else(|| "No active recording session.".to_string())?;

    capture::resume(session)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let _ = env_logger::try_init();

    let click_log = input::new_click_log();
    input::start_mouse_hook_thread();
    let recording_state: capture::RecordingState = Mutex::new(None);

    tauri::Builder::default()
        .plugin(tauri_plugin_fs::init())
        .plugin(opener_init())
        .manage(click_log)
        .manage(recording_state)
        .invoke_handler(tauri::generate_handler![
            test_click_log,
            record_click,
            get_click_log,
            start_recording,
            stop_recording,
            pause_recording,
            resume_recording
        ])
        .run(tauri::generate_context!())
        .expect("error while running demosnap");
}
