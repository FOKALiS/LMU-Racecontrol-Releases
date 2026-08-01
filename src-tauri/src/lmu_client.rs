//! Client für die offizielle, in Le Mans Ultimate eingebaute REST-API.
//!
//! LMU startet standardmäßig einen lokalen HTTP-Server auf Port 6397

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
    pub speed_kmh: f64,
    pub in_pits: bool,
    /// Kontakt-Daten aus dem Shared Memory (VehicleTelemetry)
    #[serde(default)]
    pub impact_et: f64,
    #[serde(default)]
    pub impact_mag: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SessionInfo {
    pub session_type: String,
    pub track_name: String,
    pub time_of_day: String,
    pub session_time_remaining_s: f64,
    pub session_time_elapsed_s: f64,
    pub num_cars: i32,
}

#[derive(Clone)]
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
            .with_context(|| format!("GET {} fehlgeschlagen", url))?
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
            .header("Content-Type", "application/json")
            .json(&serde_json::json!({}))
            .send()
            .await
            .with_context(|| format!("PUT {} fehlgeschlagen", url))?
            .error_for_status()
            .with_context(|| format!("PUT {} lieferte Fehlerstatus", url))?;
        Ok(())
    }

    async fn post_json(&self, path: &str, body: &Value) -> Result<()> {
        let url = format!("{}{}", self.base_url, path);
        let resp = self
            .http
            .post(&url)
            .header("Content-Type", "application/json")
            .json(body)
            .send()
            .await
            .with_context(|| format!("POST {} fehlgeschlagen", url))?;
        let status = resp.status();
        let text = resp.text().await.unwrap_or_default();
        if text.is_empty() || text == "OK" || text == "true" || text.contains("\"cameraName\"") {
            Ok(())
        } else {
            Err(anyhow::anyhow!("POST {}: {} ({})", url, status, text))
        }
    }

    pub async fn get_standings(&self) -> Result<Vec<CarStanding>> {
        let raw = self.get_json("/rest/watch/standings").await?;
        parse_standings(&raw)
    }

    pub async fn get_session_info(&self) -> Result<SessionInfo> {
        let raw = self.get_json("/rest/watch/sessionInfo").await?;
        // DEBUG: Einmal die rohe sessionInfo-Antwort loggen
        static SESSION_RAW_LOGGED: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);
        if !SESSION_RAW_LOGGED.load(std::sync::atomic::Ordering::SeqCst) {
            SESSION_RAW_LOGGED.store(true, std::sync::atomic::Ordering::SeqCst);
            eprintln!("[session_raw] sessionInfo JSON: {}", serde_json::to_string_pretty(&raw).unwrap_or_default());
        }
        Ok(parse_session_info(&raw))
    }

    pub async fn seek_replay_to(&self, seconds_since_start: f64) -> Result<()> {
        let path = format!("/rest/watch/replaytime/{:.1}", seconds_since_start);
        self.put(&path).await
    }

    pub async fn switch_to_live(&self) -> Result<()> {
        self.put("/rest/watch/replayCommand/live").await
    }

    pub async fn switch_to_replay(&self) -> Result<()> {
        self.put("/rest/watch/replayCommand/replay").await
    }

    pub async fn replay_play(&self) -> Result<()> {
        self.put("/rest/watch/replayCommand/VCRCOMMAND_PLAY").await
    }

    pub async fn pre_arm_replay(&self) -> Result<()> {
        self.put("/rest/watch/replayCommand/PreArmReplay").await
    }

    pub async fn is_replay_active(&self) -> Result<bool> {
        let val = self.get_json("/rest/replay/isActive").await?;
        Ok(val.as_bool().unwrap_or(false))
    }

    pub async fn focus_slot(&self, slot_id: i64) -> Result<()> {
        let path = format!("/rest/watch/focus/{}", slot_id);
        self.put(&path).await
    }

    pub async fn clear_focus(&self) -> Result<()> {
        self.put("/rest/watch/focus/clear").await
    }

    /// Kamera wechseln – probiert alle bekannten Endpunkte durch
    pub async fn select_camera(&self, cam_type: &str) -> Result<()> {
        let camera_id = match cam_type {
            "TV" | "Tv" | "tv" => 4,
            "Onboard" | "OB" | "Helmet" | "Cockpit" => 6,
            "Nose" | "Front" | "Bonnet" | "Bodywork" => 0,
            "Heli" | "Top" | "Trackside" => 1,
            "Swing" | "Swingman" | "Rear" | "Heck" => 2,
            _ => return Err(anyhow::anyhow!("Unbekannte Kamera: {}", cam_type)),
        };

        // 1. CameraController mit ID
        if self.post_json("/rest/replay/CameraController/switchCameraFamily", &serde_json::json!({"id": camera_id})).await.is_ok() {
            return Ok(());
        }

        // 2. CameraController mit group
        if self.post_json("/rest/replay/CameraController/switchCameraFamily", &serde_json::json!({"group": cam_type})).await.is_ok() {
            return Ok(());
        }

        // 3. Fallback: alter REST-Endpunkt
        self.put(&format!("/rest/watch/focus/{}", cam_type)).await
    }
}

