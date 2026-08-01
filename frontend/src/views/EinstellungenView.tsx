import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import type { Settings } from "../types";
import { useLanguage } from "../i18n/LanguageContext";
import ConfirmModal from "../components/ConfirmModal";

interface Props {
  settings: Settings;
  onSave: (settings: Settings) => void;
  onClearAll: () => void;
}

export default function EinstellungenView({ settings, onSave, onClearAll }: Props) {
  const { t } = useLanguage();
  const [cleared, setCleared] = useState(false);
  const [clearing, setClearing] = useState(false);
  const [showConfirm, setShowConfirm] = useState(false);

  async function clearDatabase() {
    setShowConfirm(false);
    if (clearing) return;
    setClearing(true);
    try {
      await invoke("clear_all_incidents");
      setCleared(true);
      setTimeout(() => setCleared(false), 2000);
      onClearAll();
    } catch (err) {
      window.alert("Fehler beim Leeren der Datenbank: " + String(err));
    } finally {
      setClearing(false);
    }
  }

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
      <div className="view-header-row">
        <h1>{t("settings_title")}</h1>
      </div>
      <p className="settings-hint">{t("settings_hint")}</p>

      <div className="settings-grid">
        {/* Linke Spalte */}
        <div className="settings-col">
          {/* Discord Webhook */}
          <div className="settings-block">
            <label>{t("settings_webhook_label")}</label>
            <p className="settings-block-hint">{t("settings_webhook_hint")}</p>
            <input
              className="settings-input"
              value={form.discord_webhook_url}
              placeholder="https://discord.com/api/webhooks/..."
              onChange={(e) => setForm({ ...form, discord_webhook_url: e.target.value })}
            />
          </div>

          {/* FCY-Countdown */}
          <div className="settings-block">
            <label>{t("settings_countdown")}</label>
            <div className="settings-input-row">
              <input
                className="settings-input settings-input-number"
                type="number"
                value={form.fcy_countdown_seconds}
                onChange={(e) => setForm({ ...form, fcy_countdown_seconds: Number(e.target.value) })}
              />
              <span className="settings-stepper">
                <button onClick={() => setForm({ ...form, fcy_countdown_seconds: form.fcy_countdown_seconds + 1 })}>▲</button>
                <button onClick={() => setForm({ ...form, fcy_countdown_seconds: Math.max(0, form.fcy_countdown_seconds - 1) })}>▼</button>
              </span>
            </div>
          </div>

          {/* Vorlaufzeit */}
          <div className="settings-block">
            <label>{t("settings_pre_roll_label")}</label>
            <p className="settings-block-hint">{t("settings_pre_roll_hint")}</p>
            <div className="settings-input-row">
              <input
                className="settings-input settings-input-number"
                type="number"
                min={0}
                max={120}
                step={1}
                value={form.pre_roll_seconds}
                onChange={(e) => setForm({ ...form, pre_roll_seconds: Number(e.target.value) })}
              />
              <span className="settings-stepper">
                <button onClick={() => setForm({ ...form, pre_roll_seconds: form.pre_roll_seconds + 1 })}>▲</button>
                <button onClick={() => setForm({ ...form, pre_roll_seconds: Math.max(0, form.pre_roll_seconds - 1) })}>▼</button>
              </span>
            </div>
          </div>

          {/* Vorfall-Kategorien */}
          <div className="settings-block">
            <label>{t("settings_incident_types_label")}</label>
            <p className="settings-block-hint">{t("settings_incident_types_hint")}</p>
            <textarea
              className="settings-textarea"
              rows={10}
              value={incidentTypesText}
              onChange={(e) => setIncidentTypesText(e.target.value)}
            />
          </div>
        </div>

        {/* Rechte Spalte */}
        <div className="settings-col">
          {/* FCY-Speed-Limit */}
          <div className="settings-block">
            <label>{t("settings_speed_limit")}</label>
            <div className="settings-input-row">
              <input
                className="settings-input settings-input-number"
                type="number"
                value={form.fcy_speed_limit_kmh}
                onChange={(e) => setForm({ ...form, fcy_speed_limit_kmh: Number(e.target.value) })}
              />
              <span className="settings-stepper">
                <button onClick={() => setForm({ ...form, fcy_speed_limit_kmh: form.fcy_speed_limit_kmh + 1 })}>▲</button>
                <button onClick={() => setForm({ ...form, fcy_speed_limit_kmh: Math.max(0, form.fcy_speed_limit_kmh - 1) })}>▼</button>
              </span>
            </div>
          </div>

          {/* Nachlaufzeit */}
          <div className="settings-block">
            <label>{t("settings_post_roll_label")}</label>
            <p className="settings-block-hint">{t("settings_post_roll_hint")}</p>
            <div className="settings-input-row">
              <input
                className="settings-input settings-input-number"
                type="number"
                min={0}
                max={120}
                step={1}
                value={form.post_roll_seconds}
                onChange={(e) => setForm({ ...form, post_roll_seconds: Number(e.target.value) })}
              />
              <span className="settings-stepper">
                <button onClick={() => setForm({ ...form, post_roll_seconds: form.post_roll_seconds + 1 })}>▲</button>
                <button onClick={() => setForm({ ...form, post_roll_seconds: Math.max(0, form.post_roll_seconds - 1) })}>▼</button>
              </span>
            </div>
          </div>

          {/* Entscheidungs-Optionen */}
          <div className="settings-block">
            <label>{t("settings_decision_types_label")}</label>
            <p className="settings-block-hint">{t("settings_decision_types_hint")}</p>
            <textarea
              className="settings-textarea"
              rows={10}
              value={decisionTypesText}
              onChange={(e) => setDecisionTypesText(e.target.value)}
            />
          </div>
        </div>
      </div>

      {/* Save Button */}
      <button className="settings-save-btn" onClick={save}>
        {saved ? `✓ ${t("settings_saved")}` : t("settings_save")}
      </button>

      {/* Danger Zone – Datenbank */}
      <div className="settings-danger">
        <h3>{t("settings_database")}</h3>
        <p className="settings-danger-hint">
          {t("settings_database_hint")}
        </p>
        <button
          className="settings-danger-btn"
          onClick={() => setShowConfirm(true)}
          disabled={clearing}
        >
          {cleared
            ? "✓ Datenbank geleert"
            : clearing
            ? "Lösche..."
            : t("settings_clear_database")}
        </button>
      </div>

      {showConfirm && (
        <ConfirmModal
          title="Datenbank leeren"
          message="Möchtest Du die Datenbank wirklich leeren? Alle Vorfälle (offene + archivierte) werden unwiderruflich gelöscht."
          confirmLabel="Datenbank löschen"
          cancelLabel="Abbrechen"
          danger
          onConfirm={clearDatabase}
          onCancel={() => setShowConfirm(false)}
        />
      )}
    </div>
  );
}