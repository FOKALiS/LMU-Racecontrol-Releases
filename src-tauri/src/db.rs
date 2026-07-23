//! SQLite-Persistenz für Vorfälle, passend zum Investigation-Workflow:
//! ein Vorfall hat einen "verursachenden" und optional einen "geschädigten"
//! Fahrer, Runde/Kurve/Zeitstempel, eine Vorfall-Art und (nach Entscheidung)
//! eine Maßnahme + Begründung. Solange keine Entscheidung getroffen wurde,
//! taucht er unter "Vorfälle" auf; danach wandert er ins "Archiv".

use anyhow::Result;
use chrono::{DateTime, Utc};
use rusqlite::{params, Connection, OptionalExtension};
use serde::{Deserialize, Serialize};
use std::sync::Mutex;

/// Farbe des Status-Punkts in der Fahrerfeld-Tabelle.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum FlagColor {
    Red,    // Crash-Verdacht
    Yellow, // Gelbe Flagge (Bereich)
    White,  // Weiße Flagge / langsames Fahrzeug
    None,
}
impl FlagColor {
    fn as_str(&self) -> &'static str {
        match self {
            FlagColor::Red => "red",
            FlagColor::Yellow => "yellow",
            FlagColor::White => "white",
            FlagColor::None => "none",
        }
    }
    fn from_str(s: &str) -> Self {
        match s {
            "red" => FlagColor::Red,
            "yellow" => FlagColor::Yellow,
            "white" => FlagColor::White,
            _ => FlagColor::None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Incident {
    pub id: String,
    pub incident_number: i64,
    pub created_at: DateTime<Utc>,
    pub decided_at: Option<DateTime<Utc>>,

    pub session_time_s: f64,
    pub lap: i32,
    /// Frei editierbar durch den Kommissar (Kurvennummer laut Streckenplan)
    pub corner: String,
    /// Anzeige-Zeitstempel, z.B. "0:42.347" - manuell bestätigt/korrigiert
    pub timestamp_label: String,
    pub track_name: String,

    // Verursachender Fahrer
    pub class_a: String,
    pub car_number_a: String,
    pub driver_a: String,

    // Geschädigter Fahrer (optional - leer bei Einzel-Vorfällen wie
    // FCY-Verstoß oder Track-Limits)
    pub class_b: String,
    pub car_number_b: String,
    pub driver_b: String,

    pub flag_color: FlagColor,
    pub slot_id_a: Option<i64>,

    pub incident_type: String,
    pub decision: Option<String>,
    pub reasoning: String,

    pub archived: bool,
}

pub struct Db(Mutex<Connection>);

impl Db {
    pub fn open(path: &str) -> Result<Self> {
        let conn = Connection::open(path)?;
        conn.execute(
            "CREATE TABLE IF NOT EXISTS incidents (
                id                  TEXT PRIMARY KEY,
                incident_number     INTEGER NOT NULL,
                created_at          TEXT NOT NULL,
                decided_at          TEXT,
                session_time_s      REAL NOT NULL,
                lap                 INTEGER NOT NULL,
                corner              TEXT NOT NULL DEFAULT '',
                timestamp_label     TEXT NOT NULL DEFAULT '',
                track_name          TEXT NOT NULL DEFAULT '',
                class_a             TEXT NOT NULL DEFAULT '',
                car_number_a        TEXT NOT NULL DEFAULT '',
                driver_a            TEXT NOT NULL DEFAULT '',
                class_b             TEXT NOT NULL DEFAULT '',
                car_number_b        TEXT NOT NULL DEFAULT '',
                driver_b            TEXT NOT NULL DEFAULT '',
                flag_color          TEXT NOT NULL DEFAULT 'none',
                slot_id_a           INTEGER,
                incident_type       TEXT NOT NULL DEFAULT '',
                decision            TEXT,
                reasoning           TEXT NOT NULL DEFAULT '',
                archived            INTEGER NOT NULL DEFAULT 0
            )",
            [],
        )?;
        Ok(Self(Mutex::new(conn)))
    }

    fn next_incident_number(conn: &Connection) -> Result<i64> {
        let max: Option<i64> = conn
            .query_row("SELECT MAX(incident_number) FROM incidents", [], |r| r.get(0))
            .optional()?
            .flatten();
        Ok(max.unwrap_or(0) + 1)
    }

    /// Legt einen neuen Vorfall an (z.B. automatische Erkennung oder
    /// "Neuer Vorfall"-Button). Noch ohne Entscheidung -> taucht in
    /// "Vorfälle" auf.
    pub fn insert(&self, incident: &mut Incident) -> Result<()> {
        let conn = self.0.lock().unwrap();
        incident.incident_number = Self::next_incident_number(&conn)?;
        conn.execute(
            "INSERT INTO incidents (
                id, incident_number, created_at, decided_at, session_time_s, lap,
                corner, timestamp_label, track_name,
                class_a, car_number_a, driver_a,
                class_b, car_number_b, driver_b,
                flag_color, slot_id_a, incident_type, decision, reasoning, archived
            ) VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12,?13,?14,?15,?16,?17,?18,?19,?20,?21)",
            params![
                incident.id,
                incident.incident_number,
                incident.created_at.to_rfc3339(),
                incident.decided_at.map(|d| d.to_rfc3339()),
                incident.session_time_s,
                incident.lap,
                incident.corner,
                incident.timestamp_label,
                incident.track_name,
                incident.class_a,
                incident.car_number_a,
                incident.driver_a,
                incident.class_b,
                incident.car_number_b,
                incident.driver_b,
                incident.flag_color.as_str(),
                incident.slot_id_a,
                incident.incident_type,
                incident.decision,
                incident.reasoning,
                incident.archived as i32,
            ],
        )?;
        Ok(())
    }

    /// Trägt die Entscheidung der Kommission ein und verschiebt den Vorfall
    /// damit automatisch ins Archiv (archived = true).
    pub fn decide(
        &self,
        id: &str,
        class_a: &str,
        car_number_a: &str,
        driver_a: &str,
        class_b: &str,
        car_number_b: &str,
        driver_b: &str,
        lap: i32,
        corner: &str,
        timestamp_label: &str,
        incident_type: &str,
        decision: &str,
        reasoning: &str,
    ) -> Result<Incident> {
        let conn = self.0.lock().unwrap();
        conn.execute(
            "UPDATE incidents SET
                class_a=?1, car_number_a=?2, driver_a=?3,
                class_b=?4, car_number_b=?5, driver_b=?6,
                lap=?7, corner=?8, timestamp_label=?9,
                incident_type=?10, decision=?11, reasoning=?12,
                archived=1, decided_at=?13
             WHERE id=?14",
            params![
                class_a, car_number_a, driver_a,
                class_b, car_number_b, driver_b,
                lap, corner, timestamp_label,
                incident_type, decision, reasoning,
                Utc::now().to_rfc3339(),
                id,
            ],
        )?;
        drop(conn);
        self.get(id)?.ok_or_else(|| anyhow::anyhow!("Vorfall {id} nach Update nicht gefunden"))
    }

    pub fn get(&self, id: &str) -> Result<Option<Incident>> {
        let conn = self.0.lock().unwrap();
        conn.query_row(
            "SELECT * FROM incidents WHERE id = ?1",
            params![id],
            row_to_incident,
        )
        .optional()
        .map_err(Into::into)
    }

    pub fn list_pending(&self) -> Result<Vec<Incident>> {
        self.list_where("archived = 0")
    }

    pub fn list_archived(&self) -> Result<Vec<Incident>> {
        self.list_where("archived = 1")
    }

    fn list_where(&self, clause: &str) -> Result<Vec<Incident>> {
        let conn = self.0.lock().unwrap();
        let sql = format!("SELECT * FROM incidents WHERE {clause} ORDER BY incident_number DESC");
        let mut stmt = conn.prepare(&sql)?;
        let rows = stmt.query_map([], row_to_incident)?;
        Ok(rows.filter_map(|r| r.ok()).collect())
    }
}

