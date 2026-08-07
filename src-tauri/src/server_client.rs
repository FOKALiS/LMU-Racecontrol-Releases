//! HTTP-Client für die Kommunikation mit dem LMU Racecontrol Server.
//! Sendet Vorfälle an den Server und ruft sie ab.
//! Der Server-Client wird über die Settings (server_url, api_key) konfiguriert.

use serde::{Deserialize, Serialize};

pub struct ServerClient {
    inner: reqwest::Client,
}

/// Ein Vorfall, wie er vom Server erwartet wird.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ServerIncident {
    pub id: String,
    pub incident_number: i64,
    pub car_number_a: String,
    pub car_number_b: Option<String>,
    pub flag_color: String,
    pub incident_type: String,
    pub session_type: String,
    pub lap_number: i64,
    pub timestamp: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateResponse {
    pub id: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ErrorResponse {
    pub error: String,
}

#[derive(Debug)]
pub enum ServerError {
    NotConfigured,       // Keine Server-URL gesetzt
    Unauthorized,       // API-Key ungültig
    RequestFailed(String), // Netzwerk-Fehler
    InvalidResponse(String), // Antwort unerwartet
}

impl std::fmt::Display for ServerError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ServerError::NotConfigured => write!(f, "Server-URL nicht konfiguriert"),
            ServerError::Unauthorized => write!(f, "API-Key ungültig"),
            ServerError::RequestFailed(msg) => write!(f, "Anfrage fehlgeschlagen: {}", msg),
            ServerError::InvalidResponse(msg) => write!(f, "Ungültige Antwort: {}", msg),
        }
    }
}

impl ServerClient {
    pub fn new() -> Self {
        Self {
            inner: reqwest::Client::builder()
                .timeout(std::time::Duration::from_secs(10))
                .build()
                .expect("HTTP-Client konnte nicht erstellt werden"),
        }
    }

    /// Prüft, ob der Server erreichbar ist (Health-Check).
    pub async fn check_health(&self, server_url: &str) -> Result<bool, ServerError> {
        if server_url.is_empty() {
            return Err(ServerError::NotConfigured);
        }
        let url = format!("{}/health", server_url.trim_end_matches('/'));
        match self.inner.get(&url).send().await {
            Ok(resp) => Ok(resp.status().is_success()),
            Err(e) => Err(ServerError::RequestFailed(e.to_string())),
        }
    }

    /// Sendet einen neuen Vorfall an den Server.
    pub async fn create_incident(
        &self,
        server_url: &str,
        api_key: &str,
        incident: &ServerIncident,
    ) -> Result<String, ServerError> {
        if server_url.is_empty() {
            return Err(ServerError::NotConfigured);
        }
        let url = format!("{}/api/incidents", server_url.trim_end_matches('/'));
        let resp = self
            .inner
            .post(&url)
            .header("Authorization", format!("Bearer {}", api_key))
            .json(incident)
            .send()
            .await
            .map_err(|e| ServerError::RequestFailed(e.to_string()))?;

        match resp.status().as_u16() {
            201 => {
                let body: CreateResponse = resp
                    .json()
                    .await
                    .map_err(|e| ServerError::InvalidResponse(e.to_string()))?;
                Ok(body.id)
            }
            401 => Err(ServerError::Unauthorized),
            status => {
                let body = resp.text().await.unwrap_or_default();
                Err(ServerError::RequestFailed(format!(
                    "HTTP {}: {}",
                    status, body
                )))
            }
        }
    }

    /// Löscht ALLE Vorfälle des eigenen Tenants auf dem Server.
    pub async fn delete_all_incidents(
        &self,
        server_url: &str,
        api_key: &str,
    ) -> Result<u64, ServerError> {
        if server_url.is_empty() {
            return Err(ServerError::NotConfigured);
        }
        let url = format!("{}/api/incidents", server_url.trim_end_matches('/'));
        let resp = self
            .inner
            .delete(&url)
            .header("Authorization", format!("Bearer {}", api_key))
            .send()
            .await
            .map_err(|e| ServerError::RequestFailed(e.to_string()))?;

        match resp.status().as_u16() {
            200 => {
                let body: serde_json::Value = resp
                    .json()
                    .await
                    .map_err(|e| ServerError::InvalidResponse(e.to_string()))?;
                let count = body.get("count").and_then(|c| c.as_u64()).unwrap_or(0);
                Ok(count)
            }
            401 => Err(ServerError::Unauthorized),
            status => {
                let body = resp.text().await.unwrap_or_default();
                Err(ServerError::RequestFailed(format!(
                    "HTTP {}: {}",
                    status, body
                )))
            }
        }
    }

    /// Ruft alle Vorfälle vom Server ab.
    pub async fn get_incidents(
        &self,
        server_url: &str,
        api_key: &str,
    ) -> Result<Vec<ServerIncident>, ServerError> {
        if server_url.is_empty() {
            return Err(ServerError::NotConfigured);
        }
        let url = format!("{}/api/incidents", server_url.trim_end_matches('/'));
        let resp = self
            .inner
            .get(&url)
            .header("Authorization", format!("Bearer {}", api_key))
            .send()
            .await
            .map_err(|e| ServerError::RequestFailed(e.to_string()))?;

        match resp.status().as_u16() {
            200 => {
                let incidents: Vec<ServerIncident> = resp
                    .json()
                    .await
                    .map_err(|e| ServerError::InvalidResponse(e.to_string()))?;
                Ok(incidents)
            }
            401 => Err(ServerError::Unauthorized),
            status => {
                let body = resp.text().await.unwrap_or_default();
                Err(ServerError::RequestFailed(format!(
                    "HTTP {}: {}",
                    status, body
                )))
            }
        }
    }
}