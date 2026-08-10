use axum::{
    routing::{get, post, patch, delete},
    Router, Json, extract::{Path, Request, State},
    http::{StatusCode, HeaderMap},
    response::IntoResponse,
    middleware::{self, Next},
};
use serde::{Deserialize, Serialize};
use sqlx::sqlite::{SqlitePool, SqlitePoolOptions};
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::Mutex;
use tower_http::cors::CorsLayer;
use tower_http::services::ServeDir;
use tracing_subscriber;
use std::env;

// ============================================================
// Typen
// ============================================================

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ServerConfig {
    pub key: String,
    pub value: String,
    pub updated_at: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Incident {
    pub id: String,
    pub tenant_id: String,
    pub incident_number: i64,
    pub car_number_a: String,
    pub car_number_b: Option<String>,
    pub flag_color: String,
    pub incident_type: String,
    pub session_type: String,
    pub lap_number: i64,
    pub timestamp: String,
    pub decision: Option<String>,
    pub penalty_points: Option<i64>,
    pub warning_points: Option<i64>,
    pub notes: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Deserialize)]
pub struct CreateIncidentRequest {
    pub incident_number: i64,
    pub car_number_a: String,
    pub car_number_b: Option<String>,
    pub flag_color: String,
    pub incident_type: String,
    pub session_type: String,
    pub lap_number: i64,
    pub timestamp: String,
}

#[derive(Debug, Deserialize)]
pub struct UpdateDecisionRequest {
    pub decision: String,
    pub penalty_points: Option<i64>,
    pub warning_points: Option<i64>,
    pub notes: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Tenant {
    pub id: String,
    pub name: String,
    pub tier: String,
    pub max_users: i32,
    pub created_at: String,
}

#[derive(Debug, Deserialize)]
pub struct CreateTenantRequest {
    pub name: String,
    pub tier: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ApiKeyRecord {
    pub id: String,
    pub tenant_id: String,
    pub key: String,
    pub label: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Deserialize)]
pub struct CreateApiKeyRequest {
    pub tenant_id: String,
    pub label: Option<String>,
}

// ============================================================
// Config Defaults
// ============================================================

/// Default-Werte für die Server-Konfiguration
const CONFIG_DEFAULTS: &[(&str, &str)] = &[
    ("keygen_token", "admin-2dd5cb265c251f14bf33a9341e172357d1cf90cf696d263a2cd09472706fb6a5v3"),
    ("keygen_account_id", "65f997d1-bac7-4b1c-b37d-35fce549bde6"),
    ("discord_webhook_url", ""),
    ("backup_path", ""),
];

// ============================================================
// Rate Limiter
// ============================================================

struct RateLimiter {
    max_per_second: u32,
    buckets: Mutex<std::collections::HashMap<String, (u32, Instant)>>,
}

impl RateLimiter {
    fn new(max_per_second: u32) -> Self {
        Self {
            max_per_second,
            buckets: Mutex::new(std::collections::HashMap::new()),
        }
    }

    async fn check(&self, api_key: &str) -> bool {
        let mut buckets = self.buckets.lock().await;
        let now = Instant::now();
        let entry = buckets.entry(api_key.to_string()).or_insert((0, now));
        if now.duration_since(entry.1) > Duration::from_secs(1) {
            *entry = (0, now);
        }
        if entry.0 >= self.max_per_second {
            false
        } else {
            entry.0 += 1;
            true
        }
    }
}

// ============================================================
// AppState
// ============================================================

#[derive(Clone)]
pub struct AppState {
    pub db: SqlitePool,
    pub api_keys: Vec<ApiKeyEntry>,
    rate_limiter: Arc<RateLimiter>,
    pub active_requests: Arc<AtomicU32>,
    start_time: Instant,
}

#[derive(Debug, Clone)]
pub struct ApiKeyEntry {
    pub key: String,
    pub tenant_id: String,
    pub max_users: i32,
    pub tier: String,
}

// ============================================================
// Database
// ============================================================

async fn init_db(pool: &SqlitePool) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS tenants (
            id TEXT PRIMARY KEY,
            name TEXT NOT NULL,
            tier TEXT NOT NULL DEFAULT 'enterprise_l',
            max_users INTEGER NOT NULL DEFAULT 3,
            created_at TEXT NOT NULL DEFAULT (datetime('now'))
        )
        "#,
    )
    .execute(pool)
    .await?;

    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS api_keys (
            id TEXT PRIMARY KEY,
            tenant_id TEXT NOT NULL,
            key TEXT NOT NULL UNIQUE,
            label TEXT,
            created_at TEXT NOT NULL DEFAULT (datetime('now')),
            FOREIGN KEY (tenant_id) REFERENCES tenants(id)
        )
        "#,
    )
    .execute(pool)
    .await?;

    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS incidents (
            id TEXT PRIMARY KEY,
            tenant_id TEXT NOT NULL,
            incident_number INTEGER NOT NULL,
            car_number_a TEXT NOT NULL,
            car_number_b TEXT,
            flag_color TEXT NOT NULL,
            incident_type TEXT NOT NULL,
            session_type TEXT NOT NULL,
            lap_number INTEGER NOT NULL,
            timestamp TEXT NOT NULL,
            decision TEXT,
            penalty_points INTEGER,
            warning_points INTEGER,
            notes TEXT,
            created_at TEXT NOT NULL DEFAULT (datetime('now')),
            FOREIGN KEY (tenant_id) REFERENCES tenants(id)
        )
        "#,
    )
    .execute(pool)
    .await?;

    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS server_config (
            key TEXT PRIMARY KEY,
            value TEXT NOT NULL,
            updated_at TEXT NOT NULL DEFAULT (datetime('now'))
        )
        "#,
    )
    .execute(pool)
    .await?;

    // Default-Werte für server_config einfügen (falls nicht vorhanden)
    for (key, value) in CONFIG_DEFAULTS {
        sqlx::query(
            "INSERT OR IGNORE INTO server_config (key, value) VALUES (?, ?)"
        )
        .bind(key)
        .bind(value)
        .execute(pool)
        .await?;
    }

    let tenant_count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM tenants")
        .fetch_one(pool)
        .await?;
    
    if tenant_count.0 == 0 {
        let tenant_id = "default";
        sqlx::query("INSERT INTO tenants (id, name, tier, max_users) VALUES (?, ?, ?, ?)")
            .bind(tenant_id)
            .bind("Default Tenant")
            .bind("enterprise_l")
            .bind(3)
            .execute(pool)
            .await?;

        let api_key = uuid::Uuid::new_v4().to_string();
        sqlx::query("INSERT INTO api_keys (id, tenant_id, key, label) VALUES (?, ?, ?, ?)")
            .bind(uuid::Uuid::new_v4().to_string())
            .bind(tenant_id)
            .bind(&api_key)
            .bind("Default API Key")
            .execute(pool)
            .await?;

        tracing::info!("Default Tenant erstellt. API-Key: {}", api_key);
    }

    tracing::info!("Datenbank initialisiert");
    Ok(())
}

