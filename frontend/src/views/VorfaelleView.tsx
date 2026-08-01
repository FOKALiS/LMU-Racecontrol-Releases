import { useState } from "react";
import type { Incident, FlagColor, Settings } from "../types";
import TopToolbar from "../components/TopToolbar";
import FlagFilter from "../components/FlagFilter";
import EyeIcon from "../components/EyeIcon";
import { useLanguage } from "../i18n/LanguageContext";
import { classColor } from "../classColors";
import { invoke } from "@tauri-apps/api/core";

interface Props {
  incidents: Incident[];
  settings: Settings;
  onSaveSettings: (settings: Settings) => void;
  onNewIncident: () => void;
  onInvestigate: (incident: Incident) => void;
  onReplay: (incident: Incident) => void;
  onGoToArchiv: () => void;
  onFcyClick: () => void;
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

export default function VorfaelleView({
  incidents,
  settings,
  onSaveSettings,
  onNewIncident,
  onInvestigate,
  onReplay,
  onGoToArchiv,
  onFcyClick,
  focusedSlotId,
  selectedCam = "TV",
  onCamSelect,
  onZoomStart,
  onZoomEnd,
  replayActive = false,
  onSwitchToLive,
  imageMode,
  onImageModeChange,
}: Props) {
  const { t } = useLanguage();
  const [showRed, setShowRed] = useState(true);
  const [showYellow, setShowYellow] = useState(true);
  const [showWhite, setShowWhite] = useState(true);

  // Session-Tabs (Platzhalter ohne Funktion)
  const [sessionTab, setSessionTab] = useState<"Practice" | "Qualifying" | "Race">("Practice");

  function handleImageModeChange(mode: "live" | "replay") {
    onImageModeChange(mode);
    if (mode === "live") {
      invoke("switch_to_live").catch(console.error);
    } else {
      invoke("switch_to_replay").catch(console.error);
    }
  }

  function handleFlagFilterChange(color: FlagColor, show: boolean) {
    if (color === "Red") setShowRed(show);
    else if (color === "Yellow") setShowYellow(show);
    else if (color === "White") setShowWhite(show);
  }

  function matchesFilter(incident: Incident): boolean {
    const color = incident.flag_color?.toLowerCase() ?? "none";
    if (color === "red" && !showRed) return false;
    if (color === "yellow" && !showYellow) return false;
    if (color === "white" && !showWhite) return false;
    return true;
  }

  function handlePreRollChange(value: number) {
    onSaveSettings({ ...settings, pre_roll_seconds: value });
  }

  function handlePostRollChange(value: number) {
    onSaveSettings({ ...settings, post_roll_seconds: value });
  }

  const filteredIncidents = incidents.filter(matchesFilter);

  return (
    <div className="view-vorfaelle">
      <div className="view-header-row">
        <h1>{t("vorfaelle_title")}</h1>
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

      <div className="toolbar-row-secondary">
        <div className="toolbar-group">
          <div className="toolbar-label">{t("replay_control")}</div>
          <div className="replay-inputs">
            <div className="replay-input">
              <div>
                <input
                  type="number"
                  min={0}
                  value={settings.pre_roll_seconds}
                  onChange={(e) => handlePreRollChange(Number(e.target.value))}
                />
                <span>{t("seconds_short")}</span>
              </div>
              <label>{t("pre_roll")}</label>
            </div>
            <div className="replay-input">
              <div>
                <input
                  type="number"
                  min={0}
                  value={settings.post_roll_seconds}
                  onChange={(e) => handlePostRollChange(Number(e.target.value))}
                />
                <span>{t("seconds_short")}</span>
              </div>
              <label>{t("post_roll")}</label>
            </div>
          </div>
        </div>

        <div className="toolbar-group toolbar-group-race-control">
          <div className="toolbar-label">{t("sidebar_race_control")}</div>
          <div className="toolbar-buttons">
            <button onClick={onNewIncident}>{t("new_incident")}</button>
            <button onClick={onGoToArchiv}>{t("finished_incidents")}</button>
            <button className="fcy-inline-btn" onClick={onFcyClick}>
              {t("full_course_yellow")}
            </button>
          </div>
        </div>
      </div>

      <FlagFilter
        showRed={showRed}
        showYellow={showYellow}
        showWhite={showWhite}
        onChange={handleFlagFilterChange}
      />

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
              <th>{t("col_incident")}</th>
            </tr>
          </thead>
          <tbody>
            {filteredIncidents.length === 0 && (
              <tr>
                <td colSpan={11} className="empty-row">
                  {t("vorfaelle_empty")}
                </td>
              </tr>
            )}
            {filteredIncidents.map((i) => (
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
                <td className="incident-cell">
                  <button
                    className={`flag-dot flag-${i.flag_color.toLowerCase()}`}
                    onClick={() => onReplay(i)}
                  >
                    <EyeIcon />
                  </button>
                  <button className="investigate-btn" onClick={() => onInvestigate(i)}>
                    {t("investigate")}
                  </button>
                </td>
              </tr>
            ))}
          </tbody>
        </table>
      </div>
    </div>
  );
}