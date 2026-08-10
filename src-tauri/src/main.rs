#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod db;
mod discord;
mod fcy;
mod incidents;
mod keyboard;
mod keyboard_config;
mod license;
mod lmu_client;
mod lmu_ws;
mod manufacturer;
mod server_client;
mod settings;
mod shared_memory;

use db::{Db, Incident};
use fcy::{FcyPhase, FcyState};
use incidents::{DetectionContext, IncidentDetector};
use license::{LicenseData, LicenseStore};
use lmu_client::{CarStanding, LmuClient, SessionInfo};
use serde::Serialize;
use server_client::ServerClient;
use settings::{Settings, SettingsStore};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use serde::Deserialize;
use tauri::{Emitter, Manager, State};
use tokio::sync::Mutex as AsyncMutex;
use tokio::time::{sleep, Duration};

struct AppState {
    db: Arc<Db>,
    lmu: Arc<LmuClient>,
    lmu_ws: Arc<lmu_ws::LmuWebSocket>,
    detector: Arc<AsyncMutex<IncidentDetector>>,
    fcy: Arc<FcyState>,
    settings_store: Arc<SettingsStore>,
    settings: Arc<AsyncMutex<Settings>>,
    should_poll: Arc<AtomicBool>,
    license_store: Arc<LicenseStore>,
    license: Arc<AsyncMutex<LicenseData>>,
    /// Cancel-Token für den Replay-Stop-Timer.
    /// Wird auf `true` gesetzt, wenn `switch_to_live` aufgerufen wird,
    /// damit der Timer kein F6 mehr sendet.
    replay_cancel_token: Arc<AtomicBool>,
    server_client: Arc<ServerClient>,
}

#[derive(Serialize, Clone)]
struct ConnectionStatusEvent {
    connected: bool,
}
#[derive(Serialize, Clone)]
struct StandingsEvent {
    standings: Vec<CarStanding>,
    session: SessionInfo,
    session_time_s: f64,
}
#[derive(Serialize, Clone)]
struct NewIncidentEvent {
    incident: Incident,
}
#[derive(Serialize, Clone)]
struct FcyCountdownEvent {
    remaining: i32,
}
#[derive(Serialize, Clone)]
struct FcyPhaseEvent {
    phase: FcyPhase,
}

// ---------- Commands ----------

#[tauri::command]
async fn connect_to_server(state: State<'_, AppState>) -> Result<bool, String> {
    let available = state.lmu.is_available().await;
    if available {
        state.should_poll.store(true, Ordering::SeqCst);
        // WebSocket-Verbindung starten – wichtig für die Replay-API!
        state.lmu_ws.start().await.map_err(|e| e.to_string())?;
    }
    Ok(available)
}

#[tauri::command]
async fn disconnect_from_server(state: State<'_, AppState>) -> Result<(), String> {
    state.should_poll.store(false, Ordering::SeqCst);
    state.lmu_ws.stop().await;
    Ok(())
}

#[tauri::command]
async fn get_settings(state: State<'_, AppState>) -> Result<Settings, String> {
    Ok(state.settings.lock().await.clone())
}

#[derive(Serialize, Clone)]
struct LicenseStatusResponse {
    licensed: bool,
    #[serde(flatten)]
    data: LicenseData,
}

#[tauri::command]
async fn get_license_status(state: State<'_, AppState>) -> Result<LicenseStatusResponse, String> {
    let data = state.license.lock().await.clone();
    let licensed = data.is_currently_licensed();
    Ok(LicenseStatusResponse { licensed, data })
}

#[tauri::command]
async fn deactivate_license(state: State<'_, AppState>) -> Result<LicenseStatusResponse, String> {
    let data = {
        let d = state.license.lock().await;
        d.clone()
    };
    if !data.has_key() {
        return Err("Keine aktive Lizenz zum Deaktivieren gefunden.".to_string());
    }
    
    // Bei Keygen die Maschine deregistrieren
    match license::deactivate_machine(&data.license_key, &data.license_id, &data.fingerprint).await {
        Ok(_) => {
            // Lizenz-Daten zurücksetzen
            let empty = LicenseData::default();
            state.license_store.save(&empty).map_err(|e| e.to_string())?;
            *state.license.lock().await = empty.clone();
            println!("[LICENSE] ✅ Lizenz erfolgreich deaktiviert");
            Ok(LicenseStatusResponse { licensed: false, data: empty })
        }
        Err(e) => {
            // Auch bei Fehler: Lizenz zurücksetzen (damit User sie neu aktivieren kann)
            let empty = LicenseData::default();
            state.license_store.save(&empty).map_err(|e| e.to_string())?;
            *state.license.lock().await = empty.clone();
            println!("[LICENSE] ⚠️ Deaktivierung fehlgeschlagen, aber Lizenz zurückgesetzt: {}", e);
            Ok(LicenseStatusResponse { licensed: false, data: empty })
        }
    }
}

