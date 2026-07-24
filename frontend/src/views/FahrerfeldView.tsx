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
  onFocusDriver: (carNumber: string) => void;
  selectedCam?: string;
  onCamSelect?: (cam: string) => void;
  onReplay?: (incident: Incident) => void;
}

export default function FahrerfeldView({
  standings,
  pendingIncidents,
  onInvestigate,
  onFocusDriver,
  selectedCam = "TV",
  onCamSelect,
  onReplay,
}: Props) {
  const { t } = useLanguage();
  const [imageMode, setImageMode] = useState<"live" | "replay">("live");
  const [showRed, setShowRed] = useState(true);
  const [showYellow, setShowYellow] = useState(true);
  const [showWhite, setShowWhite] = useState(true);

  // Sortierung nach Position (1., 2., 3., ...)
  const sortedStandings = useMemo(() => {
    return [...standings].sort((a, b) => a.position - b.position);
  }, [standings]);

  function handleImageModeChange(mode: "live" | "replay") {
    setImageMode(mode);
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
        />
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
              <th>{t("col_speed")}</th>
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
                <tr key={car.slot_id} onClick={() => onFocusDriver(car.car_number)}>
                  <td>{car.position}</td>
                  <td>
                    <span className={`class-badge class-badge-${classColor(car.class)}`}>{car.class}</span>
                  </td>
                  <td>{car.car_number}</td>
                  <td>{car.driver}</td>
                  <td>{car.team}</td>
                  <td>{car.car_model || "–"}</td>
                  <td>{car.speed_kmh ? car.speed_kmh.toFixed(0) : "–"}</td>
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