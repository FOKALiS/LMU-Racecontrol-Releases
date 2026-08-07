import { useState, useMemo } from "react";
import type { CarStanding, Incident, SessionInfo } from "../types";
import EyeIcon from "../components/EyeIcon";
import { useLanguage } from "../i18n/LanguageContext";
import { classColor } from "../classColors";
import { invoke } from "@tauri-apps/api/core";

interface Props {
  standings: CarStanding[];
  pendingIncidents: Incident[];
  onInvestigate: (incident: Incident) => void;
  onFocusDriver: (slotId: number, carNumber: string, driverName?: string) => void;
  focusedSlotId?: number | null;
  selectedCam?: string;
  onCamSelect?: (cam: string) => void;
  onReplay?: (incident: Incident) => void;
  onZoomStart?: (direction: "in" | "out") => void;
  onZoomEnd?: () => void;
  replayActive?: boolean;
  onSwitchToLive?: () => void;
  imageMode: "live" | "replay";
  onImageModeChange: (m: "live" | "replay") => void;
  session?: SessionInfo | null;
}

export default function FahrerfeldView({
  standings,
  pendingIncidents,
  onInvestigate,
  onFocusDriver,
  focusedSlotId,
  selectedCam = "TV",
  onCamSelect,
  onReplay,
  onZoomStart,
  onZoomEnd,
  replayActive = false,
  onSwitchToLive,
  imageMode,
  onImageModeChange,
  session,
}: Props) {
  const { t } = useLanguage();
  const [showRed, setShowRed] = useState(true);
  const [showYellow, setShowYellow] = useState(true);
  const [showWhite, setShowWhite] = useState(true);

  const sortedStandings = useMemo(() => {
    return [...standings].sort((a, b) => a.position - b.position);
  }, [standings]);

  function handleImageModeChange(mode: "live" | "replay") {
    onImageModeChange(mode);
    if (mode === "live") {
      onSwitchToLive?.();
    } else {
      invoke("replay_activate").catch(console.error);
      invoke("switch_to_replay").catch(console.error);
    }
  }

  function handleCamSelect(cam: string) {
    onCamSelect?.(cam);
  }

  function pendingFor(carNumber: string): Incident | undefined {
    return pendingIncidents
      .filter((i) => i.car_number_a === carNumber)
      .sort((a, b) => b.incident_number - a.incident_number)[0];
  }

  function matchesFilter(incident: Incident): boolean {
    const color = incident.flag_color?.toLowerCase() ?? "none";
    if (color === "red" && !showRed) return false;
    if (color === "yellow" && !showYellow) return false;
    if (color === "white" && !showWhite) return false;
    return true;
  }

  // Aktuelle Session aus session?.session_type ableiten (z.B. "Practice", "Qualifying", "Race")
  const currentSession = session?.session_type ?? null;

  return (
    <div className="view-fahrerfeld">
      {/* Zeile 1: Titel | Image Control | Cam Control */}
      <div className="fahrerfeld-toolbar-row">
        <div className="fahrerfeld-col">
          <div className="fahrerfeld-title">{t("fahrerfeld_title")}</div>
        </div>
        <div className="fahrerfeld-col">
          <div className="fahrerfeld-image-control">
            <div className="fahrerfeld-label">{t("toolbar_image_control")}</div>
            <div className="fahrerfeld-buttons-row">
              <button
                className={`fahrerfeld-ctrl-btn ${imageMode === "replay" || replayActive ? "active" : ""}`}
                onClick={() => handleImageModeChange("replay")}
                disabled={replayActive}
              >
                {replayActive ? t("toolbar_replay_active") : t("toolbar_replay")}
              </button>
              <button
                className={`fahrerfeld-ctrl-btn ${imageMode === "live" ? "active" : ""}`}
                onClick={() => handleImageModeChange("live")}
              >
                {t("toolbar_live")}
              </button>
            </div>
          </div>
        </div>
        <div className="fahrerfeld-col">
          <div className="fahrerfeld-image-control">
            <div className="fahrerfeld-label">{t("toolbar_cam_control")}</div>
            <div className="fahrerfeld-buttons-row">
              {(["TV", "Bord", "Heck"] as const).map((cam) => (
                <button
                  key={cam}
                  className={`fahrerfeld-cam-btn ${selectedCam === cam ? "active" : ""}`}
                  onClick={() => handleCamSelect(cam)}
                >
                  {cam === "TV" ? t("cam_tv") : cam === "Bord" ? t("cam_bord") : t("cam_rear")}
                </button>
              ))}
              <button
                className="fahrerfeld-cam-btn zoom-btn"
                onMouseDown={() => onZoomStart?.("in")}
                onMouseUp={() => onZoomEnd?.()}
                onMouseLeave={() => onZoomEnd?.()}
              >
                {t("cam_zoom_in")}
              </button>
              <button
                className="fahrerfeld-cam-btn zoom-btn"
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

      {/* Zeile 1b: Session (Info) | Player | Filter – auf gleicher Höhe */}
      <div className="fahrerfeld-toolbar-row">
        <div className="fahrerfeld-col">
          <div className="fahrerfeld-section">
            <div className="fahrerfeld-label">{t("col_session")}</div>
            <div className="session-tabs session-tabs-fahrerfeld">
              {(["Practice", "Qualifying", "Race"] as const).map((tab) => (
                <div
                  key={tab}
                  className={`session-tab ${currentSession === tab ? "active" : ""}`}
                  style={{ cursor: "default" }}
                >
                  {tab}
                </div>
              ))}
            </div>
          </div>
        </div>
        <div className="fahrerfeld-col">
          <div className="fahrerfeld-section">
            <div className="fahrerfeld-label">{t("col_player")}</div>
            <div className="player-bar player-bar-fahrerfeld">
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
        <div className="fahrerfeld-col">
          <div className="fahrerfeld-section">
            <div className="fahrerfeld-label">{t("col_filter")}</div>
            <div className="filter-tabs filter-tabs-fahrerfeld">
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

      {/* Zeile 3: Tabelle mit integrierter Vorfall-Spalte */}
      <div className="fahrerfeld-table-row">
        <div className="table-scroll fahrerfeld-table-scroll">
          <table className="data-table fahrerfeld-table">
            <thead>
              <tr>
                <th className="fahrerfeld-th-pos">{t("col_pos")}</th>
                <th className="fahrerfeld-th-class">{t("col_class")}</th>
                <th className="fahrerfeld-th-num">{t("col_number")}</th>
                <th className="fahrerfeld-th-logo">{t("col_car")}</th>
                <th className="fahrerfeld-th-driver">{t("col_driver_name")}</th>
                <th className="fahrerfeld-th-team">{t("col_team")}</th>
                <th className="fahrerfeld-th-status">{t("col_status")}</th>
                <th className="fahrerfeld-th-laps">{t("col_laps")}</th>
                <th className="fahrerfeld-th-ve">{t("col_ve")}</th>
                <th className="fahrerfeld-th-lap">{t("col_fastest_lap")}</th>
                <th className="fahrerfeld-th-spacer"></th>
                <th className="fahrerfeld-th-incident">{t("col_incident")}</th>
              </tr>
            </thead>
            <tbody>
              {sortedStandings.length === 0 && (
                <tr>
                  <td colSpan={10} className="empty-row">
                    {t("fahrerfeld_no_data")}
                  </td>
                </tr>
              )}
              {sortedStandings.map((car) => {
                const pending = pendingFor(car.car_number);
                const showIncident = pending && matchesFilter(pending);
                const focused = focusedSlotId === car.slot_id;
                return (
                  <tr
                    key={car.slot_id}
                    className={focused ? "row-focused" : ""}
                    onClick={() => onFocusDriver(car.slot_id, car.car_number, car.driver)}
                  >
                    <td>{car.position}</td>
                    <td>
                      <span className={`class-badge class-badge-${classColor(car.class)}`}>
                        {car.class}
                      </span>
                    </td>
                    <td>{car.car_number}</td>
                    <td className="fahrerfeld-td-logo">
                      {car.manufacturer ? (
                        <img
                          src={`/manufacturers/${car.manufacturer}.png`}
                          alt={car.manufacturer}
                          className="manufacturer-logo"
                          title={car.vehicle_model || car.manufacturer}
                          onError={(e) => {
                            (e.target as HTMLImageElement).style.display = "none";
                          }}
                        />
                      ) : null}
                    </td>
                    <td>{car.driver}</td>
                    <td>{car.team}</td>
                    <td>{car.in_pits ? "PIT" : ""}</td>
                    <td>{car.laps > 0 ? car.laps : "–"}</td>
                    <td className="fahrerfeld-td-ve">
                      {car.virtual_energy > 0 ? (
                        <span className={`ve-badge ve-badge-${veColor(car.virtual_energy)}`}>
                          {(car.virtual_energy * 100).toFixed(0)}%
                        </span>
                      ) : "N.A."}
                    </td>
                    <td>{formatLap(car.best_lap_s)}</td>
                    <td className="fahrerfeld-td-spacer"></td>
                    <td className="fahrerfeld-td-incident">
                      {showIncident && pending ? (
                        <div className="fahrerfeld-incident-inner">
                          <button
                            className={`fahrerfeld-flag-badge flag-${pending.flag_color?.toLowerCase() ?? "empty"}`}
                            onClick={(e) => { e.stopPropagation(); onReplay?.(pending); }}
                            title={pending.incident_type || ""}
                          >
                            <EyeIcon />
                          </button>
                          <button
                            className="fahrerfeld-investigate-btn"
                            onClick={(e) => { e.stopPropagation(); onInvestigate(pending); }}
                          >
                            {t("investigate")}
                          </button>
                        </div>
                      ) : (
                        <div className="fahrerfeld-incident-inner">
                          <div className="fahrerfeld-flag-badge-empty" />
                          <div className="fahrerfeld-investigate-empty" />
                        </div>
                      )}
                    </td>
                  </tr>
                );
              })}
            </tbody>
          </table>
        </div>
      </div>
    </div>
  );
}

function veColor(ve: number): "green" | "yellow" | "red" {
  if (ve >= 0.3) return "green";
  if (ve >= 0.1) return "yellow";
  return "red";
}

function formatLap(seconds: number): string {
  if (!seconds || seconds <= 0) return "–";
  const m = Math.floor(seconds / 60);
  const s = (seconds % 60).toFixed(3);
  return m > 0 ? `${m}:${s.padStart(6, "0")}` : s;
}