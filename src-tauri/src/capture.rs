#![allow(dead_code)]

use serde::{Deserialize, Serialize};
use std::fs;
use std::io::Write;
use std::path::PathBuf;
use std::process::Command;
use std::process::{Child, ChildStdin, Stdio};
use std::sync::Mutex;
use std::time::{Instant, SystemTime, UNIX_EPOCH};
use tauri::Manager;

#[cfg(target_os = "windows")]
use std::os::windows::process::CommandExt;

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
pub fn start(
    app_handle: &tauri::AppHandle,
    capture_region: Option<RecordingRegion>,
) -> Result<RecordingSession, String> {
    capture_log("start() called");
    let project_dir = get_temp_project_dir(app_handle)?;
    capture_log(&format!("project_dir ready: {:?}", project_dir));

    let video_path = project_dir.join("video.mp4");
    capture_log(&format!("spawning ffmpeg -> {:?}", video_path));
    let ffmpeg_child = spawn_screen_capture(&video_path, capture_region)
        .map_err(|e| { capture_log(&format!("spawn_screen_capture FAILED: {e}")); e })?;
    capture_log("spawn_screen_capture OK");

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
    ).map_err(|e| { capture_log(&format!("write_manifest FAILED: {e}")); e })?;
    capture_log("write_manifest OK");

    write_click_log(&project_dir, &[])
        .map_err(|e| { capture_log(&format!("write_click_log FAILED: {e}")); e })?;
    capture_log("write_click_log OK");

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
    // Resolve the real ffmpeg executable path, bypassing any symlinks.
    // WinGet installs ffmpeg as a <SYMLINK> in AppData\Local\Microsoft\WinGet\Links\,
    // and following that symlink via CreateProcess can fail with os error 448 in
    // the MSI security context. We resolve it to the real path first.
    let ffmpeg_exe = resolve_exe_path("ffmpeg");
    capture_log(&format!("ffmpeg exe resolved: {:?}", ffmpeg_exe));

    let mut command = Command::new(&ffmpeg_exe);
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
        .stderr(Stdio::null());

    #[cfg(target_os = "windows")]
    {
        command.creation_flags(0x0800_0000);
    }

    command.spawn().map_err(|error| error.to_string())
}

fn find_in_dir(dir: &std::path::Path, exe_name: &str, depth: usize) -> Option<PathBuf> {
    if depth > 5 {
        return None;
    }
    if let Ok(entries) = std::fs::read_dir(dir) {
        let mut subdirs = Vec::new();
        for entry in entries.filter_map(Result::ok) {
            let path = entry.path();
            if path.is_dir() {
                subdirs.push(path);
            } else if path.is_file() {
                if let Some(file_name) = path.file_name() {
                    if file_name.to_string_lossy().eq_ignore_ascii_case(exe_name) {
                        return Some(path);
                    }
                }
            }
        }
        for subdir in subdirs {
            if let Some(found) = find_in_dir(&subdir, exe_name, depth + 1) {
                return Some(found);
            }
        }
    }
    None
}

