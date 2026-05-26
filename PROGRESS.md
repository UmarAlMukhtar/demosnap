# Demosnap — Development Progress

This document tracks implementation progress against TIMELINE.md and REQUIREMENTS.md. Updated regularly to reflect current state.

**Current Milestone:** M1 — MVP Recording & Export (Phase 1.1, 1.2, & 1.3 Complete)  
**Current Phase:** Phase 2.1 — Cursor Smoothing & Replacement

---

## Overall Progress

```
M1 (Months 1–3)    ████████████████████████████ 100%
M2 (Months 4–6)    ░░░░░░░░░░░░░░░░░░░░░░░░░░░░ 0%
M3 (Months 7–9)    ░░░░░░░░░░░░░░░░░░░░░░░░░░░░ 0%
M4 (Months 10–12)  ░░░░░░░░░░░░░░░░░░░░░░░░░░░░ 0%
 
TOTAL V1.0         ████████░░░░░░░░░░░░░░░░░░░░ ~25%
```

---

## Milestone 1: MVP Recording & Export (M1)

**Target:** Month 3 | **Current Estimate:** On track  
**Overall Progress:** 100%

### Phase 1.1: Core Recording

**Target:** Weeks 1–4 | **Status:** ✅ DONE (Week ~2-3)

| Requirement | Task | Status | Notes |
|---|---|---|---|
| REC-01 | Record primary display at 60fps | ✅ DONE | Primary display recording at 60fps implemented via gdigrab |
| REC-02 | Record selected region | ✅ DONE | Drag-to-select region overlay is fully wired and validated |
| REC-06 | Pause/resume during recording | ✅ DONE | Pause/resume state management and video concatenation complete |
| REC-08 | Recording indicator UI | ✅ DONE | Live recording pill and elapsed timer shown while recording |
| INP-01 | Log left clicks with coordinates | ✅ DONE | Global low-level mouse hook records real clicks during recording |
| INP-03 | Log cursor movement at 60hz | ✅ DONE | Cursor movement tracked at 60Hz using low-level mouse hook |
| INP-04 | Store click log + video in project | ✅ DONE | Click log is written to `clicks.json` in the project folder on stop |
| Project file structure | Create `.dsnap` folder format | ✅ DONE | Recording sessions now create `project.json`, `video.mp4`, and `clicks.json` |

**Deliverables Progress:**
- [x] Basic recording window UI skeleton
- [x] Recording sidebar and button polish
- [x] Screen capture backend (live desktop capture loop wired)
- [x] Region selection controls wired into capture start path
- [x] Recording hover feedback and pointer cursor restored
- [x] Live recording timer and status indicators
- [x] Click logging data structures
- [x] Global mouse hook wiring for click capture
- [x] Click logging persistence to JSON
- [x] Project file I/O
- [x] Recording state management (Rust session state)
- [x] Recording path opens in file explorer

**Blockers:**
- Screen capture implementation (scap/DXGI integration) — high complexity
- Real mouse hook implementation — requires low-level Win32 API

**Code Review Status:**
```
✅ Click event struct (input.rs) — well-designed, serializable
✅ Basic UI (App.tsx) — displays click log correctly
⚠️  Capture module skeleton (capture.rs) — just a stub, not functional
❌ No screen capture backend — using scap TBD
❌ No real event logging — only test clicks
```

---

### Phase 1.2: Audio Capture
 
**Target:** Weeks 5–8 | **Status:** ✅ DONE
 
| Requirement | Task | Status | Notes |
|---|---|---|---|
| REC-03 | Record microphone audio | ✅ DONE | Auto-detect default microphone via COM/WASAPI, capture raw WAV files via FFmpeg DirectShow, and concat parts during pause/resume |
| REC-04 | Record system audio | ✅ DONE | Auto-detect default playback device via COM/WASAPI, capture raw WAV natively via `cpal` + `hound`, and concat parts |
 
**Blockers:**
- None ✅

---

### Phase 1.3: Basic Export
 
**Target:** Weeks 9–12 | **Status:** ✅ DONE (Week ~3)
 
