import { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import "./App.css";

type ClickEvent = {
  timestamp_ms: number;
  x: number;
  y: number;
  event_type: "LeftDown" | "LeftUp" | "Move";
};

function App() {
  const [clickLog, setClickLog] = useState<ClickEvent[]>([]);
  const [status, setStatus] = useState("Click anywhere in the panel to log it.");

  useEffect(() => {
    invoke<ClickEvent[]>("get_click_log")
      .then(setClickLog)
      .catch(() => setStatus("Click log backend is not ready yet."));
  }, []);

  async function logClick(event: React.MouseEvent<HTMLElement>) {
    const updatedLog = await invoke<ClickEvent[]>("record_click", {
      x: Math.round(event.clientX),
      y: Math.round(event.clientY),
    });

    setClickLog(updatedLog);
    const latest = updatedLog[updatedLog.length - 1];
    setStatus(`Logged click at (${latest.x}, ${latest.y}). Total: ${updatedLog.length}`);
  }

  return (
    <main className="container" onClick={logClick}>
      <h1>Click Log</h1>
      <p>{status}</p>
      <section>
        <h2>Recent clicks</h2>
        <ul>
          {clickLog.slice(-5).reverse().map((event) => (
            <li key={`${event.timestamp_ms}-${event.x}-${event.y}`}>
              {event.event_type} at ({event.x}, {event.y})
            </li>
          ))}
        </ul>
      </section>
    </main>
  );
}

export default App;
