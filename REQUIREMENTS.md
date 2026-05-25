# Demosnap — Project Requirements

This document defines what Demosnap must do, how it should behave, and what the boundaries are. It is the source of truth when there is disagreement about scope.

---

## 1. Overview

Demosnap is a free, open source desktop screen recorder that automatically post-processes recordings to look professionally edited. The user records their screen normally, and Demosnap handles the rest — zooming into clicks, smoothing the cursor, and adding animations — with no manual editing required.

**Target users:** developers, designers, and product people who make demos, tutorials, and product walkthroughs.

**Platforms:** Windows (primary), macOS, Linux (later phases)

---

## 2. Functional Requirements

These are things the software must be able to do.

### 2.1 Recording

| ID | Requirement | Priority |
|---|---|---|
| REC-01 | Record the primary display at up to 60fps | Must have |
| REC-02 | Record a selected region of the screen | Must have |
| REC-03 | Record microphone audio alongside video | Must have |
| REC-04 | Record system audio (what plays through speakers) | Should have |
| REC-05 | Record webcam as a picture-in-picture overlay | Should have |
| REC-06 | Support pause and resume during recording | Must have |
| REC-07 | Show a countdown before recording starts | Nice to have |
| REC-08 | Show a recording indicator while active | Must have |

### 2.2 Mouse Tracking

| ID | Requirement | Priority |
|---|---|---|
| INP-01 | Log every left click with screen coordinates and timestamp | Must have |
| INP-02 | Log every right click with screen coordinates and timestamp | Should have |
| INP-03 | Log continuous cursor movement at 60hz | Must have |
| INP-04 | Store click log alongside raw recording in a project file | Must have |

### 2.3 Auto-Zoom

| ID | Requirement | Priority |
|---|---|---|
| ZOM-01 | Automatically zoom into the cursor position on each left click | Must have |
| ZOM-02 | Zoom in smoothly using an easing curve (not a hard cut) | Must have |
| ZOM-03 | Zoom back out smoothly after a configurable delay | Must have |
| ZOM-04 | Default zoom level: 1.8x | Must have |
| ZOM-05 | User can adjust zoom level per recording | Should have |
| ZOM-06 | User can disable zoom on specific clicks in the editor | Should have |

### 2.4 Cursor

| ID | Requirement | Priority |
|---|---|---|
| CUR-01 | Replace the system cursor with a clean vector cursor in the output | Must have |
| CUR-02 | Smooth the raw cursor path using Bézier interpolation | Must have |
| CUR-03 | Show a click ripple animation on left click | Must have |
| CUR-04 | User can change cursor color | Should have |
| CUR-05 | User can change cursor size | Should have |
| CUR-06 | User can disable cursor replacement and keep system cursor | Should have |

### 2.5 Editor

| ID | Requirement | Priority |
|---|---|---|
| EDT-01 | Show a timeline of the recording with zoom events marked | Must have |
| EDT-02 | Allow trimming the start and end of the recording | Must have |
| EDT-03 | Allow deleting a section from the middle | Should have |
| EDT-04 | Live preview of edits before export | Must have |
| EDT-05 | Allow adding blur masks to hide sensitive screen regions | Should have |
| EDT-06 | Allow changing the background color/image behind the recording | Should have |
| EDT-07 | Add padding and rounded corners to the recording frame | Should have |

### 2.6 Subtitles

| ID | Requirement | Priority |
|---|---|---|
| SUB-01 | Auto-generate subtitles from microphone audio using Whisper | Should have |
| SUB-02 | Subtitles run locally, no internet connection required | Must have (if SUB-01 is built) |
| SUB-03 | Export subtitles as a separate .srt file | Nice to have |
| SUB-04 | Burn subtitles into the video on export | Nice to have |

### 2.7 Export

| ID | Requirement | Priority |
|---|---|---|
| EXP-01 | Export final video as MP4 (H.264) | Must have |
| EXP-02 | Support export resolutions: 1080p, 1440p, 4K | Must have |
| EXP-03 | Support export frame rates: 30fps, 60fps | Must have |
| EXP-04 | Show export progress with estimated time remaining | Must have |
| EXP-05 | Export runs in the background (app stays usable) | Should have |
| EXP-06 | Export as GIF | Nice to have |
| EXP-07 | Export as WebM | Nice to have |

---

## 3. Non-Functional Requirements

These define how the software should behave, not just what it does.

| ID | Requirement |
|---|---|
| NFR-01 | Capture must not drop frames at 60fps on a mid-range machine |
| NFR-02 | App must not use more than 5% CPU while idle (not recording) |
| NFR-03 | App startup time must be under 3 seconds |
| NFR-04 | Export of a 5-minute 1080p recording must complete in under 3 minutes |
| NFR-05 | App must not crash if the user runs out of disk space — show an error |
| NFR-06 | All processing is local — no user data is sent to any server |
| NFR-07 | App must work fully offline |
| NFR-08 | Installer must be under 100MB |

---

## 4. Out of Scope (v1.0)

These will not be built in the first release. Document them here so they don't creep in.

- Cloud storage or video hosting
- Sharing links (v2 feature)
- Team workspaces
- Mobile recording
- Browser extension
- AI-generated voiceover
- Multi-monitor recording (single monitor only for v1)
- Built-in screen annotation / drawing tools

---

## 5. Project File Format

A Demosnap project is a folder with this structure:

```
my-recording.dsnap/
├── video.raw           # raw captured frames
├── audio.wav           # raw microphone audio
├── clicks.json         # timestamped click log
├── webcam.raw          # optional webcam frames
└── project.json        # editor state, zoom settings, trim points
```

`project.json` holds all the user's edits so the project is re-editable after export.

---

## 6. Tech Stack

| Layer | Technology | Reason |
|---|---|---|
| Desktop shell | Tauri v2 | Cross-platform, lightweight, Rust backend |
| Core engine | Rust | Performance, memory safety, system access |
| Screen capture | `scap` crate | Handles DXGI (Win), ScreenCaptureKit (Mac), PipeWire (Linux) |
| Video encoding | FFmpeg via `ffmpeg-next` | Industry standard, handles all formats |
| Mouse tracking | Win32 `SetWindowsHookEx` / CGEventTap | Low-level OS access needed |
| UI | React + TypeScript | Dev C can work in familiar web tech |
| State management | Zustand | Lightweight, simple |
| Subtitles | Whisper.cpp | Local, open source, accurate |

---

## 7. Milestones

| Milestone | Description | Target |
|---|---|---|
| M1 | Record screen + log clicks + export basic MP4 | Month 3 |
| M2 | Auto-zoom on clicks + cursor smoothing + timeline editor | Month 6 |
| M3 | Full feature set, beta ready | Month 9 |
| M4 | Signed installers, docs, public v1.0 release | Month 12 |

---

## 8. Open Questions

Things not yet decided — need a decision before implementation:

- [ ] Do we support multi-monitor in v1 or v2?
- [ ] What is the maximum recording length we support?
- [ ] Do we bundle FFmpeg in the installer or require the user to install it?
- [ ] What Whisper model size do we ship by default (speed vs accuracy tradeoff)?
- [ ] Do we support custom keyboard shortcuts for record/stop in v1?
