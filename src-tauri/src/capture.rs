#![allow(dead_code)]

use std::path::PathBuf;

pub struct RecordingSession {
    pub output_path: PathBuf,
    pub start_time: std::time::Instant,
}

pub fn start() -> Result<RecordingSession, String> {
    let output_path = get_temp_path();

    log::info!("Capture Started, output path: {:?}", output_path);

    Ok(RecordingSession {
        output_path,
        start_time: std::time::Instant::now(),
    })
}

pub fn stop(session: RecordingSession) -> Result<String, String> {
    let duration = session.start_time.elapsed();
    log::info!("Capture Stopped, duration: {:.1}s", duration.as_secs_f32());

    Ok(session.output_path.to_string_lossy().to_string())
}

fn get_temp_path() -> PathBuf {
    let tmp = std::env::temp_dir().join("demosnap");
    std::fs::create_dir_all(&tmp).ok();

    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_millis();

    tmp.join(format!("Recording_{}.raw", timestamp))
}
