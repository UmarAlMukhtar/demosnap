import { useEffect, useState, type PointerEvent } from "react";
import { invoke } from "@tauri-apps/api/core";
import { getCurrentWindow, LogicalPosition, LogicalSize, currentMonitor } from "@tauri-apps/api/window";
import { WebviewWindow } from "@tauri-apps/api/webviewWindow";
import { emit, listen } from "@tauri-apps/api/event";
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
  const [windowLabel, setWindowLabel] = useState<string>("main");
  const [status, setStatus] = useState("Ready to record.");
  const [recordingPath, setRecordingPath] = useState<string | null>(null);
  
  // Recording states
  const [isRecording, setIsRecording] = useState(false);
  const [isPaused, setIsPaused] = useState(false);
  const [accumulatedTime, setAccumulatedTime] = useState(0);
  const [lastActiveTime, setLastActiveTime] = useState<number | null>(null);
  const [recordingElapsedMs, setRecordingElapsedMs] = useState(0);
  
  // Selection states
  const [captureMode, setCaptureMode] = useState<"full" | "region">("full");
  const [isSelectingRegion, setIsSelectingRegion] = useState(false);
  const [selectionStart, setSelectionStart] = useState<Point | null>(null);
  const [selectionRegion, setSelectionRegion] = useState<RecordingRegion | null>(null);
  
  // Countdown state (used in control window)
  const [countdown, setCountdown] = useState<number | null>(null);
  const [targetRegion, setTargetRegion] = useState<RecordingRegion | null>(null);

  // Microphone states
  const [microphoneName, setMicrophoneName] = useState<string | null>(null);
  const [micChecked, setMicChecked] = useState<boolean>(false);

  const currentWindow = getCurrentWindow();

  useEffect(() => {
    setWindowLabel(currentWindow.label);
  }, []);

  useEffect(() => {
    async function checkMicrophone() {
      try {
        const name = await invoke<string | null>("get_microphone_status");
        setMicrophoneName(name);
      } catch (err) {
        console.error("Failed to check microphone status:", err);
      } finally {
        setMicChecked(true);
      }
    }
    void checkMicrophone();
  }, []);

  // Timer interval updating
  useEffect(() => {
    if (!isRecording || lastActiveTime === null || isPaused) {
      return undefined;
    }

    const intervalId = window.setInterval(() => {
      setRecordingElapsedMs(accumulatedTime + (Date.now() - lastActiveTime));
    }, 250);

    return () => window.clearInterval(intervalId);
  }, [isRecording, lastActiveTime, isPaused, accumulatedTime]);

  // Main window: Listen to stop event from control window
  useEffect(() => {
    if (windowLabel !== "main") {
      return undefined;
    }

    const unlistenStop = listen<string>("recording-stopped", async (event) => {
      await currentWindow.unminimize();
      await currentWindow.setFocus();
      
      setIsRecording(false);
      setIsPaused(false);
      if (event.payload) {
        setRecordingPath(event.payload);
        setStatus("Recording session stopped. Project saved.");
      } else {
        setStatus("Recording cancelled or failed.");
      }
    });

    return () => {
      unlistenStop.then(f => f());
    };
  }, [windowLabel]);

  // Control window: Listen to start countdown event from main window
  useEffect(() => {
    if (windowLabel !== "recording-control") {
      return undefined;
    }

    const unlistenCountdown = listen<{ captureRegion: RecordingRegion | null }>(
      "start-countdown",
      (event) => {
        const region = event.payload.captureRegion;
        setTargetRegion(region);
        setCountdown(3);
      }
    );

    return () => {
      unlistenCountdown.then(f => f());
    };
  }, [windowLabel]);

  // Control window: Countdown logic
  useEffect(() => {
    if (windowLabel !== "recording-control" || countdown === null) {
      return undefined;
    }

    if (countdown === 0) {
      const startTimer = setTimeout(async () => {
        setCountdown(null);
        try {
          await invoke<string>("start_recording", {
            captureRegion: targetRegion,
          });
          setIsRecording(true);
          setIsPaused(false);
          setAccumulatedTime(0);
          setLastActiveTime(Date.now());
          setRecordingElapsedMs(0);
        } catch (error) {
          console.error("Failed to start recording:", error);
          await emit("recording-stopped", "");
          await currentWindow.hide();
        }
      }, 800); // Display "START" briefly

      return () => clearTimeout(startTimer);
    }

    const intervalId = setInterval(() => {
      setCountdown(prev => (prev !== null ? prev - 1 : null));
    }, 1000);

    return () => clearInterval(intervalId);
  }, [countdown, windowLabel, targetRegion]);

  async function startRecording(captureRegion: RecordingRegion | null = null) {
    try {
      const ctrlWindow = await WebviewWindow.getByLabel("recording-control");
      if (ctrlWindow) {
        const monitor = await currentMonitor();
        if (monitor) {
          const { width } = monitor.size;
          const scaleFactor = monitor.scaleFactor;
          const logicalWidth = width / scaleFactor;
          // Position overlay control window 40px from top and right
          await ctrlWindow.setPosition(new LogicalPosition(logicalWidth - 360, 40));
        }
        await ctrlWindow.show();
        await ctrlWindow.setFocus();
        await emit("start-countdown", { captureRegion });
      }
      setIsRecording(true);
      await currentWindow.minimize();
      setStatus("Starting countdown...");
    } catch (error) {
      setStatus(String(error));
    }
  }

  async function beginRegionSelection() {
    if (isRecording) {
      return;
    }
    // Set transparent borderless fullscreen main window
    await currentWindow.setDecorations(false);
    await currentWindow.setFullscreen(true);

    setSelectionStart(null);
    setSelectionRegion(null);
    setIsSelectingRegion(true);
    setStatus("Drag to select the capture region, then release to start recording.");
  }

  async function cancelRegionSelection() {
    await currentWindow.setFullscreen(false);
    await currentWindow.setDecorations(true);
    await currentWindow.setSize(new LogicalSize(800, 600));
    await currentWindow.center();

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
    await currentWindow.setFullscreen(false);
    await currentWindow.setDecorations(true);
    await currentWindow.setSize(new LogicalSize(800, 600));
    await currentWindow.center();

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

  async function handleDragStart(event: PointerEvent<HTMLDivElement>) {
    if (event.button === 0 && !(event.target as HTMLElement).closest("button")) {
      try {
        await currentWindow.startDragging();
      } catch (error) {
        console.error("Failed to drag window:", error);
      }
    }
  }

  function handleRecordButtonClick() {
    if (isRecording) {
      return;
    }

    if (captureMode === "region") {
      void beginRegionSelection();
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

  function formatElapsedTime(milliseconds: number) {
    const totalSeconds = Math.floor(milliseconds / 1000);
    const minutes = Math.floor(totalSeconds / 60).toString().padStart(2, "0");
    const seconds = (totalSeconds % 60).toString().padStart(2, "0");
    return `${minutes}:${seconds}`;
  }

  async function handlePauseResumeClick() {
    if (isPaused) {
      try {
        await invoke("resume_recording");
        setIsPaused(false);
        setLastActiveTime(Date.now());
      } catch (error) {
        console.error("Failed to resume:", error);
      }
    } else {
      try {
        await invoke("pause_recording");
        setIsPaused(true);
        const now = Date.now();
        const segmentDuration = now - lastActiveTime!;
        setAccumulatedTime(prev => prev + segmentDuration);
        setLastActiveTime(null);
        setRecordingElapsedMs(accumulatedTime + segmentDuration);
      } catch (error) {
        console.error("Failed to pause:", error);
      }
    }
  }

  async function handleStopClick() {
    try {
      const projectDir = await invoke<string>("stop_recording");
      setIsRecording(false);
      setIsPaused(false);
      setAccumulatedTime(0);
      setLastActiveTime(null);
      setRecordingElapsedMs(0);
      
      await emit("recording-stopped", projectDir);
      await currentWindow.hide();
    } catch (error) {
      console.error("Failed to stop recording:", error);
    }
  }

  useEffect(() => {
    if (!isSelectingRegion) {
      return undefined;
    }
    function handleKeyDown(event: KeyboardEvent) {
      if (event.key === "Escape") {
        void cancelRegionSelection();
      }
    }
    window.addEventListener("keydown", handleKeyDown);
    return () => window.removeEventListener("keydown", handleKeyDown);
  }, [isSelectingRegion]);

  if (windowLabel === "recording-control") {
    return (
      <div className="control-bar-shell">
        {countdown !== null ? (
          <div className="countdown-overlay">
            <span className="countdown-number">{countdown > 0 ? countdown : "START"}</span>
          </div>
        ) : (
          <div className="control-bar" onPointerDown={handleDragStart} data-tauri-drag-region="true">
            <div className="control-info" onPointerDown={handleDragStart} data-tauri-drag-region="true">
              <span className={`control-status-dot ${isPaused ? "paused" : "recording"}`} />
              <span className="control-time">{formatElapsedTime(recordingElapsedMs)}</span>
              {micChecked && (
                <span 
                  className={`control-mic-icon ${microphoneName ? "active" : "inactive"}`} 
                  title={microphoneName ? `Microphone: ${microphoneName}` : "No microphone detected (video-only)"}
                  role="img"
                  aria-label={microphoneName ? `Microphone: ${microphoneName}` : "No microphone detected (video-only)"}
                >
                  {microphoneName ? "🎤" : "🔇"}
                </span>
              )}
            </div>
            <div className="control-actions">
              <button
                className={`control-btn pause ${isPaused ? "active" : ""}`}
                type="button"
                onClick={handlePauseResumeClick}
              >
                {isPaused ? "Resume" : "Pause"}
              </button>
              <button
                className="control-btn stop"
                type="button"
                onClick={handleStopClick}
              >
                Stop
              </button>
            </div>
          </div>
        )}
      </div>
    );
  }

  return (
    <div className={`main-window-wrapper ${isSelectingRegion ? "transparent-bg" : "solid-bg"}`}>
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
                disabled={isRecording}
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
                ? "Recording is managed by the overlay control panel."
                : captureMode === "region"
                  ? "Press the red button, then drag to choose a capture region."
                  : "Press the red button to begin capturing your screen."}
            </p>

            <div className="recording-metrics" aria-label="Recording metrics">
              <span className={`recording-pill ${isRecording ? "live" : "idle"}`}>
                {isRecording ? "Recording" : "Idle"}
              </span>
              <span className="recording-pill recording-time">{formatElapsedTime(recordingElapsedMs)}</span>
              {micChecked && (
                <span className={`recording-pill mic-status-pill ${microphoneName ? "active" : "inactive"}`}>
                  <span className="mic-dot" />
                  <span className="mic-label">
                    {microphoneName ? microphoneName : "No Mic (Video-Only)"}
                  </span>
                </span>
              )}
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
    </div>
  );
}

export default App;
