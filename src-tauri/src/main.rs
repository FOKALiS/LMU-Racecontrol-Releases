#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod db;
mod discord;
mod fcy;
mod incidents;
mod keyboard;
mod license;
mod lmu_client;
mod settings;
mod shared_memory;

use db::{Db, Incident};
use fcy::{FcyPhase, FcyState};
use incidents::{DetectionContext, IncidentDetector};
use license::{LicenseData, LicenseStore};
use lmu_client::{CarStanding, LmuClient, SessionInfo};
use serde::Serialize;
use settings::{Settings, SettingsStore};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tauri::{Emitter, Manager, State};
use tokio::sync::Mutex as AsyncMutex;
use tokio::time::{sleep, Duration};

struct AppState {
    db: Arc<Db>,
    lmu: Arc<LmuClient>,
    detector: Arc<AsyncMutex<IncidentDetector>>,
    fcy: Arc<FcyState>,
    settings_store: Arc<SettingsStore>,
    settings: Arc<AsyncMutex<Settings>>,
    should_poll: Arc<AtomicBool>,
    license_store: Arc<LicenseStore>,
    license: Arc<AsyncMutex<LicenseData>>,
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
    }
    Ok(available)
}

#[tauri::command]
async fn disconnect_from_server(state: State<'_, AppState>) -> Result<(), String> {
    state.should_poll.store(false, Ordering::SeqCst);
    Ok(())
}

#[tauri::command]
async fn get_settings(state: State<'_, AppState>) -> Result<Settings, String> {
    Ok(state.settings.lock().await.clone())
}

/// Aktuellen Lizenzstatus liefern (wird beim App-Start abgefragt, um zu
/// entscheiden, ob die volle Oberfläche freigeschaltet wird).
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

/// Aktiviert einen neu eingegebenen Lizenzschlüssel für diese Installation.
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

/// Legt bei Bedarf einen neuen Vorfall an (falls `id` leer ist) und trägt
/// direkt die Entscheidung der Kommission ein -> Vorfall wandert damit
/// automatisch ins Archiv. Löst anschließend die Discord-Benachrichtigung aus.
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
        )
        .map_err(|e| e.to_string())?;

    let webhook_url = state.settings.lock().await.discord_webhook_url.clone();
    if let Err(e) = discord::send_incident_decision(&webhook_url, &updated).await {
        // Discord-Fehler soll das Archivieren nicht blockieren, aber sichtbar sein.
        eprintln!("Discord-Webhook fehlgeschlagen: {e:#}");
    }

    Ok(updated)
}

#[tauri::command]
async fn jump_to_incident_replay(
    state: State<'_, AppState>,
    session_time_s: f64,
    pre_roll_seconds: f64,
) -> Result<(), String> {
    let target = (session_time_s - pre_roll_seconds).max(0.0);
    state.lmu.seek_replay_to(target).await.map_err(|e| e.to_string())
}

#[tauri::command]
async fn set_camera(cam_id: String) -> Result<(), String> {
    // Shared Memory ist der zuverlässigere Weg (kein Fokus nötig)
    // Öffne Shared Memory NUR bei Bedarf - blockiert nicht beim Start
    if let Some(sm) = shared_memory::try_open() {
        if sm.set_camera(&cam_id).is_ok() {
            return Ok(());
        }
    }
    // Fallback: Tastatursimulation (funktioniert immer, braucht aber Fokus)
    println!("[set_camera] Fallback auf Tastatur-Simulation");
    keyboard::switch_camera(&cam_id)?;
    Ok(())
}

#[tauri::command]
async fn focus_driver(car_number: String) -> Result<(), String> {
    // Zuerst Kamera auf TV schalten
    set_camera("TV".to_string()).await?;
    tokio::time::sleep(std::time::Duration::from_millis(100)).await;
    
    // Fahrzeug-Fokus via Tastatursimulation (Strg+F + Fahrzeugnummer + Enter)
    keyboard::focus_car(&car_number)?;
    Ok(())
}

#[tauri::command]
async fn switch_to_live(state: State<'_, AppState>) -> Result<(), String> {
    state.lmu.switch_to_live().await.map_err(|e| e.to_string())
}

#[tauri::command]
async fn switch_to_replay(state: State<'_, AppState>) -> Result<(), String> {
    state.lmu.switch_to_replay().await.map_err(|e| e.to_string())
}

#[tauri::command]
async fn check_lmu_connection(state: State<'_, AppState>) -> Result<bool, String> {
    Ok(state.lmu.is_available().await)
}