async fn load_api_keys(pool: &SqlitePool) -> Vec<ApiKeyEntry> {
    let result = sqlx::query_as::<_, (String, String, String, i32)>(
        r#"
        SELECT a.key, a.tenant_id, t.tier, t.max_users
        FROM api_keys a
        JOIN tenants t ON a.tenant_id = t.id
        "#,
    )
    .fetch_all(pool)
    .await;

    match result {
        Ok(rows) => rows.into_iter().map(|r| ApiKeyEntry {
            key: r.0,
            tenant_id: r.1,
            tier: r.2,
            max_users: r.3,
        }).collect(),
        Err(e) => {
            tracing::error!("Fehler beim Laden der API-Keys: {}", e);
            vec![]
        }
    }
}

// ============================================================
// Auth Middleware
// ============================================================

async fn auth_middleware(
    State(state): State<AppState>,
    headers: HeaderMap,
    request: Request,
    next: Next,
) -> impl IntoResponse {
    let path = request.uri().path();
    // Öffentliche Endpunkte (kein API-Key nötig)
    if path == "/health" || path == "/api-key" || path == "/admin" || path == "/api/webhook/keygen" || path == "/api/lookup-api-key" {
        return Ok(next.run(request).await);
    }
    // Admin-Dashboard statische Dateien ohne Auth erlauben
    if path.starts_with("/admin/") {
        return Ok(next.run(request).await);
    }
    // Config-API ist geschützt (braucht API-Key) – wird normal geprüft

    let auth_header = headers
        .get("Authorization")
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.strip_prefix("Bearer "));

    match auth_header {
        Some(key) => {
            let valid = state.api_keys.iter().any(|entry| entry.key == key);
            if valid {
                if !state.rate_limiter.check(key).await {
                    let response = (StatusCode::TOO_MANY_REQUESTS, Json(serde_json::json!({
                        "error": "Rate Limit überschritten (max. 60 Anfragen/Sekunde)"
                    })));
                    return Err(response);
                }
                Ok(next.run(request).await)
            } else {
                let response = (StatusCode::UNAUTHORIZED, Json(serde_json::json!({
                    "error": "Ungültiger API-Key"
                })));
                Err(response)
            }
        }
        None => {
            let response = (StatusCode::UNAUTHORIZED, Json(serde_json::json!({
                "error": "Authorization: Bearer <api_key> fehlt"
            })));
            Err(response)
        }
    }
}

// ============================================================
// Logging Middleware
// ============================================================

async fn logging_middleware(
    State(state): State<AppState>,
    headers: HeaderMap,
    request: Request,
    next: Next,
) -> impl IntoResponse {
    let method = request.method().clone();
    let uri = request.uri().path().to_string();
    let api_key = headers
        .get("Authorization")
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.strip_prefix("Bearer "))
        .unwrap_or("(none)")
        .to_string();

    let active = state.active_requests.fetch_add(1, Ordering::SeqCst) + 1;
    let start = Instant::now();

    let response = next.run(request).await;

    let duration = start.elapsed();
    let status = response.status();
    let active_now = state.active_requests.fetch_sub(1, Ordering::SeqCst) - 1;

    tracing::info!(
        "[{}] {} {} -> {} ({:.0?}, {} aktiv)",
        &api_key[..api_key.len().min(8)],
        method,
        uri,
        status.as_u16(),
        duration,
        active_now
    );

    response
}

// ============================================================
// Handler – Incidents
// ============================================================

async fn post_incident(
    state: axum::extract::State<AppState>,
    headers: HeaderMap,
    Json(req): Json<CreateIncidentRequest>,
) -> impl IntoResponse {
    let api_key = headers
        .get("Authorization")
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.strip_prefix("Bearer "))
        .unwrap_or("");

    let tenant_id = state.api_keys
        .iter()
        .find(|entry| entry.key == api_key)
        .map(|entry| entry.tenant_id.clone())
        .unwrap_or_default();

    let id = uuid::Uuid::new_v4().to_string();
    let created_at = chrono::Utc::now().to_rfc3339();

    let result = sqlx::query(
        r#"
        INSERT INTO incidents (id, tenant_id, incident_number, car_number_a, car_number_b, flag_color,
                               incident_type, session_type, lap_number, timestamp, created_at)
        VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
        "#,
    )
    .bind(&id)
    .bind(&tenant_id)
    .bind(req.incident_number)
    .bind(&req.car_number_a)
    .bind(&req.car_number_b)
    .bind(&req.flag_color)
    .bind(&req.incident_type)
    .bind(&req.session_type)
    .bind(req.lap_number)
    .bind(&req.timestamp)
    .bind(&created_at)
    .execute(&state.db)
    .await;

    match result {
        Ok(_) => {
            tracing::info!("Incident {} gespeichert (Tenant: {})", id, tenant_id);
            (StatusCode::CREATED, Json(serde_json::json!({ "id": id })))
        }
        Err(e) => {
            tracing::error!("Fehler beim Speichern: {}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({ "error": e.to_string() })))
        }
    }
}

