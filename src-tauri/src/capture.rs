#![allow(dead_code)]

use serde::{Deserialize, Serialize};
use std::fs;
use std::io::Write;
use std::path::PathBuf;
use std::process::Command;
use std::process::{Child, ChildStdin, Stdio};
use std::sync::Mutex;
use std::time::{SystemTime, UNIX_EPOCH};
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
    pub audio_path: PathBuf,
    pub manifest_path: PathBuf,
    pub capture_region: Option<RecordingRegion>,
    pub created_at_ms: u64,
    pub video_parts: Vec<PathBuf>,
    pub audio_parts: Vec<PathBuf>,
    pub system_audio_path: PathBuf,
    pub system_audio_parts: Vec<PathBuf>,
    pub active_segments: Vec<(u64, Option<u64>)>, // (start_ms, Option<end_ms>)
    pub ffmpeg_child: Option<Child>,              // None if paused (video process)
    pub ffmpeg_audio_child: Option<Child>,        // None if paused/not recording (audio process)
    pub sys_audio_stop_tx: Option<crossbeam_channel::Sender<()>>,
    pub audio_recording_enabled: bool,
    pub sys_audio_recording_enabled: bool,
}

#[derive(Debug, Serialize)]
struct RecordingManifest {
    schema_version: u32,
    created_at_ms: u64,
    updated_at_ms: u64,
    status: String,
    video_file: String,
    audio_file: Option<String>,
    system_audio_file: Option<String>,
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
    let audio_path = project_dir.join("audio.wav");
    let system_audio_path = project_dir.join("system_audio.wav");
    let first_part = project_dir.join("part_0.mp4");
    let first_audio_part = project_dir.join("audio_part_0.wav");
    let first_system_audio_part = project_dir.join("system_audio_part_0.wav");
    
    capture_log(&format!("spawning ffmpeg video -> {:?}", first_part));
    let ffmpeg_child = spawn_screen_capture(&first_part, capture_region)
        .map_err(|e| { capture_log(&format!("spawn_screen_capture FAILED: {e}")); e })?;
    capture_log("spawn_screen_capture OK");

    // Audio capture setup
    let mut ffmpeg_audio_child = None;
    let mut audio_recording_enabled = false;
    let mut audio_parts = Vec::new();

    match crate::audio::get_default_microphone_name() {
        Ok(Some(mic_name)) => {
            log::info!("Recording audio using microphone: {}", mic_name);
            capture_log(&format!("spawning ffmpeg audio -> {:?}", first_audio_part));
            match spawn_audio_capture(&first_audio_part, &mic_name) {
                Ok(child) => {
                    ffmpeg_audio_child = Some(child);
                    audio_recording_enabled = true;
                    audio_parts.push(first_audio_part);
                    capture_log("spawn_audio_capture OK");
                }
                Err(e) => {
                    log::warn!("Failed to spawn audio capture: {}", e);
                    capture_log(&format!("spawn_audio_capture FAILED: {e}"));
                }
            }
        }
        Ok(None) => {
            log::info!("No microphone detected. Proceeding with video-only recording.");
            capture_log("No microphone detected");
        }
        Err(e) => {
            log::warn!("Error querying default microphone: {}", e);
            capture_log(&format!("Error querying default microphone: {e}"));
        }
    }

    // System Audio capture setup
    let mut sys_audio_stop_tx = None;
    let mut sys_audio_recording_enabled = false;
    let mut system_audio_parts = Vec::new();

