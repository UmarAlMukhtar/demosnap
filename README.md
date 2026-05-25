# Demosnap

> **Free, open-source desktop screen recorder with automatic professional editing.**

Record your screen, and Demosnap handles the rest — automatically zooming into clicks, smoothing your cursor, and enhancing the output with no manual editing required.

**Perfect for:** Developers, designers, and product teams creating demos, tutorials, and product walkthroughs.

---

## ✨ Features (Planned)

### v1.0 (Months 1–12)

**Recording:**
- ✅ Record primary display at 60fps
- ✅ Select recording region
- ✅ Record microphone audio
- ✅ Pause & resume during recording
- ✅ Recording indicator overlay

**Auto-Enhancement:**
- 🔄 Auto-zoom into clicks with smooth easing
- 🔄 Smooth cursor path using Bézier interpolation
- 🔄 Click ripple animations
- 🔄 Custom cursor styling (color, size)

**Editing:**
- 🔄 Timeline editor with zoom event markers
- 🔄 Trim start/end of recording
- 🔄 Live preview of edits
- 🔄 Blur masks for sensitive regions
- 🔄 Background customization & padding

**Export:**
- 🔄 MP4 (H.264) export at 1080p, 1440p, 4K
- 🔄 30fps & 60fps export options
- 🔄 Background export (app stays usable)
- 🔄 Auto-generated subtitles from audio (Whisper)

**Legend:** ✅ Done | 🔄 In progress/Planned | ❌ Out of scope (v1)

---

## 📋 Current Status

**Phase:** M1 — MVP Recording & Export (~25% complete)

See [PROGRESS.md](./PROGRESS.md) for detailed tracking.

| Module | Status | Next Steps |
|--------|--------|-----------|
| Screen Capture | ⚠️ Live desktop capture with drag-to-select region capture | Validate stability and tuning |
| Mouse/Click Logging | ⚠️ Partial (test only) | Implement Win32 hooks for real events |
| Audio Capture | ❌ Not started | WASAPI integration |
| Video Export | ❌ Not started | FFmpeg pipeline |
| Timeline Editor | ❌ Not started | React UI + project file format |

👉 [See full progress tracking →](./PROGRESS.md)

---

## 🚀 Quick Start

### Prerequisites

