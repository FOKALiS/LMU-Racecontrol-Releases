//! Heuristische Verdachtserkennung für die drei Marker-Farben im Fahrerfeld:
//!
//! - ROT   = möglicher Crash (starke, plötzliche Rundenzeit- oder
//!           Positions-Anomalie)
//! - GELB  = mögliche gelbe Flagge / kleinere Auffälligkeit (moderate
//!           Pace-Anomalie)
//! - WEISS = auffällig langsames Fahrzeug relativ zum Feld
//!
//! Wie schon beim ersten Entwurf gilt: Es gibt keinen bestätigten REST-
//! Endpunkt mit echtem Flaggen-Status pro Fahrzeug. Sobald ihr ein
//! entsprechendes Feld in eurer `/rest/watch/standings`-Antwort findet,
//! sollte das hier direkt statt der Heuristik verwendet werden (deutlich
//! zuverlässiger als Pace-Schätzungen).

use crate::db::{FlagColor, Incident};
use crate::lmu_client::CarStanding;
use chrono::Utc;
use std::collections::HashMap;
use uuid::Uuid;

const CRASH_FACTOR: f64 = 1.35; // >35% langsamer als eigener Schnitt -> rot
const YELLOW_FACTOR: f64 = 1.15; // >15% langsamer -> gelb
const SLOW_FIELD_FACTOR: f64 = 1.10; // >10% langsamer als Feld-Median -> weiß (langsames Fahrzeug)
const MIN_SAMPLES_FOR_BASELINE: usize = 3;
const MAX_LAP_HISTORY: usize = 8;
const POSITION_LOSS_FOR_CRASH: i32 = 3;

#[derive(Default)]
struct DriverHistory {
    recent_lap_times: Vec<f64>,
    last_position: Option<i32>,
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

            if car.last_lap_s > 0.0 && hist.recent_lap_times.len() >= MIN_SAMPLES_FOR_BASELINE {
                let own_median = median(&hist.recent_lap_times);
                if own_median > 0.0 {
                    if car.last_lap_s > own_median * CRASH_FACTOR {
                        suggestions.push(make_incident(
                            ctx,
                            car,
                            FlagColor::Red,
                            format!(
                                "Starker Rundenzeitverlust: {:.1}s vs. Ø {:.1}s – möglicher Crash. Replay prüfen.",
                                car.last_lap_s, own_median
                            ),
                        ));
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
                    }
                }
            }

            if let Some(last_pos) = hist.last_position {
                if !car.in_pits && car.position - last_pos >= POSITION_LOSS_FOR_CRASH {
                    suggestions.push(make_incident(
                        ctx,
                        car,
                        FlagColor::Red,
                        format!(
                            "{} Positionen verloren (P{} -> P{}) ohne Boxenstopp – möglicher Vorfall.",
                            car.position - last_pos,
                            last_pos,
                            car.position
                        ),
                    ));
                }
            }

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
            }

            if car.last_lap_s > 0.0 {
                hist.recent_lap_times.push(car.last_lap_s);
                if hist.recent_lap_times.len() > MAX_LAP_HISTORY {
                    hist.recent_lap_times.remove(0);
                }
            }
            hist.last_position = Some(car.position);
        }

        suggestions
    }
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
pub fn make_fcy_violation(
    ctx: &DetectionContext,
    car: &CarStanding,
    limit_kmh: f64,
) -> Incident {
    make_incident(
        ctx,
        car,
        FlagColor::Red,
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
