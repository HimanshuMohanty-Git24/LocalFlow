import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { useCallback, useEffect, useState } from "react";
import type {
  AppStatus,
  AppState,
  MicTestResult,
  Microphone,
  Settings,
} from "../types/settings";

export function useSettings() {
  const [settings, setSettings] = useState<Settings | null>(null);
  const [status, setStatus] = useState<AppStatus | null>(null);
  const [microphones, setMicrophones] = useState<Microphone[]>([]);
  const [error, setError] = useState<string | null>(null);
  const [saving, setSaving] = useState(false);
  const [recording, setRecording] = useState(false);
  const [lastRecording, setLastRecording] = useState<MicTestResult | null>(
    null,
  );
  const [lastTranscript, setLastTranscript] = useState<string | null>(null);
  const [lastCleaned, setLastCleaned] = useState<string | null>(null);

  const refresh = useCallback(async () => {
    try {
      const [nextSettings, nextStatus, nextMics] = await Promise.all([
        invoke<Settings>("get_settings"),
        invoke<AppStatus>("get_status"),
        invoke<Microphone[]>("list_microphones").catch(() => []),
      ]);
      setSettings(nextSettings);
      setStatus(nextStatus);
      setMicrophones(nextMics);
      setError(null);
    } catch (err) {
      setError(err instanceof Error ? err.message : String(err));
    }
  }, []);

  useEffect(() => {
    void refresh();
  }, [refresh]);

  useEffect(() => {
    const unlisteners: Array<() => void> = [];
    void listen<AppState>("flow-state", (event) => {
      setStatus((prev) =>
        prev ? { ...prev, state: event.payload } : prev,
      );
      setRecording(
        event.payload === "listening" ||
          event.payload === "preparing" ||
          event.payload === "transcribing",
      );
    }).then((fn) => unlisteners.push(fn));
    void listen<MicTestResult>("recording-finished", (event) => {
      setLastRecording(event.payload);
    }).then((fn) => unlisteners.push(fn));
    void listen("transcript-reset", () => {
      setLastTranscript(null);
      setLastCleaned(null);
    }).then((fn) => unlisteners.push(fn));
    void listen<string>("raw-transcript", (event) => {
      setLastTranscript(event.payload);
      setRecording(false);
    }).then((fn) => unlisteners.push(fn));
    void listen<string>("clean-transcript", (event) => {
      setLastCleaned(event.payload);
    }).then((fn) => unlisteners.push(fn));
    void listen("no-speech", () => {
      setLastCleaned("(no speech)");
      setRecording(false);
    }).then((fn) => unlisteners.push(fn));
    void listen<string>("asr-model", (event) => {
      setStatus((prev) =>
        prev ? { ...prev, asr_model: event.payload } : prev,
      );
    }).then((fn) => unlisteners.push(fn));
    void listen<string>("flow-error", (event) => {
      setError(event.payload);
      setRecording(false);
    }).then((fn) => unlisteners.push(fn));
    return () => {
      for (const fn of unlisteners) {
        fn();
      }
    };
  }, []);

  const save = useCallback(async (next: Settings) => {
    setSaving(true);
    try {
      const saved = await invoke<Settings>("update_settings", { next });
      setSettings(saved);
      const nextStatus = await invoke<AppStatus>("get_status");
      setStatus(nextStatus);
      setError(null);
    } catch (err) {
      setError(err instanceof Error ? err.message : String(err));
    } finally {
      setSaving(false);
    }
  }, []);

  const recordTest = useCallback(async () => {
    setRecording(true);
    setError(null);
    try {
      const result = await invoke<MicTestResult>("record_microphone_test");
      setLastRecording(result);
      const nextStatus = await invoke<AppStatus>("get_status");
      setStatus(nextStatus);
    } catch (err) {
      setError(err instanceof Error ? err.message : String(err));
    } finally {
      setRecording(false);
    }
  }, []);

  const revealRecording = useCallback(async (path: string) => {
    try {
      await invoke("reveal_recording", { path });
    } catch (err) {
      setError(err instanceof Error ? err.message : String(err));
    }
  }, []);

  const startLongListen = useCallback(async () => {
    try {
      await invoke("start_long_listen");
    } catch (err) {
      setError(err instanceof Error ? err.message : String(err));
    }
  }, []);

  const startShortListen = useCallback(async () => {
    try {
      await invoke("start_short_listen");
    } catch (err) {
      setError(err instanceof Error ? err.message : String(err));
    }
  }, []);

  const stopShortListen = useCallback(async () => {
    try {
      await invoke("stop_short_listen");
    } catch (err) {
      setError(err instanceof Error ? err.message : String(err));
    }
  }, []);

  return {
    settings,
    status,
    microphones,
    error,
    saving,
    recording,
    lastRecording,
    lastTranscript,
    lastCleaned,
    refresh,
    save,
    recordTest,
    revealRecording,
    startLongListen,
    startShortListen,
    stopShortListen,
  };
}