async fn get_incidents(
    state: axum::extract::State<AppState>,
    headers: HeaderMap,
) -> impl IntoResponse {
    let api_key = headers
        .get("Authorization")
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.strip_prefix("Bearer "))
        .unwrap_or("");

    let tenant_id = state.api_keys
        .iter()
        .find(|entry| entry.key == api_key)
        .map(|entry| entry.tenant_id.clone())
        .unwrap_or_default();

    let result = sqlx::query_as::<_, (String, String, i64, String, Option<String>, String, String, String, i64, String, Option<String>, Option<i64>, Option<i64>, Option<String>, String)>(
        r#"
        SELECT id, tenant_id, incident_number, car_number_a, car_number_b, flag_color,
               incident_type, session_type, lap_number, timestamp,
               decision, penalty_points, warning_points, notes, created_at
        FROM incidents
        WHERE tenant_id = ?
        ORDER BY created_at DESC
        "#,
    )
    .bind(&tenant_id)
    .fetch_all(&state.db)
    .await;

    match result {
        Ok(rows) => {
            let incidents: Vec<Incident> = rows.into_iter().map(|r| Incident {
                id: r.0,
                tenant_id: r.1,
                incident_number: r.2,
                car_number_a: r.3,
                car_number_b: r.4,
                flag_color: r.5,
                incident_type: r.6,
                session_type: r.7,
                lap_number: r.8,
                timestamp: r.9,
                decision: r.10,
                penalty_points: r.11,
                warning_points: r.12,
                notes: r.13,
                created_at: r.14,
            }).collect();
            (StatusCode::OK, Json(serde_json::json!(incidents)))
        }
        Err(e) => {
            (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({ "error": e.to_string() })))
        }
    }
}

async fn delete_all_incidents(
    state: axum::extract::State<AppState>,
    headers: HeaderMap,
) -> impl IntoResponse {
    let api_key = headers
        .get("Authorization")
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.strip_prefix("Bearer "))
        .unwrap_or("");

    let tenant_id = state.api_keys
        .iter()
        .find(|entry| entry.key == api_key)
        .map(|entry| entry.tenant_id.clone())
        .unwrap_or_default();

    let result = sqlx::query("DELETE FROM incidents WHERE tenant_id = ?")
        .bind(&tenant_id)
        .execute(&state.db)
        .await;

    match result {
        Ok(r) => {
            tracing::info!("{} Incidents für Tenant {} gelöscht", r.rows_affected(), tenant_id);
            (StatusCode::OK, Json(serde_json::json!({ "status": "deleted", "count": r.rows_affected() })))
        }
        Err(e) => {
            (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({ "error": e.to_string() })))
        }
    }
}

async fn patch_incident(
    state: axum::extract::State<AppState>,
    headers: HeaderMap,
    Path(id): Path<String>,
    Json(req): Json<UpdateDecisionRequest>,
) -> impl IntoResponse {
    let api_key = headers
        .get("Authorization")
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.strip_prefix("Bearer "))
        .unwrap_or("");

    let tenant_id = state.api_keys
        .iter()
        .find(|entry| entry.key == api_key)
        .map(|entry| entry.tenant_id.clone())
        .unwrap_or_default();

    let result = sqlx::query(
        r#"
        UPDATE incidents
        SET decision = ?, penalty_points = ?, warning_points = ?, notes = ?
        WHERE id = ? AND tenant_id = ?
        "#,
    )
    .bind(&req.decision)
    .bind(req.penalty_points)
    .bind(req.warning_points)
    .bind(&req.notes)
    .bind(&id)
    .bind(&tenant_id)
    .execute(&state.db)
    .await;

    match result {
        Ok(r) if r.rows_affected() > 0 => {
            tracing::info!("Incident {} aktualisiert (Tenant: {})", id, tenant_id);
            (StatusCode::OK, Json(serde_json::json!({ "status": "updated" })))
        }
        Ok(_) => {
            (StatusCode::NOT_FOUND, Json(serde_json::json!({ "error": "not found" })))
        }
        Err(e) => {
            (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({ "error": e.to_string() })))
        }
    }
}

// ============================================================
// Handler – Admin (geschützt)
// ============================================================

/// Server neustarten – startet die .exe selbst neu (kein systemd nötig!)
async fn post_restart_server() -> impl IntoResponse {
    tracing::warn!("⚠️ Server-Neustart angefordert (Admin Dashboard)");
    tokio::spawn(async {
        tokio::time::sleep(Duration::from_millis(500)).await;
        // Aktuelle .exe erneut starten, dann alten Prozess beenden
        #[cfg(windows)]
        {
            if let Ok(exe_path) = std::env::current_exe() {
                let _ = std::process::Command::new(exe_path)
                    .spawn();
            }
        }
        // Fallback für Linux/Mac: Systemd oder manuell
        #[cfg(not(windows))]
        {
            std::process::exit(0);
        }
        std::process::exit(0);
    });
    (StatusCode::OK, Json(serde_json::json!({ "status": "restarting" })))
}

/// Server stoppen (beendet den Prozess komplett – muss manuell neu gestartet werden)
async fn post_stop_server() -> impl IntoResponse {
    tracing::warn!("⏹️ Server-Stopp angefordert (Admin Dashboard)");
    tokio::spawn(async {
        tokio::time::sleep(Duration::from_millis(500)).await;
        std::process::exit(0);
    });
    (StatusCode::OK, Json(serde_json::json!({ "status": "stopping" })))
}

// ============================================================
// Keygen-Webhook – automatische Kundenerstellung
// ============================================================

#[derive(Debug, Deserialize)]
pub struct KeygenWebhookEvent {
    pub data: Option<KeygenEventData>,
}

#[derive(Debug, Deserialize)]
pub struct KeygenEventData {
    pub id: Option<String>,
    #[serde(rename = "type")]
    pub event_type: Option<String>,
    pub attributes: Option<KeygenEventAttributes>,
}

#[derive(Debug, Deserialize)]
pub struct KeygenEventAttributes {
    pub event: Option<String>,
}

