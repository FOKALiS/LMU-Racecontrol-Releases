//! Heuristische Verdachtserkennung für die drei Marker-Farben im Fahrerfeld:
//!
//! - ROT   = Crash (Impact >3.0, Rundenzeit >30% zum eigenen Schnitt,
//!           Stillstand <10 km/h auf der Strecke)
//! - GELB  = Auffälligkeiten (Rundenzeit >15%, Positionsverlust, FCY-Verstoß)
//! - WEISS = Dauerhaft langsames Fahrzeug (>30s unter 50 km/h)
//!
//! Grundsatz: Die Erkennung ist KONSERVATIV (weniger ist mehr).
//! Ein Rennkommissar kann manuell Vorfälle anlegen. Die automatische
//! Erkennung soll nur OFFENSICHTLICHE Anomalien melden.
//!
//! FARB-CODIERUNG:
//! - ROT   = Crash (Kontakt, starker Rundenzeitverlust, Stillstand)
//! - GELB  = Abkommen, Track-Limits, Spin-Verdacht, Positionsverlust
//! - WEISS = Dauerhaft langsames Fahrzeug (>30s <50 km/h)
//!
//! Cooldown pro Fahrzeug (30 Sekunden) verhindert Überflutung.

use crate::db::{FlagColor, Incident};
use crate::lmu_client::CarStanding;
use chrono::Utc;
use std::collections::HashMap;
use uuid::Uuid;

// ─── ROT (Crash) ──────────────────────────────────────────────────────
const CRASH_LAP_FACTOR: f64 = 1.30; // >30% langsamer als eigener Schnitt -> ROT
const FIELD_CRASH_FACTOR: f64 = 1.45; // >45% langsamer als Feld-Median (ohne Baseline) -> ROT
const IMPACT_THRESHOLD: f64 = 3.0; // impactMag > 3.0 = Kontakt (ROT)
const NEAR_STOPPED_SPEED_KMH: f64 = 10.0; // Speed <= 10 km/h = Stillstand (ROT)
const MIN_STOPPED_SPEED_KMH: f64 = 0.5; // Speed unter 0.5 km/h = geparkt (kein Vorfall)

// ─── GELB (Auffälligkeit) ─────────────────────────────────────────────
const YELLOW_FACTOR: f64 = 1.15; // >15% langsamer -> GELB
const FIELD_YELLOW_FACTOR: f64 = 1.25; // >25% langsamer als Feld-Median -> GELB
const POSITION_LOSS_FOR_CRASH: i32 = 3; // ≥3 Positionen verloren -> GELB

// ─── WEISS (dauerhaft langsam) ────────────────────────────────────────
const SLOW_SPEED_KMH: f64 = 50.0; // unter 50 km/h = langsam
const SLOW_DURATION_SECONDS: f64 = 30.0; // länger als 30s = WEISS

// ─── Allgemein ────────────────────────────────────────────────────────
const MIN_SAMPLES_FOR_BASELINE: usize = 3; // 3 Runden nötig für eigene Baseline
const MAX_LAP_HISTORY: usize = 8;
const COOLDOWN_SECONDS: f64 = 30.0; // Keine neuen Vorfälle für denselben Slot innerhalb 30s

#[derive(Default, Clone)]
struct DriverHistory {
    recent_lap_times: Vec<f64>,
    last_position: Option<i32>,
    last_incident_time: f64,
    /// Zeitstempel, seit wann das Fahrzeug langsam fährt (<50 km/h, außer Box)
    slow_since: Option<f64>,
}

#[derive(Default)]
pub struct IncidentDetector {
    history: HashMap<i64, DriverHistory>,
}

pub struct DetectionContext<'a> {
    pub session_time_s: f64,
    pub track_name: &'a str,
}

impl IncidentDetector {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn analyze(&mut self, standings: &[CarStanding], ctx: &DetectionContext) -> Vec<Incident> {
        let mut suggestions = Vec::new();

        let field_median_lap = median(
            &standings
                .iter()
                .map(|c| c.last_lap_s)
                .filter(|&t| t > 0.0)
                .collect::<Vec<_>>(),
        );

