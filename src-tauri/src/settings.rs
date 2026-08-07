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
    pub pre_roll_seconds: f64,
    pub post_roll_seconds: f64,
    #[serde(default)]
    pub license_key: String,
    /// Pfad zur LMU-Installation (für `keyboard.json`).
    /// Standard: C:\Program Files (x86)\Steam\steamapps\common\Le Mans Ultimate
    #[serde(default = "default_lmu_install_path")]
    pub lmu_install_path: String,
    /// Server-URL für Enterprise (z.B. http://v-server-ip:3000)
    #[serde(default)]
    pub server_url: String,
    /// API-Key für Enterprise-Server-Zugriff
    #[serde(default)]
    pub api_key: String,
}

fn default_lmu_install_path() -> String {
    "C:\\Program Files (x86)\\Steam\\steamapps\\common\\Le Mans Ultimate".to_string()
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
            pre_roll_seconds: 20.0,
            post_roll_seconds: 20.0,
            license_key: String::new(),
            lmu_install_path: default_lmu_install_path(),
            server_url: String::new(),
            api_key: String::new(),
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