/// Webhook von Keygen für Lizenz-Events:
/// - `license.created`  → Mandant + API-Key automatisch anlegen (Tier aus Metadaten)
/// - `license.renewed`  → bestehenden Mandanten updaten
async fn post_keygen_webhook(
    state: axum::extract::State<AppState>,
    Json(payload): Json<serde_json::Value>,
) -> impl IntoResponse {
    tracing::info!("🔑 Keygen-Webhook empfangen");

    // Event-Namen aus dem Payload extrahieren
    let event_name = payload
        .get("data")
        .and_then(|d| d.get("attributes"))
        .and_then(|a| a.get("event"))
        .and_then(|e| e.as_str())
        .or_else(|| payload.get("event").and_then(|e| e.as_str()))
        .unwrap_or("unknown");

    // Lizenz-Metadaten auslesen (Keygen sendet metadata im license-Objekt)
    let meta = payload
        .get("data")
        .and_then(|d| d.get("attributes"))
        .and_then(|a| a.get("metadata"));

    let tier = meta
        .and_then(|m| m.get("tier"))
        .and_then(|t| t.as_str())
        .unwrap_or("enterprise_l");

    let tenant_name = payload
        .get("data")
        .and_then(|d| d.get("attributes"))
        .and_then(|a| a.get("name"))
        .and_then(|n| n.as_str())
        .unwrap_or("Neuer Kunde")
        .to_string();

    let max_users = match tier {
        "enterprise_l" => 3,
        "enterprise_xl" => 10,
        "enterprise_howe" => 25,
        _ => 3,
    };

    match event_name {
        "license.created" | "machine.created" | "license.renewed" => {
            let tenant_id = uuid::Uuid::new_v4().to_string();
            let key = uuid::Uuid::new_v4().to_string();

            // Mandant anlegen
            let tenant_result = sqlx::query(
                r#"INSERT INTO tenants (id, name, tier, max_users) VALUES (?, ?, ?, ?)"#,
            )
            .bind(&tenant_id)
            .bind(&tenant_name)
            .bind(tier)
            .bind(max_users)
            .execute(&state.db)
            .await;

            match tenant_result {
                Ok(_) => {
                    // API-Key anlegen
                    let key_result = sqlx::query(
                        r#"INSERT INTO api_keys (id, tenant_id, key, label) VALUES (?, ?, ?, ?)"#,
                    )
                    .bind(uuid::Uuid::new_v4().to_string())
                    .bind(&tenant_id)
                    .bind(&key)
                    .bind(format!("Auto-Key für {}", tenant_name))
                    .execute(&state.db)
                    .await;

                    match key_result {
                        Ok(_) => {
                            tracing::info!("✅ Webhook: Mandant '{}' (Tier: {}) + API-Key erstellt", tenant_name, tier);
                            (StatusCode::CREATED, Json(serde_json::json!({
                                "status": "created",
                                "tenant_id": tenant_id,
                                "tenant_name": tenant_name,
                                "tier": tier,
                                "max_users": max_users,
                                "api_key": key,
                                "note": "API-Key ist sofort aktiv (Server-Neustart nötig für Key-Cache)"
                            })))
                        }
                        Err(e) => {
                            tracing::error!("Webhook: Fehler beim API-Key: {}", e);
                            (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({ "error": e.to_string() })))
                        }
                    }
                }
                Err(e) => {
                    tracing::error!("Webhook: Fehler beim Mandant: {}", e);
                    (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({ "error": e.to_string() })))
                }
            }
        }
        _ => {
            tracing::info!("Webhook: Event '{}' ignoriert", event_name);
            (StatusCode::OK, Json(serde_json::json!({ "status": "ignored", "event": event_name })))
        }
    }
}

/// Manuellen Lizenz-Import von Keygen (für bestehende Lizenzen)
/// Aufruf: POST /api/sync-license  { "license_key": "...", "tier": "enterprise_xl", "name": "..." }
async fn post_sync_license(
    state: axum::extract::State<AppState>,
    Json(req): Json<serde_json::Value>,
) -> impl IntoResponse {
    let license_key = req.get("license_key").and_then(|v| v.as_str()).unwrap_or("");
    let tier = req.get("tier").and_then(|v| v.as_str()).unwrap_or("enterprise_l");
    let name = req.get("name").and_then(|v| v.as_str()).unwrap_or("Importierter Kunde");
    let max_users = match tier {
        "enterprise_l" => 3,
        "enterprise_xl" => 10,
        "enterprise_howe" => 25,
        _ => 3,
    };

    // Prüfen ob bereits ein Tenant mit diesem Lizenz-Key existiert
    let existing: Result<(String,), _> = sqlx::query_as(
        "SELECT id FROM tenants WHERE id = ?"
    )
    .bind(format!("keygen_{}", license_key))
    .fetch_one(&state.db)
    .await;

    if let Ok((existing_id,)) = existing {
        // Update bestehenden Tenant
        let _ = sqlx::query("UPDATE tenants SET tier = ?, max_users = ?, name = ? WHERE id = ?")
            .bind(tier)
            .bind(max_users)
            .bind(name)
            .bind(&existing_id)
            .execute(&state.db)
            .await;
        tracing::info!("✅ Sync: Bestehender Mandant {} auf Tier {} upgegradet", existing_id, tier);
        return (StatusCode::OK, Json(serde_json::json!({ "status": "updated", "tenant_id": existing_id, "tier": tier })));
    }

    // Neuen Mandanten anlegen
    let tenant_id = format!("keygen_{}", license_key);
    let api_key = uuid::Uuid::new_v4().to_string();

    let tenant_result = sqlx::query(
        r#"INSERT INTO tenants (id, name, tier, max_users) VALUES (?, ?, ?, ?)"#,
    )
    .bind(&tenant_id)
    .bind(name)
    .bind(tier)
    .bind(max_users)
    .execute(&state.db)
    .await;

    match tenant_result {
        Ok(_) => {
            let _ = sqlx::query(
                r#"INSERT INTO api_keys (id, tenant_id, key, label) VALUES (?, ?, ?, ?)"#,
            )
            .bind(uuid::Uuid::new_v4().to_string())
            .bind(&tenant_id)
            .bind(&api_key)
            .bind(format!("Auto-Key für {}", name))
            .execute(&state.db)
            .await;

            tracing::info!("✅ Sync: Mandant '{}' (Tier: {}) importiert", name, tier);
            (StatusCode::CREATED, Json(serde_json::json!({
                "status": "created",
                "tenant_id": tenant_id,
                "tenant_name": name,
                "tier": tier,
                "max_users": max_users,
                "api_key": api_key
            })))
        }
        Err(e) => {
            (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({ "error": e.to_string() })))
        }
    }
}