        for car in standings {
            let hist = self.history.entry(car.slot_id).or_default();

            // ── Cooldown prüfen ──────────────────────────────────────────
            let on_cooldown = if hist.last_incident_time > 0.0 {
                (ctx.session_time_s - hist.last_incident_time) < COOLDOWN_SECONDS
            } else {
                false
            };

            // ── WEISS: Dauerhaft langsames Fahrzeug erkennen ─────────────
            // Läuft unabhängig vom Cooldown – kein neuer "Vorfall" sondern
            // eine Status-Anzeige. Aber wir wollen nicht jede Sekunde einen
            // neuen Eintrag. Daher: nur alle 30s einen neuen, wenn noch
            // langsam.
            let is_slow = car.speed_kmh < SLOW_SPEED_KMH && !car.in_pits && car.speed_kmh >= 0.0;
            let slow_detected = if is_slow {
                if hist.slow_since.is_none() {
                    hist.slow_since = Some(ctx.session_time_s);
                }
                // Prüfen ob >30s langsam
                if let Some(since) = hist.slow_since {
                    ctx.session_time_s - since >= SLOW_DURATION_SECONDS
                } else {
                    false
                }
            } else {
                // Nicht mehr langsam -> Timer zurücksetzen
                hist.slow_since = None;
                false
            };

            if slow_detected {
                // Wenn noch kein Cooldown oder Cooldown abgelaufen
                if !on_cooldown {
                    suggestions.push(make_incident(
                        ctx,
                        car,
                        FlagColor::White,
                        format!(
                            "Fahrzeug dauerhaft langsam ({:.0} km/h) – weiße Flagge prüfen.",
                            car.speed_kmh
                        ),
                    ));
                    hist.last_incident_time = ctx.session_time_s;
                }
                // Timer zurücksetzen, damit nicht jede Sekunde ein neuer kommt
                hist.slow_since = None;
            }

            // Wenn Cooldown aktiv, nur Historie aktualisieren und keine neuen
            // Rot/Gelb-Vorfälle
            if on_cooldown {
                update_history(hist, car, ctx);
                continue;
            }

            // ── ROT: Impact-Erkennung (Shared Memory) ────────────────────
            if car.impact_mag > IMPACT_THRESHOLD {
                suggestions.push(make_incident(
                    ctx,
                    car,
                    FlagColor::Red,
                    format!(
                        "Kontakt erkannt (Impact: {:.1}g) – Crash-Video prüfen.",
                        car.impact_mag
                    ),
                ));
                hist.last_incident_time = ctx.session_time_s;
            }

            // ── ROT: Rundenzeit >30% (Crash) ─────────────────────────────
            if car.last_lap_s > 0.0 {
                if hist.recent_lap_times.len() >= MIN_SAMPLES_FOR_BASELINE {
                    let own_median = median(&hist.recent_lap_times);
                    if own_median > 0.0 && car.last_lap_s > own_median * CRASH_LAP_FACTOR {
                        suggestions.push(make_incident(
                            ctx,
                            car,
                            FlagColor::Red,
                            format!(
                                "Crash-Verdacht: {:.1}s vs. Ø {:.1}s (>30%) – Replay prüfen.",
                                car.last_lap_s, own_median
                            ),
                        ));
                        hist.last_incident_time = ctx.session_time_s;
                    }
                } else if field_median_lap > 0.0
                    && car.last_lap_s > field_median_lap * FIELD_CRASH_FACTOR
                {
                    suggestions.push(make_incident(
                        ctx,
                        car,
                        FlagColor::Red,
                        format!(
                            "Crash-Verdacht: {:.1}s (Feld-Median: {:.1}s) – Replay prüfen.",
                            car.last_lap_s, field_median_lap
                        ),
                    ));
                    hist.last_incident_time = ctx.session_time_s;
                }
            }

            // ── ROT: Stillstand auf der Strecke ──────────────────────────
            // Nur Fahrzeuge zwischen 0.5 und 10 km/h auf der Strecke.
            // Exakt 0.0 km/h = geparkt/in der Garage (in_pits wird von der
            // LMU-API nicht immer korrekt gemeldet, siehe Pipo Derani).
            if car.speed_kmh > MIN_STOPPED_SPEED_KMH && car.speed_kmh < NEAR_STOPPED_SPEED_KMH && !car.in_pits {
                suggestions.push(make_incident(
                    ctx,
                    car,
                    FlagColor::Red,
                    format!(
                        "Fahrzeug steht auf der Strecke ({:.0} km/h) – Crash/Spin prüfen.",
                        car.speed_kmh
                    ),
                ));
                hist.last_incident_time = ctx.session_time_s;
            }

            // ── GELB: Rundenzeit >15% (Auffälligkeit) ────────────────────
            if car.last_lap_s > 0.0 {
                if hist.recent_lap_times.len() >= MIN_SAMPLES_FOR_BASELINE {
                    let own_median = median(&hist.recent_lap_times);
                    if own_median > 0.0 && car.last_lap_s > own_median * YELLOW_FACTOR {
                        suggestions.push(make_incident(
                            ctx,
                            car,
                            FlagColor::Yellow,
                            format!(
                                "Auffälliger Rundenzeitverlust: {:.1}s vs. Ø {:.1}s – Replay prüfen.",
                                car.last_lap_s, own_median
                            ),
                        ));
                        hist.last_incident_time = ctx.session_time_s;
                    }
                } else if field_median_lap > 0.0
                    && car.last_lap_s > field_median_lap * FIELD_YELLOW_FACTOR
                {
                    suggestions.push(make_incident(
                        ctx,
                        car,
                        FlagColor::Yellow,
                        format!(
                            "Auffällige Rundenzeit: {:.1}s (Feld-Median: {:.1}s) – Replay prüfen.",
                            car.last_lap_s, field_median_lap
                        ),
                    ));
                    hist.last_incident_time = ctx.session_time_s;
                }
            }

            // ── GELB: Positionsverlust ohne Boxenstopp ───────────────────
            if let Some(last_pos) = hist.last_position {
                if !car.in_pits && car.position - last_pos >= POSITION_LOSS_FOR_CRASH {
                    suggestions.push(make_incident(
                        ctx,
                        car,
                        FlagColor::Yellow,
                        format!(
                            "{} Positionen verloren (P{} -> P{}) – möglicher Vorfall.",
                            car.position - last_pos,
                            last_pos,
                            car.position
                        ),
                    ));
                    hist.last_incident_time = ctx.session_time_s;
                }
            }

            // ── Historie aktualisieren ──────────────────────────────────
            update_history(hist, car, ctx);
        }

