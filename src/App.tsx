import { useEffect, useState, type PointerEvent } from "react";
import { invoke } from "@tauri-apps/api/core";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { revealItemInDir } from "@tauri-apps/plugin-opener";
import "./App.css";

type RecordingRegion = {
  x: number;
  y: number;
  width: number;
  height: number;
};

type Point = {
  x: number;
  y: number;
};

function App() {
  const [status, setStatus] = useState("Ready to record.");
  const [recordingPath, setRecordingPath] = useState<string | null>(null);
  const [isRecording, setIsRecording] = useState(false);
  const [recordingStartedAt, setRecordingStartedAt] = useState<number | null>(null);
  const [recordingElapsedMs, setRecordingElapsedMs] = useState(0);
  const [captureMode, setCaptureMode] = useState<"full" | "region">("full");
  const [isSelectingRegion, setIsSelectingRegion] = useState(false);
  const [selectionStart, setSelectionStart] = useState<Point | null>(null);
  const [selectionRegion, setSelectionRegion] = useState<RecordingRegion | null>(null);

  useEffect(() => {
    if (!isRecording || recordingStartedAt === null) {
      return undefined;
    }

    setRecordingElapsedMs(Date.now() - recordingStartedAt);

    const intervalId = window.setInterval(() => {
      setRecordingElapsedMs(Date.now() - recordingStartedAt);
    }, 250);

    return () => window.clearInterval(intervalId);
  }, [isRecording, recordingStartedAt]);

  async function startRecording(captureRegion: RecordingRegion | null = null) {
    try {
      const projectDir = await invoke<string>("start_recording", {
        captureRegion,
      });

      setIsRecording(true);
      setRecordingStartedAt(Date.now());
      setRecordingElapsedMs(0);
      setRecordingPath(projectDir);
      setStatus(
        captureRegion
          ? `Recording session started using a ${captureRegion.width}x${captureRegion.height} region at (${captureRegion.x}, ${captureRegion.y}).`
          : "Recording session started using the full display.",
      );
    } catch (error) {
      setStatus(String(error));
    }
  }

  function beginRegionSelection() {
    if (isRecording) {
      return;
    }

    setSelectionStart(null);
    setSelectionRegion(null);
    setIsSelectingRegion(true);
    setStatus("Drag to select the capture region, then release to start recording.");
  }

  function cancelRegionSelection() {
    setIsSelectingRegion(false);
    setSelectionStart(null);
    setSelectionRegion(null);
    setStatus("Region selection cancelled.");
  }

  function getPointFromEvent(event: PointerEvent<HTMLDivElement>): Point {
    const bounds = event.currentTarget.getBoundingClientRect();

    return {
      x: Math.max(0, Math.round(event.clientX - bounds.left)),
      y: Math.max(0, Math.round(event.clientY - bounds.top)),
    };
  }

  function updateSelectionRegion(start: Point, current: Point) {
    const x = Math.min(start.x, current.x);
    const y = Math.min(start.y, current.y);
    const width = Math.abs(current.x - start.x);
    const height = Math.abs(current.y - start.y);

    setSelectionRegion({ x, y, width, height });
  }

  function createSelectionRegion(start: Point, current: Point): RecordingRegion {
    const x = Math.min(start.x, current.x);
    const y = Math.min(start.y, current.y);
    const width = Math.abs(current.x - start.x);
    const height = Math.abs(current.y - start.y);

    return { x, y, width, height };
  }

  async function toDesktopRegion(region: RecordingRegion): Promise<RecordingRegion> {
    const currentWindow = getCurrentWindow();
    const contentOrigin = await currentWindow.innerPosition();
    const scaleFactor = await currentWindow.scaleFactor();

    return {
      x: Math.round((contentOrigin.x + region.x) * scaleFactor),
      y: Math.round((contentOrigin.y + region.y) * scaleFactor),
      width: Math.round(region.width * scaleFactor),
      height: Math.round(region.height * scaleFactor),
    };
  }

  async function finishRegionSelection(regionToRecord: RecordingRegion | null) {
    if (!regionToRecord || regionToRecord.width < 10 || regionToRecord.height < 10) {
      setStatus("Select a larger region before recording.");
      setIsSelectingRegion(false);
      setSelectionStart(null);
      setSelectionRegion(null);
      return;
    }

    setIsSelectingRegion(false);
    setSelectionStart(null);
    setSelectionRegion(regionToRecord);
    await startRecording(await toDesktopRegion(regionToRecord));
  }

  function handleRecordButtonClick() {
    if (isRecording) {
      void stopRecording();
      return;
    }

    if (captureMode === "region") {
      beginRegionSelection();
      return;
    }

    void startRecording();
  }

  function handleSelectionPointerDown(event: PointerEvent<HTMLDivElement>) {
    if (!isSelectingRegion) {
      return;
    }

    event.preventDefault();
    event.currentTarget.setPointerCapture(event.pointerId);
    const point = getPointFromEvent(event);

    setSelectionStart(point);
    setSelectionRegion({ x: point.x, y: point.y, width: 0, height: 0 });
  }

  function handleSelectionPointerMove(event: PointerEvent<HTMLDivElement>) {
    if (!isSelectingRegion || !selectionStart) {
      return;
    }

    updateSelectionRegion(selectionStart, getPointFromEvent(event));
  }

  function handleSelectionPointerUp(event: PointerEvent<HTMLDivElement>) {
    if (!isSelectingRegion || !selectionStart) {
      return;
    }

    const regionToRecord = createSelectionRegion(selectionStart, getPointFromEvent(event));
    setSelectionStart(null);
    setSelectionRegion(null);
    event.currentTarget.releasePointerCapture(event.pointerId);
    void finishRegionSelection(regionToRecord);
  }

  async function stopRecording() {
    try {
      const projectDir = await invoke<string>("stop_recording");

      setIsRecording(false);
      setRecordingStartedAt(null);
      setRecordingPath(projectDir);
      setStatus("Recording session stopped. Project saved.");
    } catch (error) {
      setStatus(String(error));
    }
  }

  function formatElapsedTime(milliseconds: number) {
    const totalSeconds = Math.floor(milliseconds / 1000);
    const minutes = Math.floor(totalSeconds / 60)
      .toString()
      .padStart(2, "0");
    const seconds = (totalSeconds % 60).toString().padStart(2, "0");

    return `${minutes}:${seconds}`;
  }

  useEffect(() => {
    if (!isSelectingRegion) {
      return undefined;
    }

    function handleKeyDown(event: KeyboardEvent) {
      if (event.key === "Escape") {
        cancelRegionSelection();
      }
    }

    window.addEventListener("keydown", handleKeyDown);
    return () => window.removeEventListener("keydown", handleKeyDown);
  }, [isSelectingRegion]);

  async function openRecordingPath() {
    if (!recordingPath) {
      return;
    }

    try {
      await revealItemInDir(recordingPath);
    } catch (error) {
      setStatus(String(error));
    }
  }

  return (
    <main className="app-shell">
      <aside className="sidebar" aria-label="Navigation">
        <div className="brand">Demosnap</div>
        <button className="sidebar-tab active" type="button">
          Recording
        </button>
      </aside>

      <section className="workspace">
        <div className="recording-panel">
          <div className="record-visual">
            <div className="record-ripple" aria-hidden="true" />
            <button
              className={`record-button ${isRecording ? "recording" : "idle"}`}
              onClick={handleRecordButtonClick}
              aria-label={isRecording ? "Stop recording" : "Start recording"}
              type="button"
            >
              <span className="record-symbol" aria-hidden="true">
                <span className="record-symbol-ring" />
                <span className="record-symbol-dot" />
              </span>
              <span className="record-button-label">RECORD</span>
            </button>
          </div>

          <p className="record-title">{isRecording ? "Recording" : "Ready. Start recording"}</p>
          <p className="record-copy">
            {isRecording
              ? "Click the button again to stop the current session."
              : captureMode === "region"
                ? "Press the red button, then drag to choose a capture region."
                : "Press the red button to begin capturing your screen."}
          </p>

          <div className="recording-metrics" aria-label="Recording metrics">
            <span className={`recording-pill ${isRecording ? "live" : "idle"}`}>
              {isRecording ? "Recording" : "Idle"}
            </span>
            <span className="recording-pill recording-time">{formatElapsedTime(recordingElapsedMs)}</span>
          </div>

          <div className="capture-settings" aria-label="Capture settings">
            <div className="capture-mode">
              <button
                className={`capture-toggle ${captureMode === "full" ? "active" : ""}`}
                type="button"
                onClick={() => setCaptureMode("full")}
                disabled={isRecording}
              >
                Full screen
              </button>
              <button
                className={`capture-toggle ${captureMode === "region" ? "active" : ""}`}
                type="button"
                onClick={() => setCaptureMode("region")}
                disabled={isRecording}
              >
                Region
              </button>
            </div>
          </div>

          <div className="record-status">
            <span>{status}</span>
            {recordingPath ? (
              <button className="record-path" onClick={openRecordingPath} type="button">
                {recordingPath}
              </button>
            ) : null}
          </div>
        </div>

      </section>

      {isSelectingRegion ? (
        <div
          className="region-overlay"
          role="presentation"
          onPointerDown={handleSelectionPointerDown}
          onPointerMove={handleSelectionPointerMove}
          onPointerUp={handleSelectionPointerUp}
          onPointerCancel={cancelRegionSelection}
        >
          <div className="region-overlay-panel">
            <p className="region-overlay-title">Drag to select the capture area</p>
            <p className="region-overlay-copy">Release the mouse to begin recording that region.</p>
            <button className="region-overlay-cancel" type="button" onClick={cancelRegionSelection}>
              Cancel
            </button>
          </div>

          {selectionRegion ? (
            <div
              className="region-selection-box"
              style={{
                left: `${selectionRegion.x}px`,
                top: `${selectionRegion.y}px`,
                width: `${Math.max(selectionRegion.width, 1)}px`,
                height: `${Math.max(selectionRegion.height, 1)}px`,
              }}
              aria-hidden="true"
            >
              <span className="region-selection-label">
                {selectionRegion.width} x {selectionRegion.height}
              </span>
            </div>
          ) : null}
        </div>
      ) : null}
    </main>
  );
}

export default App;