/// Erweiterte Statistiken für das Admin-Dashboard
async fn get_stats_admin(
    state: axum::extract::State<AppState>,
) -> impl IntoResponse {
    let api_keys_count = state.api_keys.len();
    let active = state.active_requests.load(Ordering::SeqCst);
    let uptime = state.start_time.elapsed().as_secs();

    let incident_count: Result<(i64,), _> = sqlx::query_as("SELECT COUNT(*) FROM incidents")
        .fetch_one(&state.db)
        .await;
    let total_incidents = incident_count.map(|r| r.0).unwrap_or(0);

    let tenant_count: Result<(i64,), _> = sqlx::query_as("SELECT COUNT(*) FROM tenants")
        .fetch_one(&state.db)
        .await;
    let total_tenants = tenant_count.map(|r| r.0).unwrap_or(0);

    let api_key_count: Result<(i64,), _> = sqlx::query_as("SELECT COUNT(*) FROM api_keys")
        .fetch_one(&state.db)
        .await;
    let total_api_keys = api_key_count.map(|r| r.0).unwrap_or(0);

    Json(serde_json::json!({
        "status": "ok",
        "version": "0.1.0",
        "uptime_seconds": uptime,
        "total_incidents": total_incidents,
        "total_tenants": total_tenants,
        "total_api_keys": total_api_keys,
        "active_requests": active,
    }))
}

/// Mandant löschen
async fn delete_tenant(
    state: axum::extract::State<AppState>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    // Zugehörige API-Keys löschen
    let _ = sqlx::query("DELETE FROM api_keys WHERE tenant_id = ?")
        .bind(&id)
        .execute(&state.db)
        .await;
    // Zugehörige Incidents löschen
    let _ = sqlx::query("DELETE FROM incidents WHERE tenant_id = ?")
        .bind(&id)
        .execute(&state.db)
        .await;
    // Tenant löschen
    let result = sqlx::query("DELETE FROM tenants WHERE id = ?")
        .bind(&id)
        .execute(&state.db)
        .await;

    match result {
        Ok(r) if r.rows_affected() > 0 => {
            tracing::info!("Mandant {} gelöscht", id);
            (StatusCode::OK, Json(serde_json::json!({ "status": "deleted" })))
        }
        Ok(_) => (StatusCode::NOT_FOUND, Json(serde_json::json!({ "error": "not found" }))),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({ "error": e.to_string() }))),
    }
}

/// Mandant aktualisieren (Tier/Name)
async fn patch_tenant(
    state: axum::extract::State<AppState>,
    Path(id): Path<String>,
    Json(req): Json<serde_json::Value>,
) -> impl IntoResponse {
    let new_tier = req.get("tier").and_then(|v| v.as_str());
    let new_name = req.get("name").and_then(|v| v.as_str());

    if let Some(tier) = new_tier {
        let max_users = match tier {
            "enterprise_l" => 3,
            "enterprise_xl" => 10,
            "enterprise_howe" => 25,
            _ => 3,
        };
        let result = sqlx::query("UPDATE tenants SET tier = ?, max_users = ? WHERE id = ?")
            .bind(tier)
            .bind(max_users)
            .bind(&id)
            .execute(&state.db)
            .await;
        match result {
            Ok(r) if r.rows_affected() > 0 => {
                tracing::info!("Mandant {} auf Tier {} upgegradet", id, tier);
                return (StatusCode::OK, Json(serde_json::json!({ "status": "updated", "tier": tier, "max_users": max_users })));
            }
            Ok(_) => return (StatusCode::NOT_FOUND, Json(serde_json::json!({ "error": "not found" }))),
            Err(e) => return (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({ "error": e.to_string() }))),
        }
    }
    if let Some(name) = new_name {
        let result = sqlx::query("UPDATE tenants SET name = ? WHERE id = ?")
            .bind(name)
            .bind(&id)
            .execute(&state.db)
            .await;
        match result {
            Ok(r) if r.rows_affected() > 0 => {
                return (StatusCode::OK, Json(serde_json::json!({ "status": "updated", "name": name })));
            }
            Ok(_) => return (StatusCode::NOT_FOUND, Json(serde_json::json!({ "error": "not found" }))),
            Err(e) => return (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({ "error": e.to_string() }))),
        }
    }
    (StatusCode::BAD_REQUEST, Json(serde_json::json!({ "error": "no valid fields to update" })))
}

/// Alle Mandanten abrufen
async fn get_tenants(
    state: axum::extract::State<AppState>,
) -> impl IntoResponse {
    let result = sqlx::query_as::<_, (String, String, String, i32, String)>(
        r#"
        SELECT id, name, tier, max_users, created_at
        FROM tenants
        ORDER BY created_at DESC
        "#,
    )
    .fetch_all(&state.db)
    .await;

    match result {
        Ok(rows) => {
            let tenants: Vec<Tenant> = rows.into_iter().map(|r| Tenant {
                id: r.0,
                name: r.1,
                tier: r.2,
                max_users: r.3,
                created_at: r.4,
            }).collect();
            (StatusCode::OK, Json(serde_json::json!(tenants)))
        }
        Err(e) => {
            (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({ "error": e.to_string() })))
        }
    }
}

/// Mandant erstellen
async fn post_tenant(
    state: axum::extract::State<AppState>,
    Json(req): Json<CreateTenantRequest>,
) -> impl IntoResponse {
    let id = uuid::Uuid::new_v4().to_string();
    let max_users = match req.tier.as_str() {
        "enterprise_l" => 3,
        "enterprise_xl" => 10,
        "enterprise_howe" => 25,
        _ => 3,
    };

    let result = sqlx::query(
        r#"INSERT INTO tenants (id, name, tier, max_users) VALUES (?, ?, ?, ?)"#,
    )
    .bind(&id)
    .bind(&req.name)
    .bind(&req.tier)
    .bind(max_users)
    .execute(&state.db)
    .await;

    match result {
        Ok(_) => {
            tracing::info!("Mandant '{}' erstellt (ID: {})", req.name, id);
            (StatusCode::CREATED, Json(serde_json::json!({ "id": id, "name": req.name, "tier": req.tier })))
        }
        Err(e) => {
            (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({ "error": e.to_string() })))
        }
    }
}