    match crate::audio::get_default_render_name() {
        Ok(Some(render_name)) => {
            log::info!("Recording system audio using device: {}", render_name);
            capture_log(&format!("spawning cpal sys audio -> {:?}", first_system_audio_part));
            match crate::sys_audio::spawn_system_audio_capture(&first_system_audio_part, &render_name) {
                Ok(child) => {
                    sys_audio_stop_tx = Some(child);
                    sys_audio_recording_enabled = true;
                    system_audio_parts.push(first_system_audio_part);
                    capture_log("spawn_system_audio_capture OK");
                }
                Err(e) => {
                    log::warn!("Failed to spawn system audio capture: {}", e);
                    capture_log(&format!("spawn_system_audio_capture FAILED: {e}"));
                }
            }
        }
        Ok(None) => {
            log::info!("No system audio device detected.");
            capture_log("No system audio device detected");
        }
        Err(e) => {
            log::warn!("Error querying default system audio: {}", e);
            capture_log(&format!("Error querying default system audio: {e}"));
        }
    }

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
            audio_file: if audio_recording_enabled { Some("audio.wav".to_string()) } else { None },
            system_audio_file: if sys_audio_recording_enabled { Some("system_audio.wav".to_string()) } else { None },
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
        audio_path,
        system_audio_path,
        manifest_path,
        capture_region,
        created_at_ms,
        video_parts: vec![first_part],
        audio_parts,
        system_audio_parts,
        active_segments: vec![(created_at_ms, None)],
        ffmpeg_child: Some(ffmpeg_child),
        ffmpeg_audio_child,
        sys_audio_stop_tx,
        audio_recording_enabled,
        sys_audio_recording_enabled,
    })
}

// Pause the recording session by stopping the active ffmpeg capture.
pub fn pause(session: &mut RecordingSession) -> Result<(), String> {
    capture_log("pause() called");
    let child = session.ffmpeg_child.take().ok_or_else(|| "Already paused.".to_string())?;
    stop_screen_capture(child)?;

    if let Some(audio_child) = session.ffmpeg_audio_child.take() {
        if let Err(e) = stop_screen_capture(audio_child) {
            log::warn!("Failed to stop audio capture on pause: {}", e);
        }
    }

    if let Some(sys_audio_tx) = session.sys_audio_stop_tx.take() {
        let _ = sys_audio_tx.send(());
    }

    let now = now_ms();
    if let Some(last_segment) = session.active_segments.last_mut() {
        last_segment.1 = Some(now);
    }

    capture_log(&format!("paused: segment end set to {}", now));
    Ok(())
}

// Resume the recording session by spawning a new ffmpeg part.
pub fn resume(session: &mut RecordingSession) -> Result<(), String> {
    capture_log("resume() called");
    if session.ffmpeg_child.is_some() {
        return Err("Already recording.".to_string());
    }

    let now = now_ms();
    let part_filename = format!("part_{}.mp4", session.video_parts.len());
    let part_path = session.project_dir.join(part_filename);

    capture_log(&format!("spawning ffmpeg for resume -> {:?}", part_path));
    let ffmpeg_child = spawn_screen_capture(&part_path, session.capture_region)
        .map_err(|e| { capture_log(&format!("spawn_screen_capture resume FAILED: {e}")); e })?;
    capture_log("spawn_screen_capture resume OK");

    session.video_parts.push(part_path);

    if session.audio_recording_enabled {
        match crate::audio::get_default_microphone_name() {
            Ok(Some(mic_name)) => {
                let audio_part_filename = format!("audio_part_{}.wav", session.audio_parts.len());
                let audio_part_path = session.project_dir.join(audio_part_filename);
                capture_log(&format!("spawning ffmpeg audio for resume -> {:?}", audio_part_path));
                match spawn_audio_capture(&audio_part_path, &mic_name) {
                    Ok(audio_child) => {
                        session.ffmpeg_audio_child = Some(audio_child);
                        session.audio_parts.push(audio_part_path);
                        capture_log("spawn_audio_capture resume OK");
                    }
                    Err(e) => {
                        log::warn!("Failed to spawn audio capture on resume: {}", e);
                        capture_log(&format!("spawn_audio_capture resume FAILED: {e}"));
                    }
                }
            }
            _ => {
                log::warn!("Audio was enabled, but microphone could not be found or opened on resume.");
            }
        }
    }

    if session.sys_audio_recording_enabled {
        match crate::audio::get_default_render_name() {
            Ok(Some(render_name)) => {
                let sys_audio_part_filename = format!("system_audio_part_{}.wav", session.system_audio_parts.len());
                let sys_audio_part_path = session.project_dir.join(sys_audio_part_filename);
                capture_log(&format!("spawning cpal sys audio for resume -> {:?}", sys_audio_part_path));
                match crate::sys_audio::spawn_system_audio_capture(&sys_audio_part_path, &render_name) {
                    Ok(tx) => {
                        session.sys_audio_stop_tx = Some(tx);
                        session.system_audio_parts.push(sys_audio_part_path);
                        capture_log("spawn_system_audio_capture resume OK");
                    }
                    Err(e) => {
                        log::warn!("Failed to spawn system audio capture on resume: {}", e);
                        capture_log(&format!("spawn_system_audio_capture resume FAILED: {e}"));
                    }
                }
            }
            _ => {
                log::warn!("System audio was enabled, but device could not be found or opened on resume.");
            }
        }
    }

    session.active_segments.push((now, None));
    session.ffmpeg_child = Some(ffmpeg_child);

    Ok(())
}