#[tauri::command]
async fn activate_license(state: State<'_, AppState>, license_key: String) -> Result<LicenseStatusResponse, String> {
    let existing_fingerprint = {
        let data = state.license.lock().await;
        if data.fingerprint.is_empty() { None } else { Some(data.fingerprint.clone()) }
    };
    let device_name = hostname_label();
    let data = license::activate(&license_key, &device_name, existing_fingerprint)
        .await
        .map_err(|e| e.to_string())?;
    state.license_store.save(&data).map_err(|e| e.to_string())?;
    *state.license.lock().await = data.clone();
    let licensed = data.is_currently_licensed();
    Ok(LicenseStatusResponse { licensed, data })
}

fn hostname_label() -> String {
    std::env::var("COMPUTERNAME")
        .or_else(|_| std::env::var("HOSTNAME"))
        .unwrap_or_else(|_| "LMU-Racecontrol-Installation".to_string())
}

#[tauri::command]
async fn save_settings(state: State<'_, AppState>, settings: Settings) -> Result<(), String> {
    state
        .settings_store
        .save(&settings)
        .map_err(|e| e.to_string())?;
    *state.settings.lock().await = settings;
    Ok(())
}

#[tauri::command]
async fn list_pending_incidents(state: State<'_, AppState>) -> Result<Vec<Incident>, String> {
    state.db.list_pending().map_err(|e| e.to_string())
}

#[tauri::command]
async fn list_archived_incidents(state: State<'_, AppState>) -> Result<Vec<Incident>, String> {
    state.db.list_archived().map_err(|e| e.to_string())
}

/// Löscht ALLE lokalen Vorfälle (offene + archivierte).
/// Löscht auch die Server-Vorfälle des eigenen Tenants (nur das eigene Team).
/// Wird über den Button "Datenbank leeren" in den Einstellungen aufgerufen.
#[tauri::command]
async fn clear_all_incidents(state: State<'_, AppState>) -> Result<(), String> {
    state.db.clear_all().map_err(|e| e.to_string())?;

    // Auch Server-Vorfälle des eigenen Tenants löschen
    let settings = state.settings.lock().await;
    let server_url = settings.server_url.clone();
    let api_key = settings.api_key.clone();
    drop(settings);

    if !server_url.is_empty() && !api_key.is_empty() {
        match state.server_client.delete_all_incidents(&server_url, &api_key).await {
            Ok(count) => println!("[clear_all] {} Vorfälle des eigenen Tenants auf dem Server gelöscht", count),
            Err(e) => eprintln!("[clear_all] Server-Löschung fehlgeschlagen: {}", e),
        }
    }

    Ok(())
}

#[allow(clippy::too_many_arguments)]
#[tauri::command]
async fn submit_incident_decision(
    state: State<'_, AppState>,
    id: Option<String>,
    class_a: String,
    car_number_a: String,
    driver_a: String,
    class_b: String,
    car_number_b: String,
    driver_b: String,
    lap: i32,
    corner: String,
    timestamp_label: String,
    track_name: String,
    incident_type: String,
    decision: String,
    reasoning: String,
    penalty_points: Option<i32>,
    warning_points: Option<i32>,
) -> Result<Incident, String> {
    let target_id = match id {
        Some(existing) if !existing.is_empty() => existing,
        _ => {
            let mut blank = incidents::create_blank_incident(&track_name, 0.0, lap);
            let new_id = blank.id.clone();
            state.db.insert(&mut blank).map_err(|e| e.to_string())?;
            new_id
        }
    };

    let p_pts = penalty_points.unwrap_or(0);
    let w_pts = warning_points.unwrap_or(0);
    let updated = state
        .db
        .decide(
            &target_id,
            &class_a,
            &car_number_a,
            &driver_a,
            &class_b,
            &car_number_b,
            &driver_b,
            lap,
            &corner,
            &timestamp_label,
            &incident_type,
            &decision,
            &reasoning,
            p_pts,
            w_pts,
        )
        .map_err(|e| e.to_string())?;

    let webhook_url = state.settings.lock().await.discord_webhook_url.clone();
    if let Err(e) = discord::send_incident_decision(&webhook_url, &updated).await {
        eprintln!("Discord-Webhook fehlgeschlagen: {e:#}");
    }

    Ok(updated)
}

