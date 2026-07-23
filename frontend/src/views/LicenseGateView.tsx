import { useState } from "react";
import { useLanguage } from "../i18n/LanguageContext";

interface Props {
  error: string | null;
  onActivate: (key: string) => Promise<void>;
}

export default function LicenseGateView({ error, onActivate }: Props) {
  const { t } = useLanguage();
  const [key, setKey] = useState("");
  const [busy, setBusy] = useState(false);

  async function submit() {
    if (!key.trim() || busy) return;
    setBusy(true);
    try {
      await onActivate(key.trim());
    } finally {
      setBusy(false);
    }
  }

  return (
    <div className="home-view">
      <img src="/logo.png" alt="LMU Racecontrol" className="home-logo" />
      <h2 className="license-title">{t("license_title")}</h2>
      <p className="license-instructions">{t("license_instructions")}</p>

      <div className="license-form">
        <input
          value={key}
          onChange={(e) => setKey(e.target.value)}
          placeholder={t("license_key_placeholder")}
          onKeyDown={(e) => e.key === "Enter" && submit()}
          disabled={busy}
        />
        <button className="btn-solid" onClick={submit} disabled={busy || !key.trim()}>
          {busy ? t("license_activating") : t("license_activate_button")}
        </button>
      </div>

      {error && <p className="license-error">{error}</p>}

      <p className="license-hint">{t("license_no_key_hint")}</p>
    </div>
  );
}
