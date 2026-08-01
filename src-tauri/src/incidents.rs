//! Heuristische Verdachtserkennung für die drei Marker-Farben im Fahrerfeld:
//!
//! - ROT   = möglicher Crash (starke, plötzliche Rundenzeit- oder
//!           Positions-Anomalie, Stillstand/nahe-0-Speed)
//! - GELB  = mögliche gelbe Flagge / Track-Limits / Ausritt (moderate
//!           Pace-Anomalie)
//! - WEISS = auffällig langsames Fahrzeug relativ zum Feld
//!
//! Grundsatz: Die Erkennung ist KONSERVATIV (weniger ist mehr).
//! Ein Rennkommissar kann manuell Vorfälle anlegen. Die automatische
//! Erkennung soll nur OFFENSICHTLICHE Anomalien melden.
//!
//! FARB-CODIERUNG (laut User-Vorgabe):
//! - ROT   = Kontakt zwischen 2 Fahrzeugen (automatisch via Shared Memory
//!           impactET/impactMag aus der VehicleTelemetry).
//! - GELB  = Alles andere: Abkommen, Track-Limits, Spin, Crash-Verdacht,
//!           Stillstand, Positionsverlust, Rundenzeit-Anomalie.
//! - WEISS = Auffällig langsame Fahrzeuge im Vergleich zum Feld.
//!
//! Automatisch erkannte Kontakte (impactMag > 5.0) werden ROT markiert.
//! Der Rennkommissar kann die Farbe im Investigation-Modal ändern.
//!
//! Cooldown pro Fahrzeug (30 Sekunden) verhindert Überflutung.

use crate::db::{FlagColor, Incident};
use crate::lmu_client::CarStanding;
use chrono::Utc;
use std::collections::HashMap;
use uuid::Uuid;

const CRASH_FACTOR: f64 = 1.35; // >35% langsamer als eigener Schnitt -> rot
const YELLOW_FACTOR: f64 = 1.15; // >15% langsamer -> gelb
const SLOW_FIELD_FACTOR: f64 = 1.10; // >10% langsamer als Feld-Median -> weiß (langsames Fahrzeug)
const MIN_SAMPLES_FOR_BASELINE: usize = 3; // 3 Runden nötig für Baseline
const MAX_LAP_HISTORY: usize = 8;
const POSITION_LOSS_FOR_CRASH: i32 = 3;

// Speed < 10 km/h auf der Strecke = Stillstand (Crash/Spin)
const NEAR_STOPPED_SPEED_KMH: f64 = 10.0;

// Cooldown: Keine neuen Vorfälle für denselben Slot innerhalb von 30 Sekunden
const COOLDOWN_SECONDS: f64 = 30.0;

#[derive(Default, Clone)]
struct DriverHistory {
    recent_lap_times: Vec<f64>,
    last_position: Option<i32>,
    last_incident_time: f64,
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

            // Cooldown prüfen: Überspringe, wenn vor <30s ein Vorfall gemeldet wurde
            let on_cooldown = if hist.last_incident_time > 0.0 {
                (ctx.session_time_s - hist.last_incident_time) < COOLDOWN_SECONDS
            } else {
                false
            };

            if on_cooldown {
                // Nur Historie aktualisieren, keine neuen Vorfälle
                update_history(hist, car, ctx);
                continue;
            }

            // ── Rundenzeit-basierte Erkennung (IMMER GELB) ─────────────
            // Rot = NUR manuell bei bestätigtem Kontakt. Automatisch
            // erkannte Anomalien sind immer Gelb (Abkommen, Spin, etc.)
            if car.last_lap_s > 0.0 && hist.recent_lap_times.len() >= MIN_SAMPLES_FOR_BASELINE {
                let own_median = median(&hist.recent_lap_times);
                if own_median > 0.0 {
                    if car.last_lap_s > own_median * CRASH_FACTOR {
                        suggestions.push(make_incident(
                            ctx,
                            car,
                            FlagColor::Yellow,
                            format!(
                                "Starker Rundenzeitverlust: {:.1}s vs. Ø {:.1}s – möglicher Crash/Abflug. Replay prüfen.",
                                car.last_lap_s, own_median
                            ),
                        ));
                        hist.last_incident_time = ctx.session_time_s;
                    } else if car.last_lap_s > own_median * YELLOW_FACTOR {
                        suggestions.push(make_incident(
                            ctx,
                            car,
                            FlagColor::Yellow,
                            format!(
                                "Auffälliger Rundenzeitverlust: {:.1}s vs. Ø {:.1}s – ggf. gelber Bereich betroffen.",
                                car.last_lap_s, own_median
                            ),
                        ));
                        hist.last_incident_time = ctx.session_time_s;
                    }
                }
            }

            // ── Stillstand-Erkennung (IMMER GELB) ─────────────────────
            // Ein Fahrzeug, das fast steht (<10 km/h) und NICHT in der Box ist.
            if car.speed_kmh < NEAR_STOPPED_SPEED_KMH && !car.in_pits && car.speed_kmh > 0.0 {
                suggestions.push(make_incident(
                    ctx,
                    car,
                    FlagColor::Yellow,
                    format!(
                        "Fahrzeug fast stehend ({:.0} km/h) auf der Strecke – möglicher Spin/Abflug.",
                        car.speed_kmh
                    ),
                ));
                hist.last_incident_time = ctx.session_time_s;
            }

            // ── Positionsverlust-Erkennung (IMMER GELB) ────────────────
            if let Some(last_pos) = hist.last_position {
                if !car.in_pits && car.position - last_pos >= POSITION_LOSS_FOR_CRASH {
                    suggestions.push(make_incident(
                        ctx,
                        car,
                        FlagColor::Yellow,
                        format!(
                            "{} Positionen verloren (P{} -> P{}) ohne Boxenstopp – möglicher Vorfall.",
                            car.position - last_pos,
                            last_pos,
                            car.position
                        ),
                    ));
                    hist.last_incident_time = ctx.session_time_s;
                }
            }

            // ── Langsam-Feld-Erkennung (weiße Flagge) ──────────────────
            if field_median_lap > 0.0
                && car.last_lap_s > field_median_lap * SLOW_FIELD_FACTOR
                && !car.in_pits
            {
                suggestions.push(make_incident(
                    ctx,
                    car,
                    FlagColor::White,
                    format!(
                        "Deutlich langsamer als das Feld ({:.1}s vs. Feld-Median {:.1}s) – ggf. weiße Flagge.",
                        car.last_lap_s, field_median_lap
                    ),
                ));
                hist.last_incident_time = ctx.session_time_s;
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
        archived: false,
    }
}

/// Erstellt einen Vorfall für einen FCY-Geschwindigkeitsverstoß.
/// Gelb, da es kein Kontakt ist (Rot = nur manuell).
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
        archived: false,
    }
}