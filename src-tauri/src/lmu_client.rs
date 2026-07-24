//! Client für die offizielle, in Le Mans Ultimate eingebaute REST-API.
//!
//! LMU startet standardmäßig einen lokalen HTTP-Server auf Port 6397
//! (Quelle: Community-Tools wie `yiddifliddo/LMU-Replay-GUI`,
//! `snipem/go-lmu-api`, `mzluzifer/LMU-REST-API`). Es handelt sich NICHT
//! um Shared-Memory, sondern um echtes HTTP/JSON - deutlich robuster.
//!
//! WICHTIG: Die exakten Feldnamen innerhalb der JSON-Antworten (z.B. von
//! `/rest/watch/standings`) können sich zwischen LMU-Versionen ändern.
//! Dieser Client parst daher bewusst dynamisch über `serde_json::Value`
//! und liest bekannte Feldnamen mit Fallbacks. Bitte einmal mit laufendem
//! LMU gegen `http://localhost:6397/rest/watch/standings` prüfen (z.B. im
//! Browser öffnen) und ggf. die Feldnamen in `parse_standings` anpassen.
//!
//! ## API-Endpunkte (laut Swagger-Schema von mzluzifer/LMU-REST-API)
//! - GET  /rest/watch/sessionInfo
//! - GET  /rest/watch/standings
//! - GET  /rest/watch/standings/history
//! - GET  /rest/watch/replays
//! - GET  /rest/watch/trackmap
//! - PUT  /rest/watch/replaytime/{time}
//! - PUT  /rest/watch/replayCommand/{command}
//! - GET  /webdata/.*
//! - POST /webdata/.*

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::time::Duration;

