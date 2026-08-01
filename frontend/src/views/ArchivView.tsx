import { useState } from "react";
import type { Incident } from "../types";
import TopToolbar from "../components/TopToolbar";
import EyeIcon from "../components/EyeIcon";
import { useLanguage } from "../i18n/LanguageContext";
import { classColor } from "../classColors";
import { invoke } from "@tauri-apps/api/core";

interface Props {
  incidents: Incident[];
  onReplay: (incident: Incident) => void;
  focusedSlotId?: number | null;
  selectedCam?: string;
  onCamSelect?: (cam: string) => void;
  onZoomStart?: (direction: "in" | "out") => void;
  onZoomEnd?: () => void;
  replayActive?: boolean;
  onSwitchToLive?: () => void;
  imageMode: "live" | "replay";
  onImageModeChange: (m: "live" | "replay") => void;
}

export default function ArchivView({ incidents, onReplay, focusedSlotId, selectedCam = "TV", onCamSelect, onZoomStart, onZoomEnd, replayActive = false, onSwitchToLive, imageMode, onImageModeChange }: Props) {
  const { t } = useLanguage();

  // Session-Tabs (Platzhalter ohne Funktion)
  const [sessionTab, setSessionTab] = useState<"Practice" | "Qualifying" | "Race">("Practice");
  // Filter (Platzhalter ohne Funktion)
  const [showRed, setShowRed] = useState(true);
  const [showYellow, setShowYellow] = useState(true);
  const [showWhite, setShowWhite] = useState(true);

  function handleImageModeChange(mode: "live" | "replay") {
    onImageModeChange(mode);
    if (mode === "live") {
      invoke("switch_to_live").catch(console.error);
    } else {
      invoke("switch_to_replay").catch(console.error);
    }
  }

  function isPenalty(decision: string | null): boolean {
    return !!decision && !decision.toLowerCase().includes("keine") && !decision.toLowerCase().includes(" no ") && !decision.toLowerCase().startsWith("no ");
  }

  return (
    <div className="view-archiv">
      <div className="view-header-row">
        <h1>{t("archiv_title")}</h1>
        <TopToolbar imageMode={imageMode} onImageModeChange={handleImageModeChange} selectedCam={selectedCam} onCamSelect={onCamSelect} onZoomStart={onZoomStart} onZoomEnd={onZoomEnd} replayActive={replayActive} onSwitchToLive={onSwitchToLive} />
      </div>

      {/* Zweite Zeile: Session, Player, Filter */}
      <div className="toolbar-row-secondary">
        {/* Session-Tabs */}
        <div className="toolbar-group">
          <div className="toolbar-label">{t("col_session")}</div>
          <div className="session-tabs">
            {(["Practice", "Qualifying", "Race"] as const).map((tab) => (
              <button
                key={tab}
                className={`session-tab ${sessionTab === tab ? "active" : ""}`}
                onClick={() => setSessionTab(tab)}
              >
                {tab}
              </button>
            ))}
          </div>
        </div>

        {/* Player (Platzhalter ohne Funktion) */}
        <div className="toolbar-group">
          <div className="toolbar-label">{t("col_player")}</div>
          <div className="toolbar-buttons player-bar">
            <button disabled title={t("col_player_placeholder")}>
              <img src="/icons/Slow Rewind.png" alt="Slow Rewind" className="player-icon" />
            </button>
            <button disabled title={t("col_player_placeholder")}>
              <img src="/icons/Rewind.png" alt="Rewind" className="player-icon" />
            </button>
            <button disabled title={t("col_player_placeholder")}>
              <img src="/icons/Play.png" alt="Play" className="player-icon" />
            </button>
            <button disabled title={t("col_player_placeholder")}>
              <img src="/icons/Forward.png" alt="Forward" className="player-icon" />
            </button>
            <button disabled title={t("col_player_placeholder")}>
              <img src="/icons/Slow Forward.png" alt="Slow Forward" className="player-icon" />
            </button>
          </div>
        </div>

        {/* Filter */}
        <div className="toolbar-group">
          <div className="toolbar-label">{t("col_filter")}</div>
          <div className="filter-tabs">
            <button
              className={`filter-tab filter-tab-red ${showRed ? "active" : ""}`}
              onClick={() => setShowRed(!showRed)}
            >
              {t("col_filter_crash")}
            </button>
            <button
              className={`filter-tab filter-tab-yellow ${showYellow ? "active" : ""}`}
              onClick={() => setShowYellow(!showYellow)}
            >
              {t("col_filter_yellow")}
            </button>
            <button
              className={`filter-tab filter-tab-white ${showWhite ? "active" : ""}`}
              onClick={() => setShowWhite(!showWhite)}
            >
              {t("col_filter_white")}
            </button>
          </div>
        </div>
      </div>

      <div className="table-scroll">
        <table className="data-table incident-table">
          <thead>
            <tr>
              <th>{t("col_incident")}</th>
              <th>{t("col_class")}</th>
              <th>{t("col_number")}</th>
              <th>{t("col_driver_name")}</th>
              <th>{t("col_class")}</th>
              <th>{t("col_number")}</th>
              <th>{t("col_driver_name")}</th>
              <th>{t("col_lap")}</th>
              <th>{t("col_corner")}</th>
              <th>{t("col_timestamp")}</th>
              <th>{t("col_decision")}</th>
            </tr>
          </thead>
          <tbody>
            {incidents.length === 0 && (
              <tr>
                <td colSpan={11} className="empty-row">
                  {t("archiv_empty")}
                </td>
              </tr>
            )}
            {incidents.map((i) => (
              <tr key={i.id} className={focusedSlotId != null && i.slot_id_a === focusedSlotId ? "row-focused" : ""}>
                <td>{i.incident_number}</td>
                <td>{i.class_a && <span className={`class-badge class-badge-${classColor(i.class_a)}`}>{i.class_a}</span>}</td>
                <td>{i.car_number_a}</td>
                <td>{i.driver_a}</td>
                <td>{i.class_b && <span className={`class-badge class-badge-${classColor(i.class_b)}`}>{i.class_b}</span>}</td>
                <td>{i.car_number_b}</td>
                <td>{i.driver_b}</td>
                <td>{i.lap}</td>
                <td>{i.corner}</td>
                <td>{i.timestamp_label}</td>
                <td className="decision-cell">
                  <button className="view-replay-btn" onClick={() => onReplay(i)} title="Replay">
                    <EyeIcon color="#ffffff" />
                  </button>
                  <span className={`decision-badge ${isPenalty(i.decision) ? "penalty" : "nfa"}`}>
                    {isPenalty(i.decision) ? t("decision_penalty") : t("decision_nfa")}
                  </span>
                </td>
              </tr>
            ))}
          </tbody>
        </table>
      </div>
    </div>
  );
}