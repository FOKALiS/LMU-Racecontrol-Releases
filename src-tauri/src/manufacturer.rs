//! Hersteller-Erkennung für LMU-Fahrzeuge.
//!
//! Leitet aus `vehicleFilename` und `vehicleName` den Hersteller-Namen ab,
//! der für die Logo-Anzeige im Frontend verwendet wird.
//!
//! Die Logos liegen in `frontend/public/manufacturers/{name}.png`.
//!
//! Beispiel: `vehicleFilename = "397_26_BMW"` → `manufacturer = "bmw"`
//!           `vehicleName = "Aston Martin THOR Team 2026 #007:LM"` → `manufacturer = "aston_martin"`

/// Bekannte Hersteller und ihre vehicleFilename-Präfixe/Keywords (lowercase).
/// Format: (manufacturer_key, [keywords_in_vehicleFilename, ...])
const MANUFACTURER_MAP: &[(&str, &[&str])] = &[
    ("bmw", &["bmw"]),
    ("ferrari", &["ferrari"]),
    ("porsche", &["porsche", "911gt3r", "992s_pc"]),
    ("mercedes", &["mercedes", "amggt3"]),
    ("audi", &["audi"]),
    ("toyota", &["toyota", "toyot"]),
    ("peugeot", &["peugeot"]),
    ("alpine", &["alpine", "a424"]),
    ("cadillac", &["cadillac", "v-lmdh"]),
    ("lamborghini", &["lamborghini", "huracan", "sc63"]),
    ("mclaren", &["mclaren", "720sgt3", "720sgchallenge"]),
    ("ford", &["ford", "mustang"]),
    ("chevrolet", &["chevrolet", "c8r"]),
    ("aston_martin", &["aston", "valkyrie", "vantage"]),
    ("lexus", &["lexusrcf"]),
    ("corvette", &["corvette", "z06gt3r"]),
    ("oreca", &["oreca", "oreca_07"]),
    ("ligier", &["ligier", "jsp325"]),
    ("ginetta", &["ginetta", "g61evo"]),
    ("isotta", &["isotta"]),
    ("genesis", &["genesis", "gene", "gmr001"]),
    ("duqueine", &["duqueine", "d09lmp3"]),
    ("adess", &["adess", "ad25"]),
    ("nissan", &["nissan"]),
    ("honda", &["honda"]),
    ("mazda", &["mazda"]),
    ("dallara", &["dallara"]),
];

/// Team→Hersteller-Mapping für Teams, die keinen Hersteller im Namen tragen.
/// Format: (manufacturer_key, [team_keywords_in_vehicleName, ...])
const TEAM_MANUFACTURER_MAP: &[(&str, &[&str])] = &[
    // BMW-Teams
    ("bmw", &["wrt", "m team wrt"]),
    // Ferrari-Teams
    ("ferrari", &["af corse", "vista af corse"]),
    // Porsche-Teams
    ("porsche", &["pure rxcing", "clx"]),
    // Lamborghini-Teams
    ("lamborghini", &["iron dames"]),
    // McLaren-Teams
    ("mclaren", &["garage 59"]),
    // Aston Martin-Teams
    ("aston_martin", &["heart of racing", "thor"]),
    // Chevrolet/Corvette-Teams
    ("chevrolet", &["tf sport", "racing team turkey"]),
    // Oreca-Teams (LMP2)
    ("oreca", &["idec", "nielsen", "algarve pro", "apr", "tds", "inter europol", "rlr", "msport", "united autosports", "iron lynx - proton", "proton competition"]),
    // Cadillac-Teams
    ("cadillac", &["jota", "chip ganassi"]),
    // Toyota-Teams
    ("toyota", &["toyot"]),
    // Alpine-Teams
    ("alpine", &["signatech"]),
    // Ford-Teams
    ("ford", &["manthey", "m-sport"]),
    // Mercedes-Teams
    ("mercedes", &["akkodis", "getspeed", "winward"]),
];

/// Leitet den Hersteller-Key aus `vehicleFilename` und `vehicleName` ab.
///
/// Strategie:
/// 1. vehicleFilename auf bekannte Hersteller-Keywords prüfen
/// 2. Wenn nicht gefunden: vehicleName auf Hersteller-Namen prüfen
/// 3. Fallback: "unknown"
pub fn detect_manufacturer(vehicle_filename: &str, vehicle_name: &str) -> String {
    let vf_lower = vehicle_filename.to_lowercase();
    let vn_lower = vehicle_name.to_lowercase();

    // 1. vehicleFilename prüfen (z.B. "397_26_BMW" → "bmw")
    for (key, keywords) in MANUFACTURER_MAP {
        for kw in *keywords {
            if vf_lower.contains(kw) {
                return key.to_string();
            }
        }
    }

    // 2. vehicleName prüfen (z.B. "Aston Martin THOR Team 2026 #007:LM" → "aston")
    for (key, keywords) in MANUFACTURER_MAP {
        for kw in *keywords {
            if vn_lower.contains(kw) {
                return key.to_string();
            }
        }
    }

    // 3. Team→Hersteller-Mapping prüfen (z.B. "Garage 59" → "mclaren")
    for (key, team_keywords) in TEAM_MANUFACTURER_MAP {
        for kw in *team_keywords {
            if vn_lower.contains(kw) {
                return key.to_string();
            }
        }
    }

    // 4. Fallback: "unknown"
    "unknown".to_string()
}

/// Leitet das Fahrzeugmodell aus `vehicleFilename` ab.
/// Nutzt die LMU-Ordnernamen als Referenz.
///
/// Beispiel: `vehicleFilename = "397_26_BMW"` → `vehicle_model = "BMW M4 LMGT3"` (wenn bekannt)
/// Derzeit geben wir einen lesbaren Namen zurück, der aus vehicleFilename + vehicleName abgeleitet wird.
pub fn detect_vehicle_model(vehicle_filename: &str, vehicle_name: &str) -> String {
    // Versuche, das Fahrzeugmodell aus vehicleName zu extrahieren
    // vehicleName = "BMW M Team WRT 2026 #15:LM" → "BMW M Team WRT"
    // Besser: Später aus LMU-Ordnern mappen
    if let Some(end) = vehicle_name.find(" 20") {
        let model = vehicle_name[..end].trim().to_string();
        if !model.is_empty() {
            return model;
        }
    }
    // Fallback: vehicleName selbst
    if !vehicle_name.is_empty() {
        return vehicle_name.to_string();
    }
    vehicle_filename.to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bmw_from_filename() {
        assert_eq!(detect_manufacturer("397_26_BMW", "BMW GT3 Custom Team 2026 #397"), "bmw");
    }

    #[test]
    fn test_aston_martin_from_name() {
        assert_eq!(detect_manufacturer("007_26_THO73564855", "Aston Martin THOR Team 2026 #007:LM"), "aston_martin");
    }

    #[test]
    fn test_toyota_from_name() {
        assert_eq!(detect_manufacturer("7_26_TOYOT18793560", "Toyota Racing 2026 #7:LM"), "toyota");
    }

    #[test]
    fn test_ferrari_from_name() {
        assert_eq!(detect_manufacturer("", "Ferrari 499P 2023 #50"), "ferrari");
    }

    #[test]
    fn test_unknown() {
        assert_eq!(detect_manufacturer("", ""), "unknown");
    }
}