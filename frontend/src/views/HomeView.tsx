import { useLanguage } from "../i18n/LanguageContext";

interface Props {
  connected: boolean;
  onConnect: () => void;
}

export default function HomeView({ connected }: Props) {
  const { t } = useLanguage();

  return (
    <div className="home-view">
      <div className="home-content">
        {/* Logo – Figma: w-[508px] h-60 (508x240) */}
        <img className="home-logo" src="/logo.png" alt="LMU Racecontrol" />

        {/* Willkommenstext */}
        <div className="home-welcome">
          <span className="home-welcome-title">{t("home_welcome")}</span>
          <br />
          <br />
          <span className="home-welcome-instructions">{t("home_instructions")}</span>
        </div>

        {connected && <p className="home-connected-hint">✓ {t("home_connected_hint")}</p>}
      </div>
    </div>
  );
}