/// Springt zur Replay-Zeit eines Vorfalls.
/// KOMBINIERTE STRATEGIE (Tastatur + REST, wie BCUK):
///   Die REST-API (replayCommand/replay) kann den Replay-Modus NICHT
///   aus dem Watch-Modus aktivieren. Dafür ist die R-Taste (Tastatur) nötig.
///   Sobald der Replay-Modus aktiv ist, funktionieren REST-Befehle zuverlässig.
///
/// Ablauf:
///   1) WebSocket sicherstellen (nötig für REST-API)
///   2) R-Taste (Tastatur) – Replay-Modus aktivieren
///   3) PreArmReplay (REST) – Replay vorbereiten
///   4) seek_replay_to (REST) – zur Ziel-Position springen
///   5) VCRCOMMAND_PLAY (REST) – Play starten
///   6) seek_replay_to wiederholen (REST) – falls Play auf 0:00 gesetzt hat
///
/// Stop: F6 per Tastatur nach pre_roll + post_roll Sekunden
/// Der Stop-Timer wird via replay_cancel_token gecancelt,
/// wenn der User über switch_to_live auf "Live" schaltet.
#[tauri::command]
async fn jump_to_incident_replay(
    state: State<'_, AppState>,
    session_time_s: f64,
    pre_roll_seconds: f64,
    car_number: String,
    driver_name: Option<String>,
) -> Result<(), String> {
    let target = (session_time_s - pre_roll_seconds).max(0.0);
    
    println!("[replay] ===== START Replay jump zu {:.1}s (session_time={:.1}s, pre_roll={:.1}s, car=#{}, driver={:?}) =====", target, session_time_s, pre_roll_seconds, car_number, driver_name);
    
    // 0) WebSocket-Verbindung sicherstellen (nötig damit REST-API funktioniert)
    println!("[replay] (0/6) WebSocket sicherstellen...");
    state.lmu_ws.start().await.map_err(|e| e.to_string())?;
    tokio::time::sleep(std::time::Duration::from_millis(200)).await;
    println!("[replay] (0/6) WebSocket verbunden");
    
    // 1) Replay-Modus mit R-Taste aktivieren (Tastatur – REST allein kann das nicht!)
    println!("[replay] (1/6) R-Taste (Tastatur) – Replay-Modus aktivieren...");
    keyboard::replay_activate()?;
    // Wartezeit drastisch reduziert: R-Taste aktiviert Replay sofort (500ms reichen)
    println!("[replay] (1/6) R-Taste gesendet, warte 800ms auf Replay-Modus...");
    tokio::time::sleep(std::time::Duration::from_millis(800)).await;
    println!("[replay] (1/6) Replay-Modus sollte aktiv sein");
    
    // 2) PreArmReplay – Replay vorbereiten (BCUK-Kommandos) – parallel zum Warten ausführen
    println!("[replay] (2/6) PreArmReplay (REST)...");
    let _ = state.lmu.pre_arm_replay().await; // Fehler ignorieren – funktioniert nicht immer
    tokio::time::sleep(std::time::Duration::from_millis(150)).await;
    println!("[replay] (2/6) PreArmReplay gesendet");
    
    // 3) Zeitsprung zur Ziel-Position
    println!("[replay] (3/6) seek zu {:.1}s ...", target);
    state.lmu.seek_replay_to(target).await.map_err(|e| e.to_string())?;
    tokio::time::sleep(std::time::Duration::from_millis(150)).await;
    println!("[replay] (3/6) seek OK auf {:.1}s", target);
    
    // 4) Play starten – VCRCOMMAND_PLAY setzt NICHT auf 0:00 zurück
    println!("[replay] (4/6) VCRCOMMAND_PLAY (REST)...");
    state.lmu.replay_play().await.map_err(|e| e.to_string())?;
    tokio::time::sleep(std::time::Duration::from_millis(150)).await;
    println!("[replay] (4/6) VCRCOMMAND_PLAY OK");
    
    // 5) Zeitsprung wiederholen (Failsafe – falls Play doch zurückgesetzt hat)
    println!("[replay] (5/6) seek wiederholen auf {:.1}s ...", target);
    state.lmu.seek_replay_to(target).await.map_err(|e| e.to_string())?;
    tokio::time::sleep(std::time::Duration::from_millis(150)).await;
    println!("[replay] (5/6) seek wiederholt OK auf {:.1}s", target);
    
    // 6) Fahrer-Fokus direkt im Rust-Code setzen (NACH letztem seek)
    //    Strategie: Erst car_number, bei Mehrdeutigkeit Fahrername, Fallback auf ersten Slot.
    println!("[replay] (6/6) Fahrer-Fokus #{} ({:?}) (aus Rust, nach letztem seek)...", car_number, driver_name);
    if let Ok(standings) = state.lmu.get_standings().await {
        let matching: Vec<_> = standings.iter().filter(|c| c.car_number == car_number).collect();
        let slot = if matching.len() <= 1 {
            matching.first().map(|c| c.slot_id)
        } else if let Some(ref name) = driver_name {
            matching.iter().find(|c| c.driver == *name).map(|c| c.slot_id)
                .or_else(|| matching.first().map(|c| c.slot_id))
        } else {
            matching.first().map(|c| c.slot_id)
        };
        if let Some(slot_id) = slot {
            println!("[replay] (6/6) Fokussiere Slot {}", slot_id);
            let _ = state.lmu.focus_slot(slot_id).await;
            println!("[replay] (6/6) Fokus auf Slot {} gesendet", slot_id);
        } else {
            println!("[replay] (6/6) ⚠️ Slot für #{} ({:?}) nicht in Standings gefunden", car_number, driver_name);
        }
    } else {
        println!("[replay] (6/6) ⚠️ Konnte Standings nicht laden für Fokus");
    }
    
    println!("[replay] ===== Replay-Setup ABGESCHLOSSEN =====");
    
    // KEINE Kamera setzen – der Rennkommissar behält seine gewählte Kamera!
    
    // Timer: Replay automatisch stoppen nach pre_roll + post_roll Sekunden
    // Das replay_cancel_token aus dem State wird überwacht – switch_to_live setzt es auf true
    let pre_roll = state.settings.lock().await.pre_roll_seconds;
    let post_roll = state.settings.lock().await.post_roll_seconds;
    let total_play_seconds = pre_roll + post_roll;
    let timer_cancel = state.replay_cancel_token.clone();
    // Vorheriges Cancel-Token zurücksetzen (falls vom vorherigen Replay noch true)
    timer_cancel.store(false, Ordering::SeqCst);
    println!("[replay] Timer: {:.0}s + {:.0}s = {:.0}s total", pre_roll, post_roll, total_play_seconds);
    tokio::spawn(async move {
        tokio::time::sleep(std::time::Duration::from_secs_f64(total_play_seconds)).await;
        if timer_cancel.load(std::sync::atomic::Ordering::SeqCst) {
            println!("[replay] Timer gecancelt (switch_to_live) – kein Stop");
            return;
        }
        println!("[replay] Replay-Zeit vorbei – sende Pause (F11)...");
        let _ = keyboard::replay_pause();
    });
    
    Ok(())
}