fn row_to_incident(row: &rusqlite::Row) -> rusqlite::Result<Incident> {
    let created_at: String = row.get("created_at")?;
    let decided_at: Option<String> = row.get("decided_at")?;
    let flag_color: String = row.get("flag_color")?;
    Ok(Incident {
        id: row.get("id")?,
        incident_number: row.get("incident_number")?,
        created_at: DateTime::parse_from_rfc3339(&created_at)
            .map(|d| d.with_timezone(&Utc))
            .unwrap_or_else(|_| Utc::now()),
        decided_at: decided_at.and_then(|d| {
            DateTime::parse_from_rfc3339(&d).ok().map(|d| d.with_timezone(&Utc))
        }),
        session_time_s: row.get("session_time_s")?,
        lap: row.get("lap")?,
        corner: row.get("corner")?,
        timestamp_label: row.get("timestamp_label")?,
        track_name: row.get("track_name")?,
        class_a: row.get("class_a")?,
        car_number_a: row.get("car_number_a")?,
        driver_a: row.get("driver_a")?,
        class_b: row.get("class_b")?,
        car_number_b: row.get("car_number_b")?,
        driver_b: row.get("driver_b")?,
        flag_color: FlagColor::from_str(&flag_color),
        slot_id_a: row.get("slot_id_a")?,
        incident_type: row.get("incident_type")?,
        decision: row.get("decision")?,
        reasoning: row.get("reasoning")?,
        archived: row.get::<_, i32>("archived")? != 0,
    })
}
