import type { Microphone, Settings } from "../types/settings";

type Props = {
  settings: Settings;
  microphones: Microphone[];
  saving: boolean;
  onChange: (next: Settings) => void;
};

export function SettingsPage({
  settings,
  microphones,
  saving,
  onChange,
}: Props) {
  return (
    <section>
      <h1>Settings</h1>
      <p className="lede">
        These values stay on this PC. They are not persisted after quit yet.
      </p>
      <form
        className="stack"
        onSubmit={(event) => {
          event.preventDefault();
          onChange(settings);
        }}
      >
        <label>
          Microphone
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
                {mic.sample_rate
                  ? ` · ${mic.sample_rate} Hz / ${mic.channels} ch`
                  : ""}
              </option>
            ))}
          </select>
        </label>
        <label>
          Dictation hotkey
          <input value={settings.dictation_hotkey} readOnly />
        </label>
        <label>
          Hotkey mode
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
        </label>
        <label className="check">
          <input
            type="checkbox"
            checked={settings.start_on_login}
            onChange={(event) =>
              onChange({ ...settings, start_on_login: event.target.checked })
            }
          />
          Start LocalFlow when I sign in (not wired yet)
        </label>
        <label className="check">
          <input
            type="checkbox"
            checked={settings.preserve_clipboard}
            onChange={(event) =>
              onChange({
                ...settings,
                preserve_clipboard: event.target.checked,
              })
            }
          />
          Preserve clipboard after insertion
        </label>
        <label className="check">
          <input
            type="checkbox"
            checked={settings.llm_enabled}
            onChange={(event) =>
              onChange({
                ...settings,
                llm_enabled: event.target.checked,
              })
            }
          />
          AI cleanup with local Qwen (offline)
        </label>
        <label className="check">
          <input
            type="checkbox"
            checked={settings.save_text_history}
            onChange={(event) =>
              onChange({
                ...settings,
                save_text_history: event.target.checked,
              })
            }
          />
          Save text history (off by default)
        </label>
        <label className="check">
          <input
            type="checkbox"
            checked={settings.save_audio}
            onChange={(event) =>
              onChange({ ...settings, save_audio: event.target.checked })
            }
          />
          Save audio recordings (off by default)
        </label>
        <button type="submit" disabled={saving}>
          {saving ? "Saving…" : "Save"}
        </button>
      </form>
    </section>
  );
}
