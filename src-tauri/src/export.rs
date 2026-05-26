use rfd::FileDialog;
use std::path::PathBuf;
use std::process::{Command, Stdio};
use tauri::{AppHandle, Emitter};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct ProjectManifest {
    duration_ms: Option<u64>,
}

#[tauri::command]
pub fn select_export_path(default_name: String) -> Option<String> {
    let file_path = FileDialog::new()
        .add_filter("MP4 Video", &["mp4"])
        .set_file_name(&default_name)
        .save_file();
    
    file_path.map(|p| p.to_string_lossy().to_string())
}

#[tauri::command]
pub fn export_recording(
    app_handle: AppHandle,
    project_dir: String,
    output_path: String,
    resolution: String,
    framerate: String,
) -> Result<(), String> {
    let project_path = PathBuf::from(&project_dir);
    let input_video = project_path.join("video.mp4");
    let manifest_path = project_path.join("project.json");

    if !input_video.exists() {
        return Err("Input recording video.mp4 does not exist in the project directory.".to_string());
    }

    // Check for any recorded audio track in the project directory
    let audio_wav = project_path.join("audio.wav");
    let audio_aac = project_path.join("audio.aac");
    let audio_m4a = project_path.join("audio.m4a");
    let audio_mp3 = project_path.join("audio.mp3");
    let audio_path = if audio_wav.exists() {
        Some(audio_wav)
    } else if audio_aac.exists() {
        Some(audio_aac)
    } else if audio_m4a.exists() {
        Some(audio_m4a)
    } else if audio_mp3.exists() {
        Some(audio_mp3)
    } else {
        None
    };

    // 1. Read duration from project.json to calculate progress percentage
    let duration_ms = if manifest_path.exists() {
        if let Ok(content) = std::fs::read_to_string(&manifest_path) {
            if let Ok(manifest) = serde_json::from_str::<ProjectManifest>(&content) {
                manifest.duration_ms.unwrap_or(0)
            } else {
                0
            }
        } else {
            0
        }
    } else {
        0
    };

    // 2. Perform direct copy/mux if output settings match the input video
    if resolution == "Original" && framerate == "60 fps" {
        app_handle.emit("export-progress", 10).ok();
        if let Some(ref audio) = audio_path {
            // Mux video and audio streams instantly using FFmpeg copy
            let ffmpeg_exe = crate::capture::resolve_exe_path("ffmpeg");
            let mut cmd = Command::new(&ffmpeg_exe);
            cmd.arg("-y")
               .arg("-i").arg(&input_video)
               .arg("-i").arg(audio)
               .arg("-c:v").arg("copy")
               .arg("-c:a").arg("aac")
               .arg(&output_path);

            cmd.stdout(Stdio::null())
               .stderr(Stdio::null());

            #[cfg(target_os = "windows")]
            {
                use std::os::windows::process::CommandExt;
                cmd.creation_flags(0x0800_0000);
            }

            let mut child = cmd.spawn().map_err(|e| format!("Failed to spawn FFmpeg for muxing: {e}"))?;
            let status = child.wait().map_err(|e| format!("FFmpeg muxing failed: {e}"))?;
            if !status.success() {
                return Err("FFmpeg audio/video muxing failed.".to_string());
            }
        } else {
            // Direct filesystem copy (no audio present)
            std::fs::copy(&input_video, &output_path)
                .map_err(|e| format!("Failed to copy video to output destination: {e}"))?;
        }
        app_handle.emit("export-progress", 100).ok();
        return Ok(());
    }

    // 3. Otherwise, re-encode using FFmpeg
    let ffmpeg_exe = crate::capture::resolve_exe_path("ffmpeg");
    let mut cmd = Command::new(&ffmpeg_exe);

    cmd.arg("-y")
       .arg("-i")
       .arg(&input_video);

    if let Some(ref audio) = audio_path {
        cmd.arg("-i").arg(audio);
    }

    // Apply scale conversion if 1080p is selected
    if resolution == "1080p (Full HD)" {
        // Scale to fit 1920x1080 and pad with black bars to preserve aspect ratio without stretching/squishing
        cmd.arg("-vf").arg("scale=1920:1080:force_original_aspect_ratio=decrease,pad=1920:1080:(ow-iw)/2:(oh-ih)/2");
    }

    // Apply frame rate conversion
    if framerate == "30 fps" {
        cmd.arg("-r").arg("30");
    } else {
        cmd.arg("-r").arg("60");
    }

    cmd.arg("-c:v").arg("libx264")
       .arg("-preset").arg("veryfast")
       .arg("-pix_fmt").arg("yuv420p");

    if audio_path.is_some() {
        cmd.arg("-c:a").arg("aac");
    }

    cmd.arg(&output_path);

    cmd.stdout(Stdio::null())
       .stderr(Stdio::piped());

    #[cfg(target_os = "windows")]
    {
        use std::os::windows::process::CommandExt;
        cmd.creation_flags(0x0800_0000);
    }

    let mut child = cmd.spawn().map_err(|e| format!("Failed to spawn FFmpeg: {e}"))?;
    let stderr = child.stderr.take().ok_or_else(|| "Failed to capture FFmpeg stderr".to_string())?;

    let reader = std::io::BufReader::new(stderr);
    use std::io::BufRead;

    for line_result in reader.lines() {
        if let Ok(line) = line_result {
            if let Some(current_ms) = parse_ffmpeg_time(&line) {
                if duration_ms > 0 {
                    let progress = (current_ms as f64 / duration_ms as f64 * 100.0).min(99.0) as u32;
                    app_handle.emit("export-progress", progress).ok();
                }
            }
        }
    }

    let status = child.wait().map_err(|e| format!("FFmpeg execution failed: {e}"))?;
    if !status.success() {
        return Err("FFmpeg re-encoding process failed.".to_string());
    }

    app_handle.emit("export-progress", 100).ok();
    Ok(())
}

fn parse_ffmpeg_time(line: &str) -> Option<u64> {
    let time_marker = "time=";
    if let Some(idx) = line.find(time_marker) {
        let start = idx + time_marker.len();
        let sub = &line[start..];
        let time_str = sub.split_whitespace().next()?;
        
        let parts: Vec<&str> = time_str.split(':').collect();
        if parts.len() == 3 {
            let hours: u64 = parts[0].parse().ok()?;
            let minutes: u64 = parts[1].parse().ok()?;
            
            let secs_parts: Vec<&str> = parts[2].split('.').collect();
            if !secs_parts.is_empty() {
                let seconds: u64 = secs_parts[0].parse().ok()?;
                let ms: u64 = if secs_parts.len() > 1 {
                    let ms_str = secs_parts[1];
                    let padded = format!("{:0<3}", ms_str);
                    padded[..3].parse().unwrap_or(0)
                } else {
                    0
                };
                return Some(hours * 3_600_000 + minutes * 60_000 + seconds * 1_000 + ms);
            }
        }
    }
    None
}
