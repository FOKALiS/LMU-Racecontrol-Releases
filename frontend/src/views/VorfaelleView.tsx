import { useState } from "react";
import type { Incident, Settings } from "../types";
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
      invoke("replay_activate").catch(console.error);
      invoke("switch_to_replay").catch(console.error);
    }
  }

  function handleCamSelect(cam: string) {
    onCamSelect?.(cam);
  }

  function handlePreRollChange(value: number) {
    onSaveSettings({ ...settings, pre_roll_seconds: value });
  }

  function handlePostRollChange(value: number) {
    onSaveSettings({ ...settings, post_roll_seconds: value });
  }

  function matchesFilter(incident: Incident): boolean {
    const color = incident.flag_color?.toLowerCase() ?? "none";
    if (color === "red" && !showRed) return false;
    if (color === "yellow" && !showYellow) return false;
    if (color === "white" && !showWhite) return false;
    return true;
  }

  const filteredIncidents = incidents.filter(matchesFilter);

  return (
    <div className="view-vorfaelle">
      {/* Zeile 1: Titel | Image Control | Cam Control – wie Fahrerfeld */}
      <div className="vorfaelle-toolbar-row">
        <div className="vorfaelle-col">
          <div className="vorfaelle-title">{t("vorfaelle_title")}</div>
        </div>
        <div className="vorfaelle-col">
          <div className="vorfaelle-image-control">
            <div className="vorfaelle-label">{t("toolbar_image_control")}</div>
            <div className="vorfaelle-buttons-row">
              <button
                className={`vorfaelle-ctrl-btn ${imageMode === "replay" || replayActive ? "active" : ""}`}
                onClick={() => handleImageModeChange("replay")}
                disabled={replayActive}
              >
                {replayActive ? t("toolbar_replay_active") : t("toolbar_replay")}
              </button>
              <button
                className={`vorfaelle-ctrl-btn ${imageMode === "live" ? "active" : ""}`}
                onClick={() => handleImageModeChange("live")}
              >
                {t("toolbar_live")}
              </button>
            </div>
          </div>
        </div>
        <div className="vorfaelle-col">
          <div className="vorfaelle-image-control">
            <div className="vorfaelle-label">{t("toolbar_cam_control")}</div>
            <div className="vorfaelle-buttons-row">
              {(["TV", "Bord", "Heck"] as const).map((cam) => (
                <button
                  key={cam}
                  className={`vorfaelle-cam-btn ${selectedCam === cam ? "active" : ""}`}
                  onClick={() => handleCamSelect(cam)}
                >
                  {cam === "TV" ? t("cam_tv") : cam === "Bord" ? t("cam_bord") : t("cam_rear")}
                </button>
              ))}
              <button
                className="vorfaelle-cam-btn zoom-btn"
                onMouseDown={() => onZoomStart?.("in")}
                onMouseUp={() => onZoomEnd?.()}
                onMouseLeave={() => onZoomEnd?.()}
              >
                {t("cam_zoom_in")}
              </button>
              <button
                className="vorfaelle-cam-btn zoom-btn"
                onMouseDown={() => onZoomStart?.("out")}
                onMouseUp={() => onZoomEnd?.()}
                onMouseLeave={() => onZoomEnd?.()}
              >
                {t("cam_zoom_out")}
              </button>
            </div>
          </div>
        </div>
      </div>

      {/* Zeile 1b: Session | Player | Filter – wie Fahrerfeld */}
      <div className="vorfaelle-toolbar-row">
        <div className="vorfaelle-col">
          <div className="vorfaelle-section">
            <div className="vorfaelle-label">{t("col_session")}</div>
            <div className="session-tabs session-tabs-vorfaelle">
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
        </div>
        <div className="vorfaelle-col">
          <div className="vorfaelle-section">
            <div className="vorfaelle-label">{t("col_player")}</div>
            <div className="player-bar player-bar-vorfaelle">
              <button title={t("col_player_replay_reverse")}
                onMouseDown={() => invoke("replay_reverse")}
                onMouseUp={() => invoke("hold_stop")}
                onMouseLeave={() => invoke("hold_stop")}>
                <img src="/icons/Slow Rewind.png" alt="Slow Rewind" className="player-icon" />
              </button>
              <button title={t("col_player_rewind_fast")}
                onMouseDown={() => invoke("rewind_fast")}
                onMouseUp={() => invoke("hold_stop")}
                onMouseLeave={() => invoke("hold_stop")}>
                <img src="/icons/Rewind.png" alt="Rewind" className="player-icon" />
              </button>
              <button title={t("col_player_play")} onClick={() => invoke("replay_pause")}>
                <img src="/icons/Play.png" alt="Play" className="player-icon" />
              </button>
              <button title={t("col_player_forward")}
                onMouseDown={() => invoke("replay_forward")}
                onMouseUp={() => invoke("hold_stop")}
                onMouseLeave={() => invoke("hold_stop")}>
                <img src="/icons/Forward.png" alt="Forward" className="player-icon" />
              </button>
              <button title={t("col_player_slow")}
                onMouseDown={() => invoke("replay_slow")}
                onMouseUp={() => invoke("hold_stop")}
                onMouseLeave={() => invoke("hold_stop")}>
                <img src="/icons/Slow Forward.png" alt="Slow Forward" className="player-icon" />
              </button>
            </div>
          </div>
        </div>
        <div className="vorfaelle-col">
          <div className="vorfaelle-section">
            <div className="vorfaelle-label">{t("col_filter")}</div>
            <div className="filter-tabs filter-tabs-vorfaelle">
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
      </div>

      {/* Zeile 2: Replay Control + Race Control – Figma: gap-12 */}
      <div className="vorfaelle-control-row">
        <div className="vorfaelle-replay-control">
          <div className="vorfaelle-label">{t("replay_control")}</div>
          <div className="vorfaelle-replay-inputs">
            <div className="vorfaelle-replay-pair" style={{ flex: 1 }}>
              <span className="vorfaelle-replay-pair-label">{t("pre_roll")}</span>
              <div className="vorfaelle-replay-input">
                <div className="vorfaelle-replay-input-number">
                  <input
                    type="number"
                    min={0}
                    value={settings.pre_roll_seconds}
                    onChange={(e) => handlePreRollChange(Number(e.target.value))}
                  />
                </div>
                <span className="vorfaelle-replay-input-label">{t("seconds_short")}</span>
                <div className="vorfaelle-replay-input-arrows">
                  <button
                    className="vorfaelle-replay-arrow-btn"
                    onClick={() => handlePreRollChange(settings.pre_roll_seconds + 1)}
                  >
                    <img src="/icons/Pfeil oben.png" alt="+" className="vorfaelle-replay-arrow-icon" />
                  </button>
                  <button
                    className="vorfaelle-replay-arrow-btn"
                    onClick={() => handlePreRollChange(Math.max(0, settings.pre_roll_seconds - 1))}
                  >
                    <img src="/icons/Pfeil unten.png" alt="-" className="vorfaelle-replay-arrow-icon" />
                  </button>
                </div>
              </div>
            </div>
            <div className="vorfaelle-replay-pair" style={{ flex: 1 }}>
              <span className="vorfaelle-replay-pair-label">{t("post_roll")}</span>
              <div className="vorfaelle-replay-input">
                <div className="vorfaelle-replay-input-number">
                  <input
                    type="number"
                    min={0}
                    value={settings.post_roll_seconds}
                    onChange={(e) => handlePostRollChange(Number(e.target.value))}
                  />
                </div>
                <span className="vorfaelle-replay-input-label">{t("seconds_short")}</span>
                <div className="vorfaelle-replay-input-arrows">
                  <button
                    className="vorfaelle-replay-arrow-btn"
                    onClick={() => handlePostRollChange(settings.post_roll_seconds + 1)}
                  >
                    <img src="/icons/Pfeil oben.png" alt="+" className="vorfaelle-replay-arrow-icon" />
                  </button>
                  <button
                    className="vorfaelle-replay-arrow-btn"
                    onClick={() => handlePostRollChange(Math.max(0, settings.post_roll_seconds - 1))}
                  >
                    <img src="/icons/Pfeil unten.png" alt="-" className="vorfaelle-replay-arrow-icon" />
                  </button>
                </div>
              </div>
            </div>
          </div>
        </div>
        <div className="vorfaelle-race-control">
          <div className="vorfaelle-label">{t("sidebar_race_control")}</div>
          <div className="vorfaelle-race-buttons">
            <button className="vorfaelle-race-btn" onClick={onNewIncident}>
              {t("new_incident")}
            </button>
            <button className="vorfaelle-race-btn" onClick={onGoToArchiv}>
              {t("finished_incidents")}
            </button>
            <button className="vorfaelle-race-btn vorfaelle-fcy-btn" onClick={onFcyClick}>
              {t("full_course_yellow")}
            </button>
          </div>
        </div>
      </div>

      {/* Zeile 3: Tabelle mit integrierter Vorfall-Spalte – wie Fahrerfeld */}
      <div className="vorfaelle-table-row">
        <div className="table-scroll vorfaelle-table-scroll">
          <table className="data-table vorfaelle-table">
            <thead>
              <tr>
                <th className="vorfaelle-th-incident">{t("col_incident")}</th>
                <th className="vorfaelle-th-class">{t("col_class")}</th>
                <th className="vorfaelle-th-num">{t("col_number")}</th>
                <th className="vorfaelle-th-driver">{t("col_driver_name")}</th>
                <th className="vorfaelle-th-class">{t("col_class")}</th>
                <th className="vorfaelle-th-num">{t("col_number")}</th>
                <th className="vorfaelle-th-driver">{t("col_driver_name")}</th>
                <th className="vorfaelle-th-lap">{t("col_lap")}</th>
                <th className="vorfaelle-th-timestamp">{t("col_timestamp")}</th>
                <th className="vorfaelle-th-spacer"></th>
                <th className="vorfaelle-th-incident-col">{t("col_incident")}</th>
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
                <tr
                  key={i.id}
                  className={focusedSlotId != null && i.slot_id_a === focusedSlotId ? "row-focused" : ""}
                >
                  <td className="vorfaelle-td-incident">{i.incident_number}</td>
                  <td>
                    {i.class_a && (
                      <span className={`class-badge class-badge-${classColor(i.class_a)}`}>
                        {i.class_a}
                      </span>
                    )}
                  </td>
                  <td>{i.car_number_a}</td>
                  <td>{i.driver_a}</td>
                  <td>
                    {i.class_b && (
                      <span className={`class-badge class-badge-${classColor(i.class_b)}`}>
                        {i.class_b}
                      </span>
                    )}
                  </td>
                  <td>{i.car_number_b}</td>
                  <td>{i.driver_b}</td>
                  <td>{i.lap}</td>
                  <td>{i.timestamp_label}</td>
                  <td className="vorfaelle-td-spacer"></td>
                  <td className="vorfaelle-td-incident-col">
                    <div className="vorfaelle-incident-inner">
                      <button
                        className={`vorfaelle-flag-badge flag-${i.flag_color.toLowerCase()}`}
                        onClick={() => onReplay(i)}
                      >
                        <EyeIcon />
                      </button>
                      <button
                        className="vorfaelle-investigate-btn"
                        onClick={() => onInvestigate(i)}
                      >
                        {t("investigate")}
                      </button>
                    </div>
                  </td>
                </tr>
              ))}
            </tbody>
          </table>
        </div>
      </div>
    </div>
  );
}