# Demosnap — Development Timeline

This timeline breaks down the REQUIREMENTS.md into phases, showing which features should be built and by when. The project targets a 12-month v1.0 release across 4 milestones.

---

## Milestone 1: MVP Recording & Export (Month 1–3)

**Target Release:** End of Month 3  
**Goal:** Functional recording + basic post-processing pipeline

### Phase 1.1: Core Recording (Weeks 1–4)

**Must-have features:**
- REC-01: Record primary display at 60fps
- REC-02: Record selected region of the screen
- REC-06: Pause/resume during recording
- REC-08: Recording indicator UI
- INP-01: Log left clicks with coordinates and timestamps
- INP-03: Log continuous cursor movement at 60hz
- INP-04: Store click log + raw video in project file

**Deliverables:**
- Basic recording window with region selection UI
- Screen capture backend (using `scap` crate)
- Click/movement logging to JSON
- Project file structure created (`.dsnap` folder)
- Recording state management (Zustand)

**Non-functional targets:**
- NFR-07: Works fully offline
- NFR-06: No data leaves the machine

---

### Phase 1.2: Audio Capture (Weeks 5–8)

**Must-have features:**
- REC-03: Record microphone audio alongside video
- Audio file stored in project alongside video

**Should-have features:**
- REC-04: Record system audio (defer if complex)

**Deliverables:**
- Microphone audio capture using WASAPI (Windows) / Core Audio (Mac)
- Audio synced with video frames
- Raw `.wav` file stored in project

**Note:** System audio capture deferred to Phase 1.3 if timeline pressure

---

### Phase 1.3: Basic Export (Weeks 9–12)

**Must-have features:**
- EXP-01: Export as MP4 (H.264)
- EXP-02: Support 1080p export resolution
- EXP-03: Support 30fps and 60fps export
- EXP-04: Show export progress + ETA

**Deliverables:**
- FFmpeg integration via `ffmpeg-next` crate
- Video + audio mux pipeline
- Export progress dialog
- Basic error handling for disk space (NFR-05)

**Performance targets:**
- 5-minute 1080p export completes in < 3 minutes (NFR-04)
- CPU overhead < 5% when idle (NFR-02)
- Installer < 100MB (NFR-08)

---

## Milestone 2: Smart Editing & Auto-Zoom (Month 4–6)

**Target Release:** End of Month 6  
**Goal:** Click-aware editing with cursor enhancement

### Phase 2.1: Cursor Smoothing & Replacement (Weeks 13–16)

**Must-have features:**
- CUR-01: Replace system cursor with clean vector cursor
- CUR-02: Smooth cursor path using Bézier interpolation
- CUR-03: Show click ripple animation on left click

**Deliverables:**
- Vector cursor renderer (SVG or canvas-based)
- Bézier curve interpolation on raw mouse coordinates
- Ripple animation timing synced to click log
- Re-encode video with new cursor overlay

**Dependencies:** Phase 1.3 export pipeline complete

---

### Phase 2.2: Auto-Zoom on Clicks (Weeks 17–20)

**Must-have features:**
- ZOM-01: Auto-zoom into cursor position on left click
- ZOM-02: Smooth zoom using easing curve
- ZOM-03: Smooth zoom-out after configurable delay
- ZOM-04: Default 1.8x zoom level

**Deliverables:**
- Zoom keyframe generator based on click log
- FFmpeg filter graph for smooth zoom transitions
- Configurable zoom duration and easing function
- Re-encode video with zoom effects baked in

**Dependencies:** Phase 2.1 complete + click log (Phase 1.1)

---

### Phase 2.3: Timeline Editor (Weeks 21–24)

**Must-have features:**
- EDT-01: Timeline UI showing zoom events
- EDT-02: Trim start/end of recording
- EDT-04: Live preview of edits

**Should-have features:**
- EDT-03: Delete middle sections (defer to M3 if needed)

**Deliverables:**
- Timeline scrubber UI in React
- Trim point selection + preview
- Frame-accurate playback window
- Edit state persisted to `project.json`

**Dependencies:** Phase 2.2 complete + video preview infrastructure

---

