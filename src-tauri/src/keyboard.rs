//! Tastatursimulation für die LMU-Kamera-Steuerung und Fahrzeug-Fokus.
//!
//! LMU (Le Mans Ultimate) / rFactor2 verwendet standardmäßig die F1-F6 Tasten
//! zum Umschalten zwischen Kameraperspektiven. Da die LMU-REST-API KEINEN
//! Endpunkt für die Kamera-Steuerung bietet, simulieren wir die entsprechenden
//! Tastendrücke via `enigo` crate.
//!
//! ## Tastenbelegung (rFactor2/LMU-Standard)
//! - F1 = TV/Broadcast Cam
//! - F2 = Helmet Cam
//! - F3 = Front (Bumper) Cam
//! - F4 = Rear (Chase) Cam
//! - F5 = Top/Bonnet Cam
//! - F6 = Behind/Free Cam
//!
//! ## Fahrzeug-Fokus
//! Strg+F öffnet den Fahrzeug-Fokus-Dialog, dann wird die Fahrzeugnummer
//! eingegeben und mit Enter bestätigt.

use enigo::{
    Direction, Enigo, Key, Keyboard, Settings,
};

/// Schaltet die LMU-Kamera auf die angegebene Kamera-ID um.
pub fn switch_camera(cam_id: &str) -> Result<(), String> {
    let key = match cam_id {
        "TV" => Key::F1,
        "Helmet" => Key::F2,
        "Front" => Key::F3,
        "Heck" | "Rear" => Key::F4,
        "Top" => Key::F5,
        "Behind" => Key::F6,
        _ => return Err(format!(
            "Unbekannte Kamera-ID: {}. Gültig: TV, Helmet, Front, Heck, Top, Behind",
            cam_id
        )),
    };

    let mut enigo = Enigo::new(&Settings::default()).map_err(|e| format!("Enigo-Init fehlgeschlagen: {}", e))?;

    // Taste drücken und loslassen
    enigo.key(key, Direction::Click)
        .map_err(|e| format!("Tastendruck fehlgeschlagen: {}", e))?;

    Ok(())
}

/// Fokussiert die Kamera auf ein bestimmtes Fahrzeug (über Strg+F + Fahrzeugnummer + Enter).
/// Das ist die Standard-Methode in rFactor 2 / LMU, um ein bestimmtes Fahrzeug zu fokussieren.
pub fn focus_car(car_number: &str) -> Result<(), String> {
    let mut enigo = Enigo::new(&Settings::default()).map_err(|e| format!("Enigo-Init fehlgeschlagen: {}", e))?;

    // 1) Strg+F für Fahrzeug-Fokus-Dialog
    enigo.key(Key::Control, Direction::Press)
        .map_err(|e| format!("Strg drücken fehlgeschlagen: {}", e))?;
    std::thread::sleep(std::time::Duration::from_millis(20));
    enigo.key(Key::Unicode('f'), Direction::Click)
        .map_err(|e| format!("F drücken fehlgeschlagen: {}", e))?;
    std::thread::sleep(std::time::Duration::from_millis(20));
    enigo.key(Key::Control, Direction::Release)
        .map_err(|e| format!("Strg loslassen fehlgeschlagen: {}", e))?;

    // Kurz warten, bis der Fokus-Dialog erscheint
    std::thread::sleep(std::time::Duration::from_millis(300));

    // 2) Fahrzeugnummer eingeben (Zeichen für Zeichen)
    for c in car_number.chars() {
        enigo.key(Key::Unicode(c), Direction::Click)
            .map_err(|e| format!("Zeichen '{}' eingeben fehlgeschlagen: {}", c, e))?;
        std::thread::sleep(std::time::Duration::from_millis(30));
    }

    // 3) Enter drücken
    std::thread::sleep(std::time::Duration::from_millis(50));
    enigo.key(Key::Return, Direction::Click)
        .map_err(|e| format!("Enter drücken fehlgeschlagen: {}", e))?;

    Ok(())
}