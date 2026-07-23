import { useState } from "react";
import type { Incident, FlagColor } from "../types";
import type { ActionResult } from "../App";
import TopToolbar from "../components/TopToolbar";
import EyeIcon from "../components/EyeIcon";
import { useLanguage } from "../i18n/LanguageContext";
import type { TranslationKey } from "../i18n/translations";
import { classColor } from "../classColors";

interface Props {
  incidents: Incident[];
  onSaveReplaySettings: (preRoll: number, postRoll: number) => void;
  preRoll: number;
  postRoll: number;
  onNewIncident: () => void;
  onInvestigate: (incident: Incident) => void;
  onJumpToReplay: (incident: Incident) => Promise<ActionResult>;
  onGoToArchiv: () => void;
  onFcyClick: () => void;
}

const FILTER_OPTIONS: { color: FlagColor; labelKey: TranslationKey; dotClass: string }[] = [
  { color: "Red", labelKey: "filter_crashes", dotClass: "red" },
  { color: "Yellow", labelKey: "filter_yellow", dotClass: "yellow" },
  { color: "White", labelKey: "filter_white", dotClass: "white" },
  { color: "None", labelKey: "filter_overtakes", dotClass: "none" },
];

export default function VorfaelleView({
  incidents,
  preRoll,
  postRoll,
  onSaveReplaySettings,
  onNewIncident,
  onInvestigate,
  onJumpToReplay,
  onGoToArchiv,
  onFcyClick,
}: Props) {
  const { t } = useLanguage();
  const [imageMode, setImageMode] = useState<"live" | "replay">("live");
  const [selectedCam, setSelectedCam] = useState("TV");
  const [status, setStatus] = useState<{ ok: boolean; message: string } | null>(null);
  const [activeFilters, setActiveFilters] = useState<Set<FlagColor>>(
    new Set(["Red", "Yellow", "White", "None"])
  );

  function showStatus(result: { ok: boolean; message: string }) {
    setStatus(result);
    window.setTimeout(() => setStatus(null), 6000);
  }

  async function handleEyeClick(incident: Incident) {
    const result = await onJumpToReplay(incident);
    showStatus(result);
    if (result.ok) setImageMode("replay");
  }

  function toggleFilter(color: FlagColor) {
    setActiveFilters((prev) => {
      const next = new Set(prev);
      if (next.has(color)) {
        next.delete(color);
      } else {
        next.add(color);
      }
      return next;
    });
  }

  const visibleIncidents = incidents.filter((i) => activeFilters.has(i.flag_color));

  return (
    <div className="view-vorfaelle">
      <div className="view-header-row">
        <h1>{t("vorfaelle_title")}</h1>
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

      <div className="toolbar-row-secondary">
        <div className="toolbar-group">
          <div className="toolbar-label">{t("replay_control")}</div>
          <div className="replay-inputs">
            <div className="replay-input">
              <div>
                <input
                  type="number"
                  min={0}
                  value={preRoll}
                  onChange={(e) => onSaveReplaySettings(Number(e.target.value), postRoll)}
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
                  value={postRoll}
                  onChange={(e) => onSaveReplaySettings(preRoll, Number(e.target.value))}
                />
                <span>{t("seconds_short")}</span>
              </div>
              <label>{t("post_roll")}</label>
            </div>
          </div>
        </div>

        <div className="toolbar-group incident-filter">
          <div className="toolbar-label">
            {t("filter_crashes")}/{t("filter_yellow")}/{t("filter_white")}/{t("filter_overtakes")}
          </div>
          <div className="incident-filter-chips">
            {FILTER_OPTIONS.map((opt) => (
              <button
                key={opt.color}
                className={`incident-filter-chip ${activeFilters.has(opt.color) ? "active" : ""}`}
                onClick={() => toggleFilter(opt.color)}
              >
                <span className={`dot ${opt.dotClass}`} />
                {t(opt.labelKey)}
              </button>
            ))}
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
            {visibleIncidents.length === 0 && (
              <tr>
                <td colSpan={11} className="empty-row">
                  {t("vorfaelle_empty")}
                </td>
              </tr>
            )}
            {visibleIncidents.map((i) => (
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
                <td className="incident-cell">
                  <button
                    className={`flag-dot flag-${i.flag_color.toLowerCase()}`}
                    onClick={() => handleEyeClick(i)}
                    title={i.incident_type}
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