## Milestone 3: Full Feature Set (Month 7–9)

**Target Release:** End of Month 9  
**Goal:** Feature-complete beta ready for user testing

### Phase 3.1: Optional Recording Features (Weeks 25–28)

**Should-have features:**
- REC-04: System audio capture (if deferred from M1)
- REC-05: Webcam PIP overlay
- REC-07: Countdown before recording starts

**Deliverables:**
- System audio capture backend (WASAPI-loopback / Core Audio)
- Webcam frame capture + compositing
- Countdown timer UI

---

### Phase 3.2: Advanced Editor (Weeks 29–32)

**Should-have features:**
- EDT-03: Delete middle sections
- EDT-05: Blur masks for sensitive regions
- EDT-06: Background color/image behind recording
- EDT-07: Padding and rounded corners

**Deliverables:**
- Section deletion with audio/video re-sync
- Mask editor UI with Bezier path drawing
- Background renderer (solid color, image, or blur)
- Frame compositing pipeline

---

### Phase 3.3: Subtitle Support (Weeks 33–36)

**Should-have features:**
- SUB-01: Auto-generate subtitles from audio via Whisper
- SUB-02: Local processing (no internet)
- SUB-03: Export as `.srt` file (nice-to-have)
- SUB-04: Burn subtitles into video (nice-to-have)

**Deliverables:**
- Whisper.cpp integration for local transcription
- Subtitle sync UI in timeline editor
- SRT generation and video re-encode with burned subs
- Model download + caching

---

### Phase 3.4: Optional Cursor Features (Weeks 37–40)

**Should-have features:**
- CUR-04: Cursor color customization
- CUR-05: Cursor size customization
- CUR-06: Toggle cursor replacement off

**Deliverables:**
- Cursor settings panel
- Re-encode with new cursor styling

---

### Phase 3.5: Advanced Zoom & Export (Weeks 41–44)

**Should-have features:**
- ZOM-05: Adjust zoom level per recording
- ZOM-06: Disable zoom on specific clicks
- EXP-05: Background export
- EXP-06: GIF export (nice-to-have)
- EXP-07: WebM export (nice-to-have)

**Deliverables:**
- Per-click zoom override UI
- Background task queue for export
- GIF/WebM encoding pipelines

---

## Milestone 4: Release & Polish (Month 10–12)

**Target Release:** End of Month 12  
**Goal:** Production-ready v1.0 with marketing materials

### Phase 4.1: Performance & Stability (Weeks 45–48)

**Focus:**
- Achieve non-functional requirements (NFR-01 through NFR-08)
- Frame drop testing at 60fps on mid-range hardware
- Memory leak detection
- Crash handler + error reporting
- Disk-full error handling

**Deliverables:**
- Performance benchmark suite
- Profiling reports + optimizations
- Error telemetry (if privacy-approved)
- Graceful degradation docs

---

### Phase 4.2: Installer & Distribution (Weeks 49–51)

**Focus:**
- Signed installers (Windows, Mac, Linux)
- Auto-update mechanism
- FFmpeg bundling or installation guide
- Installer size < 100MB

**Deliverables:**
- MSIX / DMG / AppImage installers
- Code signing certificates
- Installation docs
- Uninstall cleanup

---

### Phase 4.3: Documentation & Marketing (Weeks 52–52)

**Deliverables:**
- User manual (quick start, tutorials)
- FAQ and troubleshooting guide
- System requirements doc
- Demo video walkthrough
- Release notes
- Contributing guide (for open source community)

---

## Feature Priority Map

### Phase 1 (M1) — Foundation
| Feature | Module | Priority |
|---------|--------|----------|
| Screen capture | Capture | Must |
| Region selection | UI | Must |
| Click logging | Input | Must |
| Cursor tracking | Input | Must |
| Microphone audio | Audio | Must |
| Basic export | Export | Must |
| Project file format | Storage | Must |

### Phase 2 (M2) — Polish
| Feature | Module | Priority |
|---------|--------|----------|
| Cursor smoothing | Render | Must |
| Cursor replacement | Render | Must |
| Click ripple | Render | Must |
| Auto-zoom | Editor | Must |
| Timeline editor | UI | Must |
| Trim editing | Editor | Must |
| Live preview | UI | Must |