/// Alle API-Keys abrufen (Admin)
async fn get_api_keys_admin(
    state: axum::extract::State<AppState>,
) -> impl IntoResponse {
    let result = sqlx::query_as::<_, (String, String, String, Option<String>, String)>(
        r#"
        SELECT id, tenant_id, key, label, created_at
        FROM api_keys
        ORDER BY created_at DESC
        "#,
    )
    .fetch_all(&state.db)
    .await;

    match result {
        Ok(rows) => {
            let keys: Vec<ApiKeyRecord> = rows.into_iter().map(|r| ApiKeyRecord {
                id: r.0,
                tenant_id: r.1,
                key: r.2,
                label: r.3,
                created_at: r.4,
            }).collect();
            (StatusCode::OK, Json(serde_json::json!(keys)))
        }
        Err(e) => {
            (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({ "error": e.to_string() })))
        }
    }
}

/// API-Key erstellen (Admin)
async fn post_api_key(
    state: axum::extract::State<AppState>,
    Json(req): Json<CreateApiKeyRequest>,
) -> impl IntoResponse {
    let id = uuid::Uuid::new_v4().to_string();
    let key = uuid::Uuid::new_v4().to_string();

    let result = sqlx::query(
        r#"INSERT INTO api_keys (id, tenant_id, key, label) VALUES (?, ?, ?, ?)"#,
    )
    .bind(&id)
    .bind(&req.tenant_id)
    .bind(&key)
    .bind(&req.label)
    .execute(&state.db)
    .await;

    match result {
        Ok(_) => {
            tracing::info!("API-Key erstellt für Tenant {}", req.tenant_id);
            // Neu laden
            let new_keys = load_api_keys(&state.db).await;
            // In einer echten App würde man den State mutieren – hier aktualisieren wir den Vec
            // Da AppState per Arc geteilt ist, müssten wir mit Mutex arbeiten. Fürs erste
            // lassen wir die API-Keys nur beim Start laden.
            (StatusCode::CREATED, Json(serde_json::json!({ "id": id, "key": key })))
        }
        Err(e) => {
            (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({ "error": e.to_string() })))
        }
    }
}

// ============================================================
// Config-Helper
// ============================================================

/// Holt einen Konfigurationswert aus der DB
async fn get_config_value(pool: &SqlitePool, key: &str) -> Option<String> {
    sqlx::query_as::<_, (String,)>(
        "SELECT value FROM server_config WHERE key = ?"
    )
    .bind(key)
    .fetch_optional(pool)
    .await
    .ok()
    .and_then(|r| r.map(|v| v.0))
}

/// Setzt einen Konfigurationswert in der DB
async fn set_config_value(pool: &SqlitePool, key: &str, value: &str) -> Result<(), sqlx::Error> {
    sqlx::query(
        "INSERT INTO server_config (key, value, updated_at) VALUES (?, ?, datetime('now'))
         ON CONFLICT(key) DO UPDATE SET value = excluded.value, updated_at = datetime('now')"
    )
    .bind(key)
    .bind(value)
    .execute(pool)
    .await?;
    Ok(())
}

// ============================================================
// Handler – Config (geschützt)
// ============================================================

/// Alle Konfigurationswerte abrufen
async fn get_config(
    state: axum::extract::State<AppState>,
) -> impl IntoResponse {
    let result = sqlx::query_as::<_, (String, String, String)>(
        "SELECT key, value, updated_at FROM server_config ORDER BY key"
    )
    .fetch_all(&state.db)
    .await;

    match result {
        Ok(rows) => {
            let configs: Vec<ServerConfig> = rows.into_iter().map(|r| ServerConfig {
                key: r.0,
                value: r.1,
                updated_at: r.2,
            }).collect();
            (StatusCode::OK, Json(serde_json::json!(configs)))
        }
        Err(e) => {
            (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({ "error": e.to_string() })))
        }
    }
}

/// Einen Konfigurationswert aktualisieren
/// POST /api/config  { "key": "keygen_token", "value": "..." }
async fn post_config(
    state: axum::extract::State<AppState>,
    Json(req): Json<serde_json::Value>,
) -> impl IntoResponse {
    let key = req.get("key").and_then(|v| v.as_str()).unwrap_or("");
    let value = req.get("value").and_then(|v| v.as_str()).unwrap_or("");

    if key.is_empty() {
        return (StatusCode::BAD_REQUEST, Json(serde_json::json!({ "error": "key fehlt" })));
    }

    match set_config_value(&state.db, key, value).await {
        Ok(_) => {
            tracing::info!("🔧 Config '{}' aktualisiert", key);
            (StatusCode::OK, Json(serde_json::json!({ "status": "updated", "key": key })))
        }
        Err(e) => {
            tracing::error!("Fehler beim Aktualisieren von Config '{}': {}", key, e);
            (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({ "error": e.to_string() })))
        }
    }
}

// ============================================================
// Handler – Öffentlich
// ============================================================