// Stop the recording session, concat all video parts, shift click event timestamps, and finalize metadata.
pub fn stop(mut session: RecordingSession, raw_clicks: &[ClickEvent]) -> Result<String, String> {
    capture_log("stop() called");
    let now = now_ms();

    // 1. Stop the active ffmpeg processes if they are still running
    if let Some(child) = session.ffmpeg_child.take() {
        stop_screen_capture(child)?;
        if let Some(last_segment) = session.active_segments.last_mut() {
            last_segment.1 = Some(now);
        }
    }

    if let Some(audio_child) = session.ffmpeg_audio_child.take() {
        if let Err(e) = stop_screen_capture(audio_child) {
            log::warn!("Failed to stop audio capture on stop: {}", e);
        }
    }

    if let Some(sys_audio_tx) = session.sys_audio_stop_tx.take() {
        let _ = sys_audio_tx.send(());
    }

    // 2. Filter and shift clicks/hover event timestamps to align with the concatenated video
    let mut adjusted_clicks = Vec::new();
    for click in raw_clicks {
        let t = click.timestamp_ms;
        let mut elapsed_active = 0;

        for (start, end_opt) in &session.active_segments {
            let end = end_opt.unwrap_or(now);
            if t >= *start && t <= end {
                let offset_in_video = elapsed_active + (t - *start);
                let mut adjusted_click = click.clone();
                adjusted_click.timestamp_ms = session.created_at_ms + offset_in_video;
                adjusted_clicks.push(adjusted_click);
                break;
            }
            elapsed_active += end - *start;
        }
    }

    write_click_log(&session.project_dir, &adjusted_clicks)
        .map_err(|e| { capture_log(&format!("write_click_log FAILED: {e}")); e })?;

    // 3. Concat all video parts
    let mut valid_video_parts = Vec::new();
    for p in &session.video_parts {
        if p.exists() && p.metadata().map(|m| m.len()).unwrap_or(0) > 0 {
            valid_video_parts.push(p.clone());
        }
    }

    if valid_video_parts.is_empty() {
        return Err("No video files recorded.".to_string());
    }

    if valid_video_parts.len() == 1 {
        let part_path = &valid_video_parts[0];
        std::fs::rename(part_path, &session.video_path)
            .map_err(|e| format!("Failed to rename part to video.mp4: {e}"))?;
    } else {
        concat_files(&session.project_dir, &valid_video_parts, &session.video_path, "concat_video.txt")?;
    }

    // 3b. Concat all audio parts (if any were recorded)
    let mut valid_audio_parts = Vec::new();
    for p in &session.audio_parts {
        if p.exists() && p.metadata().map(|m| m.len()).unwrap_or(0) > 0 {
            valid_audio_parts.push(p.clone());
        }
    }

    if !valid_audio_parts.is_empty() {
        if valid_audio_parts.len() == 1 {
            let part_path = &valid_audio_parts[0];
            std::fs::rename(part_path, &session.audio_path)
                .map_err(|e| format!("Failed to rename audio part to audio.wav: {e}"))?;
        } else {
            let _ = concat_files(&session.project_dir, &valid_audio_parts, &session.audio_path, "concat_audio.txt");
        }
    }

    let mut valid_sys_audio_parts = Vec::new();
    for p in &session.system_audio_parts {
        if p.exists() && p.metadata().map(|m| m.len()).unwrap_or(0) > 0 {
            valid_sys_audio_parts.push(p.clone());
        }
    }

    if !valid_sys_audio_parts.is_empty() {
        if valid_sys_audio_parts.len() == 1 {
            let part_path = &valid_sys_audio_parts[0];
            let _ = std::fs::rename(part_path, &session.system_audio_path);
        } else {
            let _ = concat_files(&session.project_dir, &valid_sys_audio_parts, &session.system_audio_path, "concat_sys_audio.txt");
        }
    }

    // Calculate total video duration in ms
    let total_duration_ms: u64 = session
        .active_segments
        .iter()
        .map(|(start, end_opt)| end_opt.unwrap_or(now) - *start)
        .sum();

    log::info!("Capture Stopped, total active duration: {:.1}s", (total_duration_ms as f32) / 1000.0);

    // 4. Write finalized manifest
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
            audio_file: if session.audio_path.exists() {
                Some(
                    session
                        .audio_path
                        .file_name()
                        .unwrap_or_default()
                        .to_string_lossy()
                        .to_string(),
                )
            } else {
                None
            },
            system_audio_file: if session.system_audio_path.exists() {
                Some(
                    session
                        .system_audio_path
                        .file_name()
                        .unwrap_or_default()
                        .to_string_lossy()
                        .to_string(),
                )
            } else {
                None
            },
            clicks_file: "clicks.json".to_string(),
            capture_region: session.capture_region,
            duration_ms: Some(total_duration_ms as u128),
        },
    )?;

    // 5. Clean up temporary part files
    for part in &session.video_parts {
        if part.exists() {
            let _ = std::fs::remove_file(part);
        }
    }
    for part in &session.audio_parts {
        if part.exists() {
            let _ = std::fs::remove_file(part);
        }
    }
    for part in &session.system_audio_parts {
        if part.exists() {
            let _ = std::fs::remove_file(part);
        }
    }

    Ok(session.project_dir.to_string_lossy().to_string())
}

