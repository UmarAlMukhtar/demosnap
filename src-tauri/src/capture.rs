#![allow(dead_code)]

use serde::Serialize;
use std::fs;
use std::io::Write;
use std::path::PathBuf;
use std::process::Command;
use std::process::{Child, ChildStdin, Stdio};
use std::sync::Mutex;
use std::time::{Instant, SystemTime, UNIX_EPOCH};

pub type RecordingState = Mutex<Option<RecordingSession>>;

#[derive(Debug)]
pub struct RecordingSession {
    pub project_dir: PathBuf,
    pub video_path: PathBuf,
    pub manifest_path: PathBuf,
    pub start_time: Instant,
    pub created_at_ms: u64,
    ffmpeg_child: Child,
}

#[derive(Debug, Serialize)]
struct RecordingManifest {
    schema_version: u32,
    created_at_ms: u64,
    updated_at_ms: u64,
    status: String,
    video_file: String,
    duration_ms: Option<u128>,
}

// Start a new recording session and prepare the project folder.
pub fn start() -> Result<RecordingSession, String> {
    let project_dir = get_temp_project_dir();
    fs::create_dir_all(&project_dir).map_err(|error| error.to_string())?;

    let video_path = project_dir.join("video.mp4");
    let ffmpeg_child = spawn_screen_capture(&video_path)?;

    let manifest_path = project_dir.join("project.json");
    let created_at_ms = now_ms();
    write_manifest(
        &manifest_path,
        &RecordingManifest {
            schema_version: 1,
            created_at_ms,
            updated_at_ms: created_at_ms,
            status: "recording".to_string(),
            video_file: "video.mp4".to_string(),
            duration_ms: None,
        },
    )?;

    log::info!("Capture session started at {:?}", project_dir);
    Ok(RecordingSession {
        project_dir,
        video_path,
        manifest_path,
        start_time: Instant::now(),
        created_at_ms,
        ffmpeg_child,
    })
}

// Stop the recording session and finalize the project metadata.
pub fn stop(session: RecordingSession) -> Result<String, String> {
    let duration = session.start_time.elapsed();
    log::info!("Capture Stopped, duration: {:.1}s", duration.as_secs_f32());

    stop_screen_capture(session.ffmpeg_child)?;

    let now = now_ms();
    write_manifest(
        &session.manifest_path,
        &RecordingManifest {
            schema_version: 1,
            created_at_ms: session.created_at_ms,
            updated_at_ms: now,
            status: "stopped".to_string(),
            video_file: session
                .video_path
                .file_name()
                .unwrap_or_default()
                .to_string_lossy()
                .to_string(),
            duration_ms: Some(duration.as_millis()),
        },
    )?;

    Ok(session.project_dir.to_string_lossy().to_string())
}

fn spawn_screen_capture(output_path: &PathBuf) -> Result<Child, String> {
    Command::new("ffmpeg")
        .arg("-y")
        .arg("-f")
        .arg("gdigrab")
        .arg("-framerate")
        .arg("60")
        .arg("-i")
        .arg("desktop")
        .arg("-draw_mouse")
        .arg("1")
        .arg("-vcodec")
        .arg("libx264")
        .arg("-preset")
        .arg("veryfast")
        .arg("-pix_fmt")
        .arg("yuv420p")
        .arg(output_path)
        .stdin(Stdio::piped())
        .stdout(Stdio::null())
        .stderr(Stdio::inherit())
        .spawn()
        .map_err(|error| error.to_string())
}

fn stop_screen_capture(mut child: Child) -> Result<(), String> {
    if let Some(mut stdin) = child.stdin.take() {
        send_ffmpeg_quit(&mut stdin)?;
    }

    let status = child.wait().map_err(|error| error.to_string())?;
    if status.success() {
        Ok(())
    } else {
        Err(format!("ffmpeg exited with status: {status}"))
    }
}

fn send_ffmpeg_quit(stdin: &mut ChildStdin) -> Result<(), String> {
    stdin.write_all(b"q\n").map_err(|error| error.to_string())?;
    stdin.flush().map_err(|error| error.to_string())
}

// Generate a temporary project directory for the recording.
fn get_temp_project_dir() -> PathBuf {
    let tmp = std::env::temp_dir().join("demosnap");
    let timestamp = now_ms();

    tmp.join(format!("Recording_{}.dsnap", timestamp))
}

fn now_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis() as u64
}

fn write_manifest(path: &PathBuf, manifest: &RecordingManifest) -> Result<(), String> {
    let manifest_json =
        serde_json::to_string_pretty(manifest).map_err(|error| error.to_string())?;
    fs::write(path, manifest_json).map_err(|error| error.to_string())
}