const DEFAULT_BASE_URL: &str = "http://localhost:6397";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CarStanding {
    pub slot_id: i64,
    pub position: i32,
    pub car_number: String,
    pub driver: String,
    pub team: String,
    pub class: String,
    pub car_model: String,
    pub class_position: i32,
    pub laps: i32,
    pub gap: String,
    pub last_lap_s: f64,
    pub best_lap_s: f64,
    pub sector1_s: f64,
    pub sector2_s: f64,
    pub sector3_s: f64,
    pub top_speed_kmh: f64,
    /// Aktuelle Momentan-Geschwindigkeit in km/h.
    /// LMU liefert `speed` in m/s → wir konvertieren intern.
    pub speed_kmh: f64,
    pub in_pits: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SessionInfo {
    pub session_type: String,
    pub track_name: String,
    pub time_of_day: String,
    pub session_time_remaining_s: f64,
    pub num_cars: i32,
}

pub struct LmuClient {
    http: reqwest::Client,
    base_url: String,
}

impl LmuClient {
    pub fn new() -> Self {
        Self::with_base_url(DEFAULT_BASE_URL)
    }

    pub fn with_base_url(base_url: &str) -> Self {
        let http = reqwest::Client::builder()
            .timeout(Duration::from_secs(2))
            .build()
            .expect("HTTP-Client konnte nicht erstellt werden");
        Self {
            http,
            base_url: base_url.trim_end_matches('/').to_string(),
        }
    }

    pub async fn is_available(&self) -> bool {
        self.get_json("/rest/watch/sessionInfo").await.is_ok()
    }

    async fn get_json(&self, path: &str) -> Result<Value> {
        let url = format!("{}{}", self.base_url, path);
        let resp = self
            .http
            .get(&url)
            .send()
            .await
            .with_context(|| format!("GET {} fehlgeschlagen (läuft LMU?)", url))?
            .error_for_status()
            .with_context(|| format!("GET {} lieferte Fehlerstatus", url))?;
        resp.json::<Value>()
            .await
            .with_context(|| format!("Antwort von {} war kein gültiges JSON", url))
    }

    async fn put(&self, path: &str) -> Result<()> {
        let url = format!("{}{}", self.base_url, path);
        self.http
            .put(&url)
            .send()
            .await
            .with_context(|| format!("PUT {} fehlgeschlagen", url))?
            .error_for_status()
            .with_context(|| format!("PUT {} lieferte Fehlerstatus", url))?;
        Ok(())
    }

    pub async fn get_standings(&self) -> Result<Vec<CarStanding>> {
        let raw = self.get_json("/rest/watch/standings").await?;
        parse_standings(&raw)
    }

    pub async fn get_session_info(&self) -> Result<SessionInfo> {
        let raw = self.get_json("/rest/watch/sessionInfo").await?;
        Ok(parse_session_info(&raw))
    }

    pub async fn seek_replay_to(&self, seconds_since_start: f64) -> Result<()> {
        let path = format!("/rest/watch/replaytime/{}", seconds_since_start as i64);
        self.put(&path).await
    }

    pub async fn switch_to_live(&self) -> Result<()> {
        self.put("/rest/watch/replayCommand/live").await
    }

    pub async fn switch_to_replay(&self) -> Result<()> {
        self.put("/rest/watch/replayCommand/replay").await
    }

    /// Fokussiert die Kamera auf ein bestimmtes Fahrzeug per Slot-ID.
    /// Wird von Broadcast Control UK genauso verwendet - funktioniert!
    pub async fn focus_slot(&self, slot_id: i64) -> Result<()> {
        let path = format!("/rest/watch/focus/{}", slot_id);
        self.put(&path).await
    }

    /// Löscht den Fahrzeug-Fokus (zurück zur freien Kamera).
    pub async fn clear_focus(&self) -> Result<()> {
        self.put("/rest/watch/focus/clear").await
    }

    /// Wählt eine Kamera via REST-API.
    ///
    /// Basierend auf der offiziellen LMU-UI (Studio 397) `BroadcastService.js`:
    /// `PUT /rest/watch/focus/{type}/{trackSideGroup}/{shouldAdvance}`
    ///
    /// Kamera-Typen (aus `getCameras()`):
    /// - `SCV_COCKPIT`   → index 0 = On-board, index 1 = Cockpit
    /// - `SCV_NOSECAM`   → index 2 = Nose
    /// - `SCV_SWINGMAN`  → index 3 = Rear / Swingman
    /// - `SCV_TRACKSIDE` → index 4 = Trackside
    /// - `SCV_SPECTATOR` → index 5 = Spectator / TV-Zyklus
    ///
    /// **Wichtig:** LMU verwendet NICHT die rFactor2 Shared Memory Offsets
    /// 0x24/0x28 für die Kamera-Steuerung! Die Kamera-Steuerung erfolgt
    /// ausschließlich über die REST-API.
    pub async fn select_camera(&self, cam_type: &str, track_side_group: u32, should_advance: bool) -> Result<()> {
        let advance_flag = if should_advance { 1 } else { 0 };
        let path = format!("/rest/watch/focus/{}/{}/{}", cam_type, track_side_group, advance_flag);
        self.put(&path).await
    }
}

/// Wandelt die rohe JSON-Struktur von `/rest/watch/standings` in unser
/// Domänenmodell um. Bewusst tolerant gegenüber fehlenden/zusätzlichen
/// Feldern, da die exakte LMU-JSON-Struktur je nach Version variieren kann.
fn parse_standings(raw: &Value) -> Result<Vec<CarStanding>> {
    // Debug: ALLE Feldnamen des ersten Eintrags ausgeben, damit wir die
    // exakte JSON-Struktur von LMU sehen (insb. für speed_kmh)
    if let Some(arr) = raw.as_array() {
        if let Some(first) = arr.first() {
            if let Some(obj) = first.as_object() {
                println!("[lmu_client] === ALLE FELDER des ersten Fahrzeugs ===");
                for (key, val) in obj.iter() {
                    println!("  {} = {}", key, val);
                }
                println!("[lmu_client] ===========================================");
            }
        }
        println!("[lmu_client] Anzahl Fahrzeuge: {}", arr.len());
    } else if let Some(cars) = raw.get("cars").and_then(|v| v.as_array()) {
        if let Some(first) = cars.first() {
            if let Some(obj) = first.as_object() {
                println!("[lmu_client] === ALLE FELDER des ersten Fahrzeugs (cars) ===");
                for (key, val) in obj.iter() {
                    println!("  {} = {}", key, val);
                }
                println!("[lmu_client] =================================================");
            }
        }
    } else if let Some(vehicles) = raw.get("vehicles").and_then(|v| v.as_array()) {
        if let Some(first) = vehicles.first() {
            if let Some(obj) = first.as_object() {
                println!("[lmu_client] === ALLE FELDER des ersten Fahrzeugs (vehicles) ===");
                for (key, val) in obj.iter() {
                    println!("  {} = {}", key, val);
                }
                println!("[lmu_client] =====================================================");
            }
        }
    } else {
        // Fallback: Zeige die gesamte rohe JSON-Antwort (gekürzt auf 5000 Zeichen)
        let raw_str = serde_json::to_string(raw).unwrap_or_default();
        let preview = if raw_str.len() > 5000 { &raw_str[..5000] } else { &raw_str };
        println!("[lmu_client] RAW JSON (erste 5000 Zeichen):\n{}", preview);
    }

    let list = raw
        .as_array()
        .cloned()
        .or_else(|| raw.get("cars").and_then(|v| v.as_array()).cloned())
        .or_else(|| raw.get("vehicles").and_then(|v| v.as_array()).cloned())
        .or_else(|| raw.get("standings").and_then(|v| v.as_array()).cloned())
        .or_else(|| raw.get("entries").and_then(|v| v.as_array()).cloned())
        .context(
            "Konnte kein Array in der standings-Antwort finden - \
             bitte JSON-Struktur mit laufendem LMU prüfen und \
             parse_standings() in lmu_client.rs anpassen",
        )?;

    let mut out = Vec::with_capacity(list.len());
    for (idx, entry) in list.iter().enumerate() {
        // Geschwindigkeit: LMU liefert `carVelocity.velocity` in m/s (aus typedefs.js bestätigt).
        // Werte wie 87.4 m/s = 314 km/h. Konvertiere * 3.6 für km/h.
        let speed_mps = entry
            .get("carVelocity")
            .and_then(|v| v.get("velocity"))
            .and_then(|v| v.as_f64())
            .or_else(|| field_f64(entry, &["speed", "currentSpeed", "speedKmh", "kmh", "groundSpeed"]))
            .unwrap_or(0.0);
        let speed_kmh = speed_mps * 3.6;

        out.push(CarStanding {
            // slotID (großes D) ist der offizielle LMU-Name (aus typedefs.js)
            slot_id: field_i64(entry, &["slotID", "slotId", "id", "vehicleId"])
                .unwrap_or(idx as i64),
            position: field_i64(entry, &["position", "place", "pos"]).unwrap_or(0) as i32,
            car_number: field_string(entry, &["carNumber", "number", "carNum"]),
            // fullTeamName ist der offizielle LMU-Name (aus typedefs.js)
            team: field_string(entry, &["fullTeamName", "team", "teamName"]),
            // driverName ist der offizielle LMU-Name (aus typedefs.js)
            driver: field_string(entry, &["driverName", "driver", "name"]),
            // carClass ist der offizielle LMU-Name (aus typedefs.js)
            class: field_string(entry, &["carClass", "class", "vehicleClass"]),
            car_model: field_string(entry, &["vehicleName", "carModel", "vehicle", "carType"]),
            class_position: field_i64(entry, &["classPosition", "picPosition", "pic"])
                .unwrap_or(0) as i32,
            // lapsCompleted ist der offizielle LMU-Name (aus typedefs.js)
            laps: field_i64(entry, &["lapsCompleted", "laps", "totalLaps"]).unwrap_or(0) as i32,
            gap: field_string(entry, &["gap", "gapToLeader"]),
            // lastLapTime ist der offizielle LMU-Name (aus typedefs.js)
            last_lap_s: field_f64(entry, &["lastLapTime", "lastLap"]).unwrap_or(0.0),
            // bestLapTime ist der offizielle LMU-Name (aus typedefs.js)
            best_lap_s: field_f64(entry, &["bestLapTime", "bestLap"]).unwrap_or(0.0),
            sector1_s: field_f64(entry, &["sector1", "s1", "sector1Time"]).unwrap_or(0.0),
            sector2_s: field_f64(entry, &["sector2", "s2", "sector2Time"]).unwrap_or(0.0),
            sector3_s: field_f64(entry, &["sector3", "s3", "sector3Time"]).unwrap_or(0.0),
            top_speed_kmh: field_f64(entry, &["topSpeed", "vmax", "maxSpeed"]).unwrap_or(0.0),
            speed_kmh,
            // pitting ist der offizielle LMU-Name (aus typedefs.js)
            in_pits: field_bool(entry, &["pitting", "inPits", "isInPit", "pit"]).unwrap_or(false),
        });
    }
    Ok(out)
}

fn parse_session_info(raw: &Value) -> SessionInfo {
    SessionInfo {
        session_type: field_string(raw, &["sessionType", "session", "type"]),
        track_name: field_string(raw, &["trackName", "track"]),
        time_of_day: field_string(raw, &["timeOfDay", "tod"]),
        session_time_remaining_s: field_f64(raw, &["timeRemaining", "sessionTimeRemaining"])
            .unwrap_or(0.0),
        num_cars: field_i64(raw, &["numCars", "carCount", "numVehicles"]).unwrap_or(0) as i32,
    }
}

fn field_i64(v: &Value, keys: &[&str]) -> Option<i64> {
    keys.iter().find_map(|k| v.get(k).and_then(Value::as_i64))
}
fn field_f64(v: &Value, keys: &[&str]) -> Option<f64> {
    keys.iter().find_map(|k| v.get(k).and_then(Value::as_f64))
}
fn field_bool(v: &Value, keys: &[&str]) -> Option<bool> {
    keys.iter().find_map(|k| v.get(k).and_then(Value::as_bool))
}
fn field_string(v: &Value, keys: &[&str]) -> String {
    keys.iter()
        .find_map(|k| v.get(k).and_then(Value::as_str))
        .unwrap_or("")
        .to_string()
}