fn parse_standings(raw: &Value) -> Result<Vec<CarStanding>> {
    let list = raw
        .as_array()
        .cloned()
        .or_else(|| raw.get("cars").and_then(|v| v.as_array()).cloned())
        .or_else(|| raw.get("vehicles").and_then(|v| v.as_array()).cloned())
        .or_else(|| raw.get("standings").and_then(|v| v.as_array()).cloned())
        .or_else(|| raw.get("entries").and_then(|v| v.as_array()).cloned())
        .context("Konnte kein Array in der standings-Antwort finden")?;

    let mut out = Vec::with_capacity(list.len());
    for (idx, entry) in list.iter().enumerate() {
        let speed_mps = entry
            .get("carVelocity")
            .and_then(|v| v.get("velocity"))
            .and_then(|v| v.as_f64())
            .or_else(|| field_f64(entry, &["speed", "currentSpeed", "speedKmh", "kmh", "groundSpeed"]))
            .unwrap_or(0.0);
        let speed_kmh = speed_mps * 3.6;

        out.push(CarStanding {
            slot_id: field_i64(entry, &["slotID", "slotId", "id", "vehicleId"]).unwrap_or(idx as i64),
            position: field_i64(entry, &["position", "place", "pos"]).unwrap_or(0) as i32,
            car_number: field_string(entry, &["carNumber", "number", "carNum"]),
            team: field_string(entry, &["fullTeamName", "team", "teamName"]),
            driver: field_string(entry, &["driverName", "driver", "name"]),
            class: field_string(entry, &["carClass", "class", "vehicleClass"]),
            car_model: field_string(entry, &["vehicleName", "carModel", "vehicle", "carType"]),
            class_position: field_i64(entry, &["classPosition", "picPosition", "pic"]).unwrap_or(0) as i32,
            laps: field_i64(entry, &["lapsCompleted", "laps", "totalLaps"]).unwrap_or(0) as i32,
            gap: field_string(entry, &["gap", "gapToLeader"]),
            last_lap_s: field_f64(entry, &["lastLapTime", "lastLap"]).unwrap_or(0.0),
            best_lap_s: field_f64(entry, &["bestLapTime", "bestLap"]).unwrap_or(0.0),
            sector1_s: field_f64(entry, &["sector1", "s1", "sector1Time"]).unwrap_or(0.0),
            sector2_s: field_f64(entry, &["sector2", "s2", "sector2Time"]).unwrap_or(0.0),
            sector3_s: field_f64(entry, &["sector3", "s3", "sector3Time"]).unwrap_or(0.0),
            top_speed_kmh: field_f64(entry, &["topSpeed", "vmax", "maxSpeed"]).unwrap_or(0.0),
            speed_kmh,
            in_pits: field_bool(entry, &["pitting", "inPits", "isInPit", "pit"]).unwrap_or(false),
            impact_et: 0.0,
            impact_mag: 0.0,
        });
    }
    Ok(out)
}

fn parse_session_info(raw: &Value) -> SessionInfo {
    SessionInfo {
        session_type: field_string(raw, &["sessionType", "session", "type"]),
        track_name: field_string(raw, &["trackName", "track"]),
        time_of_day: field_string(raw, &["timeOfDay", "tod"]),
        session_time_remaining_s: field_f64(raw, &["timeRemaining", "sessionTimeRemaining", "timeRemainingInGamePhase"]).unwrap_or(0.0),
        session_time_elapsed_s: field_f64(raw, &["currentEventTime", "sessionTime", "elapsed", "currentTime", "timeElapsed"]).unwrap_or(0.0),
        num_cars: field_i64(raw, &["numCars", "carCount", "numVehicles", "numberOfVehicles"]).unwrap_or(0) as i32,
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