        suggestions
    }
}

/// Aktualisiert die Fahrer-Historie (Rundenzeiten, Position)
fn update_history(hist: &mut DriverHistory, car: &CarStanding, _ctx: &DetectionContext) {
    if car.last_lap_s > 0.0 {
        hist.recent_lap_times.push(car.last_lap_s);
        if hist.recent_lap_times.len() > MAX_LAP_HISTORY {
            hist.recent_lap_times.remove(0);
        }
    }
    hist.last_position = Some(car.position);
}

fn make_incident(
    ctx: &DetectionContext,
    car: &CarStanding,
    flag: FlagColor,
    description: String,
) -> Incident {
    Incident {
        id: Uuid::new_v4().to_string(),
        incident_number: 0, // wird beim Insert von der DB vergeben
        created_at: Utc::now(),
        decided_at: None,
        session_time_s: ctx.session_time_s,
        lap: car.laps,
        corner: String::new(),
        timestamp_label: format_timestamp(ctx.session_time_s),
        track_name: ctx.track_name.to_string(),
        class_a: car.class.clone(),
        car_number_a: car.car_number.clone(),
        driver_a: car.driver.clone(),
        class_b: String::new(),
        car_number_b: String::new(),
        driver_b: String::new(),
        flag_color: flag,
        slot_id_a: Some(car.slot_id),
        incident_type: description,
        decision: None,
        reasoning: String::new(),
        penalty_points: 0,
        warning_points: 0,
        archived: false,
    }
}

/// Erstellt einen Vorfall für einen FCY-Geschwindigkeitsverstoß.
pub fn make_fcy_violation(
    ctx: &DetectionContext,
    car: &CarStanding,
    limit_kmh: f64,
) -> Incident {
    make_incident(
        ctx,
        car,
        FlagColor::Yellow,
        format!(
            "FCY-Verstoß: {:.0} km/h bei Limit {:.0} km/h.",
            car.speed_kmh, limit_kmh
        ),
    )
}

pub fn format_timestamp(seconds: f64) -> String {
    if seconds <= 0.0 {
        return "0:00.000".to_string();
    }
    let m = (seconds / 60.0).floor() as i64;
    let s = seconds % 60.0;
    format!("{}:{:06.3}", m, s)
}

fn median(values: &[f64]) -> f64 {
    if values.is_empty() {
        return 0.0;
    }
    let mut sorted = values.to_vec();
    sorted.sort_by(|a, b| a.partial_cmp(b).unwrap());
    let mid = sorted.len() / 2;
    if sorted.len() % 2 == 0 {
        (sorted[mid - 1] + sorted[mid]) / 2.0
    } else {
        sorted[mid]
    }
}

/// Erstellt einen manuell durch den Kommissar direkt entschiedenen Vorfall
/// (Button "Neuer Vorfall" bzw. leeres Investigation-Formular).
pub fn create_blank_incident(track_name: &str, session_time_s: f64, lap: i32) -> Incident {
    Incident {
        id: Uuid::new_v4().to_string(),
        incident_number: 0,
        created_at: Utc::now(),
        decided_at: None,
        session_time_s,
        lap,
        corner: String::new(),
        timestamp_label: format_timestamp(session_time_s),
        track_name: track_name.to_string(),
        class_a: String::new(),
        car_number_a: String::new(),
        driver_a: String::new(),
        class_b: String::new(),
        car_number_b: String::new(),
        driver_b: String::new(),
        flag_color: FlagColor::None,
        slot_id_a: None,
        incident_type: String::new(),
        decision: None,
        reasoning: String::new(),
        penalty_points: 0,
        warning_points: 0,
        archived: false,
    }
}
