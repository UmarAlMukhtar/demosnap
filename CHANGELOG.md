# CHANGELOG

## [Unreleased]
- Started live desktop capture recording with ffmpeg
- Added drag-to-select region capture and passed the chosen rectangle into ffmpeg
- Added live recording timer and active-state indicator
- Restored pointer cursor and hover feedback on interactive controls
- Stabilized the status area so the project path appears once without shifting the panel
- Added global mouse-hook click logging and persisted clicks to `clicks.json`
- Wrote project files into each `.dsnap` folder (`project.json`, `video.mp4`, `clicks.json`)
- Updated PROGRESS.md with REC-01 status