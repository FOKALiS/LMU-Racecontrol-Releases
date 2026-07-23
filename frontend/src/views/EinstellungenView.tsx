import { useState, useEffect } from "react";
import type { Settings } from "../types";
import { useLanguage } from "../i18n/LanguageContext";

interface Props {
  settings: Settings;
  onSave: (settings: Settings) => void;
}

export default function EinstellungenView({ settings, onSave }: Props) {
  const { t } = useLanguage();
  const [form, setForm] = useState<Settings>(settings);
  const [incidentTypesText, setIncidentTypesText] = useState(settings.incident_types.join("\n"));
  const [decisionTypesText, setDecisionTypesText] = useState(settings.decision_types.join("\n"));
  const [saved, setSaved] = useState(false);

  useEffect(() => {
    setForm(settings);
    setIncidentTypesText(settings.incident_types.join("\n"));
    setDecisionTypesText(settings.decision_types.join("\n"));
  }, [settings]);

  function save() {
    const updated: Settings = {
      ...form,
      incident_types: incidentTypesText.split("\n").map((s) => s.trim()).filter(Boolean),
      decision_types: decisionTypesText.split("\n").map((s) => s.trim()).filter(Boolean),
    };
    onSave(updated);
    setSaved(true);
    setTimeout(() => setSaved(false), 2000);
  }

  return (
    <div className="view-einstellungen">
      <h1>{t("settings_title")}</h1>
      <p className="settings-hint">{t("settings_hint")}</p>

      <div className="settings-grid">
        <div className="settings-block">
          <label>{t("settings_webhook_label")}</label>
          <p className="settings-block-hint">{t("settings_webhook_hint")}</p>
          <input
            value={form.discord_webhook_url}
            placeholder="https://discord.com/api/webhooks/..."
            onChange={(e) => setForm({ ...form, discord_webhook_url: e.target.value })}
          />
        </div>

        <div className="settings-block">
          <label>{t("settings_speed_limit")}</label>
          <input
            type="number"
            value={form.fcy_speed_limit_kmh}
            onChange={(e) => setForm({ ...form, fcy_speed_limit_kmh: Number(e.target.value) })}
          />
        </div>

        <div className="settings-block">
          <label>{t("settings_countdown")}</label>
          <input
            type="number"
            value={form.fcy_countdown_seconds}
            onChange={(e) => setForm({ ...form, fcy_countdown_seconds: Number(e.target.value) })}
          />
        </div>

        <div className="settings-block settings-block-wide">
          <label>{t("settings_incident_types_label")}</label>
          <p className="settings-block-hint">{t("settings_incident_types_hint")}</p>
          <textarea
            rows={8}
            value={incidentTypesText}
            onChange={(e) => setIncidentTypesText(e.target.value)}
          />
        </div>

        <div className="settings-block settings-block-wide">
          <label>{t("settings_decision_types_label")}</label>
          <p className="settings-block-hint">{t("settings_decision_types_hint")}</p>
          <textarea
            rows={8}
            value={decisionTypesText}
            onChange={(e) => setDecisionTypesText(e.target.value)}
          />
        </div>
      </div>

      <button className="btn-solid" onClick={save}>
        {saved ? `✓ ${t("settings_saved")}` : t("settings_save")}
      </button>
    </div>
  );
}
