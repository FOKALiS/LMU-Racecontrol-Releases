import { useState, useMemo } from "react";
import type { CarStanding, Incident, FlagColor } from "../types";
import TopToolbar from "../components/TopToolbar";
import FlagFilter from "../components/FlagFilter";
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
}: Props) {
  const { t } = useLanguage();
  const [showRed, setShowRed] = useState(true);
  const [showYellow, setShowYellow] = useState(true);
  const [showWhite, setShowWhite] = useState(true);

  // Session-Tabs (Platzhalter ohne Funktion)
  const [sessionTab, setSessionTab] = useState<"Practice" | "Qualifying" | "Race">("Practice");

  const sortedStandings = useMemo(() => {
    return [...standings].sort((a, b) => a.position - b.position);
  }, [standings]);

  function handleImageModeChange(mode: "live" | "replay") {
    onImageModeChange(mode);
    if (mode === "live") {
      invoke("switch_to_live").catch(console.error);
    } else {
      invoke("switch_to_replay").catch(console.error);
    }
  }

  function handleCamSelect(cam: string) {
    onCamSelect?.(cam);
  }

  function handleFlagFilterChange(color: FlagColor, show: boolean) {
    if (color === "Red") setShowRed(show);
    else if (color === "Yellow") setShowYellow(show);
    else if (color === "White") setShowWhite(show);
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

  return (
    <div className="view-fahrerfeld">
      <div className="view-header-row">
        <h1>{t("fahrerfeld_title")}</h1>
        <TopToolbar
          imageMode={imageMode}
          onImageModeChange={handleImageModeChange}
          selectedCam={selectedCam}
          onCamSelect={handleCamSelect}
          onZoomStart={onZoomStart}
          onZoomEnd={onZoomEnd}
          replayActive={replayActive}
          onSwitchToLive={onSwitchToLive}
        />
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

      <FlagFilter
        showRed={showRed}
        showYellow={showYellow}
        showWhite={showWhite}
        onChange={handleFlagFilterChange}
      />

      <div className="table-scroll">
        <table className="data-table">
          <thead>
            <tr>
              <th>{t("col_pos")}</th>
              <th>{t("col_class")}</th>
              <th>{t("col_number")}</th>
              <th>{t("col_driver_name")}</th>
              <th>{t("col_team")}</th>
              <th>{t("col_car")}</th>
              <th>{t("col_status")}</th>
              <th>{t("col_fastest_lap")}</th>
              <th>{t("col_incident")}</th>
            </tr>
          </thead>
          <tbody>
            {sortedStandings.length === 0 && (
              <tr>
                <td colSpan={9} className="empty-row">
                  {t("fahrerfeld_no_data")}
                </td>
              </tr>
            )}
            {sortedStandings.map((car) => {
              const pending = pendingFor(car.car_number);
              const showIncident = pending && matchesFilter(pending);
              return (
                <tr
                  key={car.slot_id}
                  className={focusedSlotId === car.slot_id ? "row-focused" : ""}
                  onClick={() => onFocusDriver(car.slot_id, car.car_number, car.driver)}
                >
                  <td>{car.position}</td>
                  <td>
                    <span className={`class-badge class-badge-${classColor(car.class)}`}>{car.class}</span>
                  </td>
                  <td>{car.car_number}</td>
                  <td>{car.driver}</td>
                  <td>{car.team}</td>
                  <td>{car.car_model || "–"}</td>
                  <td>{car.in_pits ? "PIT" : ""}</td>
                  <td>{formatLap(car.best_lap_s)}</td>
                  <td className="incident-cell">
                    {showIncident && pending && (
                      <button
                        className={`flag-dot flag-${pending.flag_color?.toLowerCase() ?? "empty"}`}
                        onClick={(e) => { e.stopPropagation(); onReplay?.(pending); }}
                        title={pending.incident_type || ""}
                      >
                        <EyeIcon />
                      </button>
                    )}
                    {showIncident && pending && (
                      <button
                        className="investigate-btn"
                        onClick={(e) => { e.stopPropagation(); onInvestigate(pending); }}
                      >
                        {t("investigate")}
                      </button>
                    )}
                  </td>
                </tr>
              );
            })}
          </tbody>
        </table>
      </div>
    </div>
  );
}

function formatLap(seconds: number): string {
  if (!seconds || seconds <= 0) return "–";
  const m = Math.floor(seconds / 60);
  const s = (seconds % 60).toFixed(3);
  return m > 0 ? `${m}:${s.padStart(6, "0")}` : s;
}