# Contributing to Demosnap

Thank you for your interest in contributing to Demosnap! This guide covers everything you need to set up your development environment, collaborate with others, and submit changes.

---

## Table of Contents

1. [Getting Started](#getting-started)
2. [Development Setup](#development-setup)
3. [Project Structure](#project-structure)
4. [Building & Testing](#building--testing)
5. [Git Workflow](#git-workflow)
6. [Collaboration (Vibe Coding)](#collaboration-vibe-coding)
7. [Code Style & Conventions](#code-style--conventions)
8. [Pull Request Process](#pull-request-process)
9. [Reporting Issues](#reporting-issues)
10. [Architecture & Design](#architecture--design)

---

## Getting Started

**Before you start:**
- Read [REQUIREMENTS.md](./REQUIREMENTS.md) to understand project scope
- Check [TIMELINE.md](./TIMELINE.md) to see which phase we're in
- Review [PROGRESS.md](./PROGRESS.md) to see what's blocked or in progress

**New to Rust?**
- Install [Rust](https://www.rust-lang.org/tools/install)
- Read [The Rust Book](https://doc.rust-lang.org/book/) (chapters 1–8 minimum)

**New to React/TypeScript?**
- Familiarity with React hooks, TypeScript basics assumed
- See official docs: [React](https://react.dev), [TypeScript](https://www.typescriptlang.org/docs/)

---

## Development Setup

### Prerequisites

- **Rust:** `rustc` + `cargo` (install via [rustup](https://rustup.rs/))
- **Node.js:** v18+ with npm
- **OS:** Windows 10/11 (macOS/Linux support planned for v2)
- **IDE:** VS Code + extensions:
  - [Tauri](https://marketplace.visualstudio.com/items?itemName=tauri-apps.tauri-vscode)
  - [rust-analyzer](https://marketplace.visualstudio.com/items?itemName=rust-lang.rust-analyzer)
  - [ES7+ React/Redux/React-Native snippets](https://marketplace.visualstudio.com/items?itemName=dsznajder.es7-react-js-snippets)

### Clone & Install

```bash
# Clone the repository
git clone https://github.com/your-org/demosnap.git
cd demosnap

# Install Node dependencies
npm install

# Install Rust dependencies (runs automatically)
cargo fetch

# Verify setup
npm run build
```

### First Run

```bash
# Start dev server with hot reload
npm run dev

# In another terminal, run Tauri dev (full app + hot reload)
cargo tauri dev
```

You should see:
- React app on `http://localhost:5173`
- Tauri window with click logger
- Console showing "APP STARTING - you should see this"

**Troubleshooting:**
- `Error: Command "cargo" not found` → Install Rust via [rustup](https://rustup.rs/)
- `npm ERR! code ERESOLVE` → Run `npm install --legacy-peer-deps`
- Tauri window won't open → Check Windows 11 WebView2 is installed: `winget install Microsoft.WebView2Runtime`

---

## Project Structure

```
demosnap/
├── src/                          # React frontend
│   ├── App.tsx                   # Main component
│   ├── App.css                   # Styles
│   ├── components/               # Reusable UI components (future)
│   ├── hooks/                    # Custom React hooks (future)
│   └── store/                    # Zustand state (future)
│
├── src-tauri/                    # Rust backend
│   ├── src/
│   │   ├── lib.rs               # Tauri command registry + app entry
│   │   ├── capture.rs           # Screen recording module
│   │   ├── input.rs             # Mouse/keyboard logging
│   │   ├── audio.rs             # Audio capture (future)
│   │   ├── export.rs            # FFmpeg integration (future)
│   │   └── project.rs           # File I/O (future)
│   ├── Cargo.toml               # Rust dependencies
│   └── tauri.conf.json          # Tauri config
│
├── REQUIREMENTS.md              # Project specification
├── TIMELINE.md                  # Development phases (12 months)
├── PROGRESS.md                  # Current implementation status
├── CONTRIBUTING.md              # This file
├── CLAUDE.md                    # AI assistant guidance (future)
├── package.json                 # Node dependencies
├── tsconfig.json                # TypeScript config
├── vite.config.ts               # Vite bundler config
└── README.md                    # User-facing docs
```

### Module Responsibilities

| Module | Purpose | Status | Owner |
|--------|---------|--------|-------|
| `capture.rs` | Screen recording (DXGI/scap) | ❌ Stub | Unassigned |
| `input.rs` | Mouse/keyboard hooks | ⚠️ Partial | Unassigned |
| `audio.rs` | Microphone + system audio | ❌ TBD | Unassigned |
| `export.rs` | FFmpeg video encoding | ❌ TBD | Unassigned |
| `project.rs` | `.dsnap` project file I/O | ❌ TBD | Unassigned |
| `App.tsx` | Main React UI | ⚠️ Partial | Unassigned |

---

## Building & Testing

### Build Commands

```bash
# Development build (unoptimized, fast compile)
npm run dev

# Production build (optimized, slower compile)
npm run build

# Tauri dev (full app with hot reload)
cargo tauri dev

# Tauri build (full release binary)
cargo tauri build
```

### Running Tests

```bash
# Run all Rust tests
cargo test

# Run tests for a specific module
cargo test input::
cargo test capture::

# Run with output
cargo test -- --nocapture

# Run in release mode (slower compile, faster execution)
cargo test --release

# React/TypeScript tests (when added)
npm test

# Type check only
npx tsc --noEmit
```

### Performance Testing

```bash
# Check compile time
time cargo build

# Profile runtime
cargo build --release
cargo flamegraph  # Requires: cargo install flamegraph

# Memory profiling
cargo build --release
valgrind ./target/release/demosnap  # Requires: apt install valgrind
```

### Local Testing Checklist

Before pushing, verify:
- [ ] `cargo test` passes
- [ ] `cargo clippy` has no warnings (linter)
- [ ] `cargo fmt --check` passes (formatting)
- [ ] App starts: `cargo tauri dev` succeeds
- [ ] Click logging works: click in app window, see it logged
- [ ] No console errors in browser DevTools (F12)

```bash
# Run all checks
cargo fmt --check && cargo clippy && cargo test && npm run build
```

---

## Git Workflow

### Branch Naming

```
feature/phase-X-Y-description        # New feature
bugfix/issue-123-short-desc          # Bug fix
refactor/module-name-cleanup         # Refactoring
docs/update-readme                   # Documentation
chore/update-dependencies            # Dependencies
```

Examples:
```
feature/phase-1.1-screen-capture
feature/phase-1.2-microphone-audio
bugfix/issue-42-click-log-not-saving
refactor/input-module-simplify
```

### Commit Messages

Follow [Conventional Commits](https://www.conventionalcommits.org/):

```
feat(capture): add DXGI screen capture integration

- Integrated scap crate v0.14 for frame capture
- Supports 60fps on 1920x1080
- Returns BGRA format frames
- Added unit tests for frame validation

Closes #42
```

**Commit types:**
- `feat:` New feature
- `fix:` Bug fix
- `refactor:` Code restructuring (no behavior change)
- `test:` Tests only
- `docs:` Documentation only
- `chore:` Dependencies, configs, etc.

**Template:**
```
<type>(<scope>): <subject>

<body (optional)>

<footer (optional)>
```

**Subject rules:**
- Imperative mood: "add", not "added" or "adds"
- No period at end
- Under 50 characters
- Reference issue: `Closes #42`

### Push & Pull Request

```bash
# Create feature branch from main
git checkout main
git pull origin main
git checkout -b feature/my-feature

# Make changes, commit
git add src-tauri/src/capture.rs
git commit -m "feat(capture): implement screen capture"

# Push to remote
git push -u origin feature/my-feature

# Open PR on GitHub
# (or use: gh pr create)
```

**Before pushing:**
```bash
# Sync with main in case others pushed
git fetch origin
git rebase origin/main

# Run tests one more time
cargo test && npm run build
```

---

## Collaboration (Vibe Coding)

### Synchronous Sessions

**Before the session:**
1. Create an issue + PR draft to claim work
2. Post in #dev-collab (Discord/Slack) with:
   - Time + duration (e.g., "May 26, 2–4 PM UTC")
   - Which phase/features (from TIMELINE.md)
   - Voice channel link

**During the session:**
```
14:00 Kickoff (5 min)
  • Review task scope
  • Show architecture sketch (Excalidraw)
  • Assign 1 feature per person

14:05 – 14:50 Code (45 min)
  • Use VS Code Live Share for pair/mob
  • Shared Slack for quick questions
  • Shared voice for design decisions

14:50 Check-in (5 min)
  • Demo locally what you built
  • Flag blockers: "Need FFmpeg decision"

15:00 Merge & Test (10 min)
  • Each person pushes branch
  • Run full test suite
  • Merge to feature branch

15:10 Reflect (5 min)
  • Update PROGRESS.md
  • Plan next session
```

### Asynchronous Contributions

**If async, document heavily:**

1. **Detailed commit messages** (see Git Workflow section)
2. **ADR (Architecture Decision Record)** in `/docs/adr/`:
   ```markdown
   # ADR-001: Use scap for Screen Capture

   ## Context
   Need cross-platform screen capture for Windows/Mac/Linux.

   ## Decision
   Use scap crate (maintained, handles DXGI/ScreenCaptureKit/PipeWire).

   ## Tradeoffs
   ✅ Cross-platform
   ✅ Maintained
   ❌ Slight performance hit vs raw DXGI
   ```

3. **Video walkthrough** (3–5 min Loom):
   - "Here's the module I built"
   - "Here's the API/interface"
   - "Here's how to test it"

4. **PR description** links to ADR + context

---

## Code Style & Conventions

### Rust

**Formatting:**
```bash
cargo fmt
```

**Linting:**
```bash
cargo clippy
```

**Style guide:**
- Use `snake_case` for functions and variables
- Use `PascalCase` for types and structs
- Keep functions under 50 lines (readability)
- Avoid `unwrap()` in production code; use `Result<T, E>` instead
- Document public APIs with `///` doc comments

**Example:**
```rust
/// Log a click event with coordinates and timestamp
pub fn push_click(log: &ClickLog, x: i32, y: i32) -> Result<(), String> {
    let event = ClickEvent {
        timestamp_ms: now_ms(),
        x,
        y,
        event_type: MouseEventType::LeftDown,
    };
    
    log.lock()
        .map_err(|e| format!("Mutex lock failed: {}", e))?
        .push(event);
    
    Ok(())
}
```

### TypeScript/React

**Formatting:**
```bash
npx prettier --write src/
```

**Linting:**
```bash
npx eslint src/
```

**Style guide:**
- Use `const` for all declarations (no `let` unless reassignment needed)
- Use arrow functions `() => {}`
- Keep components under 200 lines (split into smaller components)
- Use TypeScript types, not `any`
- Prop names in camelCase

**Example:**
```typescript
type RecordingState = {
  isRecording: boolean;
  elapsedSeconds: number;
  clickLog: ClickEvent[];
};

interface RecorderProps {
  onStart: () => void;
  onStop: () => void;
}

export const Recorder: React.FC<RecorderProps> = ({ onStart, onStop }) => {
  const [state, setState] = useState<RecordingState>({
    isRecording: false,
    elapsedSeconds: 0,
    clickLog: [],
  });

  return (
    <button onClick={state.isRecording ? onStop : onStart}>
      {state.isRecording ? "Stop" : "Start"}
    </button>
  );
};
```

### General Rules

- **No `TODO` comments** — Create an issue instead
- **No commented-out code** — Delete it; git history has it
- **Error messages are user-facing** — Make them clear and actionable
- **Test names describe behavior:** `test_click_log_stores_event` not `test_click`

---

## Pull Request Process

### Before Submitting

1. **Update PROGRESS.md:**
   - Mark features as ✅ DONE / ⚠️ IN PROGRESS
   - Update phase percentage
   - Add blockers if any

2. **Self-review:**
   ```bash
   git diff main...HEAD
   # Read your own changes; catch obvious issues
   ```

3. **Run full test suite:**
   ```bash
   cargo fmt && cargo clippy && cargo test && npm run build && npm test
   ```

4. **Test manually:**
   - Click in the app, verify behavior
   - Check console for errors
   - Try edge cases (empty input, disk full, etc.)

### PR Template

```markdown
## Description
Brief summary of what this PR does.

## Relates To
Closes #42
Related to TIMELINE.md Phase X.Y

## Changes
- [ ] Added feature X
- [ ] Updated module Y
- [ ] Added tests for Z

## Testing
- [ ] Manual testing: Click logging works
- [ ] `cargo test` passes
- [ ] `npm run build` succeeds

## Screenshots (if UI change)
[Paste screenshot or GIF]

## Notes
- Used scap v0.14 (new dependency)
- Performance impact: ~5% slower but gains cross-platform support
```

### Review Process

- At least 1 approval required before merge
- Automated tests must pass (CI)
- No merge conflicts
- Commits should be squashed if > 5 commits

---

## Reporting Issues

### Bug Report Template

```markdown
## Describe the bug
Clear description of the problem.

## Steps to reproduce
1. Open Demosnap
2. Click in the window
3. Observe: ...

## Expected behavior
What should happen.

## Actual behavior
What actually happens.

## Environment
- OS: Windows 11
- Demosnap version: main branch
- Steps to reproduce consistently: Yes/No

## Logs
Paste console output (F12 DevTools)
```

### Feature Request Template

```markdown
## Summary
One-line description.

## Use case
Why is this needed?

## Proposed solution
How should it work?

## Related requirements
References to REQUIREMENTS.md (e.g., REC-01, ZOM-02)

## Acceptance criteria
- [ ] Feature works as described
- [ ] Tests pass
- [ ] Documented
```

---

## Architecture & Design

### Key Concepts

**ClickLog (Thread-safe event storage):**
```rust
pub type ClickLog = Arc<Mutex<Vec<ClickEvent>>>;
```
- `Arc`: Shared ownership across threads
- `Mutex`: Synchronized access (one thread at a time)
- Stored in Tauri state, accessible from React via `invoke()`

**Tauri Commands (React ↔ Rust IPC):**
```rust
#[tauri::command]
fn record_click(log: State<ClickLog>, x: i32, y: i32) -> Vec<ClickEvent> {
    // React calls: invoke("record_click", { x, y })
    // Rust receives, stores, returns updated log
}
```

**Project File Format (`.dsnap`):**
```
my-recording.dsnap/
├── video.raw                 # Raw captured frames
├── audio.wav                 # Microphone audio
├── clicks.json               # Click log (JSON)
└── project.json              # Editor state + settings
```

### Design Patterns

**Module interfaces (Rust traits):**
```rust
// Separation of concerns — easy to test/swap implementations
pub trait ScreenCapture {
    fn start(&mut self, region: Rect) -> Result<(), String>;
    fn stop(&mut self) -> Result<Vec<Frame>, String>;
}
```

**React hooks for state (future):**
```typescript
const { isRecording, clickLog } = useRecording();
// Encapsulates Tauri IPC calls + Zustand state management
```

### Performance Considerations

- **60fps constraint:** 16.7ms per frame max
- **Thread-safe logging:** Click logging must not block capture
- **Memory efficiency:** Store raw frames, not decoded (BGRA only)
- **Batch operations:** Collect 100 frames before writing to disk

See [REQUIREMENTS.md Section 3](./REQUIREMENTS.md#3-non-functional-requirements) for full NFR list.

---

## Need Help?

- **Questions?** Open a discussion in GitHub Discussions
- **Bug?** File an issue with the bug template above
- **Want to pair?** Post in #dev-collab with availability
- **Architecture decision needed?** Create an ADR in `/docs/adr/`
- **Blocked?** Comment in your PR; maintainers will help unblock

---

## Code of Conduct

- Be respectful and constructive
- Assume good intent
- Welcome questions from less-experienced contributors
- Celebrate wins (merges, fixes, learnings)

---

**Happy coding! 🚀**
