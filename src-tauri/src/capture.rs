#![allow(dead_code)]

use serde::{Deserialize, Serialize};
use std::fs;
use std::io::Write;
use std::path::PathBuf;
use std::process::Command;
use std::process::{Child, ChildStdin, Stdio};
use std::sync::Mutex;
use std::time::{Instant, SystemTime, UNIX_EPOCH};

use crate::input::ClickEvent;

pub type RecordingState = Mutex<Option<RecordingSession>>;

#[derive(Debug, Clone, Copy, Deserialize, Serialize)]
pub struct RecordingRegion {
    pub x: i32,
    pub y: i32,
    pub width: i32,
    pub height: i32,
}

#[derive(Debug)]
pub struct RecordingSession {
    pub project_dir: PathBuf,
    pub video_path: PathBuf,
    pub manifest_path: PathBuf,
    pub capture_region: Option<RecordingRegion>,
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
    clicks_file: String,
    capture_region: Option<RecordingRegion>,
    duration_ms: Option<u128>,
}

// Start a new recording session and prepare the project folder.
pub fn start(capture_region: Option<RecordingRegion>) -> Result<RecordingSession, String> {
    let project_dir = get_temp_project_dir();
    fs::create_dir_all(&project_dir).map_err(|error| error.to_string())?;

    let video_path = project_dir.join("video.mp4");
    let ffmpeg_child = spawn_screen_capture(&video_path, capture_region)?;

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
            clicks_file: "clicks.json".to_string(),
            capture_region,
            duration_ms: None,
        },
    )?;

    write_click_log(&project_dir, &[])?;

    log::info!("Capture session started at {:?}", project_dir);
    Ok(RecordingSession {
        project_dir,
        video_path,
        manifest_path,
        capture_region,
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
            clicks_file: "clicks.json".to_string(),
            capture_region: session.capture_region,
            duration_ms: Some(duration.as_millis()),
        },
    )?;

    Ok(session.project_dir.to_string_lossy().to_string())
}

fn spawn_screen_capture(
    output_path: &PathBuf,
    capture_region: Option<RecordingRegion>,
) -> Result<Child, String> {
    let mut command = Command::new("ffmpeg");
    let capture_region = capture_region.map(normalize_capture_region);

    command
        .arg("-y")
        .arg("-f")
        .arg("gdigrab")
        .arg("-framerate")
        .arg("60")
        .arg("-draw_mouse")
        .arg("1");

    if let Some(region) = capture_region {
        if region.width <= 0 || region.height <= 0 {
            return Err("Capture region width and height must be greater than zero.".to_string());
        }

        command
            .arg("-offset_x")
            .arg(region.x.to_string())
            .arg("-offset_y")
            .arg(region.y.to_string())
            .arg("-video_size")
            .arg(format!("{}x{}", region.width, region.height));
    }

    command
        .arg("-i")
        .arg("desktop")
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

fn normalize_capture_region(region: RecordingRegion) -> RecordingRegion {
    RecordingRegion {
        x: region.x,
        y: region.y,
        width: region.width - (region.width % 2),
        height: region.height - (region.height % 2),
    }
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

pub fn write_click_log(project_dir: &PathBuf, events: &[ClickEvent]) -> Result<PathBuf, String> {
    let click_log_path = project_dir.join("clicks.json");
    let click_log_json = serde_json::to_string_pretty(events).map_err(|error| error.to_string())?;
    fs::write(&click_log_path, click_log_json).map_err(|error| error.to_string())?;
    Ok(click_log_path)
}