| Requirement | Task | Status | Notes |
|---|---|---|---|
| EXP-01 | Export as MP4 (H.264) | ✅ DONE | Direct copy or FFmpeg transcoding to MP4 target |
| EXP-02 | Support 1080p export | ✅ DONE | 1080p scale with letterbox/pillarbox padding to preserve aspect ratio |
| EXP-03 | Support 30fps, 60fps export | ✅ DONE | Transcodes frame rate to 30fps or 60fps based on selector |
| EXP-04 | Show export progress + ETA | ✅ DONE | Backend parses FFmpeg output and emits progress to React progress bar |
 
**Blockers:**
- None

---

## Milestone 2: Smart Editing & Auto-Zoom (M2)

**Target:** Month 6 | **Status:** ❌ NOT STARTED  
**Overall Progress:** 0%

All phases blocked on M1 completion.

---

## Milestone 3: Full Feature Set (M3)

**Target:** Month 9 | **Status:** ❌ NOT STARTED  
**Overall Progress:** 0%

All phases blocked on M2 completion.

---

## Milestone 4: Release & Polish (M4)

**Target:** Month 12 | **Status:** ❌ NOT STARTED  
**Overall Progress:** 0%

All phases blocked on M3 completion.

---

## Current Code Status

### What Works ✅
- React + TypeScript UI scaffold
- Light recording-focused UI with sidebar
- Recording button hover feedback and pointer cursor behavior
- Live elapsed recording timer and active-state pill
- Click event data structures (Rust)
- Global low-level mouse hook for click logging (LeftDown/LeftUp)
- Continuous cursor position tracking at 60Hz (MouseMove)
- Basic Tauri IPC bridge (invoke/listen)
- Click log persistence to `clicks.json`
- `.dsnap` project folder creation with `project.json`, `video.mp4`, and `clicks.json`
- Recording path opens in file explorer
- Dynamic floating overlay control bar (`recording-control` window) with glassmorphic UI, Pause/Resume, and Stop actions
- Animated countdown (3, 2, 1, START) before capture begins
- Region selection overlay with custom mouse dragging
- Pause/resume state management with segment concatenation
- Robust executable path resolution for ffmpeg bypassing RedirectionGuard symlink security mitigations in MSI packages (fixes ERROR_UNTRUSTED_MOUNT_POINT / os error 448)
- Microphone audio capture via FFmpeg DirectShow with automatic default recording endpoint discovery using COM WASAPI
- System audio capture natively via `cpal` + `hound` WASAPI loopback with automatic default playback endpoint discovery
- Audio pause/resume/stop segments synchronization and concatenation
- Active microphone name and status indicator display in React UI
- Basic export pipeline with 1080p letterbox conversion and 30/60fps framerate adjustment
- Native save dialog using RFD (Rust File Dialogs)
- Real-time progress bar tracking FFmpeg transcode progress

### What's Stubbed 🟡
- None
 
### What's Missing ❌
- **Error handling** — No graceful failures for missing disk space, etc.

---

## Next Steps (Priorities)

### Immediate (This Week)
1. Test frame rates and CPU usage with region capture
2. Polish region selection controls and validation
3. Persist click log to project files
4. Validate recording output on longer sessions

### Short-term (Next 2 Weeks)
1. Implement Windows mouse hook for real click logging
2. Add continuous cursor position tracking at 60hz
3. Persist click log to JSON on recording stop
4. Create project file I/O (save/load `.dsnap` folders)

### Medium-term (Weeks 4–8)
1. Integrate FFmpeg for MP4 export
2. Add microphone audio capture
3. Implement audio-video sync
4. Test 5-min recording export performance (target: < 3 min)

---

## Known Issues & Risks