#[tauri::command]
async fn start_fcy(app: tauri::AppHandle, state: State<'_, AppState>) -> Result<(), String> {
    if state.fcy.current_phase() != FcyPhase::Idle {
        return Ok(()); // läuft schon
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

/// Wird vom Splashscreen-Fenster aufgerufen, sobald der Countdown/die
/// Update-Prüfung abgeschlossen ist: zeigt das (bereits im Hintergrund
/// geladene) Hauptfenster maximiert an und schließt den Splashscreen.
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
    let mut session_start: Option<std::time::Instant> = None;

    loop {
        if !state.should_poll.load(Ordering::SeqCst) {
            if was_connected {
                was_connected = false;
                session_start = None;
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
                session_start = Some(std::time::Instant::now());
            }
        }

        if available {
            if let (Ok(standings), Ok(session)) =
                (state.lmu.get_standings().await, state.lmu.get_session_info().await)
            {
                let session_time_s = session_start
                    .map(|t| t.elapsed().as_secs_f64())
                    .unwrap_or(0.0);

                // 1) Automatische Verdachtserkennung (rot/gelb/weiß)
                {
                    let mut detector = state.detector.lock().await;
                    let ctx = DetectionContext {
                        session_time_s,
                        track_name: &session.track_name,
                    };
                    for incident in detector.analyze(&standings, &ctx) {
                        let mut incident = incident;
                        if state.db.insert(&mut incident).is_ok() {
                            let _ = app.emit("new-incident", NewIncidentEvent { incident });
                        }
                    }
                }

                // 2) FCY-Geschwindigkeitsüberwachung
                if state.fcy.current_phase() == FcyPhase::Active {
                    let limit = state.settings.lock().await.fcy_speed_limit_kmh;
                    let tolerance = 3.0; // +3 km/h Toleranz
                    let threshold = limit + tolerance;
                    let ctx = DetectionContext {
                        session_time_s,
                        track_name: &session.track_name,
                    };
                    for car in &standings {
                        // Prüfe: speed_kmh > (Limit + Toleranz)
                        if car.speed_kmh > threshold && state.fcy.should_flag(car.slot_id) {
                            let mut incident = incidents::make_fcy_violation(&ctx, car, limit);
                            if state.db.insert(&mut incident).is_ok() {
                                let _ = app.emit("new-incident", NewIncidentEvent { incident });
                            }
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
            // Kein Internet oder Server nicht erreichbar -> NICHT sofort
            // sperren, sondern die zwischengespeicherte Kulanzfrist greifen
            // lassen (siehe LicenseData::is_currently_licensed). Nur den
            // Fehlertext fürs UI aktualisieren.
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
            let lmu = Arc::new(LmuClient::new());
            let detector = Arc::new(AsyncMutex::new(IncidentDetector::new()));
            let fcy = Arc::new(FcyState::default());
            let settings_store = Arc::new(SettingsStore::new(&app_dir));
            let loaded_settings = settings_store.load();
            let settings = Arc::new(AsyncMutex::new(loaded_settings));
            let should_poll = Arc::new(AtomicBool::new(false));
            let license_store = Arc::new(LicenseStore::new(&app_dir));
            let loaded_license = license_store.load();
            let license = Arc::new(AsyncMutex::new(loaded_license));

            let state = Arc::new(AppState {
                db: db.clone(),
                lmu: lmu.clone(),
                detector: detector.clone(),
                fcy: fcy.clone(),
                settings_store: settings_store.clone(),
                settings: settings.clone(),
                should_poll: should_poll.clone(),
                license_store: license_store.clone(),
                license: license.clone(),
            });

            app.manage(AppState {
                db,
                lmu,
                detector,
                fcy,
                settings_store,
                settings,
                should_poll,
                license_store: license_store.clone(),
                license: license.clone(),
            });

            let app_handle = app.handle().clone();
            tauri::async_runtime::spawn(poll_loop(app_handle, state));

            // Beim Start einmalig die gespeicherte Lizenz online nachprüfen
            // (falls eine hinterlegt ist). Läuft im Hintergrund, blockiert
            // den Programmstart nicht - die zwischengespeicherte Gültigkeit
            // (inkl. Offline-Kulanzfrist) gilt bis das Ergebnis da ist.
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
            list_pending_incidents,
            list_archived_incidents,
            submit_incident_decision,
            jump_to_incident_replay,
            set_camera,
            focus_driver,
            check_lmu_connection,
            switch_to_live,
            switch_to_replay,
            start_fcy,
            clear_fcy,
            show_main_window,
        ])
        .run(tauri::generate_context!())
        .expect("Fehler beim Starten von LMU Race Control");
}