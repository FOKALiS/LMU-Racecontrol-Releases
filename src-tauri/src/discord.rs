//! Sendet eine Discord-Embed-Nachricht, sobald ein Vorfall von der
//! Rennkommission entschieden und archiviert wurde. Nutzt einen normalen
//! Discord-Webhook (Server-Einstellungen -> Integrationen -> Webhooks).
//!
//! Format der Nachricht (Farbe #b66a00 = 11958784):
//!
//! Vorfall #1
//! Zum Vorfall Reko gegen #Carnumber im Rennen, Runde x - Kurve x, hat die
//! Rennkommission wie folgt entschieden:
//!
//! Vorwurf: <incident_type>
//! Strafe: <decision>
//! Strafpunkte: <penalty_points>
//! Verwarnpunkte: <warning_points>
//!
//! Begründung:
//! <reasoning>

use crate::db::Incident;
use anyhow::Result;

pub async fn send_incident_decision(webhook_url: &str, incident: &Incident) -> Result<()> {
    let webhook_url = webhook_url.trim();
    if webhook_url.is_empty() {
        return Ok(());
    }

    // Farbe #b66a00 = 11958784
    let color = 0xB66A00;

    // Beschreibungstext
    let car_number = if !incident.car_number_a.is_empty() {
        &incident.car_number_a
    } else {
        "??"
    };
    let driver = if !incident.driver_a.is_empty() {
        &incident.driver_a
    } else {
        "Unbekannt"
    };
    let description = format!(
        "Zum Vorfall Rennkommission gegen **{}** im Rennen, Runde **{}** - Kurve **{}**, hat die Rennkommission wie folgt entschieden:\n\n\
        **Vorwurf:** {}\n\
        **Strafe:** {}\n\
        **Verwarnpunkte:** {}\n\
        **Strafpunkte:** {}\n\n\
        **Begründung:**\n{}",
        driver,
        incident.lap,
        if incident.corner.trim().is_empty() { "N.A." } else { &incident.corner },
        non_empty(&incident.incident_type),
        non_empty(incident.decision.as_deref().unwrap_or("")),
        incident.warning_points,
        incident.penalty_points,
        if incident.reasoning.trim().is_empty() { "–" } else { &incident.reasoning },
    );

    let payload = serde_json::json!({
        "embeds": [{
            "title": format!("Vorfall #{}", incident.incident_number),
            "color": color,
            "description": description,
            "footer": {
                "text": "Bei Fragen zur Entscheidung der Rennkommission öffnet im Ticketsystem bitte ein Ticket. Um einen Einspruch einzulegen (1 x pro Saison) nutzt bitte unsere Website „Strafantrag / Einspruch“."
            }
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