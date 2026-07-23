import { useState } from "react";
import type { CarStanding, Incident } from "../types";
import TopToolbar from "../components/TopToolbar";
import EyeIcon from "../components/EyeIcon";
import { useLanguage } from "../i18n/LanguageContext";
import { classColor } from "../classColors";

interface Props {
  standings: CarStanding[];
  pendingIncidents: Incident[];
  onInvestigate: (incident: Incident) => void;
  onFocusDriver: (carNumber: string) => void;
  onCamSelect?: (cam: string) => void;
  onReplay?: (incident: Incident) => void;
}

export default function FahrerfeldView({
  standings,
  pendingIncidents,
  onInvestigate,
  onFocusDriver,
  onCamSelect,
  onReplay,
}: Props) {
  const { t } = useLanguage();
  const [imageMode, setImageMode] = useState<"live" | "replay">("live");

  function pendingFor(carNumber: string): Incident | undefined {
    return pendingIncidents
      .filter((i) => i.car_number_a === carNumber)
      .sort((a, b) => b.incident_number - a.incident_number)[0];
  }

  return (
    <div className="view-fahrerfeld">
      <div className="view-header-row">
        <h1>{t("fahrerfeld_title")}</h1>
        <TopToolbar imageMode={imageMode} onImageModeChange={setImageMode} onCamSelect={onCamSelect} />
      </div>

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
            {standings.length === 0 && (
              <tr>
                <td colSpan={9} className="empty-row">
                  {t("fahrerfeld_no_data")}
                </td>
              </tr>
            )}
            {standings.map((car) => {
              const pending = pendingFor(car.car_number);
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
                    {pending && (
                      <button
                        className={`flag-dot flag-${pending?.flag_color?.toLowerCase() ?? "empty"}`}
                        onClick={(e) => { e.stopPropagation(); onReplay?.(pending); }}
                        title={pending?.incident_type || ""}
                      >
                        <EyeIcon />
                      </button>
                    )}
                    {pending && (
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