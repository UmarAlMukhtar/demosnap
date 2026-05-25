# Demosnap — Development Progress

This document tracks implementation progress against TIMELINE.md and REQUIREMENTS.md. Updated regularly to reflect current state.

**Last Updated:** 2026-05-25  
**Current Milestone:** M1 — MVP Recording & Export  
**Current Phase:** Phase 1.1 (Partial) — Core Recording

---

## Overall Progress

```
M1 (Months 1–3)    ████░░░░░░░░░░░░░░░░░░░░░░░ ~25%
M2 (Months 4–6)    ░░░░░░░░░░░░░░░░░░░░░░░░░░░░ 0%
M3 (Months 7–9)    ░░░░░░░░░░░░░░░░░░░░░░░░░░░░ 0%
M4 (Months 10–12)  ░░░░░░░░░░░░░░░░░░░░░░░░░░░░ 0%

TOTAL V1.0         █░░░░░░░░░░░░░░░░░░░░░░░░░░░ ~6%
```

---

## Milestone 1: MVP Recording & Export (M1)

**Target:** Month 3 | **Current Estimate:** On track  
**Overall Progress:** 25%

### Phase 1.1: Core Recording

**Target:** Weeks 1–4 | **Status:** 🟡 IN PROGRESS (Week ~2–3)

| Requirement | Task | Status | Notes |
|---|---|---|---|
| REC-01 | Record primary display at 60fps | ❌ NOT STARTED | Needs `scap` integration + DXGI capture |
| REC-02 | Record selected region | ❌ NOT STARTED | Depends on capture impl + region UI |
| REC-06 | Pause/resume during recording | ❌ NOT STARTED | State management needed |
| REC-08 | Recording indicator UI | ⚠️ PARTIAL | Basic indicator exists; needs polish |
| INP-01 | Log left clicks with coordinates | ✅ DONE | Test click logging working; need real hook |
| INP-03 | Log cursor movement at 60hz | ❌ NOT STARTED | Requires Win32 `SetWindowsHookEx` |
| INP-04 | Store click log + video in project | ⚠️ PARTIAL | Click log in memory; persistence not implemented |
| Project file structure | Create `.dsnap` folder format | ❌ NOT STARTED | Need file I/O + `project.json` writer |

**Deliverables Progress:**
- [x] Basic recording window UI skeleton
- [ ] Screen capture backend (scap crate integration)
- [x] Click logging data structures
- [ ] Click logging persistence to JSON
- [ ] Project file I/O
- [ ] Recording state management (Zustand setup)

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

**Target:** Weeks 5–8 | **Status:** ❌ NOT STARTED

| Requirement | Task | Status |
|---|---|---|
| REC-03 | Record microphone audio | ❌ NOT STARTED |
| REC-04 | Record system audio (deferred) | ❌ DEFERRED |

**Blockers:**
- Phase 1.1 recording must work first
- WASAPI / Core Audio integration TBD

---

### Phase 1.3: Basic Export

**Target:** Weeks 9–12 | **Status:** ❌ NOT STARTED

| Requirement | Task | Status |
|---|---|---|
| EXP-01 | Export as MP4 (H.264) | ❌ NOT STARTED |
| EXP-02 | Support 1080p export | ❌ NOT STARTED |
| EXP-03 | Support 30fps, 60fps export | ❌ NOT STARTED |
| EXP-04 | Show export progress + ETA | ❌ NOT STARTED |

**Blockers:**
- Phase 1.1 recording must work first
- FFmpeg integration via `ffmpeg-next` TBD

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
- Click event data structures (Rust)
- Test click logging (simulated mouse events)
- Click log display in React
- Basic Tauri IPC bridge (invoke/listen)
- Project folder structure concept

### What's Stubbed 🟡
- `capture.rs`: Module exists but doesn't actually capture screen
- Recording start/stop: No real capture backend
- Project file format: Structure defined, no I/O implemented
- Export pipeline: No FFmpeg integration

### What's Missing ❌
- **Screen capture** — `scap` crate not integrated
- **Real mouse hooks** — No Win32 `SetWindowsHookEx` implementation
- **Cursor tracking** — No continuous movement logging (60hz)
- **Audio capture** — Not started
- **Video encoding** — FFmpeg not integrated
- **Project persistence** — Click log not saved to disk
- **Error handling** — No graceful failures for missing disk space, etc.

---

## Next Steps (Priorities)

### Immediate (This Week)
1. Integrate `scap` crate for screen capture
2. Implement basic screen recording loop (capture → buffer → save raw)
3. Add region selection UI for recording area
4. Test frame rates and CPU usage

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
| No real screen capture implemented | 🔴 Critical | Blocks entire M1 | Start `scap` integration immediately |
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
| `scap` | Screen capture | ❌ TBD | Not yet added to Cargo.toml |
| `ffmpeg-next` | Video encoding | ❌ TBD | Not yet added to Cargo.toml |
| Win32 API | Mouse hooks | ❌ TBD | Not yet integrated |
| WASAPI | Audio capture | ❌ TBD | Not yet integrated |
| Zustand | State management | ❌ TBD | Not yet added |
| `serde` | JSON serialization | ⚠️ Available | Used for ClickEvent struct |

---

## Testing Status

- [ ] Unit tests for input module
- [ ] Integration tests for IPC (Tauri commands)
- [ ] System test: record screen → export → verify video
- [ ] Performance test: 60fps frame capture
- [ ] Audio sync test: video + microphone audio
- [ ] Export performance test: 5-min 1080p < 3 min
- [ ] Offline mode test: no network calls
- [ ] Disk-full error handling test

---

## Documentation Status

- [x] REQUIREMENTS.md — Project spec ✅
- [x] TIMELINE.md — Development phases ✅
- [x] PROGRESS.md — This file ✅
- [ ] README.md — Updated with current status
- [ ] CONTRIBUTING.md — Developer guide (M4)
- [ ] USER_MANUAL.md — End-user docs (M4)

---

## How to Update This File

1. **Weekly:** Update phase progress percentages and blockers
2. **Per feature:** Mark as ❌ NOT STARTED → ⚠️ IN PROGRESS → ✅ DONE
3. **Per commit:** Add to Git Commit History section
4. **Issues found:** Add to Known Issues & Risks
5. **Dependency changes:** Update Tech Stack Status

**Last updated:** Commit `9b5a301` (click logging feature)