#[tauri::command]
async fn zoom_start(direction: String, _state: State<'_, AppState>) -> Result<(), String> {
    println!("[zoom_start] 🔄 Starte Dauer-Zoom {}...", direction);
    keyboard::zoom_start(&direction)?;
    println!("[zoom_start] ✅ Dauer-Zoom {} gestartet", direction);
    Ok(())
}

#[tauri::command]
async fn zoom_stop(_state: State<'_, AppState>) -> Result<(), String> {
    println!("[zoom_stop] 🔄 Stoppe Dauer-Zoom...");
    keyboard::zoom_stop();
    println!("[zoom_stop] ✅ Dauer-Zoom gestoppt");
    Ok(())
}

#[tauri::command]
async fn replay_slow(_state: State<'_, AppState>) -> Result<(), String> {
    println!("[replay_slow] 🔄 Replay Slow-Motion (F10)...");
    keyboard::replay_slow()
}

#[tauri::command]
async fn replay_forward(_state: State<'_, AppState>) -> Result<(), String> {
    println!("[replay_forward] 🔄 Replay Vorspulen (F9)...");
    keyboard::replay_forward()
}

#[tauri::command]
async fn rewind_fast(_state: State<'_, AppState>) -> Result<(), String> {
    println!("[rewind_fast] 🔄 Replay schnell Zurück (F8)...");
    keyboard::rewind_fast()
}

#[tauri::command]
async fn replay_reverse(_state: State<'_, AppState>) -> Result<(), String> {
    println!("[replay_reverse] 🔄 Replay Rückwärts (F7)...");
    keyboard::replay_reverse()
}

#[tauri::command]
async fn replay_activate(_state: State<'_, AppState>) -> Result<(), String> {
    println!("[replay_activate] 🔄 Replay aktivieren (R-Taste)...");
    keyboard::replay_activate()
}

#[tauri::command]
async fn replay_pause(_state: State<'_, AppState>) -> Result<(), String> {
    println!("[replay_pause] 🔄 Play/Pause (F11)...");
    keyboard::replay_pause()
}

#[tauri::command]
async fn hold_stop(_state: State<'_, AppState>) -> Result<(), String> {
    println!("[hold_stop] 🔄 Hold-Key stoppen...");
    keyboard::hold_stop();
    keyboard::zoom_stop();
    Ok(())
}

#[tauri::command]
async fn set_camera(cam_id: String, state: State<'_, AppState>) -> Result<(), String> {
    println!("[set_camera] 🔄 Setze Kamera {} via Tastatur-Simulation...", cam_id);
    
    // CameraController (REST-API) funktioniert NICHT zuverlässig.
    // Daher verwenden wir direkt die Tastatur-Simulation (Scancodes via SendInput).
    // Die App holt LMU kurz in den Vordergrund, sendet den Scancode 1x und geht zurück.
    keyboard::switch_camera(&cam_id)?;
    
    println!("[set_camera] ✅ Tastatur-Simulation für {} gesendet", cam_id);
    Ok(())
}

