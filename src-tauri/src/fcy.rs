//! Statusmaschine für "Full Course Yellow", ausgelöst über den FCY-Button:
//!
//! Idle --(start_fcy)--> Countdown (10,9,...,0) --> Active --(clear_fcy)--> Idle
//!
//! Während "Active" prüft der Polling-Loop in main.rs die Live-Geschwindigkeit
//! aller Fahrzeuge gegen das konfigurierte Limit und markiert Verstöße
//! automatisch als Vorfall (einmal pro Fahrzeug und FCY-Phase, damit nicht
//! bei jedem Tick derselbe Verstoß erneut gemeldet wird).

use serde::Serialize;
use std::collections::HashSet;
use std::sync::Mutex;

#[derive(Debug, Clone, Copy, Serialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum FcyPhase {
    Idle,
    Countdown,
    Active,
}

pub struct FcyState {
    pub phase: Mutex<FcyPhase>,
    pub already_flagged: Mutex<HashSet<i64>>,
}

impl Default for FcyState {
    fn default() -> Self {
        Self {
            phase: Mutex::new(FcyPhase::Idle),
            already_flagged: Mutex::new(HashSet::new()),
        }
    }
}

impl FcyState {
    pub fn current_phase(&self) -> FcyPhase {
        *self.phase.lock().unwrap()
    }

    pub fn set_phase(&self, phase: FcyPhase) {
        *self.phase.lock().unwrap() = phase;
        if phase == FcyPhase::Active {
            self.already_flagged.lock().unwrap().clear();
        }
    }

    pub fn should_flag(&self, slot_id: i64) -> bool {
        let mut flagged = self.already_flagged.lock().unwrap();
        if flagged.contains(&slot_id) {
            false
        } else {
            flagged.insert(slot_id);
            true
        }
    }
}