/// API-Key anhand des License-Keys abfragen (öffentlich, kein API-Key nötig).
/// Validiert den License-Key bei Keygen, legt bei Bedarf Tenant + API-Key an
/// und gibt den API-Key zurück. Jeder License-Key bekommt SEINEN eigenen API-Key.
async fn post_lookup_api_key(
    state: axum::extract::State<AppState>,
    Json(req): Json<serde_json::Value>,
) -> impl IntoResponse {
    let license_key = req.get("license_key").and_then(|v| v.as_str()).unwrap_or("");

    if license_key.is_empty() {
        return (StatusCode::BAD_REQUEST, Json(serde_json::json!({ "error": "license_key fehlt" })));
    }

    let tenant_id = format!("keygen_{}", license_key);

    // 1. Prüfen ob bereits ein Tenant existiert – dann API-Key zurückgeben
    let existing = sqlx::query_as::<_, (String, String, String)>(
        r#"SELECT a.key, a.tenant_id, t.name FROM api_keys a JOIN tenants t ON a.tenant_id = t.id WHERE a.tenant_id = ?"#,
    )
    .bind(&tenant_id)
    .fetch_optional(&state.db)
    .await;

    match existing {
        Ok(Some((api_key, tid, tname))) => {
            tracing::info!("API-Key für License-Key {} abgefragt (Tenant: {})", &license_key[..license_key.len().min(8)], tid);
            return (StatusCode::OK, Json(serde_json::json!({
                "api_key": api_key,
                "tenant_id": tid,
                "tenant_name": tname
            })));
        }
        Ok(None) => {
            // 2. Noch kein Tenant – bei Keygen validieren und anlegen
            // Token aus DB lesen, Fallback auf Umgebungsvariable
            let keygen_token = get_config_value(&state.db, "keygen_token").await
                .filter(|s| !s.is_empty())
                .or_else(|| std::env::var("KEYGEN_TOKEN").ok())
                .unwrap_or_default();
            if keygen_token.is_empty() {
                return (StatusCode::SERVICE_UNAVAILABLE, Json(serde_json::json!({
                    "error": "KEYGEN_TOKEN nicht konfiguriert – Server-Admin kontaktieren"
                })));
            }

            // Account ID aus DB lesen, Fallback auf Umgebungsvariable
            let keygen_account_id = get_config_value(&state.db, "keygen_account_id").await
                .filter(|s| !s.is_empty())
                .or_else(|| std::env::var("KEYGEN_ACCOUNT_ID").ok())
                .unwrap_or_default();

            let client = reqwest::Client::builder()
                .timeout(std::time::Duration::from_secs(10))
                .build()
                .unwrap();

            let keygen_url = format!(
                "https://api.keygen.sh/v1/accounts/{}/licenses/{}",
                keygen_account_id,
                license_key
            );

            let resp = match client
                .get(&keygen_url)
                .header("Authorization", format!("Bearer {}", keygen_token))
                .header("Accept", "application/vnd.api+json")
                .send()
                .await
            {
                Ok(r) => r,
                Err(e) => {
                    return (StatusCode::BAD_GATEWAY, Json(serde_json::json!({
                        "error": format!("Keygen nicht erreichbar: {}", e)
                    })));
                }
            };

            if !resp.status().is_success() {
                return (StatusCode::UNAUTHORIZED, Json(serde_json::json!({
                    "error": "Ungültiger License-Key – bei Keygen nicht gefunden"
                })));
            }

            // Tier aus Keygen-Metadaten auslesen
            let keygen_body: serde_json::Value = resp.json().await.unwrap_or_default();
            let tier = keygen_body
                .get("data")
                .and_then(|d| d.get("attributes"))
                .and_then(|a| a.get("metadata"))
                .and_then(|m| m.get("tier"))
                .and_then(|t| t.as_str())
                .unwrap_or("enterprise_l");

            let name = keygen_body
                .get("data")
                .and_then(|d| d.get("attributes"))
                .and_then(|a| a.get("name"))
                .and_then(|n| n.as_str())
                .unwrap_or("API-Key Abfrage");

            let max_users = match tier {
                "enterprise_l" => 3,
                "enterprise_xl" => 10,
                "enterprise_howe" => 25,
                _ => 3,
            };

            let api_key = uuid::Uuid::new_v4().to_string();

            // Tenant anlegen
            let tenant_result = sqlx::query(
                r#"INSERT INTO tenants (id, name, tier, max_users) VALUES (?, ?, ?, ?)"#,
            )
            .bind(&tenant_id)
            .bind(name)
            .bind(tier)
            .bind(max_users)
            .execute(&state.db)
            .await;

            match tenant_result {
                Ok(_) => {
                    // API-Key anlegen
                    let _ = sqlx::query(
                        r#"INSERT INTO api_keys (id, tenant_id, key, label) VALUES (?, ?, ?, ?)"#,
                    )
                    .bind(uuid::Uuid::new_v4().to_string())
                    .bind(&tenant_id)
                    .bind(&api_key)
                    .bind(format!("Auto-Key für {}", name))
                    .execute(&state.db)
                    .await;

                    tracing::info!("✅ Lookup: Mandant '{}' (Tier: {}) + API-Key erstellt für License-Key {}", name, tier, &license_key[..license_key.len().min(8)]);

                    (StatusCode::CREATED, Json(serde_json::json!({
                        "api_key": api_key,
                        "tenant_id": tenant_id,
                        "tenant_name": name,
                        "tier": tier
                    })))
                }
                Err(e) => {
                    (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({
                        "error": format!("Fehler beim Anlegen des Mandanten: {}", e)
                    })))
                }
            }
        }
        Err(e) => {
            tracing::error!("Fehler bei API-Key-Lookup: {}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({ "error": e.to_string() })))
        }
    }
}

async fn health() -> impl IntoResponse {
    Json(serde_json::json!({ "status": "ok", "version": "0.1.0" }))
}

async fn get_first_api_key(
    state: axum::extract::State<AppState>,
) -> impl IntoResponse {
    let keys = &state.api_keys;
    if keys.is_empty() {
        (StatusCode::NOT_FOUND, Json(serde_json::json!({ "error": "Keine API-Keys gefunden" })))
    } else {
        (StatusCode::OK, Json(serde_json::json!({ "api_key": keys[0].key, "tenant_id": keys[0].tenant_id, "tier": keys[0].tier, "max_users": keys[0].max_users })))
    }
}

// ============================================================
// Handler – Backup
// ============================================================