/// Fokussiert einen Fahrer.
/// WICHTIG: KEINE Kamera setzen! Das macht App.tsx separat nach dem Fokus.
/// Wenn `driver_name` angegeben ist, wird zusätzlich zur Startnummer
/// auch der Fahrername gematcht (eindeutig bei doppelten Startnummern).
#[tauri::command]
async fn focus_driver(
    car_number: String,
    driver_name: Option<String>,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let standings = state.lmu.get_standings().await.map_err(|e| e.to_string())?;
    
    // Strategie: Erst nach car_number suchen, nur bei Mehrdeutigkeit den Fahrernamen nutzen.
    // Bei Fahrerwechseln (Endurance) ist der Name möglicherweise anders – dann Fallback auf ersten Treffer.
    let matching: Vec<_> = standings
        .iter()
        .filter(|c| c.car_number == car_number)
        .collect();
    
    let slot = if matching.len() <= 1 {
        matching.first().map(|c| c.slot_id)
    } else if let Some(ref name) = driver_name {
        // Mehrere Slots mit gleicher Nummer → mit Fahrername filtern
        let by_name = matching.iter().find(|c| c.driver == *name).map(|c| c.slot_id);
        if by_name.is_some() {
            println!("[focus_driver] 🔍 Mehrere #{} – Fahrer '{}' gefunden", car_number, name);
        } else {
            println!("[focus_driver] ⚠️ Fahrer '{}' bei #{} nicht gefunden (Fahrerwechsel?) – nehme ersten Slot", name, car_number);
        }
        by_name.or_else(|| matching.first().map(|c| c.slot_id))
    } else {
        // Kein Fahrername angegeben → ersten Slot nehmen
        matching.first().map(|c| c.slot_id)
    };
    
    if let Some(slot_id) = slot {
        println!(
            "[focus_driver] 🔄 Fokussiere Slot {} (Fahrzeug #{}, Fahrer: {:?})",
            slot_id, car_number, driver_name
        );
        state.lmu.focus_slot(slot_id).await.map_err(|e| e.to_string())?;
        println!("[focus_driver] ✅ Fokus auf Slot {} gesendet", slot_id);
        Ok(())
    } else {
        println!(
            "[focus_driver] ⚠️ Slot für #{} (Fahrer: {:?}) nicht gefunden, Fallback auf Tastatur",
            car_number, driver_name
        );
        keyboard::focus_car(&car_number)?;
        Ok(())
    }
}

#[tauri::command]
async fn focus_slot(slot_id: i64, state: State<'_, AppState>) -> Result<(), String> {
    state.lmu.focus_slot(slot_id).await.map_err(|e| e.to_string())
}

#[tauri::command]
async fn clear_focus(state: State<'_, AppState>) -> Result<(), String> {
    state.lmu.clear_focus().await.map_err(|e| e.to_string())
}

#[tauri::command]
async fn switch_to_live(state: State<'_, AppState>) -> Result<(), String> {
    // Cancel-Token setzen, damit der Replay-Stop-Timer kein F11 mehr sendet
    state.replay_cancel_token.store(true, Ordering::SeqCst);
    println!("[switch_to_live] Replay-Timer gecancelt, wechsle zu Live...");
    
    // Trick: Replay ans Ende springen lassen – LMU schaltet dann automatisch zu Live!
    // Ein extrem großer Zeitwert (86400s = 24h) bringt das Replay sicher ans Ende.
    println!("[switch_to_live] Sende seek ans Ende (86400s)...");
    let _ = state.lmu.seek_replay_to(86400.0).await;
    
    // Kurze Pause, damit LMU den Sprung verarbeitet
    tokio::time::sleep(std::time::Duration::from_millis(500)).await;
    
    // Play senden, damit das Replay wieder läuft (und LMU dann auto zu Live schaltet)
    println!("[switch_to_live] Sende Play...");
    let _ = state.lmu.replay_play().await;
    
    println!("[switch_to_live] Zurück zu Live!");
    Ok(())
}

#[tauri::command]
async fn switch_to_replay(state: State<'_, AppState>) -> Result<(), String> {
    state.lmu.switch_to_replay().await.map_err(|e| e.to_string())
}

/// Gibt die aktuell geladene Tastenbelegung aus der LMU keyboard.json zurück.
/// Zeigt die relevanten Tasten (Kamera, Replay, Zoom) mit lesbaren Namen an.
#[tauri::command]
async fn get_keyboard_mapping() -> Result<Vec<keyboard::KeyboardMappingEntryFrontend>, String> {
    Ok(keyboard::get_relevant_bindings())
}

