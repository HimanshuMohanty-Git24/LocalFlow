import type { Microphone, MicTestResult, Settings } from "../types/settings";
import { Toggle } from "../ui/Toggle";

export type SettingsSection = "general" | "system" | "privacy";

type Props = {
  section: SettingsSection;
  settings: Settings;
  microphones: Microphone[];
  saving: boolean;
  recording: boolean;
  lastRecording: MicTestResult | null;
  onChange: (next: Settings) => void;
  onRecordTest: () => void;
  onRevealRecording: (path: string) => void;
};

const TITLES: Record<SettingsSection, string> = {
  general: "General",
  system: "System",
  privacy: "Data and privacy",
};

export function SettingsPage({
  section,
  settings,
  microphones,
  saving,
  recording,
  lastRecording,
  onChange,
  onRecordTest,
  onRevealRecording,
}: Props) {
  return (
    <section className="page">
      <h1>{TITLES[section]}</h1>
      <p className="lede">
        These values stay on this PC. They are not persisted after quit yet.
      </p>

      {section === "general" ? (
        <div className="card">
          <div className="setting-row">
            <div className="setting-copy">
              <span className="setting-title">Microphone</span>
              <span className="setting-desc">
                Used for dictation and the 5-second test.
              </span>
            </div>
            <select
              value={settings.microphone_id}
              onChange={(event) =>
                onChange({ ...settings, microphone_id: event.target.value })
              }
            >
              <option value="">System default</option>
              {microphones.map((mic) => (
                <option key={mic.id} value={mic.id}>
                  {mic.name}
                  {mic.is_default ? " (default)" : ""}
                </option>
              ))}
            </select>
          </div>
          <div className="setting-row">
            <div className="setting-copy">
              <span className="setting-title">Shortcuts</span>
              <span className="setting-desc">
                Hold {settings.dictation_hotkey} for short listen. Double-tap
                for long listen.
              </span>
            </div>
            <input
              className="field"
              value={settings.dictation_hotkey}
              readOnly
            />
          </div>
          <div className="setting-row">
            <div className="setting-copy">
              <span className="setting-title">Hotkey mode</span>
              <span className="setting-desc">
                Push to talk holds the key. Toggle starts and stops with
                separate presses.
              </span>
            </div>
            <select
              value={settings.hotkey_mode}
              onChange={(event) =>
                onChange({
                  ...settings,
                  hotkey_mode: event.target.value as Settings["hotkey_mode"],
                })
              }
            >
              <option value="push_to_talk">Push to talk</option>
              <option value="toggle">Toggle</option>
            </select>
          </div>
          <div className="setting-row">
            <div className="setting-copy">
              <span className="setting-title">Test microphone</span>
              <span className="setting-desc">
                Records 5 seconds and saves a WAV on this PC.
                {lastRecording
                  ? ` Last test: ${lastRecording.duration_ms / 1000}s.`
                  : ""}
              </span>
            </div>
            <div className="actions" style={{ margin: 0 }}>
              <button
                type="button"
                className="ghost"
                onClick={onRecordTest}
                disabled={recording}
              >
                {recording ? "Recording…" : "Record"}
              </button>
              {lastRecording ? (
                <button
                  type="button"
                  className="ghost"
                  onClick={() => onRevealRecording(lastRecording.path)}
                >
                  Show WAV
                </button>
              ) : null}
            </div>
          </div>
        </div>
      ) : null}

      {section === "system" ? (
        <>
          <p className="group-title">App settings</p>
          <div className="card">
            <div className="setting-row">
              <div className="setting-copy">
                <span className="setting-title">Launch app at login</span>
                <span className="setting-desc">
                  Start LocalFlow when you sign in to Windows. Not wired yet.
                </span>
              </div>
              <Toggle
                on={settings.start_on_login}
                label="Launch app at login"
                onChange={(start_on_login) =>
                  onChange({ ...settings, start_on_login })
                }
              />
            </div>
            <div className="setting-row">
              <div className="setting-copy">
                <span className="setting-title">Preserve clipboard</span>
                <span className="setting-desc">
                  Restore the previous clipboard after inserting dictation.
                </span>
              </div>
              <Toggle
                on={settings.preserve_clipboard}
                label="Preserve clipboard"
                onChange={(preserve_clipboard) =>
                  onChange({ ...settings, preserve_clipboard })
                }
              />
            </div>
          </div>
          <p className="group-title">Cleanup</p>
          <div className="card">
            <div className="setting-row">
              <div className="setting-copy">
                <span className="setting-title">AI cleanup</span>
                <span className="setting-desc">
                  Optional local Qwen. Dictation still works without it.
                </span>
              </div>
              <Toggle
                on={settings.llm_enabled}
                label="AI cleanup"
                onChange={(llm_enabled) =>
                  onChange({ ...settings, llm_enabled })
                }
              />
            </div>
          </div>
          <p className="hint">
            Qwen is not in the installer. Download{" "}
            <code>Qwen3-0.6B-Q8_0.gguf</code> from Hugging Face
            (Qwen/Qwen3-0.6B-GGUF), copy it into{" "}
            <code>%LOCALAPPDATA%\LocalFlow\models</code>, then quit and reopen.
          </p>
        </>
      ) : null}

      {section === "privacy" ? (
        <div className="card">
          <div className="setting-row">
            <div className="setting-copy">
              <span className="setting-title">Network</span>
              <span className="setting-desc">
                Audio, transcripts, and settings never leave this machine. There
                is no account and no cloud sync.
              </span>
            </div>
            <span className="badge">Offline</span>
          </div>
          <div className="setting-row">
            <div className="setting-copy">
              <span className="setting-title">Save text history</span>
              <span className="setting-desc">
                Keep recent transcripts on disk. Off by default.
              </span>
            </div>
            <Toggle
              on={settings.save_text_history}
              label="Save text history"
              onChange={(save_text_history) =>
                onChange({ ...settings, save_text_history })
              }
            />
          </div>
          <div className="setting-row">
            <div className="setting-copy">
              <span className="setting-title">Save audio recordings</span>
              <span className="setting-desc">
                Keep dictation WAVs on this PC. Off by default.
              </span>
            </div>
            <Toggle
              on={settings.save_audio}
              label="Save audio recordings"
              onChange={(save_audio) => onChange({ ...settings, save_audio })}
            />
          </div>
        </div>
      ) : null}

      {saving ? <p className="saving">Saving…</p> : null}
    </section>
  );
}