fn concat_files(project_dir: &PathBuf, parts: &[PathBuf], output_path: &PathBuf, txt_filename: &str) -> Result<(), String> {
    let concat_txt_path = project_dir.join(txt_filename);
    let mut f = std::fs::File::create(&concat_txt_path)
        .map_err(|e| format!("Failed to create {txt_filename}: {e}"))?;
    
    for part in parts {
        let file_name = part.file_name().ok_or_else(|| "Invalid part filename".to_string())?;
        writeln!(f, "file '{}'", file_name.to_string_lossy())
            .map_err(|e| format!("Failed to write to {txt_filename}: {e}"))?;
    }
    drop(f);

    let ffmpeg_exe = resolve_exe_path("ffmpeg");
    let mut command = Command::new(&ffmpeg_exe);
    command
        .arg("-y")
        .arg("-f")
        .arg("concat")
        .arg("-safe")
        .arg("0")
        .arg("-i")
        .arg(txt_filename)
        .arg("-c")
        .arg("copy")
        .arg(output_path)
        .current_dir(project_dir)
        .stdout(Stdio::null())
        .stderr(Stdio::null());

    #[cfg(target_os = "windows")]
    {
        command.creation_flags(0x0800_0000);
    }

    let status = command.status()
        .map_err(|e| format!("Failed to execute concat command: {e}"))?;

    let _ = std::fs::remove_file(&concat_txt_path);

    if !status.success() {
        return Err(format!("ffmpeg concat failed with status: {status}"));
    }
    Ok(())
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

fn spawn_audio_capture(
    output_path: &PathBuf,
    mic_name: &str,
) -> Result<Child, String> {
    let ffmpeg_exe = resolve_exe_path("ffmpeg");
    capture_log(&format!("ffmpeg exe resolved for audio: {:?}", ffmpeg_exe));

    let mut command = Command::new(&ffmpeg_exe);
    command
        .arg("-y")
        .arg("-f")
        .arg("dshow")
        .arg("-i")
        .arg(format!("audio={}", mic_name))
        .arg("-acodec")
        .arg("pcm_s16le")
        .arg("-ar")
        .arg("44100")
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
        let _ = send_ffmpeg_quit(&mut stdin);
    }

    let _ = child.wait();
    Ok(())
}

fn send_ffmpeg_quit(stdin: &mut ChildStdin) -> Result<(), String> {
    if let Err(e) = stdin.write_all(b"q\n") {
        if e.kind() == std::io::ErrorKind::BrokenPipe {
            return Ok(());
        }
        return Err(e.to_string());
    }
    if let Err(e) = stdin.flush() {
        if e.kind() == std::io::ErrorKind::BrokenPipe {
            return Ok(());
        }
        return Err(e.to_string());
    }
    Ok(())
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

