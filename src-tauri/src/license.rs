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

/// Lizenz-Tier (abgestufte Version)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum LicenseTier {
    #[serde(rename = "demo")]
    Demo,
    #[serde(rename = "basic")]
    Basic,
    #[serde(rename = "enterprise_l")]
    EnterpriseL,
    #[serde(rename = "enterprise_xl")]
    EnterpriseXl,
}

impl LicenseTier {
    pub fn as_str(&self) -> &'static str {
        match self {
            LicenseTier::Demo => "Demo",
            LicenseTier::Basic => "Basic",
            LicenseTier::EnterpriseL => "Enterprise L",
            LicenseTier::EnterpriseXl => "Enterprise XL",
        }
    }

    /// Ob dieser Tier Server-Zugriff erlaubt
    pub fn allows_server(&self) -> bool {
        matches!(self, LicenseTier::EnterpriseL | LicenseTier::EnterpriseXl)
    }

    /// Maximale Anzahl gleichzeitiger User (nur für Server-Tiers relevant)
    pub fn max_users(&self) -> Option<i32> {
        match self {
            LicenseTier::EnterpriseL => Some(3),
            LicenseTier::EnterpriseXl => Some(5),
            _ => None,
        }
    }
}

impl Default for LicenseTier {
    fn default() -> Self {
        LicenseTier::Basic
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct LicenseData {
    pub license_key: String,
    /// Keygen-interne Lizenz-ID (UUID)
    pub license_id: String,
    /// Eindeutige Kennung DIESER Installation
    pub fingerprint: String,
    pub valid: bool,
    pub last_validated_at: Option<DateTime<Utc>>,
    /// Klartext-Grund, warum eine Prüfung zuletzt fehlgeschlagen ist
    pub last_error: Option<String>,
    /// Lizenz-Tier: "demo", "basic", "enterprise_l", "enterprise_xl"
    pub tier: LicenseTier,
}

impl LicenseData {
    pub fn has_key(&self) -> bool {
        !self.license_key.is_empty()
    }

    /// Ob die App aktuell freigeschaltet sein soll
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

    /// Zeigt an, ob der Lizenz-Tier Server-Zugriff erlaubt
    pub fn is_enterprise(&self) -> bool {
        self.tier.allows_server()
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
struct LicenseAttributes {
    #[serde(default)]
    metadata: Option<serde_json::Value>,
}
#[derive(Debug, Deserialize)]
struct LicenseDataObj {
    id: String,
    attributes: LicenseAttributes,
}
/// Keygen liefert Metadata als separate "included"-Ressourcen
#[derive(Debug, Deserialize)]
struct IncludedResource {
    #[serde(rename = "type")]
    resource_type: String,
    attributes: serde_json::Value,
}
#[derive(Debug, Deserialize)]
struct ValidateResponse {
    meta: ValidateMeta,
    data: Option<LicenseDataObj>,
    #[serde(default)]
    included: Vec<IncludedResource>,
}

impl ValidateResponse {
    /// Holt die Metadata aus dem "included"-Array (Keygen Standard).
    /// Wenn nicht vorhanden, fällt es zurück auf data.attributes.metadata.
    fn metadata(&self) -> Option<&serde_json::Value> {
        // 1) Versuche "included" (Keygen Standard für Metadata)
        for inc in &self.included {
            if inc.resource_type == "metadata" {
                if let Some(value) = inc.attributes.get("value") {
                    // Keygen Metadata kann entweder flach sein:
                    //   { "key": "tier", "value": "enterprise_xl" }
                    // oder verschachtelt:
                    //   { "tier": "enterprise_xl" }
                    // Wir versuchen beides zu lesen.
                    return Some(&inc.attributes);
                }
            }
        }
        // 2) Fallback: data.attributes.metadata
        self.data.as_ref().and_then(|d| d.attributes.metadata.as_ref())
    }

    fn tier_from_metadata(&self) -> LicenseTier {
        match self.metadata() {
            Some(m) => {
                // Debug
                println!("[LICENSE] Metadata-Wert: {}", m);
                // Format 1: { "key": "tier", "value": "enterprise_xl" }
                if let Some(key) = m.get("key").and_then(|v| v.as_str()) {
                    if key == "tier" {
                        let value = m.get("value").and_then(|v| v.as_str());
                        if let Some(v) = value {
                            return match v {
                                "demo" => LicenseTier::Demo,
                                "basic" => LicenseTier::Basic,
                                "enterprise_l" => LicenseTier::EnterpriseL,
                                "enterprise_xl" => LicenseTier::EnterpriseXl,
                                _ => LicenseTier::Basic,
                            };
                        }
                    }
                }
                // Format 2: { "attributes": { "key": "tier", "value": "..." } }
                if let Some(attrs) = m.get("attributes") {
                    if let Some(key) = attrs.get("key").and_then(|v| v.as_str()) {
                        if key == "tier" {
                            let value = attrs.get("value").and_then(|v| v.as_str());
                            if let Some(v) = value {
                                return match v {
                                    "demo" => LicenseTier::Demo,
                                    "basic" => LicenseTier::Basic,
                                    "enterprise_l" => LicenseTier::EnterpriseL,
                                    "enterprise_xl" => LicenseTier::EnterpriseXl,
                                    _ => LicenseTier::Basic,
                                };
                            }
                        }
                    }
                }
                // Format 3: Flach: { "tier": "enterprise_xl" }
                extract_tier(Some(m))
            }
            None => LicenseTier::Basic,
        }
    }
}

/// Extrahiert den Tier aus dem Keygen-Metadata-Feld.
/// Erwartet: `metadata.tier` als String ("demo", "basic", "enterprise_l", "enterprise_xl")
fn extract_tier(metadata: Option<&serde_json::Value>) -> LicenseTier {
    match metadata {
        Some(m) => {
            match m.get("tier").and_then(|v| v.as_str()) {
                Some("demo") => LicenseTier::Demo,
                Some("enterprise_l") => LicenseTier::EnterpriseL,
                Some("enterprise_xl") => LicenseTier::EnterpriseXl,
                _ => LicenseTier::Basic, // Default: Basic
            }
        }
        None => LicenseTier::Basic,
    }
}

/// Fragt bei Keygen den Status eines Lizenzschlüssels ab
async fn validate_key(license_key: &str, fingerprint: Option<&str>) -> Result<ValidateResponse> {
    let client = reqwest::Client::new();
    let mut meta = json!({ "key": license_key });
    if let Some(fp) = fingerprint {
        meta["scope"] = json!({ "fingerprint": fp });
    }
    let body = json!({ "meta": meta });

    // WICHTIG: Include muss "metadata" enthalten, damit wir den Tier lesen können.
    // In Keygen ist metadata ein spezielles Attribut, das über ?include=metadata
    // in der Response mitgeliefert wird.
    let url = format!("{}/licenses/actions/validate-key", api_base());
    let client = reqwest::Client::new();
    let resp_builder = client
        .post(&url)
        .query(&[("include", "metadata")])
        .header("Content-Type", "application/vnd.api+json")
        .header("Accept", "application/vnd.api+json")
        .json(&body);

    // Debug: Roh-Antwort loggen (nur im Dev-Modus)
    let resp_text = resp_builder
        .send()
        .await
        .context("Konnte Keygen nicht erreichen (Internetverbindung prüfen)")?
        .text()
        .await
        .context("Konnte Antworttext nicht lesen")?;

    // Debug-Log
    println!("[LICENSE] Keygen-Antwort: {}", &resp_text[..resp_text.len().min(2000)]);

    let resp: ValidateResponse = serde_json::from_str(&resp_text)
        .context("Unerwartete Antwort von Keygen")?;

    // Debug: Tier aus Metadata loggen
    if let Some(ref data) = resp.data {
        if let Some(ref meta) = data.attributes.metadata {
            println!("[LICENSE] Metadata gefunden: {}", meta);
            if let Some(tier) = meta.get("tier").and_then(|v| v.as_str()) {
                println!("[LICENSE] Tier aus Metadata: {}", tier);
            } else {
                println!("[LICENSE] Kein 'tier' in Metadata gefunden!");
            }
        } else {
            println!("[LICENSE] Keine Metadata in der Antwort!");
        }
    }

    Ok(resp)
}

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

fn needs_machine_activation(code: &str) -> bool {
    matches!(
        code,
        "NO_MACHINE" | "NO_MACHINES" | "FINGERPRINT_SCOPE_MISMATCH" | "FINGERPRINT_SCOPE_REQUIRED"
    )
}

/// Aktiviert einen neu eingegebenen Lizenzschlüssel für dieses Gerät
pub async fn activate(
    license_key: &str,
    device_name: &str,
    existing_fingerprint: Option<String>,
) -> Result<LicenseData> {
    let license_key = license_key.trim();
    let fingerprint = existing_fingerprint.unwrap_or_else(|| uuid::Uuid::new_v4().to_string());

    let first = validate_key(license_key, Some(&fingerprint)).await?;
    let license_id = first.data.as_ref().map(|d| d.id.clone()).context("Keine Lizenz-ID in der Antwort")?;

    // Tier aus Metadata extrahieren (via included oder data.attributes.metadata)
    let tier = first.tier_from_metadata();
    println!("[LICENSE] Extrahierter Tier: {:?}", tier);

    if !first.meta.valid {
        if !needs_machine_activation(&first.meta.code) {
            anyhow::bail!(friendly_error(&first.meta.code, first.meta.detail.as_deref()));
        }

        activate_machine(license_key, &license_id, &fingerprint, device_name).await?;

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
        tier,
    })
}

/// Prüft eine bereits aktivierte Lizenz erneut (App-Start, regelmäßig im Hintergrund).
pub async fn revalidate(license_key: &str, fingerprint: &str) -> Result<()> {
    let resp = validate_key(license_key, Some(fingerprint)).await?;
    if !resp.meta.valid {
        anyhow::bail!(friendly_error(&resp.meta.code, resp.meta.detail.as_deref()));
    }
    Ok(())
}

/// Deaktiviert DIESE Maschine (Fingerprint) bei Keygen.
/// Nützlich bei Rechnerwechsel – der User kann die Lizenz am alten Rechner
/// freigeben, um sie auf einem neuen Rechner zu aktivieren.
/// Ruft die Keygen Machine-API auf, um die aktuelle Maschine zu löschen.
pub async fn deactivate_machine(license_key: &str, license_id: &str, fingerprint: &str) -> Result<()> {
    let client = reqwest::Client::new();

    // 1) Maschinen-Liste für diese Lizenz abrufen
    let machines_url = format!("{}/licenses/{}/machines", api_base(), license_id);
    let machines_resp = client
        .get(&machines_url)
        .header("Accept", "application/vnd.api+json")
        .header("Authorization", format!("License {license_key}"))
        .send()
        .await
        .context("Konnte Maschinen-Liste nicht abrufen")?;

    let machines_text = machines_resp.text().await.unwrap_or_default();
    println!("[LICENSE] Maschinen-Antwort: {}", &machines_text[..machines_text.len().min(500)]);

    // 2) Maschinen parsen und nach Fingerprint suchen
    #[derive(Deserialize)]
    struct MachineListResponse {
        data: Vec<MachineItem>,
    }
    #[derive(Deserialize)]
    struct MachineItem {
        id: String,
        attributes: MachineAttributes,
    }
    #[derive(Deserialize)]
    struct MachineAttributes {
        #[serde(default)]
        fingerprint: Option<String>,
    }

    let machines: MachineListResponse = serde_json::from_str(&machines_text)
        .context("Konnte Maschinen-Liste nicht parsen")?;

    let machine_id = machines.data.iter()
        .find(|m| m.attributes.fingerprint.as_deref() == Some(fingerprint))
        .map(|m| m.id.clone());

    if let Some(machine_id) = machine_id {
        // 3) Maschine löschen
        let delete_url = format!("{}/machines/{}", api_base(), machine_id);
        let delete_resp = client
            .delete(&delete_url)
            .header("Accept", "application/vnd.api+json")
            .header("Authorization", format!("License {license_key}"))
            .send()
            .await
            .context("Konnte Maschine nicht löschen")?;

        if delete_resp.status().is_success() || delete_resp.status().as_u16() == 404 {
            println!("[LICENSE] ✅ Maschine {} erfolgreich deregistriert", machine_id);
            return Ok(());
        } else {
            let status = delete_resp.status();
            let text = delete_resp.text().await.unwrap_or_default();
            anyhow::bail!("Maschine konnte nicht gelöscht werden ({status}): {text}");
        }
    } else {
        // Keine Maschine mit unserem Fingerprint gefunden – das ist ok
        println!("[LICENSE] ⚠️ Keine Maschine mit Fingerprint {} gefunden – bereits deregistriert?", fingerprint);
        Ok(())
    }
}
