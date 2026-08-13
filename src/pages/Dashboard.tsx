import type { AppStatus, MicTestResult } from "../types/settings";

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

function Row({ label, value }: { label: string; value: string }) {
  return (
    <div className="row-line">
      <span>{label}</span>
      <strong>{value}</strong>
    </div>
  );
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
  return (
    <section>
      <h1>LocalFlow</h1>
      <p className="lede">
        Completely offline. Click into Notepad, then use one of the two
        shortcuts below. Silence is ignored. Local rules plus optional Qwen
        (on this PC) clean the text before paste.
      </p>
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
          <strong>Short</strong>
          <span>Hold this button, or hold Ctrl+B. Speak, then release.</span>
        </button>
        <button type="button" className="mode-card" onClick={onLongListen}>
          <strong>Long</strong>
          <span>
            Click here, or press Ctrl+B twice. Speak. Press Space or Esc to
            stop.
          </span>
        </button>
      </div>
      <div className="card">
        <Row label="Status" value={status?.state ?? "loading"} />
        <Row label="Microphone" value={status?.microphone ?? "—"} />
        <Row label="Model" value={status?.asr_model ?? "—"} />
        <Row label="AI cleanup" value={status?.llm_enabled ? "On" : "Off"} />
        <Row label="Hotkey" value={status?.hotkey ?? "—"} />
        <Row
          label="Network"
          value={status?.offline ? "Offline by design" : "Unexpected"}
        />
      </div>
      <div className="actions">
        <button type="button" onClick={onRecordTest} disabled={recording}>
          {recording ? "Recording 5 seconds…" : "Test microphone"}
        </button>
        <button type="button" onClick={onOpenSettings}>
          Open settings
        </button>
      </div>
      {lastCleaned !== null || lastTranscript !== null ? (
        <div className="card">
          {lastCleaned !== null ? (
            <Row label="Inserted" value={lastCleaned || "(empty)"} />
          ) : null}
          {lastTranscript !== null && lastTranscript !== lastCleaned ? (
            <Row label="Raw Whisper" value={lastTranscript || "(empty)"} />
          ) : null}
        </div>
      ) : (
        <p className="lede">
          Put a Whisper ggml file in <code>models/</code> (see
          docs/models.md). Then use Short or Long dictation.
        </p>
      )}
      {lastRecording ? (
        <p className="lede">
          Saved {lastRecording.duration_ms / 1000}s WAV at{" "}
          {lastRecording.sample_rate} Hz, {lastRecording.channels} ch (
          {lastRecording.frames} frames).
          <br />
          <button
            type="button"
            onClick={() => onRevealRecording(lastRecording.path)}
          >
            Show WAV in Explorer
          </button>
        </p>
      ) : null}
    </section>
  );
}
