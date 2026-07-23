//! Sendet eine Discord-Embed-Nachricht, sobald ein Vorfall von der
//! Rennkommission entschieden und archiviert wurde. Nutzt einen normalen
//! Discord-Webhook (Server-Einstellungen -> Integrationen -> Webhooks).

use crate::db::Incident;
use anyhow::Result;

pub async fn send_incident_decision(webhook_url: &str, incident: &Incident) -> Result<()> {
    let webhook_url = webhook_url.trim();
    if webhook_url.is_empty() {
        // Kein Webhook konfiguriert - bewusst kein Fehler, damit das
        // Archivieren von Vorfällen auch ohne Discord-Anbindung klappt.
        return Ok(());
    }

    let is_no_action = incident
        .decision
        .as_deref()
        .map(|d| d.to_lowercase().contains("keine"))
        .unwrap_or(false);
    let color = if is_no_action { 0x2FBF4F } else { 0xE62837 };

    let mut fields = vec![
        serde_json::json!({"name": "Runde", "value": incident.lap.to_string(), "inline": true}),
        serde_json::json!({"name": "Kurve", "value": non_empty(&incident.corner), "inline": true}),
        serde_json::json!({"name": "Zeitstempel", "value": non_empty(&incident.timestamp_label), "inline": true}),
        serde_json::json!({
            "name": "Verursacher",
            "value": format!("#{} {} ({})", non_empty(&incident.car_number_a), non_empty(&incident.driver_a), non_empty(&incident.class_a)),
            "inline": false
        }),
    ];
    if !incident.driver_b.trim().is_empty() {
        fields.push(serde_json::json!({
            "name": "Geschädigter",
            "value": format!("#{} {} ({})", non_empty(&incident.car_number_b), non_empty(&incident.driver_b), non_empty(&incident.class_b)),
            "inline": false
        }));
    }
    fields.push(serde_json::json!({"name": "Vorfall-Art", "value": non_empty(&incident.incident_type), "inline": false}));
    fields.push(serde_json::json!({
        "name": "Entscheidung",
        "value": non_empty(incident.decision.as_deref().unwrap_or("")),
        "inline": false
    }));
    if !incident.reasoning.trim().is_empty() {
        fields.push(serde_json::json!({"name": "Begründung", "value": incident.reasoning, "inline": false}));
    }

    let payload = serde_json::json!({
        "embeds": [{
            "title": format!("Vorfall #{} entschieden", incident.incident_number),
            "color": color,
            "fields": fields,
        }]
    });

    let client = reqwest::Client::new();
    client
        .post(webhook_url)
        .json(&payload)
        .send()
        .await?
        .error_for_status()?;
    Ok(())
}

fn non_empty(s: &str) -> String {
    if s.trim().is_empty() {
        "–".to_string()
    } else {
        s.to_string()
    }
}