/// Lädt die Tastenbelegung aus der LMU keyboard.json neu.
/// Wird aufgerufen, nachdem der User den LMU-Pfad geändert hat.
#[tauri::command]
async fn reload_keyboard_mapping(state: State<'_, AppState>) -> Result<Vec<keyboard::KeyboardMappingEntryFrontend>, String> {
    let lmu_install_path = state.settings.lock().await.lmu_install_path.clone();
    let kb_config = keyboard_config::KeyboardConfig::load_from(&lmu_install_path);
    keyboard::init(kb_config);
    Ok(keyboard::get_relevant_bindings())
}

#[tauri::command]
async fn check_lmu_connection(state: State<'_, AppState>) -> Result<bool, String> {
    Ok(state.lmu.is_available().await)
}

/// Prüft, ob der konfigurierte Server erreichbar ist.
#[tauri::command]
async fn check_server_connection(state: State<'_, AppState>) -> Result<bool, String> {
    let settings = state.settings.lock().await;
    let server_url = settings.server_url.clone();
    if server_url.is_empty() {
        return Ok(false);
    }
    state.server_client.check_health(&server_url).await.map_err(|e| e.to_string())
}

/// Hilfsfunktion: Sendet einen Vorfall an den Server, falls konfiguriert.
/// Fehler werden nur geloggt, nicht zurückgegeben – der Sync ist optional.
async fn sync_incident_if_configured(state: &AppState, incident: &Incident) {
    let settings = state.settings.lock().await;
    let server_url = settings.server_url.clone();
    let api_key = settings.api_key.clone();
    drop(settings);

    if server_url.is_empty() || api_key.is_empty() {
        return; // Server nicht konfiguriert – kein Sync
    }

    let server_incident = server_client::ServerIncident {
        id: incident.id.clone(),
        incident_number: incident.incident_number,
        car_number_a: incident.car_number_a.clone(),
        car_number_b: Some(incident.car_number_b.clone()),
        flag_color: incident.flag_color.as_str().to_string(),
        incident_type: incident.incident_type.clone(),
        session_type: incident.track_name.clone(),
        lap_number: incident.lap as i64,
        timestamp: incident.timestamp_label.clone(),
    };

    match state.server_client.create_incident(&server_url, &api_key, &server_incident).await {
        Ok(_) => eprintln!("[sync] Incident {} an Server gesynct", incident.id),
        Err(e) => eprintln!("[sync] Incident-Sync fehlgeschlagen: {} (id={})", e, incident.id),
    }
}

/// Sendet einen lokalen Vorfall an den Server.
#[tauri::command]
async fn sync_incident_to_server(state: State<'_, AppState>, incident: Incident) -> Result<(), String> {
    sync_incident_if_configured(&state, &incident).await;
    Ok(())
}

/// Holt alle Vorfälle vom Server und gibt sie zurück.
#[tauri::command]
async fn fetch_incidents_from_server(state: State<'_, AppState>) -> Result<Vec<server_client::ServerIncident>, String> {
    let settings = state.settings.lock().await;
    let server_url = settings.server_url.clone();
    let api_key = settings.api_key.clone();
    
    if server_url.is_empty() || api_key.is_empty() {
        return Err("Server-URL oder API-Key nicht konfiguriert".to_string());
    }

    state.server_client.get_incidents(&server_url, &api_key).await
        .map_err(|e| e.to_string())
}

/// Ruft den API-Key anhand des License-Keys vom Server ab.
/// Wird verwendet, wenn der User in Einstellungen > Server auf "API-Key abfragen" klickt.
#[tauri::command]
async fn fetch_api_key_from_server(server_url: String, license_key: String) -> Result<String, String> {
    if server_url.is_empty() || license_key.is_empty() {
        return Err("Server-URL oder License-Key ist leer".to_string());
    }
    
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(10))
        .build()
        .map_err(|e| format!("HTTP-Client-Fehler: {}", e))?;
    
    let url = format!("{}/api/lookup-api-key", server_url.trim_end_matches('/'));
    let resp = client
        .post(&url)
        .json(&serde_json::json!({ "license_key": license_key }))
        .send()
        .await
        .map_err(|e| format!("Server nicht erreichbar: {}", e))?;
    
    let status = resp.status();
    let body: serde_json::Value = resp.json().await
        .map_err(|e| format!("Ungültige Antwort: {}", e))?;
    
    if status.is_success() {
        body.get("api_key")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string())
            .ok_or_else(|| "Kein API-Key in der Antwort".to_string())
    } else {
        let error = body.get("error").and_then(|v| v.as_str()).unwrap_or("Unbekannter Fehler");
        Err(format!("Server-Fehler ({}): {}", status.as_u16(), error))
    }
}

