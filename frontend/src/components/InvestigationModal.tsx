import { useState, useEffect } from "react";
import type { CarStanding, IncidentDraft, Settings } from "../types";
import { useLanguage } from "../i18n/LanguageContext";

interface Props {
  draft: IncidentDraft;
  standings: CarStanding[];
  settings: Settings;
  onClose: () => void;
  onSubmit: (draft: IncidentDraft) => void;
}

export default function InvestigationModal({ draft, standings, settings, onClose, onSubmit }: Props) {
  const { t } = useLanguage();
  const [form, setForm] = useState<IncidentDraft>(draft);

  useEffect(() => setForm(draft), [draft]);

  function set<K extends keyof IncidentDraft>(key: K, value: IncidentDraft[K]) {
    setForm((f) => ({ ...f, [key]: value }));
  }

  function selectDriverA(carNumber: string) {
    const car = standings.find((c) => c.car_number === carNumber);
    if (car) {
      setForm((f) => ({ ...f, car_number_a: car.car_number, driver_a: car.driver, class_a: car.class }));
    } else {
      set("car_number_a", carNumber);
    }
  }
  function selectDriverB(carNumber: string) {
    const car = standings.find((c) => c.car_number === carNumber);
    if (car) {
      setForm((f) => ({ ...f, car_number_b: car.car_number, driver_b: car.driver, class_b: car.class }));
    } else {
      set("car_number_b", carNumber);
    }
  }

  return (
    <div className="modal-backdrop" onClick={onClose}>
      <div className="modal investigation-modal" onClick={(e) => e.stopPropagation()}>
        <div className="modal-header">
          <h1>{t("modal_title")}</h1>
          <button className="modal-close" onClick={onClose}>
            ✕
          </button>
        </div>
        <div className="modal-divider" />

        <div className="modal-grid">
          <div className="field">
            <label>{t("modal_causing_driver")}</label>
            <div className="select-wrapper">
              <select value={form.car_number_a} onChange={(e) => selectDriverA(e.target.value)}>
                <option value="">{t("modal_select_driver")}</option>
                {standings.map((c) => (
                  <option key={c.slot_id} value={c.car_number}>
                    #{c.car_number} {c.driver}
                  </option>
                ))}
              </select>
            </div>
          </div>
          <div className="field">
            <label>{t("modal_affected_driver")}</label>
            <div className="select-wrapper">
              <select value={form.car_number_b} onChange={(e) => selectDriverB(e.target.value)}>
                <option value="">{t("modal_select_driver_optional")}</option>
                {standings.map((c) => (
                  <option key={c.slot_id} value={c.car_number}>
                    #{c.car_number} {c.driver}
                  </option>
                ))}
              </select>
            </div>
          </div>

          <div className="field field-inline">
            <div>
              <label>{t("modal_lap")}</label>
              <div className="modal-input-with-arrows">
                <input
                  type="number"
                  value={form.lap}
                  onChange={(e) => set("lap", Number(e.target.value))}
                />
                <div className="input-arrows">
                  <button className="input-arrow-btn" onClick={() => set("lap", form.lap + 1)}>
                    <img src="/icons/Pfeil oben.png" alt="+" className="input-arrow-icon" />
                  </button>
                  <button className="input-arrow-btn" onClick={() => set("lap", Math.max(0, form.lap - 1))}>
                    <img src="/icons/Pfeil unten.png" alt="-" className="input-arrow-icon" />
                  </button>
                </div>
              </div>
            </div>
            <div>
              <label>{t("modal_corner")}</label>
              <div className="modal-input-with-arrows">
                <input
                  type="number"
                  min={0}
                  value={form.corner || 1}
                  onChange={(e) => set("corner", e.target.value)}
                />
                <div className="input-arrows">
                  <button className="input-arrow-btn" onClick={() => set("corner", String(Number(form.corner || 1) + 1))}>
                    <img src="/icons/Pfeil oben.png" alt="+" className="input-arrow-icon" />
                  </button>
                  <button className="input-arrow-btn" onClick={() => set("corner", String(Math.max(0, Number(form.corner || 1) - 1)))}>
                    <img src="/icons/Pfeil unten.png" alt="-" className="input-arrow-icon" />
                  </button>
                </div>
              </div>
            </div>
          </div>
          <div className="field">
            <label>{t("modal_timestamp")}</label>
            <input
              value={form.timestamp_label}
              onChange={(e) => set("timestamp_label", e.target.value)}
            />
          </div>

          <div className="field">
            <label>{t("modal_incident_type")}</label>
            <div className="select-wrapper">
              <select
                value={form.incident_type}
                onChange={(e) => set("incident_type", e.target.value)}
              >
                <option value="">{t("modal_select_incident_type")}</option>
                {settings.incident_types.map((type) => (
                  <option key={type} value={type}>
                    {type}
                  </option>
                ))}
              </select>
            </div>
          </div>
          <div className="field">
            <label>{t("modal_decision")}</label>
            <div className="select-wrapper">
              <select value={form.decision} onChange={(e) => set("decision", e.target.value)}>
                <option value="">{t("modal_select_decision")}</option>
                {settings.decision_types.map((type) => (
                  <option key={type} value={type}>
                    {type}
                  </option>
                ))}
              </select>
            </div>
          </div>

          <div className="field">
            <label>{t("modal_warning_points")}</label>
            <div className="modal-input-with-arrows">
              <input
                type="number"
                min={0}
                value={form.warning_points}
                onChange={(e) => set("warning_points", Number(e.target.value))}
              />
              <div className="input-arrows">
                <button className="input-arrow-btn" onClick={() => set("warning_points", form.warning_points + 1)}>
                  <img src="/icons/Pfeil oben.png" alt="+" className="input-arrow-icon" />
                </button>
                <button className="input-arrow-btn" onClick={() => set("warning_points", Math.max(0, form.warning_points - 1))}>
                  <img src="/icons/Pfeil unten.png" alt="-" className="input-arrow-icon" />
                </button>
              </div>
            </div>
          </div>
          <div className="field">
            <label>{t("modal_penalty_points")}</label>
            <div className="modal-input-with-arrows">
              <input
                type="number"
                min={0}
                value={form.penalty_points}
                onChange={(e) => set("penalty_points", Number(e.target.value))}
              />
              <div className="input-arrows">
                <button className="input-arrow-btn" onClick={() => set("penalty_points", form.penalty_points + 1)}>
                  <img src="/icons/Pfeil oben.png" alt="+" className="input-arrow-icon" />
                </button>
                <button className="input-arrow-btn" onClick={() => set("penalty_points", Math.max(0, form.penalty_points - 1))}>
                  <img src="/icons/Pfeil unten.png" alt="-" className="input-arrow-icon" />
                </button>
              </div>
            </div>
          </div>

          <div className="field field-full">
            <label>{t("modal_reasoning")}</label>
            <textarea
              rows={5}
              placeholder={t("modal_reasoning_placeholder")}
              value={form.reasoning}
              onChange={(e) => set("reasoning", e.target.value)}
            />
          </div>
        </div>

        <div className="modal-actions">
          <button className="btn-outline" onClick={onClose}>
            {t("modal_cancel")}
          </button>
          <button
            className="btn-solid"
            onClick={() => onSubmit(form)}
            disabled={!form.incident_type || !form.decision}
          >
            {t("modal_submit")}
          </button>
        </div>
      </div>
    </div>
  );
}
