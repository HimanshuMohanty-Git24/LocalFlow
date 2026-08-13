export type HotkeyMode = "push_to_talk" | "toggle";

export type AppState =
  | "idle"
  | "preparing"
  | "listening"
  | "speech_detected"
  | "transcribing"
  | "normalizing"
  | "injecting"
  | "error";

export type Settings = {
  dictation_hotkey: string;
  hotkey_mode: HotkeyMode;
  microphone_id: string;
  start_on_login: boolean;
  preserve_clipboard: boolean;
  save_text_history: boolean;
  save_audio: boolean;
  llm_enabled: boolean;
};

export type Microphone = {
  id: string;
  name: string;
  sample_rate: number;
  channels: number;
  is_default: boolean;
};

export type MicTestResult = {
  path: string;
  duration_ms: number;
  sample_rate: number;
  channels: number;
  frames: number;
};

export type AppStatus = {
  state: AppState;
  microphone: string;
  asr_model: string;
  llm_enabled: boolean;
  hotkey: string;
  offline: boolean;
};