#[tauri::command]
async fn start_fcy(app: tauri::AppHandle, state: State<'_, AppState>) -> Result<(), String> {
    if state.fcy.current_phase() != FcyPhase::Idle {
        return Ok(());
    }
    let countdown_from = state.settings.lock().await.fcy_countdown_seconds;
    state.fcy.set_phase(FcyPhase::Countdown);

    let fcy = state.fcy.clone();
    tauri::async_runtime::spawn(async move {
        let mut remaining = countdown_from;
        while remaining >= 0 {
            let _ = app.emit("fcy-countdown", FcyCountdownEvent { remaining });
            if remaining == 0 {
                break;
            }
            sleep(Duration::from_secs(1)).await;
            remaining -= 1;
        }
        fcy.set_phase(FcyPhase::Active);
        let _ = app.emit("fcy-phase", FcyPhaseEvent { phase: FcyPhase::Active });
    });

    Ok(())
}

#[tauri::command]
async fn clear_fcy(app: tauri::AppHandle, state: State<'_, AppState>) -> Result<(), String> {
    state.fcy.set_phase(FcyPhase::Idle);
    let _ = app.emit("fcy-phase", FcyPhaseEvent { phase: FcyPhase::Idle });
    Ok(())
}

#[tauri::command]
async fn show_main_window(app: tauri::AppHandle) -> Result<(), String> {
    if let Some(main) = app.get_webview_window("main") {
        main.show().map_err(|e| e.to_string())?;
        main.set_focus().map_err(|e| e.to_string())?;
        main.maximize().map_err(|e| e.to_string())?;
    }
    if let Some(splash) = app.get_webview_window("splashscreen") {
        splash.close().map_err(|e| e.to_string())?;
    }
    Ok(())
}

// ---------- Hintergrund-Polling-Loop ----------

async fn poll_loop(app: tauri::AppHandle, state: Arc<AppState>) {
    let mut was_connected = false;
    let mut last_session_time_remaining: Option<f64> = None;
    let mut last_poll_time: Option<std::time::Instant> = None;

    loop {
        if !state.should_poll.load(Ordering::SeqCst) {
            if was_connected {
                was_connected = false;
                last_session_time_remaining = None;
                last_poll_time = None;
                let _ = app.emit("connection-status", ConnectionStatusEvent { connected: false });
            }
            sleep(Duration::from_millis(500)).await;
            continue;
        }

        let available = state.lmu.is_available().await;
        if available != was_connected {
            let _ = app.emit("connection-status", ConnectionStatusEvent { connected: available });
            was_connected = available;
            if available {
                last_poll_time = Some(std::time::Instant::now());
            }
        }

        if available {
            if let (Ok(mut standings), Ok(session)) =
                (state.lmu.get_standings().await, state.lmu.get_session_info().await)
            {
                // Impact-Daten aus Shared Memory auslesen und in Standings einfügen
                let impact_data = shared_memory::read_impact_data();
                for car in &mut standings {
                    if let Some(&(impact_et, impact_mag)) = impact_data.get(&car.slot_id) {
                        car.impact_et = impact_et;
                        car.impact_mag = impact_mag;
                    }
                }
                // Echte Session-Zeit aus der LMU-API berechnen:
                // 1. session_time_elapsed_s direkt aus der API (falls vorhanden)
                // 2. Fallback: session_time_remaining_s + vergangene Zeit seit letztem Poll
                // 3. Letzter Fallback: Echtzeit seit Verbindungsaufbau
                let session_time_s = if session.session_time_elapsed_s > 0.0 {
                    session.session_time_elapsed_s
                } else if session.session_time_remaining_s > 0.0 {
                    // Annahme: Session ist ca. 24h (86400s) – daraus elapsed berechnen
                    let total_estimate = 86400.0;
                    (total_estimate - session.session_time_remaining_s).max(0.0)
                } else {
                    // Fallback: Echtzeit seit Verbindungsaufbau
                    last_poll_time
                        .map(|t| t.elapsed().as_secs_f64())
                        .unwrap_or(0.0)
                };
                
                // Session-Zeit wird jetzt korrekt aus currentEventTime gelesen (lmu_client.rs)

                let fcy_active = state.fcy.current_phase() == FcyPhase::Active;
                if fcy_active {
                    let limit = state.settings.lock().await.fcy_speed_limit_kmh;
                    let tolerance = 3.0;
                    let threshold = limit + tolerance;
                    let ctx = DetectionContext {
                        session_time_s,
                        track_name: &session.track_name,
                    };
                    for car in &standings {
                        if car.speed_kmh > threshold && state.fcy.should_flag(car.slot_id) {
                            let mut incident = incidents::make_fcy_violation(&ctx, car, limit);
                            if state.db.insert(&mut incident).is_ok() {
                                sync_incident_if_configured(&state, &incident).await;
                                let _ = app.emit("new-incident", NewIncidentEvent { incident });
                            }
                        }
                    }
                }

                {
                    let mut detector = state.detector.lock().await;
                    let ctx = DetectionContext {
                        session_time_s,
                        track_name: &session.track_name,
                    };
                    for incident in detector.analyze(&standings, &ctx) {
                        let mut incident = incident;
                        if state.db.insert(&mut incident).is_ok() {
                            sync_incident_if_configured(&state, &incident).await;
                            let _ = app.emit("new-incident", NewIncidentEvent { incident });
                        }
                    }
                }

                let _ = app.emit(
                    "standings-update",
                    StandingsEvent { standings, session, session_time_s },
                );
            }
        }

        sleep(Duration::from_millis(1000)).await;
    }
}

