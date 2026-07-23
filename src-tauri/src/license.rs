//! Lizenzprüfung über die Keygen License-API (https://keygen.sh).
//!
//! Verwendet die Konto-ID (UUID) statt des Slugs - laut Keygens eigenen
//! Sicherheitshinweisen die robustere Wahl, da die ID sich nie ändert (der
//! Slug könnte später umbenannt werden und würde dann alle Apps aussperren).
const KEYGEN_ACCOUNT: &str = "65f997d1-bac7-4b1c-b37d-35fce549bde6";

/// Wie viele Tage die App nach der letzten erfolgreichen Online-Prüfung noch
/// OHNE Internet weiterläuft, bevor sie sich wieder sperrt. Verhindert, dass
/// ein Rennwochenende mit schlechtem Internet die Kommission aussperrt.
const OFFLINE_GRACE_DAYS: i64 = 14;

use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::path::PathBuf;

fn api_base() -> String {
    format!("https://api.keygen.sh/v1/accounts/{KEYGEN_ACCOUNT}")
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct LicenseData {
    pub license_key: String,
    /// Keygen-interne Lizenz-ID (UUID), wird bei der ersten Aktivierung ermittelt.
    pub license_id: String,
    /// Eindeutige Kennung DIESER Installation - einmalig zufällig erzeugt und
    /// dauerhaft gespeichert, NICHT bei jedem Start neu (sonst würde jeder
    /// Start als neues Gerät zählen und die Aktivierungen aufbrauchen).
    pub fingerprint: String,
    pub valid: bool,
    pub last_validated_at: Option<DateTime<Utc>>,
    /// Klartext-Grund, warum eine Prüfung zuletzt fehlgeschlagen ist - wird
    /// im UI angezeigt, damit der Nutzer weiß, woran es liegt.
    pub last_error: Option<String>,
}

impl LicenseData {
    pub fn has_key(&self) -> bool {
        !self.license_key.is_empty()
    }

    /// Ob die App aktuell freigeschaltet sein soll: entweder frisch online
    /// bestätigt gültig, ODER innerhalb der Offline-Kulanzfrist seit der
    /// letzten erfolgreichen Prüfung.
    pub fn is_currently_licensed(&self) -> bool {
        if !self.has_key() {
            return false;
        }
        if self.valid {
            return true;
        }
        match self.last_validated_at {
            Some(last) => Utc::now().signed_duration_since(last).num_days() < OFFLINE_GRACE_DAYS,
            None => false,
        }
    }
}

pub struct LicenseStore {
    path: PathBuf,
}

impl LicenseStore {
    pub fn new(app_dir: &std::path::Path) -> Self {
        Self {
            path: app_dir.join("license.json"),
        }
    }

    pub fn load(&self) -> LicenseData {
        std::fs::read_to_string(&self.path)
            .ok()
            .and_then(|s| serde_json::from_str(&s).ok())
            .unwrap_or_default()
    }

    pub fn save(&self, data: &LicenseData) -> Result<()> {
        std::fs::write(&self.path, serde_json::to_string_pretty(data)?)?;
        Ok(())
    }
}

#[derive(Debug, Deserialize)]
struct ValidateMeta {
    valid: bool,
    code: String,
    detail: Option<String>,
}
#[derive(Debug, Deserialize)]
struct ResourceRef {
    id: String,
}
#[derive(Debug, Deserialize)]
struct ValidateResponse {
    meta: ValidateMeta,
    data: Option<ResourceRef>,
}

/// Fragt bei Keygen den Status eines Lizenzschlüssels ab. `fingerprint`
/// optional mitgeben, um zu prüfen, ob die Lizenz FÜR DIESES GERÄT
/// freigeschaltet ist (nicht nur grundsätzlich gültig).
async fn validate_key(license_key: &str, fingerprint: Option<&str>) -> Result<ValidateResponse> {
    let client = reqwest::Client::new();
    let mut meta = json!({ "key": license_key });
    if let Some(fp) = fingerprint {
        meta["scope"] = json!({ "fingerprint": fp });
    }
    let body = json!({ "meta": meta });

    let resp: ValidateResponse = client
        .post(format!("{}/licenses/actions/validate-key", api_base()))
        .header("Content-Type", "application/vnd.api+json")
        .header("Accept", "application/vnd.api+json")
        .json(&body)
        .send()
        .await
        .context("Konnte Keygen nicht erreichen (Internetverbindung prüfen)")?
        .json()
        .await
        .context("Unerwartete Antwort von Keygen")?;
    Ok(resp)
}

/// Registriert dieses Gerät (per Fingerprint) bei der Lizenz. Nutzt den
/// Lizenzschlüssel selbst als Berechtigungsnachweis (setzt voraus, dass die
/// Policy in Keygen auf Authentifizierungsstrategie "License" oder "Mixed"
/// steht - siehe Einrichtungs-Anleitung).
async fn activate_machine(license_key: &str, license_id: &str, fingerprint: &str, name: &str) -> Result<()> {
    let client = reqwest::Client::new();
    let body = json!({
        "data": {
            "type": "machines",
            "attributes": { "fingerprint": fingerprint, "name": name },
            "relationships": {
                "license": { "data": { "type": "licenses", "id": license_id } }
            }
        }
    });

    let resp = client
        .post(format!("{}/machines", api_base()))
        .header("Content-Type", "application/vnd.api+json")
        .header("Accept", "application/vnd.api+json")
        .header("Authorization", format!("License {license_key}"))
        .json(&body)
        .send()
        .await
        .context("Konnte Keygen nicht erreichen")?;

    // 201 = neu aktiviert. 409 = dieses Gerät war schon aktiviert -> auch ok.
    if resp.status().is_success() || resp.status().as_u16() == 409 {
        Ok(())
    } else {
        let status = resp.status();
        let text = resp.text().await.unwrap_or_default();
        anyhow::bail!("Geräte-Aktivierung fehlgeschlagen ({status}): {text}")
    }
}

fn friendly_error(code: &str, detail: Option<&str>) -> String {
    let base = match code {
        "EXPIRED" => "Diese Lizenz ist abgelaufen.",
        "SUSPENDED" => "Diese Lizenz wurde gesperrt.",
        "NOT_FOUND" => "Dieser Lizenzschlüssel wurde nicht gefunden. Bitte prüfen.",
        "TOO_MANY_MACHINES" => "Diese Lizenz ist bereits auf der maximalen Anzahl Geräte aktiviert.",
        "FINGERPRINT_SCOPE_MISMATCH" | "NO_MACHINE" | "NO_MACHINES" | "FINGERPRINT_SCOPE_REQUIRED" => {
            "Dieses Gerät ist für diese Lizenz noch nicht freigeschaltet."
        }
        _ => "Lizenz ist ungültig.",
    };
    match detail {
        Some(d) if !d.is_empty() => format!("{base} ({d})"),
        _ => base.to_string(),
    }
}

/// Codes, bei denen die Lizenz an sich in Ordnung ist, nur für DIESES
/// Gerät (diesen Fingerprint) noch keine Aktivierung existiert - in diesem
/// Fall versuchen wir eine Geräte-Aktivierung, statt sofort abzubrechen.
fn needs_machine_activation(code: &str) -> bool {
    matches!(
        code,
        "NO_MACHINE" | "NO_MACHINES" | "FINGERPRINT_SCOPE_MISMATCH" | "FINGERPRINT_SCOPE_REQUIRED"
    )
}

/// Aktiviert einen neu eingegebenen Lizenzschlüssel für dieses Gerät.
/// `existing_fingerprint`: falls schon eine Fingerprint-ID für diese
/// Installation gespeichert war (z.B. erneute Aktivierung eines neuen
/// Schlüssels auf demselben Rechner), wird sie wiederverwendet statt eine
/// neue Geräte-Aktivierung zu verbrauchen.
pub async fn activate(
    license_key: &str,
    device_name: &str,
    existing_fingerprint: Option<String>,
) -> Result<LicenseData> {
    let license_key = license_key.trim();
    let fingerprint = existing_fingerprint.unwrap_or_else(|| uuid::Uuid::new_v4().to_string());

    // WICHTIG: Die Policy hat "Require Fingerprint Scope" aktiviert - jede
    // Validierungsanfrage MUSS deshalb von Anfang an einen Fingerprint
    // mitschicken, sonst weist Keygen die Anfrage direkt zurück. Der volle
    // Lizenz-Datensatz (inkl. license_id) wird trotzdem immer mitgeliefert,
    // auch wenn die Prüfung selbst noch "ungültig" zurückgibt.
    let first = validate_key(license_key, Some(&fingerprint)).await?;
    let license_id = first.data.map(|d| d.id).context("Keine Lizenz-ID in der Antwort")?;

    if !first.meta.valid {
        if !needs_machine_activation(&first.meta.code) {
            anyhow::bail!(friendly_error(&first.meta.code, first.meta.detail.as_deref()));
        }

        // Dieses Gerät ist noch nicht bei der Lizenz registriert -> jetzt tun
        activate_machine(license_key, &license_id, &fingerprint, device_name).await?;

        // Final bestätigen, dass die Lizenz jetzt für DIESES Gerät gültig ist
        let confirm = validate_key(license_key, Some(&fingerprint)).await?;
        if !confirm.meta.valid {
            anyhow::bail!(friendly_error(&confirm.meta.code, confirm.meta.detail.as_deref()));
        }
    }

    Ok(LicenseData {
        license_key: license_key.to_string(),
        license_id,
        fingerprint,
        valid: true,
        last_validated_at: Some(Utc::now()),
        last_error: None,
    })
}

/// Prüft eine bereits aktivierte Lizenz erneut (App-Start, regelmäßig im
/// Hintergrund).
pub async fn revalidate(license_key: &str, fingerprint: &str) -> Result<()> {
    let resp = validate_key(license_key, Some(fingerprint)).await?;
    if !resp.meta.valid {
        anyhow::bail!(friendly_error(&resp.meta.code, resp.meta.detail.as_deref()));
    }
    Ok(())
}