/// Erstellt ein Backup der Datenbank im konfigurierten Backup-Pfad
/// POST /api/backup (Auth erforderlich)
async fn post_backup(
    state: axum::extract::State<AppState>,
) -> impl IntoResponse {
    // Backup-Pfad aus der Config laden
    let backup_path = get_config_value(&state.db, "backup_path").await.unwrap_or_default();
    
    if backup_path.is_empty() {
        return (StatusCode::BAD_REQUEST, Json(serde_json::json!({
            "error": "Kein Backup-Pfad konfiguriert. Bitte zuerst in der Konfiguration setzen."
        })));
    }

    // Backup-Verzeichnis erstellen falls nicht vorhanden
    let backup_dir = std::path::Path::new(&backup_path);
    if let Err(e) = std::fs::create_dir_all(backup_dir) {
        return (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({
            "error": format!("Backup-Verzeichnis konnte nicht erstellt werden: {}", e)
        })));
    }

    // Timestamp für den Dateinamen
    let timestamp = chrono::Utc::now().format("%Y%m%d_%H%M%S");
    let db_path = std::path::Path::new("lmu-race-control.db");
    let backup_file = backup_dir.join(format!("lmu-race-control-backup-{}.db", timestamp));

    // Datenbank kopieren (SQLite ist dateibasiert, einfaches Copy reicht)
    match std::fs::copy(db_path, &backup_file) {
        Ok(_) => {
            tracing::info!("📦 Backup erstellt: {}", backup_file.display());
            (StatusCode::OK, Json(serde_json::json!({
                "status": "ok",
                "message": "Backup erfolgreich erstellt",
                "path": backup_file.to_string_lossy().to_string()
            })))
        }
        Err(e) => {
            tracing::error!("Backup fehlgeschlagen: {}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({
                "error": format!("Backup fehlgeschlagen: {}", e)
            })))
        }
    }
}

// ============================================================
// Auto-Bereinigung: Vorfälle > 26 Stunden löschen
// ============================================================

async fn purge_old_incidents(pool: &SqlitePool) {
    let result = sqlx::query(
        r#"DELETE FROM incidents WHERE datetime(created_at) < datetime('now', '-26 hours')"#,
    )
    .execute(pool)
    .await;

    match result {
        Ok(r) if r.rows_affected() > 0 => {
            tracing::info!("🧹 {} alte Vorfälle (>26h) automatisch gelöscht", r.rows_affected());
        }
        Ok(_) => {} // Nichts zu löschen
        Err(e) => {
            tracing::error!("Fehler beim Bereinigen alter Vorfälle: {}", e);
        }
    }
}

// ============================================================
// Main
// ============================================================

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "sqlite://lmu-race-control.db?mode=rwc".to_string());

    let pool = SqlitePoolOptions::new()
        .max_connections(10)
        .connect(&database_url)
        .await
        .expect("Datenbank-Verbindung fehlgeschlagen");

    init_db(&pool).await.expect("Datenbank-Init fehlgeschlagen");

    let api_keys = load_api_keys(&pool).await;
    tracing::info!("{} API-Keys geladen", api_keys.len());

    let rate_limiter = Arc::new(RateLimiter::new(60));
    let active_requests = Arc::new(AtomicU32::new(0));
    let start_time = Instant::now();

    let state = AppState {
        db: pool.clone(),
        api_keys,
        rate_limiter,
        active_requests: active_requests.clone(),
        start_time,
    };

    // Hintergrund-Task: Alte Vorfälle (>26h) automatisch löschen (alle 30 Minuten)
    let purge_pool = pool.clone();
    tokio::spawn(async move {
        // Beim Start sofort einmal bereinigen
        purge_old_incidents(&purge_pool).await;
        // Dann alle 30 Minuten
        loop {
            tokio::time::sleep(Duration::from_secs(1800)).await;
            purge_old_incidents(&purge_pool).await;
        }
    });

    // Admin-Dashboard statische Dateien servieren
    let serve_dir = ServeDir::new("admin");

    let app = Router::new()
        // Öffentliche Endpunkte
        .route("/health", get(health))
        .route("/api-key", get(get_first_api_key))
        .route("/api/lookup-api-key", post(post_lookup_api_key))
        // Admin-Dashboard (HTML)
        .nest_service("/admin", serve_dir)
        // Keygen-Webhook (öffentlich) + Sync
        .route("/api/webhook/keygen", post(post_keygen_webhook))
        .route("/api/sync-license", post(post_sync_license))
        // Admin-API (geschützt)
        .route("/api/restart", post(post_restart_server))
        .route("/api/stop", post(post_stop_server))
        .route("/api/stats_admin", get(get_stats_admin))
        .route("/api/tenants", get(get_tenants).post(post_tenant).patch(patch_tenant))
        .route("/api/tenants/:id", delete(delete_tenant))
        .route("/api/keys", get(get_api_keys_admin).post(post_api_key))
        // Config-API (geschützt)
        .route("/api/config", get(get_config).post(post_config))
        // Incident-API (geschützt)
        .route("/api/stats", get(get_stats_admin))
        .route("/api/incidents", post(post_incident).get(get_incidents).delete(delete_all_incidents))
        .route("/api/incidents/:id", patch(patch_incident))
        // Backup-API (geschützt)
        .route("/api/backup", post(post_backup))
        // Middleware
        .layer(CorsLayer::permissive())
        .layer(middleware::from_fn_with_state(state.clone(), logging_middleware))
        .layer(middleware::from_fn_with_state(state.clone(), auth_middleware))
        .with_state(state);

    let addr = "0.0.0.0:3000";
    tracing::info!("===========================================");
    tracing::info!("  LMU RACECONTROL SERVER v0.2.0");
    tracing::info!("===========================================");
    tracing::info!("  Admin-Dashboard: http://{}:{}/admin/", "0.0.0.0", 3000);
    tracing::info!("  Health-Check:   GET /health");
    tracing::info!("  API-Key:        GET /api-key");
    tracing::info!("  Statistiken:    GET /api/stats    (Auth)");
    tracing::info!("  Vorfälle:       GET /api/incidents (Auth)");
    tracing::info!("  Vorfälle:       POST /api/incidents (Auth)");
    tracing::info!("  Entscheidung:   PATCH /api/incidents/:id (Auth)");
    tracing::info!("  Mandanten:      GET /api/tenants   (Auth)");
    tracing::info!("  Mandanten:      POST /api/tenants  (Auth)");
    tracing::info!("  API-Keys:       GET /api/keys      (Auth)");
    tracing::info!("  API-Keys:       POST /api/keys     (Auth)");
    tracing::info!("===========================================");

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}