/// Finds an executable by name in PATH and resolves any symlinks/reparse points.
/// This is required because WinGet (and Scoop) install executables as symlinks,
/// and CreateProcess in MSI security context cannot follow those symlinks.
fn resolve_exe_path(name: &str) -> PathBuf {
    #[cfg(target_os = "windows")]
    {
        let exe = format!("{name}.exe");
        if let Ok(path_var) = std::env::var("PATH") {
            for dir in path_var.split(';') {
                let candidate = PathBuf::from(dir.trim()).join(&exe);
                // Use symlink_metadata so we can detect symlinks without following them
                if candidate.symlink_metadata().is_ok() {
                    // Follow the full symlink chain to get the real executable
                    let mut current = candidate.clone();
                    let mut resolved = false;
                    for _ in 0..16 {
                        match std::fs::read_link(&current) {
                            Ok(target) => {
                                current = if target.is_absolute() {
                                    target
                                } else {
                                    current.parent()
                                        .map(|p| p.join(&target))
                                        .unwrap_or(target)
                                };
                                resolved = true;
                            }
                            Err(_) => break,
                        }
                    }
                    if resolved {
                        capture_log(&format!("resolve_exe_path: {:?} -> {:?}", candidate, current));
                        return current;
                    }

                    // Fallback if read_link failed (e.g. blocked by RedirectionGuard in MSI package)
                    capture_log(&format!("resolve_exe_path: read_link failed for {:?}", candidate));
                    let path_str = candidate.to_string_lossy().to_lowercase();
                    if path_str.contains("winget\\links") {
                        if let Ok(local_app_data) = std::env::var("LOCALAPPDATA") {
                            let packages_dir = PathBuf::from(local_app_data)
                                .join("Microsoft")
                                .join("WinGet")
                                .join("Packages");
                            capture_log(&format!("WinGet fallback: searching in {:?}", packages_dir));
                            if let Some(real_path) = find_in_dir(&packages_dir, &exe, 0) {
                                capture_log(&format!("WinGet fallback found: {:?}", real_path));
                                return real_path;
                            }
                        }
                    } else if path_str.contains("scoop\\shims") {
                        if let Ok(user_profile) = std::env::var("USERPROFILE") {
                            let scoop_apps_dir = PathBuf::from(user_profile)
                                .join("scoop")
                                .join("apps");
                            capture_log(&format!("Scoop fallback: searching in {:?}", scoop_apps_dir));
                            if let Some(real_path) = find_in_dir(&scoop_apps_dir, &exe, 0) {
                                capture_log(&format!("Scoop fallback found: {:?}", real_path));
                                return real_path;
                            }
                        }
                    }
                }
            }
        }
    }
    PathBuf::from(name)
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

// Write a diagnostic line to %TEMP%\demosnap_capture.log
fn capture_log(msg: &str) {
    use std::io::Write;
    let path = std::env::temp_dir().join("demosnap_capture.log");
    if let Ok(mut f) = std::fs::OpenOptions::new().create(true).append(true).open(&path) {
        writeln!(f, "{msg}").ok();
    }
}

// Generate a temporary project directory for the recording.
//
// IMPORTANT: We intentionally use the LOCALAPPDATA environment variable instead of
// Tauri's local_data_dir() API. On Windows, Tauri's API can return a path that
// passes through NTFS junction points (reparse points), which triggers
// ERROR_UNTRUSTED_MOUNT_POINT (os error 448) in the security context of MSI-installed
// apps. The %LOCALAPPDATA% env var is set by Windows at login to the real, physical
// path without any junction traversal, making it safe to use directly.
fn get_temp_project_dir(app_handle: &tauri::AppHandle) -> Result<PathBuf, String> {
    // Primary: use LOCALAPPDATA env var (always junction-free on Windows)
    let base_dir = if let Ok(local_app_data) = std::env::var("LOCALAPPDATA") {
        let path = PathBuf::from(local_app_data)
            .join("demosnap")
            .join("Recordings");
        capture_log(&format!("base_dir via LOCALAPPDATA env: {:?}", path));
        path
    } else {
        // Fallback: use Tauri path API (may not work in all MSI contexts)
        let path = app_handle
            .path()
            .local_data_dir()
            .map_err(|e| format!("could not resolve local data dir: {e}"))?
            .join("Demosnap")
            .join("Recordings");
        capture_log(&format!("base_dir via Tauri API (LOCALAPPDATA not set): {:?}", path));
        path
    };

    match std::fs::create_dir_all(&base_dir) {
        Ok(_) => capture_log("create_dir_all(base_dir): OK"),
        Err(e) => {
            capture_log(&format!("create_dir_all(base_dir): FAILED: {e}"));
            return Err(format!("Failed to create recordings dir: {e}"));
        }
    }

    let timestamp = now_ms();
    let project_dir = base_dir.join(format!("Recording_{}.dsnap", timestamp));

    match std::fs::create_dir_all(&project_dir) {
        Ok(_) => capture_log(&format!("project_dir created: {:?}", project_dir)),
        Err(e) => {
            capture_log(&format!("create_dir_all(project_dir): FAILED: {e}"));
            return Err(format!("Failed to create project dir: {e}"));
        }
    }

    capture_log(&format!("returning project_dir: {:?}", project_dir));
    Ok(project_dir)
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
