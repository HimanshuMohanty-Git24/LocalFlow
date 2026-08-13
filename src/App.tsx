import { listen } from "@tauri-apps/api/event";
import { useEffect, useState } from "react";
import { useSettings } from "./hooks/useSettings";
import { AboutPage } from "./pages/About";
import { Dashboard } from "./pages/Dashboard";
import { SettingsPage } from "./pages/Settings";

type Page = "dashboard" | "settings" | "about";

export default function App() {
  const [page, setPage] = useState<Page>("dashboard");
  const {
    settings,
    status,
    microphones,
    error,
    saving,
    recording,
    lastRecording,
    lastTranscript,
    lastCleaned,
    save,
    recordTest,
    revealRecording,
    startLongListen,
    startShortListen,
    stopShortListen,
  } = useSettings();

  useEffect(() => {
    let unlisten: (() => void) | undefined;
    void listen<string>("navigate", (event) => {
      if (event.payload === "settings") {
        setPage("settings");
      }
    }).then((fn) => {
      unlisten = fn;
    });
    return () => {
      unlisten?.();
    };
  }, []);

  return (
    <div className="shell">
      <nav>
        <button
          type="button"
          className={page === "dashboard" ? "active" : ""}
          onClick={() => setPage("dashboard")}
        >
          Dashboard
        </button>
        <button
          type="button"
          className={page === "settings" ? "active" : ""}
          onClick={() => setPage("settings")}
        >
          Settings
        </button>
        <button
          type="button"
          className={page === "about" ? "active" : ""}
          onClick={() => setPage("about")}
        >
          About
        </button>
      </nav>
      <main>
        {error ? <p className="error">{error}</p> : null}
        {page === "dashboard" ? (
          <Dashboard
            status={status}
            recording={recording}
            lastRecording={lastRecording}
            lastTranscript={lastTranscript}
            lastCleaned={lastCleaned}
            onOpenSettings={() => setPage("settings")}
            onRecordTest={() => void recordTest()}
            onRevealRecording={(path) => void revealRecording(path)}
            onLongListen={() => void startLongListen()}
            onShortDown={() => void startShortListen()}
            onShortUp={() => void stopShortListen()}
          />
        ) : null}
        {page === "settings" && settings ? (
          <SettingsPage
            settings={settings}
            microphones={microphones}
            saving={saving}
            onChange={save}
          />
        ) : null}
        {page === "about" ? <AboutPage /> : null}
      </main>
    </div>
  );
}
