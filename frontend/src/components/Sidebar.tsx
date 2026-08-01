import { useState, useEffect, useRef } from "react";
import type { View, FcyPhase } from "../types";
import { useLanguage } from "../i18n/LanguageContext";
import { getVersion } from "@tauri-apps/api/app";
import { open } from "@tauri-apps/plugin-shell";
import LanguageToggle from "./LanguageToggle";
import HelpModal from "./HelpModal";

interface Props {
  view: View;
  onNavigate: (v: View) => void;
  connected: boolean;
  onConnect: () => void;
  fcyPhase: FcyPhase;
  fcyRemaining: number;
  onFcyClick: () => void;
  /** Solange keine gültige Lizenz vorliegt: nur Logo, Sprache, Hilfe und
   * Website sichtbar - keine Funktionen, keine Navigation zu den übrigen
   * Ansichten. */
  licensed: boolean;
}

export default function Sidebar({
  view,
  onNavigate,
  connected,
  onConnect,
  fcyPhase,
  fcyRemaining,
  onFcyClick,
  licensed,
}: Props) {
  const { t } = useLanguage();
  const [helpOpen, setHelpOpen] = useState(false);
  const [appVersion, setAppVersion] = useState("");
  const [isHoveringConnect, setIsHoveringConnect] = useState(false);

  useEffect(() => {
    getVersion().then(setAppVersion).catch(console.error);
  }, []);

  function getConnectLabel() {
    if (!connected) return t("connect_to_server");
    if (isHoveringConnect) return t("server_disconnect");
    return t("server_connected");
  }

  return (
    <aside className="sidebar">
      <div className="sidebar-logo">
        <img src="/logo.png" alt="LMU Racecontrol" className="sidebar-logo-img" />
      </div>

      {licensed && (
        <div className="sidebar-body">
          <div className="sidebar-section">
            <div className="sidebar-section-label">{t("sidebar_functions")}</div>
            <button
              className={`nav-btn nav-btn-connect ${connected ? "is-connected" : ""}`}
              onClick={onConnect}
              onMouseEnter={() => setIsHoveringConnect(true)}
              onMouseLeave={() => setIsHoveringConnect(false)}
            >
              {getConnectLabel()}
            </button>
          </div>

          {connected && (
            <>
              <div className="sidebar-section">
                <div className="sidebar-section-label">{t("sidebar_control")}</div>
                <button
                  className={`nav-btn ${view === "fahrerfeld" ? "active" : ""}`}
                  onClick={() => onNavigate("fahrerfeld")}
                >
                  {t("nav_fahrerfeld")}
                </button>
              </div>

              <div className="sidebar-section">
                <div className="sidebar-section-label">{t("sidebar_race_control")}</div>
                <button
                  className={`nav-btn ${view === "vorfaelle" ? "active" : ""}`}
                  onClick={() => onNavigate("vorfaelle")}
                >
                  {t("nav_vorfaelle")}
                </button>
                <button
                  className={`nav-btn ${view === "archiv" ? "active" : ""}`}
                  onClick={() => onNavigate("archiv")}
                >
                  {t("nav_archiv")}
                </button>
              </div>
            </>
          )}
        </div>
      )}

      {licensed && connected && (
        <button className={`fcy-btn fcy-${fcyPhase}`} onClick={onFcyClick}>
          {fcyPhase === "idle" && "FCY"}
          {fcyPhase === "countdown" && fcyRemaining}
          {fcyPhase === "active" && t("fcy_active_short")}
        </button>
      )}

      <div className="sidebar-body sidebar-body-bottom">
        <div className="sidebar-section">
          <div className="sidebar-section-label">{t("sidebar_software_infos")}</div>
          <LanguageToggle full />
          {licensed && (
            <button
              className={`nav-btn-outline nav-btn-outline-block ${view === "einstellungen" ? "active" : ""}`}
              onClick={() => onNavigate("einstellungen")}
            >
              {t("nav_einstellungen")}
            </button>
          )}
          <button
            className="nav-btn-outline nav-btn-outline-block"
            onClick={() => setHelpOpen(true)}
            title={t("help_tooltip")}
          >
            {t("nav_hilfe")}
          </button>
          <button
            className="nav-btn-outline nav-btn-outline-block"
            onClick={() => open("https://www.lmu-racecontrol.gg").catch(console.error)}
          >
            www.lmu-racecontrol.gg
          </button>
        </div>
      </div>

      <div className="sidebar-footer">
        {t("footer_copyright")}
        <span className="sidebar-footer-product">{t("footer_product")}</span>
        <span className="sidebar-footer-version">Version {appVersion}</span>
      </div>

      {helpOpen && <HelpModal onClose={() => setHelpOpen(false)} />}
    </aside>
  );
}