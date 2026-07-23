import { useLanguage } from "../i18n/LanguageContext";
import type { TranslationKey } from "../i18n/translations";

interface Props {
  imageMode: "live" | "replay";
  onImageModeChange: (m: "live" | "replay") => void;
  selectedCam?: string;
  onCamSelect?: (cam: string) => void;
}

const CAM_KEYS: { key: TranslationKey; id: string }[] = [
  { key: "cam_tv", id: "TV" },
  { key: "cam_helmet", id: "Helmet" },
  { key: "cam_front", id: "Front" },
  { key: "cam_rear", id: "Heck" },
  { key: "cam_top", id: "Top" },
  { key: "cam_behind", id: "Behind" },
];

export default function TopToolbar({
  imageMode,
  onImageModeChange,
  selectedCam,
  onCamSelect,
}: Props) {
  const { t } = useLanguage();

  return (
    <div className="top-toolbar">
      <div className="toolbar-group">
        <div className="toolbar-label">{t("toolbar_image_control")}</div>
        <div className="toolbar-buttons">
          <button
            className={imageMode === "live" ? "active" : ""}
            onClick={() => onImageModeChange("live")}
          >
            {t("toolbar_live")}
          </button>
          <button
            className={imageMode === "replay" ? "active" : ""}
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
        </div>
      </div>
    </div>
  );
}