import { useState } from "react";
import type { Incident } from "../types";
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

export default function ArchivView({
  incidents,
  onReplay,
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

  function isPenalty(decision: string | null): boolean {
    return (
      !!decision &&
      !decision.toLowerCase().includes("keine") &&
      !decision.toLowerCase().includes(" no ") &&
      !decision.toLowerCase().startsWith("no ")
    );
  }

  return (
    <div className="view-archiv">
      {/* Zeile 1: 3-Spalten-Toolbar – Figma: gap-12 (48px) */}
      <div className="archiv-toolbar-row">
        {/* Spalte 1: Titel – Figma: flex-1, self-stretch, gap-9 */}
        <div className="archiv-col-first">
          <div className="archiv-title">{t("archiv_title")}</div>
        </div>

        {/* Spalte 2: Image Control – Figma: flex-1, h-20 (80px) */}
        <div className="archiv-col">
          <div className="archiv-image-control">
            <div className="archiv-label">{t("toolbar_image_control")}</div>
            <div className="archiv-buttons-row">
              <button
                className={`archiv-ctrl-btn ${imageMode === "replay" || replayActive ? "active" : ""}`}
                onClick={() => handleImageModeChange("replay")}
                disabled={replayActive}
              >
                {replayActive ? t("toolbar_replay_active") : t("toolbar_replay")}
              </button>
              <button
                className={`archiv-ctrl-btn ${imageMode === "live" ? "active" : ""}`}
                onClick={() => handleImageModeChange("live")}
              >
                {t("toolbar_live")}
              </button>
            </div>
          </div>
        </div>

        {/* Spalte 3: Cam Control – Figma: flex-1, h-20 */}
        <div className="archiv-col">
          <div className="archiv-image-control">
            <div className="archiv-label">{t("toolbar_cam_control")}</div>
            <div className="archiv-buttons-row">
              {(["TV", "Bord", "Heck"] as const).map((cam) => (
                <button
                  key={cam}
                  className={`archiv-cam-btn ${selectedCam === cam ? "active" : ""}`}
                  onClick={() => handleCamSelect(cam)}
                >
                  {cam === "TV" ? t("cam_tv") : cam === "Bord" ? t("cam_bord") : t("cam_rear")}
                </button>
              ))}
              <button
                className="archiv-cam-btn zoom-btn"
                onMouseDown={() => onZoomStart?.("in")}
                onMouseUp={() => onZoomEnd?.()}
                onMouseLeave={() => onZoomEnd?.()}
              >
                {t("cam_zoom_in")}
              </button>
              <button
                className="archiv-cam-btn zoom-btn"
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

      {/* Zeile 2: Tabelle mit integrierter Entscheidung-Spalte – wie Fahrerfeld */}
      <div className="archiv-table-row">
        <div className="table-scroll archiv-table-scroll">
          <table className="data-table archiv-table">
            <thead>
              <tr>
                <th className="archiv-th-incident">{t("col_incident")}</th>
                <th className="archiv-th-class">{t("col_class")}</th>
                <th className="archiv-th-num">{t("col_number")}</th>
                <th className="archiv-th-driver">{t("col_driver_name")}</th>
                <th className="archiv-th-class">{t("col_class")}</th>
                <th className="archiv-th-num">{t("col_number")}</th>
                <th className="archiv-th-driver">{t("col_driver_name")}</th>
                <th className="archiv-th-lap">{t("col_lap")}</th>
                <th className="archiv-th-timestamp">{t("col_timestamp")}</th>
                <th className="archiv-th-spacer"></th>
                <th className="archiv-th-decision-col">{t("col_decision")}</th>
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
                <tr
                  key={i.id}
                  className={focusedSlotId != null && i.slot_id_a === focusedSlotId ? "row-focused" : ""}
                >
                  <td className="archiv-td-incident">{i.incident_number}</td>
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
                  <td className="archiv-td-spacer"></td>
                  <td className="archiv-td-decision-col">
                    <div className="archiv-decision-inner">
                      <button
                        className="archiv-decision-view-btn"
                        onClick={() => onReplay(i)}
                        title="Replay"
                      >
                        <EyeIcon color="#ffffff" />
                      </button>
                      <span className={`archiv-decision-badge ${isPenalty(i.decision) ? "penalty" : "nfa"}`}>
                        {isPenalty(i.decision) ? t("decision_penalty") : t("decision_nfa")}
                      </span>
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