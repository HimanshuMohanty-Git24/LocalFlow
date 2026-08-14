import { useState } from "react";
import type { AppStatus, MicTestResult } from "../types/settings";
import { IconCopy, IconFolder } from "../ui/icons";

type Props = {
  status: AppStatus | null;
  recording: boolean;
  lastRecording: MicTestResult | null;
  lastTranscript: string | null;
  lastCleaned: string | null;
  onOpenSettings: () => void;
  onRecordTest: () => void;
  onRevealRecording: (path: string) => void;
  onLongListen: () => void;
  onShortDown: () => void;
  onShortUp: () => void;
};

const STATE_LABEL: Record<string, string> = {
  idle: "Ready",
  preparing: "Preparing",
  listening: "Listening",
  speech_detected: "Hearing you",
  transcribing: "Transcribing",
  normalizing: "Cleaning",
  injecting: "Inserting",
  error: "Error",
};

function modelName(value: string | undefined) {
  if (!value || value === "—") return "Not loaded";
  return value.replace(/^.*[\\/]/, "").replace(/\.(bin|gguf)$/i, "");
}

export function Dashboard({
  status,
  recording,
  lastRecording,
  lastTranscript,
  lastCleaned,
  onOpenSettings,
  onRecordTest,
  onRevealRecording,
  onLongListen,
  onShortDown,
  onShortUp,
}: Props) {
  const [copied, setCopied] = useState(false);
  const inserted = lastCleaned ?? lastTranscript;
  const hotkey = status?.hotkey ?? "Ctrl+B";

  const copyInserted = async () => {
    if (!inserted) return;
    try {
      await navigator.clipboard.writeText(inserted);
      setCopied(true);
      window.setTimeout(() => setCopied(false), 1400);
    } catch {
      setCopied(false);
    }
  };

  return (
    <section className="page">
      <h1>Dictation</h1>
      <p className="lede">
        Completely offline. Click into a text field, then hold {hotkey}.
        Silence is ignored. Optional local Qwen cleans the text before paste.
      </p>

      <div className="banner">
        <div>
          <h2>
            Speak once. Paste <em>here</em>.
          </h2>
          <p>
            Hold <kbd>{hotkey}</kbd> to dictate. Double-tap it for long listen.
            Nothing leaves this PC.
          </p>
        </div>
      </div>

      <div className="stat-grid">
        <div className="stat">
          <span>Status</span>
          <strong>
            {STATE_LABEL[status?.state ?? "idle"] ?? status?.state ?? "Loading"}
          </strong>
        </div>
        <div className="stat">
          <span>Microphone</span>
          <strong>{status?.microphone ?? "—"}</strong>
        </div>
        <div className="stat">
          <span>Whisper</span>
          <strong>{modelName(status?.asr_model)}</strong>
        </div>
      </div>

      <div className="mode-grid">
        <button
          type="button"
          className="mode-card"
          onPointerDown={(event) => {
            event.preventDefault();
            onShortDown();
          }}
          onPointerUp={onShortUp}
          onPointerCancel={onShortUp}
        >
          <strong>Short listen</strong>
          <span>Hold this, or hold {hotkey}. Speak, then release.</span>
        </button>
        <button type="button" className="mode-card" onClick={onLongListen}>
          <strong>Long listen</strong>
          <span>Click here, or press {hotkey} twice. Space or Esc to stop.</span>
        </button>
      </div>

      {inserted ? (
        <div className="card">
          <div className="feed-item">
            <div className="feed-meta">
              <span>{copied ? "Copied" : "Latest"}</span>
              <div className="feed-actions">
                <button
                  type="button"
                  className="icon-btn"
                  title="Copy"
                  onClick={() => void copyInserted()}
                >
                  <IconCopy size={16} />
                </button>
              </div>
            </div>
            <div>{inserted || "(empty)"}</div>
            {lastTranscript && lastCleaned && lastTranscript !== lastCleaned ? (
              <p className="hint">Raw Whisper: {lastTranscript}</p>
            ) : null}
          </div>
        </div>
      ) : (
        <p className="hint">
          Put a Whisper ggml file in <code>models/</code>, then use Short or
          Long. See docs/models.md.
        </p>
      )}

      <div className="actions">
        <button
          type="button"
          className="primary"
          onClick={onRecordTest}
          disabled={recording}
        >
          {recording ? "Recording…" : "Test microphone"}
        </button>
        <button type="button" className="ghost" onClick={onOpenSettings}>
          Open settings
        </button>
        {lastRecording ? (
          <button
            type="button"
            className="ghost"
            onClick={() => onRevealRecording(lastRecording.path)}
          >
            <IconFolder size={16} /> Show WAV
          </button>
        ) : null}
      </div>
    </section>
  );
}
