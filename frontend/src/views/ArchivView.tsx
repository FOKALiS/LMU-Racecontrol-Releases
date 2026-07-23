import { useState } from "react";
import type { Incident } from "../types";
import type { ActionResult } from "../App";
import TopToolbar from "../components/TopToolbar";
import EyeIcon from "../components/EyeIcon";
import { useLanguage } from "../i18n/LanguageContext";
import { classColor } from "../classColors";

interface Props {
  incidents: Incident[];
  onReplay: (incident: Incident) => Promise<ActionResult>;
}

export default function ArchivView({ incidents, onReplay }: Props) {
  const { t } = useLanguage();
  const [imageMode, setImageMode] = useState<"live" | "replay">("live");
  const [status, setStatus] = useState<{ ok: boolean; message: string } | null>(null);

  function isPenalty(decision: string | null): boolean {
    return (
      !!decision &&
      !decision.toLowerCase().includes("keine") &&
      !decision.toLowerCase().includes(" no ") &&
      !decision.toLowerCase().startsWith("no ")
    );
  }

  async function handleReplay(incident: Incident) {
    const result = await onReplay(incident);
    setStatus(result);
    window.setTimeout(() => setStatus(null), 6000);
  }

  return (
    <div className="view-archiv">
      <div className="view-header-row">
        <h1>{t("archiv_title")}</h1>
        <TopToolbar imageMode={imageMode} onImageModeChange={setImageMode} />
      </div>

      {status && (
        <div className={`action-status ${status.ok ? "ok" : "error"}`}>{status.message}</div>
      )}

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
              <tr key={i.id}>
                <td>{i.incident_number}</td>
                <td>{i.class_a && <span className={`class-badge ${classColor(i.class_a)}`}>{i.class_a}</span>}</td>
                <td>{i.car_number_a}</td>
                <td>{i.driver_a}</td>
                <td>{i.class_b && <span className={`class-badge ${classColor(i.class_b)}`}>{i.class_b}</span>}</td>
                <td>{i.car_number_b}</td>
                <td>{i.driver_b}</td>
                <td>{i.lap}</td>
                <td>{i.corner}</td>
                <td>{i.timestamp_label}</td>
                <td className="decision-cell">
                  <button className="view-replay-btn" onClick={() => handleReplay(i)} title="Replay">
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