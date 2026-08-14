import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { useEffect, useState } from "react";
import { useSettings } from "./hooks/useSettings";
import { AboutPage } from "./pages/About";
import { Dashboard } from "./pages/Dashboard";
import { SettingsPage, type SettingsSection } from "./pages/Settings";
import {
  IconDownload,
  IconInfo,
  IconMic,
  IconMonitor,
  IconShield,
  IconSliders,
} from "./ui/icons";

type Page = "dashboard" | SettingsSection | "about";

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

  const openProductSite = async () => {
    try {
      await invoke("open_product_site");
    } catch (err) {
      setPage("about");
      console.error(err);
    }
  };

  useEffect(() => {
    let unlisten: (() => void) | undefined;
    void listen<string>("navigate", (event) => {
      if (event.payload === "settings") {
        setPage("general");
      }
    }).then((fn) => {
      unlisten = fn;
    });
    return () => {
      unlisten?.();
    };
  }, []);

  const settingsOpen =
    page === "general" || page === "system" || page === "privacy";

  return (
    <div className="shell">
      <nav className="sidebar">
        <div className="brand">
          <img src="/logo.png" width="28" height="28" alt="" />
          <span>LocalFlow</span>
          <span className="badge">Local</span>
        </div>
        <div className="nav-label">Dictate</div>
        <button
          type="button"
          className={page === "dashboard" ? "active" : ""}
          onClick={() => setPage("dashboard")}
        >
          <IconMic />
          Dictation
        </button>
        <div className="nav-label">Settings</div>
        <button
          type="button"
          className={page === "general" ? "active" : ""}
          onClick={() => setPage("general")}
        >
          <IconSliders />
          General
        </button>
        <button
          type="button"
          className={page === "system" ? "active" : ""}
          onClick={() => setPage("system")}
        >
          <IconMonitor />
          System
        </button>
        <button
          type="button"
          className={page === "privacy" ? "active" : ""}
          onClick={() => setPage("privacy")}
        >
          <IconShield />
          Data and privacy
        </button>
        <div className="nav-label">App</div>
        <button
          type="button"
          className={page === "about" ? "active" : ""}
          onClick={() => setPage("about")}
        >
          <IconInfo />
          About
        </button>
        <div className="sidebar-spacer" />
        <div className="sidebar-foot">
          <span>LocalFlow v0.1.1</span>
          <button
            type="button"
            className="nav-quiet"
            title="Download page"
            onClick={() => void openProductSite()}
          >
            <IconDownload size={16} />
          </button>
        </div>
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
            onOpenSettings={() => setPage("general")}
            onRecordTest={() => void recordTest()}
            onRevealRecording={(path) => void revealRecording(path)}
            onLongListen={() => void startLongListen()}
            onShortDown={() => void startShortListen()}
            onShortUp={() => void stopShortListen()}
          />
        ) : null}
        {settingsOpen && !settings ? <p className="lede">Loading settings…</p> : null}
        {settingsOpen && settings ? (
          <SettingsPage
            section={page}
            settings={settings}
            microphones={microphones}
            saving={saving}
            onChange={save}
            onRecordTest={() => void recordTest()}
            recording={recording}
            lastRecording={lastRecording}
            onRevealRecording={(path) => void revealRecording(path)}
          />
        ) : null}
        {page === "about" ? <AboutPage /> : null}
      </main>
    </div>
  );
}
