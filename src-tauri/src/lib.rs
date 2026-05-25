mod capture;
mod input;

use tauri::State;

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

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    println!("APP STARTING - you should see this");
    env_logger::init();

    let click_log = input::new_click_log();
    
    tauri::Builder::default()
        .manage(click_log)
        .invoke_handler(tauri::generate_handler![
            test_click_log,
            record_click,
            get_click_log
        ])
        .run(tauri::generate_context!())
        .expect("error while running demosnap");
}