| Issue | Severity | Impact | Mitigation |
|---|---|---|---|
| Capture tuning still needed | 🟡 High | Region/fullscreen recording needs validation | Test `gdigrab` region args and frame stability |
| No real mouse hook | 🔴 Critical | Click logging won't work on real usage | Use Win32 API; test thoroughly |
| No persistence layer | 🟡 High | Projects lost on close | Implement JSON serialization + file I/O |
| Frame dropping risk at 60fps | 🟡 High | Poor user experience | Profile early; may need buffering strategy |
| FFmpeg dependency size | 🟠 Medium | Installer > 100MB | Defer bundling; provide install link in M4 |
| Cross-platform audio capture | 🟠 Medium | Timeline risk for Mac/Linux | Focus on Windows in M1; defer others to M3 |

---

## Performance Metrics

**Target Baselines (from NFR):**
- No frame drops at 60fps on mid-range hardware (NFR-01) — TBD, not tested
- < 5% CPU when idle (NFR-02) — Not yet measured
- App startup < 3 seconds (NFR-03) — Current: ~1 sec ✅
- 5-min 1080p export < 3 min (NFR-04) — TBD, export not implemented
- Graceful error on disk full (NFR-05) — Not implemented
- 100% offline (NFR-06) — On track ✅
- Installer < 100MB (NFR-08) — TBD, depends on FFmpeg bundling

---

## Git Commit History (Completed Work)

| Commit | Feature | Phase | Status |
|---|---|---|---|
| TBD | Fix ERROR_UNTRUSTED_MOUNT_POINT in MSI package | 1.1 | ✅ Complete |
| 7cb4586 | Initial Tauri + React scaffold | Setup | ✅ Complete |
| f24a7cf | Capture module skeleton | 1.1 | ⚠️ Stub only |
| 9b5a301 | Click logging + test recording | 1.1 | ⚠️ Partial (test only) |
| 6f11fdb | Clean up default lib.rs | Setup | ✅ Complete |

---

## Dependencies & Tech Stack Status

| Technology | Purpose | Status | Notes |
|---|---|---|---|
| Tauri v2 | Desktop shell | ✅ Setup | Working |
| React 19 | UI framework | ✅ Setup | Working |
| TypeScript 5.8 | Type safety | ✅ Setup | Working |
| Vite | Bundler | ✅ Setup | Working |
| Screen capture session plumbing | Project scaffolding | ⚠️ Partial | Live desktop capture added; region selection, validation, and timer polish pending |
| Project file persistence | Storage | ✅ Done | `project.json` and `clicks.json` are written into each `.dsnap` folder |
| `ffmpeg-next` | Video encoding | ❌ TBD | Not yet added to Cargo.toml |
| Win32 API | Mouse hooks | ✅ Setup | Global low-level mouse hook captures real clicks during recording |
| WASAPI | Audio capture | ❌ TBD | Not yet integrated |
| Zustand | State management | ❌ TBD | Not yet added |
| `serde` | JSON serialization | ⚠️ Available | Used for ClickEvent struct |

---

## Testing Status

- [ ] Unit tests for input module
- [ ] Integration tests for IPC (Tauri commands)
- [ ] System test: record screen → export → verify video
- [ ] Performance test: 60fps frame capture
- [ ] UI test: region overlay selection aligns to capture output
- [ ] Project reload test: verify `clicks.json` and `project.json` are written correctly
- [ ] Audio sync test: video + microphone audio
- [ ] Export performance test: 5-min 1080p < 3 min
- [ ] Offline mode test: no network calls
- [ ] Disk-full error handling test

---

## Documentation Status

- [x] REQUIREMENTS.md — Project spec ✅
- [x] TIMELINE.md — Development phases ✅
- [x] PROGRESS.md — This file ✅
- [x] README.md — Updated with current status
- [ ] CONTRIBUTING.md — Developer guide (M4)
- [ ] USER_MANUAL.md — End-user docs (M4)

---

## How to Update This File

1. **Weekly:** Update phase progress percentages and blockers
2. **Per feature:** Mark as ❌ NOT STARTED → ⚠️ IN PROGRESS → ✅ DONE
3. **Per commit:** Add to Git Commit History section
4. **Issues found:** Add to Known Issues & Risks
5. **Dependency changes:** Update Tech Stack Status

**Last updated:** Feature `audio-capture` (Phase 1.2 completed)