- **Rust:** [Install rustup](https://rustup.rs/)
- **Node.js:** v18+
- **OS:** Windows 10/11 (macOS/Linux planned for v2)
- **IDE:** [VS Code](https://code.visualstudio.com/) + extensions:
  - [Tauri](https://marketplace.visualstudio.com/items?itemName=tauri-apps.tauri-vscode)
  - [rust-analyzer](https://marketplace.visualstudio.com/items?itemName=rust-lang.rust-analyzer)

### Setup

```bash
# Clone repository
git clone https://github.com/your-org/demosnap.git
cd demosnap

# Install dependencies
npm install

# Start dev server
npm run dev

# In another terminal: Start Tauri dev (full app + hot reload)
cargo tauri dev
```

**Expected output:**
- React dev server on `http://localhost:5173`
- Tauri window with click logger
- Try clicking in the window to log clicks

### Common Commands

```bash
# Build frontend + compile Rust
npm run build

# Run tests
cargo test

# Format & lint
cargo fmt && cargo clippy

# Full dev workflow
cargo tauri dev

# Full release build
cargo tauri build
```

👉 [See full dev guide →](./CONTRIBUTING.md#building--testing)

---

## 📚 Documentation

| Document | Purpose |
|----------|---------|
| [REQUIREMENTS.md](./REQUIREMENTS.md) | **Spec:** What Demosnap must do, functional & non-functional requirements |
| [TIMELINE.md](./TIMELINE.md) | **Plan:** 12-month development roadmap with 4 milestones |
| [PROGRESS.md](./PROGRESS.md) | **Status:** Real-time tracking of what's done/in-progress/blocked |
| [CONTRIBUTING.md](./CONTRIBUTING.md) | **Dev Guide:** Setup, collaboration, git workflow, code style |

---

## 🏗️ Architecture

**Tech Stack:**
- **Desktop Shell:** [Tauri v2](https://tauri.app/) (lightweight cross-platform framework)
- **Frontend:** React 19 + TypeScript 5.8
- **Backend:** Rust (performance, memory safety, system access)
- **Screen Capture:** Rust now launches ffmpeg desktop capture on recording start and supports drag-to-select regions; stability and tuning are still pending
- **Video Encoding:** FFmpeg via [`ffmpeg-next`](https://github.com/zmwang622/ffmpeg-next)
- **State Management:** [Zustand](https://github.com/pmndrs/zustand) (lightweight, simple)
- **Subtitles:** [Whisper.cpp](https://github.com/ggerganov/whisper.cpp) (local, no internet)

**Project Structure:**
```
src/                          # React frontend
├── App.tsx                   # Main UI
├── components/               # Reusable components (future)
└── store/                    # Zustand state (future)

src-tauri/src/                # Rust backend
├── lib.rs                    # Tauri command registry
├── capture.rs                # Screen recording module
├── input.rs                  # Mouse/keyboard logging
├── audio.rs                  # Audio capture (future)
├── export.rs                 # FFmpeg integration (future)
└── project.rs                # File I/O (future)
```

👉 [See full architecture →](./CONTRIBUTING.md#architecture--design)

---

## 🤝 Contributing

**Want to help?** We're building this in the open with collaborative development.

1. **New to the project?** Start with [CONTRIBUTING.md](./CONTRIBUTING.md#getting-started)
2. **Want to pick up a task?** Check [PROGRESS.md](./PROGRESS.md) for what's blocked or pending
3. **Ready to code?** See [git workflow](./CONTRIBUTING.md#git-workflow) and [collaboration guidelines](./CONTRIBUTING.md#collaboration-vibe-coding)

### Getting Started with Development

```bash
# 1. Read the docs (15 min)
cat REQUIREMENTS.md      # What we're building
cat TIMELINE.md          # How we're building it
cat PROGRESS.md          # Where we are now

# 2. Set up dev environment (10 min)
npm install
cargo fetch

# 3. Run tests to verify setup (5 min)
cargo test && npm run build

# 4. Pick a task from PROGRESS.md and open an issue

# 5. Create a branch and start coding
git checkout -b feature/your-feature
```

### Vibe Coding Sessions

We run synchronous collaboration sessions to move fast. Join us on **Discord** to see when sessions happen and contribute in real-time.

Session structure:
- **Start:** Review task scope + interfaces
- **45 min:** Code together (VS Code Live Share)
- **Check-in:** Demo locally + flag blockers
- **Merge:** Run tests + merge to feature branch

👉 [See collaboration guidelines →](./CONTRIBUTING.md#collaboration-vibe-coding)

---

## 🎯 Development Roadmap

### Milestone 1: MVP (Month 3)
✅ Record screen + log clicks + export basic MP4

### Milestone 2: Polish (Month 6)
🔄 Auto-zoom + cursor smoothing + timeline editor

### Milestone 3: Features (Month 9)
🔄 Full feature set (webcam, subtitles, masks, GIF/WebM)

### Milestone 4: Release (Month 12)
🔄 Performance tuning + installers + v1.0 release

👉 [See full timeline →](./TIMELINE.md)

---

## ⚙️ Configuration

**Env Variables:** Create `.env.local` (not committed):
```bash
# Optional: Enable verbose logging
RUST_LOG=debug
```

**Tauri Config:** Edit `src-tauri/tauri.conf.json` to customize:
- Window size
- App title
- Security settings
- Build options

---

## 🐛 Reporting Issues

Found a bug or have a feature idea? Open an issue with:

**Bug Report:**
- Steps to reproduce
- Expected vs actual behavior
- Screenshots/console logs
- OS/version

**Feature Request:**
- Use case
- Proposed solution
- Related REQUIREMENTS.md IDs (e.g., REC-01, ZOM-02)

👉 [See issue templates →](./CONTRIBUTING.md#reporting-issues)

---

## 📄 License

[License TBD — Check LICENSE file]

---

## 🙏 Acknowledgments

- Built with [Tauri](https://tauri.app/)
- Screen capture via [`scap`](https://github.com/nashaofu/scap)
- Video encoding via [FFmpeg](https://ffmpeg.org/)
- Inspired by tools like [Loom](https://www.loom.com/) and [ScreenFlow](https://www.telestream.net/screenflow/)

---

## 💬 Questions?

- **Dev setup help?** See [CONTRIBUTING.md](./CONTRIBUTING.md#getting-started)
- **Want to contribute?** See [CONTRIBUTING.md](./CONTRIBUTING.md)
- **Need architecture context?** See [CONTRIBUTING.md#architecture--design](./CONTRIBUTING.md#architecture--design)
- **Tracking tasks?** See [PROGRESS.md](./PROGRESS.md)

**Connect:** Open a GitHub Discussion or check the #dev-collab Discord channel
