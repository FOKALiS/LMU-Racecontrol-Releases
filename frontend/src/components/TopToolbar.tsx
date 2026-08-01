import { useLanguage } from "../i18n/LanguageContext";
import type { TranslationKey } from "../i18n/translations";

interface Props {
  imageMode: "live" | "replay";
  onImageModeChange: (m: "live" | "replay") => void;
  selectedCam?: string;
  onCamSelect?: (cam: string) => void;
  onZoomStart?: (direction: "in" | "out") => void;
  onZoomEnd?: () => void;
  /** Zeigt an, ob ein Replay gerade läuft (durch Auge-Klick gestartet) */
  replayActive?: boolean;
  /** Wird aufgerufen, wenn der LIVE-Button geklickt wird (nur sichtbar während Replay) */
  onSwitchToLive?: () => void;
}

const CAM_KEYS: { key: TranslationKey; id: string }[] = [
  { key: "cam_tv", id: "TV" },
  { key: "cam_bord", id: "Bord" },
  { key: "cam_rear", id: "Heck" },
];

export default function TopToolbar({
  imageMode,
  onImageModeChange,
  selectedCam,
  onCamSelect,
  onZoomStart,
  onZoomEnd,
  replayActive = false,
  onSwitchToLive,
}: Props) {
  const { t } = useLanguage();

  function handleZoomInMouseDown(e: React.MouseEvent) {
    e.preventDefault();
    onZoomStart?.("in");
  }

  function handleZoomOutMouseDown(e: React.MouseEvent) {
    e.preventDefault();
    onZoomStart?.("out");
  }

  function handleZoomMouseUp() {
    onZoomEnd?.();
  }

  return (
    <div className="top-toolbar">
      <div className="toolbar-group">
        <div className="toolbar-label">{t("toolbar_image_control")}</div>
        <div className="toolbar-buttons">
          <button
            className={!replayActive && imageMode === "live" ? "active" : ""}
            onClick={() => replayActive ? onSwitchToLive?.() : onImageModeChange("live")}
            title={replayActive ? t("toolbar_back_to_live") : ""}
          >
            {t("toolbar_live")} {replayActive ? " ↩" : ""}
          </button>
          <button
            className={replayActive || imageMode === "replay" ? "active" : ""}
            onClick={() => onImageModeChange("replay")}
          >
            {t("toolbar_replay")}
          </button>
        </div>
      </div>

      <div className="toolbar-group toolbar-group-align">
        <div className="toolbar-label">{t("toolbar_cam_control")}</div>
        <div className="toolbar-buttons">
          {CAM_KEYS.map((cam) => (
            <button
              key={cam.id}
              className={selectedCam === cam.id ? "active" : ""}
              onClick={() => onCamSelect?.(cam.id)}
            >
              {t(cam.key)}
            </button>
          ))}
          <div className="zoom-btn-group">
            <button
              className="zoom-btn"
              onMouseDown={handleZoomInMouseDown}
              onMouseUp={handleZoomMouseUp}
              onMouseLeave={handleZoomMouseUp}
            >
              {t("cam_zoom_in")}
            </button>
            <button
              className="zoom-btn"
              onMouseDown={handleZoomOutMouseDown}
              onMouseUp={handleZoomMouseUp}
              onMouseLeave={handleZoomMouseUp}
            >
              {t("cam_zoom_out")}
            </button>
          </div>
        </div>
      </div>
    </div>
  );
}