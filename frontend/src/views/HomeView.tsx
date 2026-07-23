import { useLanguage } from "../i18n/LanguageContext";

interface Props {
  connected: boolean;
  onConnect: () => void;
}

export default function HomeView({ connected }: Props) {
  const { t } = useLanguage();

  return (
    <div className="home-view">
      <img src="/logo.png" alt="LMU Racecontrol" className="home-logo" />
      <p>
        {t("home_welcome")}
        <br />
        <br />
        {t("home_instructions")}
      </p>
      {connected && <p className="home-connected-hint">✓ {t("home_connected_hint")}</p>}
    </div>
  );
}