async fn revalidate_license_on_startup(store: Arc<LicenseStore>, license: Arc<AsyncMutex<LicenseData>>) {
    let key = {
        let data = license.lock().await;
        if !data.has_key() {
            return;
        }
        data.clone()
    };

    match license::revalidate(&key.license_key, &key.fingerprint).await {
        Ok(_) => {
            let mut data = license.lock().await;
            data.valid = true;
            data.last_validated_at = Some(chrono::Utc::now());
            data.last_error = None;
            let _ = store.save(&data);
        }
        Err(e) => {
            let mut data = license.lock().await;
            data.last_error = Some(e.to_string());
            let _ = store.save(&data);
        }
    }
}

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_updater::Builder::new().build())
        .plugin(tauri_plugin_process::init())
        .setup(|app| {
            let app_dir = app
                .path()
                .app_data_dir()
                .expect("App-Datenverzeichnis konnte nicht ermittelt werden");
            std::fs::create_dir_all(&app_dir).ok();

            let db = Arc::new(
                Db::open(app_dir.join("incidents.sqlite3").to_str().unwrap())
                    .expect("DB-Init fehlgeschlagen"),
            );
            // Vorfälle älter als 26 Stunden automatisch entfernen,
            // damit die Datenbank nicht überläuft. Alles jüngere bleibt erhalten.
            if let Err(e) = db.purge_older_than(26) {
                eprintln!("[startup] ⚠️ Konnte alte Vorfälle (>26h) nicht aufräumen: {e}");
            }
            let lmu = Arc::new(LmuClient::new());
            let lmu_ws = Arc::new(lmu_ws::LmuWebSocket::new());
            let detector = Arc::new(AsyncMutex::new(IncidentDetector::new()));
            let fcy = Arc::new(FcyState::default());
            let settings_store = Arc::new(SettingsStore::new(&app_dir));
            let loaded_settings = settings_store.load();
            let lmu_install_path = loaded_settings.lmu_install_path.clone();
            let settings = Arc::new(AsyncMutex::new(loaded_settings));
            let should_poll = Arc::new(AtomicBool::new(false));
            let license_store = Arc::new(LicenseStore::new(&app_dir));
            let loaded_license = license_store.load();
            let license = Arc::new(AsyncMutex::new(loaded_license));

            // Tastenbelegung aus der LMU keyboard.json laden
            let kb_config = keyboard_config::KeyboardConfig::load_from(&lmu_install_path);
            keyboard::init(kb_config);

            let replay_cancel_token = Arc::new(AtomicBool::new(false));

            let server_client = Arc::new(ServerClient::new());

            let state = Arc::new(AppState {
                db: db.clone(),
                lmu: lmu.clone(),
                lmu_ws: lmu_ws.clone(),
                detector: detector.clone(),
                fcy: fcy.clone(),
                settings_store: settings_store.clone(),
                settings: settings.clone(),
                should_poll: should_poll.clone(),
                license_store: license_store.clone(),
                license: license.clone(),
                replay_cancel_token: replay_cancel_token.clone(),
                server_client: server_client.clone(),
            });

            app.manage(AppState {
                db,
                lmu,
                lmu_ws,
                detector,
                fcy,
                settings_store,
                settings,
                should_poll,
                license_store: license_store.clone(),
                license: license.clone(),
                replay_cancel_token,
                server_client,
            });

            let app_handle = app.handle().clone();
            tauri::async_runtime::spawn(poll_loop(app_handle, state));

            tauri::async_runtime::spawn(revalidate_license_on_startup(license_store, license));

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            connect_to_server,
            disconnect_from_server,
            get_settings,
            save_settings,
            get_license_status,
            activate_license,
            deactivate_license,
            list_pending_incidents,
            list_archived_incidents,
            clear_all_incidents,
            submit_incident_decision,
            jump_to_incident_replay,
            zoom_start,
            zoom_stop,
            set_camera,
            focus_driver,
            focus_slot,
            clear_focus,
            get_keyboard_mapping,
            reload_keyboard_mapping,
            check_lmu_connection,
            check_server_connection,
            sync_incident_to_server,
            fetch_incidents_from_server,
            fetch_api_key_from_server,
            switch_to_live,
            switch_to_replay,
            start_fcy,
            clear_fcy,
            show_main_window,
            replay_slow,
            replay_forward,
            rewind_fast,
            replay_reverse,
            replay_activate,
            replay_pause,
            hold_stop,
        ])
        .run(tauri::generate_context!())
        .expect("Fehler beim Starten von LMU Race Control");
}