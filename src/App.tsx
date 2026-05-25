import { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { revealItemInDir } from "@tauri-apps/plugin-opener";
import "./App.css";

function App() {
  const [status, setStatus] = useState("Ready to record.");
  const [recordingPath, setRecordingPath] = useState<string | null>(null);
  const [isRecording, setIsRecording] = useState(false);

  useEffect(() => {
    return undefined;
  }, []);

  async function startRecording() {
    try {
      const projectDir = await invoke<string>("start_recording");

      setIsRecording(true);
      setRecordingPath(projectDir);
      setStatus(`Recording session started in ${projectDir}`);
    } catch (error) {
      setStatus(String(error));
    }
  }

  async function stopRecording() {
    try {
      const projectDir = await invoke<string>("stop_recording");

      setIsRecording(false);
      setRecordingPath(projectDir);
      setStatus(`Recording session stopped. Project saved to ${projectDir}`);
    } catch (error) {
      setStatus(String(error));
    }
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
              onClick={isRecording ? stopRecording : startRecording}
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
              : "Press the red button to begin capturing your screen."}
          </p>

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
    </main>
  );
}

export default App;
