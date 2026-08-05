import { useState, useEffect } from "react";
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
      {/* Logo – Figma: py-9, bg-sky-800/20, 184x87 */}
      <div className="sidebar-logo">
        <img src="/logo.png" alt="LMU Racecontrol" />
      </div>

      {/* Language Toggle – Figma: SEPARATER Block zwischen Logo und Server */}
      <div className="sidebar-lang">
        <LanguageToggle full />
      </div>

      {/* Server Section – Figma: Label "Server" + Divider + Connect-Button (56px) */}
      <div className="sidebar-section sidebar-section-server">
        <div className="sidebar-section-label">{t("sidebar_server")}</div>
        <div className="sidebar-divider" />
        <button
          className={`nav-btn nav-btn-connect ${connected ? "is-connected" : ""}`}
          onClick={onConnect}
          onMouseEnter={() => setIsHoveringConnect(true)}
          onMouseLeave={() => setIsHoveringConnect(false)}
        >
          {getConnectLabel()}
        </button>
      </div>

      {!connected && (
        <>
          {/* Placeholder_SB – Figma: Container Fill, min-height 393px */}
          <div className="sidebar-placeholder">
            <div className="sidebar-placeholder-inner" />
          </div>
        </>
      )}

      {connected && licensed && (
        <>
          {/* Steuerung Section – Figma: vertical: fill (Spacer) */}
          <div className="sidebar-section sidebar-section-control">
            <div className="sidebar-section-label">{t("sidebar_live_view")}</div>
            <button
              className={`nav-btn ${view === "fahrerfeld" ? "active" : ""}`}
              onClick={() => onNavigate("fahrerfeld")}
            >
              {t("nav_fahrerfeld")}
            </button>
          </div>

          {/* Race Control Section – Figma: flex-1, gap 6px */}
          <div className="sidebar-section sidebar-section-race">
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
            {/* FCY-Button – Figma: 193x85, gelb, border-radius 5px */}
            <button className={`fcy-btn fcy-${fcyPhase}`} onClick={onFcyClick}>
              {fcyPhase === "idle" && "FCY"}
              {fcyPhase === "countdown" && fcyRemaining}
              {fcyPhase === "active" && t("fcy_active_short")}
            </button>
          </div>
        </>
      )}

      {/* Software Infos Section – Figma: Label + Divider + Buttons */}
      <div className="sidebar-section sidebar-section-infos">
        <div className="sidebar-section-label">{t("sidebar_software_infos")}</div>
        <div className="sidebar-divider" />
        <button
          className="nav-btn-outline"
          onClick={() => setHelpOpen(true)}
          title={t("help_tooltip")}
        >
          {t("nav_hilfe")}
        </button>
        <button
          className={`nav-btn-outline ${view === "einstellungen" ? "active" : ""}`}
          onClick={() => onNavigate("einstellungen")}
        >
          {t("nav_einstellungen")}
        </button>
        <button
          className="nav-btn-outline"
          onClick={() => open("https://www.lmu-racecontrol.gg").catch(console.error)}
        >
          www.lmu-racecontrol.gg
        </button>
      </div>

      {/* Copyright – Figma: 3-zeilig, bg-sky-800/20, padding 14px 10px */}
      <div className="sidebar-footer">
        <span>{t("footer_copyright")}</span>
        <span className="sidebar-footer-product">{t("footer_product")}</span>
        <span className="sidebar-footer-version">Version {appVersion}</span>
      </div>

      {helpOpen && <HelpModal onClose={() => setHelpOpen(false)} />}
    </aside>
  );
}