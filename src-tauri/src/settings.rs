//! Vom Kommissariat selbst pflegbare Einstellungen: die Dropdown-Listen für
//! "Vorfall auswählen" / "Entscheidung der Rennkommission", die Discord-
//! Webhook-URL sowie die FCY-Parameter. Wird als `settings.json` im
//! App-Datenverzeichnis abgelegt - überlebt also App-Updates.

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Settings {
    pub discord_webhook_url: String,
    pub incident_types: Vec<String>,
    pub decision_types: Vec<String>,
    pub fcy_speed_limit_kmh: f64,
    pub fcy_countdown_seconds: i32,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            discord_webhook_url: String::new(),
            // Platzhalter-Werte - bitte in den "Einstellungen" der App durch
            // eure verbindlichen Regelwerk-Kategorien ersetzen.
            incident_types: vec![
                "Kollision Fahrzeug-Fahrzeug".into(),
                "Alleinunfall".into(),
                "Track Limits".into(),
                "Unsportliches Verhalten".into(),
                "Blocking".into(),
                "FCY-Verstoß".into(),
                "Sonstiges".into(),
            ],
            decision_types: vec![
                "Keine Maßnahme (NFA)".into(),
                "Verwarnung".into(),
                "Drive Through".into(),
                "Stop & Go".into(),
                "Zeitstrafe".into(),
                "Rennausschluss".into(),
            ],
            fcy_speed_limit_kmh: 60.0,
            fcy_countdown_seconds: 10,
        }
    }
}

pub struct SettingsStore {
    path: PathBuf,
}

impl SettingsStore {
    pub fn new(app_dir: &std::path::Path) -> Self {
        Self {
            path: app_dir.join("settings.json"),
        }
    }

    pub fn load(&self) -> Settings {
        std::fs::read_to_string(&self.path)
            .ok()
            .and_then(|s| serde_json::from_str(&s).ok())
            .unwrap_or_default()
    }

    pub fn save(&self, settings: &Settings) -> Result<()> {
        let json = serde_json::to_string_pretty(settings)?;
        std::fs::write(&self.path, json)?;
        Ok(())
    }
}
