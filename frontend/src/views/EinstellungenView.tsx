import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import type { Settings } from "../types";
import { useLanguage } from "../i18n/LanguageContext";
import ConfirmModal from "../components/ConfirmModal";

interface KeyboardMappingEntry {
  action: string;
  key_name: string;
  scan: number;
  extended: boolean;
}

const KM_ACTION_LABELS: Record<string, string> = {
  "Tracking Cameras": "TV",
  "Driving Cameras": "Bord",
  "Swingman Camera": "Heck",
  "Swingman Zoom In": "Zoom+",
  "Swingman Zoom Out": "Zoom-",
  "Instant Replay": "Replay",
  "Replay Play": "Play",
  "Replay Stop": "Stop",
  "Replay Slowmotion": "Slow",
  "Replay Fast Forward": "Vor",
  "Replay Fast Rewind": "Schnell Zurück",
  "Replay Reverse": "Rück",
};

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
  const [keyboardMapping, setKeyboardMapping] = useState<KeyboardMappingEntry[]>([]);
  const [reloaded, setReloaded] = useState(false);

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

  // Tastenbelegung aus dem Backend laden
  useEffect(() => {
    loadKeyboardMapping();
  }, []);

  async function loadKeyboardMapping() {
    try {
      const mapping = await invoke<KeyboardMappingEntry[]>("get_keyboard_mapping");
      setKeyboardMapping(mapping);
    } catch (err) {
      console.error("Fehler beim Laden der Tastenbelegung:", err);
    }
  }

  async function reloadKeyboardMapping() {
    try {
      const mapping = await invoke<KeyboardMappingEntry[]>("reload_keyboard_mapping");
      setKeyboardMapping(mapping);
      setReloaded(true);
      setTimeout(() => setReloaded(false), 2000);
    } catch (err) {
      console.error("Fehler beim Neuladen der Tastenbelegung:", err);
    }
  }

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
      {/* Titel – Figma: text-4xl, leading-[54px], tracking-wide, center */}
      <div className="einstellungen-title">{t("settings_title")}</div>

      {/* Hinweis – Figma: white/60, text-xs, Michroma, leading-4 */}
      <p className="einstellungen-hint">{t("settings_hint")}</p>

      {/* 2-Spalten-Grid – Figma: gap-12 (48px), flex-1 */}
      <div className="einstellungen-grid">
        {/* Linke Spalte */}
        <div className="einstellungen-col">
          {/* Lizenz Informationen – Figma: self-stretch, flex-col, gap-2.5 */}
          <div className="einstellungen-field">
            <div className="einstellungen-field-label">Lizenz Informationen</div>
            <div className="einstellungen-field-hint">Nachfolgende Lizenznummer ist für Dein Gerät registriert. Wünschst Du eine weitere Lizenz oder möchtest Deine Lizenz erweitern, dann kontaktiere uns gerne unter www.lmu-racecontrol.gg</div>
            <div className="einstellungen-input-wrapper">
              <input
                className="einstellungen-input"
                value={form.license_key}
                placeholder="123B-234C-345YZ"
                readOnly
              />
            </div>
          </div>

          {/* FCY-Countdown (Sekunden) – Figma: self-stretch, flex-col, gap-2.5 */}
          <div className="einstellungen-field">
            <div className="einstellungen-field-label">{t("settings_countdown")}</div>
            <div className="einstellungen-input-simple">
              <input
                className="einstellungen-input-simple-field"
                type="number"
                value={form.fcy_countdown_seconds}
                onChange={(e) => setForm({ ...form, fcy_countdown_seconds: Number(e.target.value) })}
              />
              <div className="input-arrows">
                <button className="input-arrow-btn" onClick={() => setForm({ ...form, fcy_countdown_seconds: form.fcy_countdown_seconds + 1 })}>
                  <img src="/icons/Pfeil oben.png" alt="+" className="input-arrow-icon" />
                </button>
                <button className="input-arrow-btn" onClick={() => setForm({ ...form, fcy_countdown_seconds: Math.max(0, form.fcy_countdown_seconds - 1) })}>
                  <img src="/icons/Pfeil unten.png" alt="-" className="input-arrow-icon" />
                </button>
              </div>
            </div>
          </div>

          {/* Vorfall Voralaufzeit im Replay – Figma */}
          <div className="einstellungen-field">
            <div className="einstellungen-field-label">{t("settings_pre_roll_label")}</div>
            <div className="einstellungen-field-hint">{t("settings_pre_roll_hint")}</div>
            <div className="einstellungen-input-simple">
              <input
                className="einstellungen-input-simple-field"
                type="number"
                min={0}
                max={120}
                value={form.pre_roll_seconds}
                onChange={(e) => setForm({ ...form, pre_roll_seconds: Number(e.target.value) })}
              />
              <div className="input-arrows">
                <button className="input-arrow-btn" onClick={() => setForm({ ...form, pre_roll_seconds: form.pre_roll_seconds + 1 })}>
                  <img src="/icons/Pfeil oben.png" alt="+" className="input-arrow-icon" />
                </button>
                <button className="input-arrow-btn" onClick={() => setForm({ ...form, pre_roll_seconds: Math.max(0, form.pre_roll_seconds - 1) })}>
                  <img src="/icons/Pfeil unten.png" alt="-" className="input-arrow-icon" />
                </button>
              </div>
            </div>
          </div>

          {/* Vorfall Kategorien – "Vorfall auswählen" – Figma */}
          <div className="einstellungen-field">
            <div className="einstellungen-field-label">{t("settings_incident_types_label")}</div>
            <div className="einstellungen-field-hint">{t("settings_incident_types_hint")}</div>
            <div className="einstellungen-textarea-wrapper">
              <textarea
                className="einstellungen-textarea"
                value={incidentTypesText}
                onChange={(e) => setIncidentTypesText(e.target.value)}
              />
            </div>
          </div>

          {/* Save Button – in der linken Spalte unter den Kategorien */}
          <button className="einstellungen-save-btn" onClick={save}>
            {saved ? `✓ ${t("settings_saved")}` : t("settings_save")}
          </button>
        </div>

        {/* Rechte Spalte */}
        <div className="einstellungen-col">
          {/* Discord Webhook-URL – Figma */}
          <div className="einstellungen-field">
            <div className="einstellungen-field-label">{t("settings_webhook_label")}</div>
            <div className="einstellungen-field-hint">{t("settings_webhook_hint")}</div>
            <div className="einstellungen-input-wrapper">
              <input
                className="einstellungen-input"
                value={form.discord_webhook_url}
                placeholder="https://discord.com/api/webhooks/..."
                onChange={(e) => setForm({ ...form, discord_webhook_url: e.target.value })}
              />
            </div>
          </div>

          {/* FCY-Geschwindigkeit (km/h) – Figma */}
          <div className="einstellungen-field">
            <div className="einstellungen-field-label">{t("settings_speed_limit")}</div>
            <div className="einstellungen-input-simple">
              <input
                className="einstellungen-input-simple-field"
                type="number"
                value={form.fcy_speed_limit_kmh}
                onChange={(e) => setForm({ ...form, fcy_speed_limit_kmh: Number(e.target.value) })}
              />
              <div className="input-arrows">
                <button className="input-arrow-btn" onClick={() => setForm({ ...form, fcy_speed_limit_kmh: form.fcy_speed_limit_kmh + 1 })}>
                  <img src="/icons/Pfeil oben.png" alt="+" className="input-arrow-icon" />
                </button>
                <button className="input-arrow-btn" onClick={() => setForm({ ...form, fcy_speed_limit_kmh: Math.max(0, form.fcy_speed_limit_kmh - 1) })}>
                  <img src="/icons/Pfeil unten.png" alt="-" className="input-arrow-icon" />
                </button>
              </div>
            </div>
          </div>

          {/* Vorfall Nachlaufzeit im Replay – Figma */}
          <div className="einstellungen-field">
            <div className="einstellungen-field-label">{t("settings_post_roll_label")}</div>
            <div className="einstellungen-field-hint">{t("settings_post_roll_hint")}</div>
            <div className="einstellungen-input-simple">
              <input
                className="einstellungen-input-simple-field"
                type="number"
                min={0}
                max={120}
                value={form.post_roll_seconds}
                onChange={(e) => setForm({ ...form, post_roll_seconds: Number(e.target.value) })}
              />
              <div className="input-arrows">
                <button className="input-arrow-btn" onClick={() => setForm({ ...form, post_roll_seconds: form.post_roll_seconds + 1 })}>
                  <img src="/icons/Pfeil oben.png" alt="+" className="input-arrow-icon" />
                </button>
                <button className="input-arrow-btn" onClick={() => setForm({ ...form, post_roll_seconds: Math.max(0, form.post_roll_seconds - 1) })}>
                  <img src="/icons/Pfeil unten.png" alt="-" className="input-arrow-icon" />
                </button>
              </div>
            </div>
          </div>

          {/* Kategorien – "Entscheidung der Rennkommission" – Figma */}
          <div className="einstellungen-field">
            <div className="einstellungen-field-label">{t("settings_decision_types_label")}</div>
            <div className="einstellungen-field-hint">{t("settings_decision_types_hint")}</div>
            <div className="einstellungen-textarea-wrapper">
              <textarea
                className="einstellungen-textarea"
                value={decisionTypesText}
                onChange={(e) => setDecisionTypesText(e.target.value)}
              />
            </div>
          </div>
        </div>
      </div>

      {/* LMU-Tastenbelegung – zwischen Grid und Danger Zone */}
      <div className="einstellungen-lmu-keys">
        <div className="einstellungen-field-label">{t("settings_lmu_keys_title")}</div>
        <div className="einstellungen-field-hint">{t("settings_lmu_keys_hint")}</div>
        
        {/* LMU-Pfad-Eingabe */}
        <div className="einstellungen-lmu-path-row">
          <div className="einstellungen-lmu-path-input-wrapper">
            <div className="einstellungen-field-label-small">{t("settings_lmu_path_label")}</div>
            <div className="einstellungen-field-hint-small">{t("settings_lmu_path_hint")}</div>
            <input
              className="einstellungen-input"
              value={form.lmu_install_path}
              onChange={(e) => setForm({ ...form, lmu_install_path: e.target.value })}
            />
          </div>
          <button
            className="einstellungen-lmu-reload-btn"
            onClick={reloadKeyboardMapping}
          >
            {reloaded ? `✓ ${t("settings_lmu_reloaded")}` : t("settings_lmu_reload")}
          </button>
        </div>

        {/* Tastenanzeige */}
        {keyboardMapping.length > 0 && (
          <div className="einstellungen-lmu-keys-grid">
            {keyboardMapping.map((entry) => (
              <div key={entry.action} className="einstellungen-lmu-key-item">
                <span className="einstellungen-lmu-key-label">
                  {KM_ACTION_LABELS[entry.action] || entry.action}
                </span>
                <span className="einstellungen-lmu-key-value">
                  {entry.key_name}
                </span>
              </div>
            ))}
          </div>
        )}
        {keyboardMapping.length === 0 && (
          <p className="einstellungen-lmu-keys-empty">Keine Tastenbelegung geladen.</p>
        )}
      </div>

      {/* Danger Zone – Datenbank leeren – Figma: h-44, py-6, bg-red-600/20, rounded-[10px], outline-red-600 */}
      <div className="einstellungen-danger">
        <div className="einstellungen-danger-title">{t("settings_database")}</div>
        <div className="einstellungen-danger-hint">{t("settings_database_hint")}</div>
        <button
          className="einstellungen-danger-btn"
          onClick={() => setShowConfirm(true)}
          disabled={clearing}
        >
          <span className="einstellungen-danger-btn-primary">{t("settings_clear_database")}</span>
          <span className="einstellungen-danger-btn-sub">(Alle Vorfälle)</span>
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