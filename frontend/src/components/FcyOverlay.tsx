import type { FcyPhase } from "../types";
import { useLanguage } from "../i18n/LanguageContext";

interface Props {
  phase: FcyPhase;
  remaining: number;
  speedLimit: number;
}

export default function FcyOverlay({ phase, remaining, speedLimit }: Props) {
  const { t } = useLanguage();
  if (phase === "idle") return null;

  return (
    <div className={`fcy-overlay fcy-overlay-${phase}`}>
      {phase === "countdown" && (
        <>
          <div className="fcy-overlay-label">{t("fcy_overlay_countdown")}</div>
          <div className="fcy-overlay-count">{remaining}</div>
        </>
      )}
      {phase === "active" && (
        <>
          <div className="fcy-overlay-label">{t("fcy_overlay_active")}</div>
          <div className="fcy-overlay-sub">
            {t("fcy_overlay_sub", { limit: speedLimit.toFixed(0) })}
          </div>
        </>
      )}
    </div>
  );
}
