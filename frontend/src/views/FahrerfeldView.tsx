import { useState } from "react";
import type { CarStanding, Incident } from "../types";
import type { ActionResult } from "../App";
import TopToolbar from "../components/TopToolbar";
import EyeIcon from "../components/EyeIcon";
import { useLanguage } from "../i18n/LanguageContext";
import { classColor } from "../classColors";

interface Props {
  standings: CarStanding[];
  pendingIncidents: Incident[];
  onInvestigate: (incident: Incident) => void;
  onFocusDriver: (slotId: number, camType: string) => Promise<ActionResult>;
  onJumpToReplay: (incident: Incident) => Promise<ActionResult>;
}

export default function FahrerfeldView({
  standings,
  pendingIncidents,
  onInvestigate,
  onFocusDriver,
  onJumpToReplay,
}: Props) {
  const { t } = useLanguage();
  const [imageMode, setImageMode] = useState<"live" | "replay">("live");
  const [selectedCam, setSelectedCam] = useState("TV");
  const [status, setStatus] = useState<{ ok: boolean; message: string } | null>(null);

  function showStatus(result: { ok: boolean; message: string }) {
    setStatus(result);
    window.setTimeout(() => setStatus(null), 6000);
  }

  function pendingFor(carNumber: string): Incident | undefined {
    return pendingIncidents
      .filter((i) => i.car_number_a === carNumber)
      .sort((a, b) => b.incident_number - a.incident_number)[0];
  }

  async function handleRowClick(car: CarStanding) {
    const result = await onFocusDriver(car.slot_id, selectedCam);
    showStatus(result);
  }

  async function handleEyeClick(incident: Incident) {
    const result = await onJumpToReplay(incident);
    showStatus(result);
    if (result.ok) setImageMode("replay");
  }

  return (
    <div className="view-fahrerfeld">
      <div className="view-header-row">
        <h1>{t("fahrerfeld_title")}</h1>
        <TopToolbar
          imageMode={imageMode}
          onImageModeChange={setImageMode}
          selectedCam={selectedCam}
          onCamSelect={setSelectedCam}
        />
      </div>

      {status && (
        <div className={`action-status ${status.ok ? "ok" : "error"}`}>{status.message}</div>
      )}

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
                <tr
                  key={car.slot_id}
                  className="clickable-row"
                  title={`${t("nav_fahrerfeld")}: ${car.driver} (${selectedCam})`}
                  onClick={() => handleRowClick(car)}
                >
                  <td>{car.position}</td>
                  <td>
                    <span className={`class-badge ${classColor(car.class)}`}>{car.class}</span>
                  </td>
                  <td>{car.car_number}</td>
                  <td>{car.driver}</td>
                  <td>{car.team}</td>
                  <td>{car.car_model || "–"}</td>
                  <td>{car.speed_kmh ? car.speed_kmh.toFixed(0) : "–"}</td>
                  <td>{formatLap(car.best_lap_s)}</td>
                  <td className="incident-cell" onClick={(e) => e.stopPropagation()}>
                    <button
                      className={`flag-dot flag-${pending?.flag_color?.toLowerCase() ?? "empty"}`}
                      disabled={!pending}
                      title={pending?.incident_type || ""}
                      onClick={() => pending && handleEyeClick(pending)}
                    >
                      {pending && <EyeIcon />}
                    </button>
                    {pending && (
                      <button className="investigate-btn" onClick={() => onInvestigate(pending)}>
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