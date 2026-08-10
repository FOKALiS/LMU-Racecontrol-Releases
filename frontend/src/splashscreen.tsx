import React, { useEffect, useState, useCallback } from "react";
import ReactDOM from "react-dom/client";
import { invoke } from "@tauri-apps/api/core";
import { getVersion } from "@tauri-apps/api/app";
import { check, type Update } from "@tauri-apps/plugin-updater";
import { relaunch } from "@tauri-apps/plugin-process";
import { open } from "@tauri-apps/plugin-shell";
import "./fonts.css";
import "./styles.css";

const COUNTDOWN_SECONDS = 5;

function detectLang(): "de" | "en" {
  const stored = localStorage.getItem("lmu-rc-lang");
  if (stored === "de" || stored === "en") return stored;
  return (navigator.language || "en").toLowerCase().startsWith("de") ? "de" : "en";
}

const TEXT = {
  de: {
    updateAvailable: "UPDATE VERFÜGBAR",
    version: "Version",
    installing: "Update wird installiert...",
    installError: "Update-Installation fehlgeschlagen",
    website: "www.lmu-racecontrol.com",
    copyright: "Copyright © 2026 by FOKALiS - Film & Medienagentur",
  },
  en: {
    updateAvailable: "UPDATE AVAILABLE",
    version: "Version",
    installing: "Installing update...",
    installError: "Update installation failed",
    website: "www.lmu-racecontrol.com",
    copyright: "Copyright © 2026 by FOKALiS - Film & Medienagentur",
  },
};

function Splashscreen() {
  const lang = detectLang();
  const t = TEXT[lang];

  const [appVersion, setAppVersion] = useState("");
  const [secondsLeft, setSecondsLeft] = useState(COUNTDOWN_SECONDS);
  const [update, setUpdate] = useState<Update | null>(null);
  const [installing, setInstalling] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [proceeded, setProceeded] = useState(false);

  const proceedToMain = useCallback(() => {
    setProceeded((already) => {
      if (already) return true;
      invoke("show_main_window").catch(console.error);
      return true;
    });
  }, []);

  useEffect(() => {
    getVersion().then(setAppVersion).catch(console.error);
    check()
      .then((result) => {
        if (result) setUpdate(result);
      })
      .catch((err) => console.error("Update-Prüfung fehlgeschlagen:", err));

    const start = Date.now();
    const interval = setInterval(() => {
      const remaining = Math.max(0, COUNTDOWN_SECONDS - Math.floor((Date.now() - start) / 1000));
      setSecondsLeft(remaining);
      if (remaining === 0) clearInterval(interval);
    }, 200);
    return () => clearInterval(interval);
  }, []);

  useEffect(() => {
    if (secondsLeft === 0 && !installing) proceedToMain();
  }, [secondsLeft, installing, proceedToMain]);

  async function handleInstallUpdate() {
    if (!update) return;
    setInstalling(true);
    setError(null);
    try {
      await update.downloadAndInstall();
      await relaunch();
    } catch (err) {
      console.error(err);
      setError(t.installError);
      setInstalling(false);
    }
  }

  return (
    <main className="splash">
      {/* Figma exakt: 850x600, bg-neutral-900, rounded-[10px], outline, flex-col, items-center, gap-10 */}
      <div className="splash-frame">
        {/* Logo – Figma: w-96 h-52 (384x208) */}
        <img className="splash-logo" src="/logo.png" alt="LMU RaceControl" />

        {/* Version – Figma: 728x48, text-center, white, text-sm, Michroma, leading-6, tracking-wide */}
        <div className="splash-version">
          {t.version} {appVersion}
        </div>

        {/* Notices – Figma: self-stretch, flex-col, items-center, gap-3 */}
        <div className="splash-notices">
          {update && (
            <button
              className="splash-bar splash-bar-update"
              onClick={handleInstallUpdate}
              disabled={installing}
            >
              {installing ? (
                t.installing
              ) : (
                <>
                  {t.updateAvailable}
                  <br />
                  {t.version} {update.version}
                </>
              )}
            </button>
          )}

          <button
            className="splash-bar splash-bar-link"
            onClick={() => open("https://www.lmu-racecontrol.com").catch(console.error)}
          >
            {t.website}
          </button>
        </div>

        {error && <div className="splash-error">{error}</div>}

        {/* Footer – Figma: self-stretch, text-center, white/60, text-xs, Michroma, leading-5, tracking-tight */}
        <footer className="splash-footer">
          {t.copyright}
        </footer>
      </div>
    </main>
  );
}

ReactDOM.createRoot(document.getElementById("root")!).render(
  <React.StrictMode>
    <Splashscreen />
  </React.StrictMode>
);