### Phase 3 (M3) — Features
| Feature | Module | Priority |
|---------|--------|----------|
| Webcam PIP | Capture | Should |
| System audio | Audio | Should |
| Section deletion | Editor | Should |
| Blur masks | Editor | Should |
| Background styling | Render | Should |
| Subtitles (Whisper) | Audio | Should |
| Cursor customization | Render | Should |
| Per-click zoom | Editor | Should |
| GIF export | Export | Nice |
| WebM export | Export | Nice |

### Phase 4 (M4) — Release
| Task | Category |
|------|----------|
| Performance tuning | Engineering |
| Installer signing | Engineering |
| Documentation | Content |
| Launch marketing | Marketing |

---

## Dependency Graph

```
Phase 1.1: Recording + Click Logging
  ↓
Phase 1.2: Audio Capture
  ↓
Phase 1.3: Basic Export (requires 1.1 + 1.2)
  ↓
Phase 2.1: Cursor Smoothing (requires 1.3)
  ↓
Phase 2.2: Auto-Zoom (requires 2.1 + 1.1)
  ↓
Phase 2.3: Timeline Editor (requires 2.2)
  ↓
Phase 3.x: Optional features (requires M2)
  ↓
Phase 4.x: Release prep (requires M3)
```

---

## Risk & Mitigation

| Risk | Impact | Mitigation |
|------|--------|-----------|
| FFmpeg integration delays | Blocks M1 export | Start early, use known version |
| Cross-platform audio capture | Delays M1 | Windows-only in M1, macOS/Linux in M3 |
| 60fps frame dropping on weak hardware | Blocks M4 | Use GPU-accelerated desktop duplication (`ddagrab`) and ultrafast encoding preset |
| Whisper model download size | Exceeds 100MB installer | Don't bundle; download on first use |
| Subtitle sync accuracy | User-facing quality | Manual sync adjustment UI + testing |

---

## Success Metrics

- [ ] M1: Can record 5-min video + export MP4 without crashes
- [ ] M2: Auto-zoom produces smooth, professional-looking output
- [ ] M3: Feature-complete, ready for closed beta
- [ ] M4: v1.0 stable, installers signed, docs published

---

## Assumptions

1. **Windows first:** M1–M3 target Windows only; macOS/Linux in M4 or post-launch
2. **Single monitor:** v1 does not support multi-monitor setups
3. **FFmpeg bundled or pre-installed:** Installer bundles a customized, stripped-down FFmpeg sidecar binary to keep size minimal (~15–20MB)
4. **Whisper runs locally:** No cloud API calls; model downloaded at first use
5. **Maximum recording length:** TBD (see Open Questions in REQUIREMENTS.md)

---

## Production Capture & Export Evolution Roadmap

To transition Demosnap from a functional MVP to a production-grade commercial release, the media pipeline will evolve as follows:

### Phase 1: Custom FFmpeg Sidecar (Milestone 4 / Initial Public Release)
- **Architecture:** Continue spawning FFmpeg as a background subprocess, but package it as a Tauri Sidecar.
- **Optimization:** Compile a custom, stripped-down version of FFmpeg containing only the codecs and devices needed (e.g. `ddagrab`, `libx264`, `aac`, `amix`, `concat`), reducing executable size from 100MB+ to ~15–20MB.

### Phase 2: Direct C-Library Linking & Native Capture (Post-1.0 Commercial Release)
- **Architecture:** Move away from subprocess execution entirely. Link the Rust backend directly to the FFmpeg shared libraries (`libavcodec`, `libavformat`, etc.) using dynamic linking.
- **Native Capture:** Replace FFmpeg screen grabbing with direct OS-native capture APIs (e.g., `Windows.Graphics.Capture` via the `windows-rs` crate on Windows, and `ScreenCaptureKit` on macOS).
- **Encoding Pipeline:** Feed raw frame/audio buffers directly from the native capture loops into the linked FFmpeg encoder in memory. This eliminates spawned process management and provides maximum performance and crash resilience.
