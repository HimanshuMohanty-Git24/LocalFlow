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
      if (
        event.payload === "listening" ||
        event.payload === "preparing"
      ) {
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
  const processing = state === "transcribing";
  const listenHint =
    listenMode === "long"
      ? "Long listen — Space or Esc to stop"
      : listenMode === "awaiting"
        ? "Tap Ctrl+B again for long, or wait"
        : "Listening — release to stop";
  const hint = listening
    ? listenHint
    : processing
      ? "Processing"
      : text
        ? text
        : "Dictate Ctrl + B";

  return (
    <div
      className={`overlay ${listening ? "is-listening" : ""} ${processing ? "is-processing" : ""}`}
    >
      <div className="hint">
        {listening || processing ? (
          <>
            {hint}
            {listenMode !== "long" ? (
              <>
                {" "}
                <strong>Ctrl + B</strong>
              </>
            ) : null}
          </>
        ) : (
          hint
        )}
      </div>
      <div className="mic" aria-hidden="true">
        <svg viewBox="0 0 24 24" width="18" height="18">
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
