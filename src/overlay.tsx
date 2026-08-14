import React from "react";
import ReactDOM from "react-dom/client";
import { listen } from "@tauri-apps/api/event";
import type { AppState } from "./types/settings";
import "./styles/overlay.css";

function Overlay() {
  const [state, setState] = React.useState<AppState>("idle");
  const [text, setText] = React.useState("");
  const [listenMode, setListenMode] = React.useState("short");

  React.useEffect(() => {
    const unlisteners: Array<() => void> = [];
    void listen<AppState>("flow-state", (event) => {
      setState(event.payload);
      if (event.payload === "listening" || event.payload === "preparing") {
        setText("");
      }
    }).then((fn) => unlisteners.push(fn));
    void listen<string>("clean-transcript", (event) => {
      setText(event.payload);
    }).then((fn) => unlisteners.push(fn));
    void listen<string>("listen-mode", (event) => {
      setListenMode(event.payload);
    }).then((fn) => unlisteners.push(fn));
    void listen("transcript-reset", () => {
      setText("");
    }).then((fn) => unlisteners.push(fn));
    return () => {
      for (const fn of unlisteners) {
        fn();
      }
    };
  }, []);

  const listening = state === "listening" || state === "preparing";
  const processing =
    state === "transcribing" ||
    state === "normalizing" ||
    state === "injecting";
  const showWave = state === "listening" || state === "speech_detected";

  const listenHint =
    listenMode === "long"
      ? "Long listen — Space or Esc"
      : listenMode === "awaiting"
        ? "Tap Ctrl+B again for long"
        : "Listening";

  if (showWave) {
    return (
      <div className="overlay is-listening">
        <div className="pill pill-listen" aria-label={listenHint}>
          <span className="pill-dot cancel" />
          <div className="wave">
            {Array.from({ length: 13 }, (_, i) => (
              <i key={i} style={{ animationDelay: `${i * 0.06}s` }} />
            ))}
          </div>
          <span className="pill-dot confirm" />
        </div>
      </div>
    );
  }

  const label = processing
    ? "Processing"
    : listening
      ? listenHint
      : text
        ? text
        : "Dictate";
  const shortcut =
    !processing && !text && listenMode !== "long" ? "Ctrl + B" : null;

  return (
    <div
      className={`overlay ${listening ? "is-listening" : ""} ${processing ? "is-processing" : ""}`}
    >
      <div className="pill pill-hint">
        <span className="pill-text">
          {label}
          {shortcut ? <strong> {shortcut}</strong> : null}
        </span>
      </div>
      <div className="pill pill-mic" aria-hidden="true">
        <svg viewBox="0 0 24 24" width="16" height="16">
          <path
            fill="currentColor"
            d="M12 14a3 3 0 0 0 3-3V6a3 3 0 0 0-6 0v5a3 3 0 0 0 3 3zm5-3a5 5 0 0 1-10 0H5a7 7 0 0 0 6 6.93V20H9v2h6v-2h-2v-2.07A7 7 0 0 0 19 11h-2z"
          />
        </svg>
      </div>
    </div>
  );
}

ReactDOM.createRoot(document.getElementById("root") as HTMLElement).render(
  <React.StrictMode>
    <Overlay />
  </React.StrictMode>,
);
