//! Liest die LMU-Tastenbelegung aus `keyboard.json` und übersetzt die
//! DirectInput-Scancodes (DIK) in Windows-Scancodes + extended-Flag für SendInput.
//!
//! Die LMU `keyboard.json` speichert die Scancodes im DirectInput-Format (DIK):
//! - Normale Tasten (R, F6, F11, Numpad, ...) haben Werte < 0xC7 (199)
//! - Extended-Tasten (Home, End, PageUp, PageDown, Insert, Delete) haben
//!   Werte im Bereich 0xC7–0xD3. Deren Windows-Scancode ist `Wert - 0x80`
//!   und sie benötigen das extended-Flag bei SendInput.

use serde::Deserialize;
use std::collections::HashMap;
use std::path::PathBuf;

/// Ein aufgelöster Tasten-Binding: Windows-Scancode + extended-Flag.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct KeyBinding {
    pub scan: u16,
    pub extended: bool,
}

/// Die vollständige Tastenbelegung, aus der `keyboard.json` geladen.
#[derive(Debug, Clone)]
pub struct KeyboardConfig {
    /// Aktion (z.B. "Onboard Cameras") → Windows-Binding
    bindings: HashMap<String, KeyBinding>,
    /// Aktion (z.B. "Onboard Cameras") → lesbarer Tastenname (z.B. "Insert")
    key_names: HashMap<String, String>,
}

impl Default for KeyboardConfig {
    fn default() -> Self {
        Self {
            bindings: HashMap::new(),
            key_names: HashMap::new(),
        }
    }
}

impl KeyboardConfig {
    /// Standard-Fallbacks, falls die `keyboard.json` nicht geladen werden kann.
    /// Basierend auf der LMU-Standardbelegung (Preset).
    pub fn with_defaults() -> Self {
        let mut config = Self::default();
        // Aktion → (DIK-Scancode) – basierend auf LMU keyboard.json
        let defaults: &[(&str, u16)] = &[
            ("Tracking Cameras", 0xD1),     // PageDown (TV - Verfolger)
            ("Driving Cameras", 0xD2),      // Insert (Bord - Fahrkameras)
            ("Swingman Camera", 0xC9),      // PageUp (Heck - Schwenkkopf)
            ("Swingman Zoom In", 0x47),     // KP7 (Zoom+)
            ("Swingman Zoom Out", 0x49),    // KP9 (Zoom-)
            ("Instant Replay", 0x13),       // R
            ("Replay Play", 0x57),          // F11
            ("Replay Stop", 0x40),          // F6
            ("Replay Slowmotion", 0x44),    // F10
            ("Replay Fast Forward", 0x43),  // F9
            ("Replay Fast Rewind", 0x42),   // F8
            ("Replay Reverse", 0x41),       // F7
        ];
        for (action, dik) in defaults {
            config.set(action, *dik);
        }
        config
    }

    /// Liest die `keyboard.json` aus dem LMU-Installations-Pfad.
    /// Fallback auf die Standardbelegung, falls die Datei fehlt oder ungültig ist.
    pub fn load_from(lmu_install_path: &str) -> Self {
        let base = PathBuf::from(lmu_install_path);
        let path = base
            .join("UserData")
            .join("player")
            .join("keyboard.json");

        match std::fs::read_to_string(&path) {
            Ok(content) => match serde_json::from_str::<LmuKeyboardFile>(&content) {
                Ok(file) => {
                    let mut config = Self::default();
                    for (action, dik) in file.input {
                        config.set(&action, dik);
                    }
                    // Sicherstellen, dass alle benötigten Aktionen existieren
                    config.ensure_defaults();
                    eprintln!(
                        "[keyboard_config] Erfolgreich geladen: {}",
                        path.display()
                    );
                    config
                }
                Err(e) => {
                    eprintln!("[keyboard_config] Parse-Fehler in {}: {}", path.display(), e);
                    Self::with_defaults()
                }
            },
            Err(e) => {
                eprintln!(
                    "[keyboard_config] Konnte {} nicht lesen: {} – nutze Standardbelegung",
                    path.display(),
                    e
                );
                Self::with_defaults()
            }
        }
    }

    /// Setzt eine einzelne Aktion (DIK-Scancode → Windows-Binding).
    fn set(&mut self, action: &str, dik: u16) {
        let binding = dik_to_windows(dik);
        self.bindings.insert(action.to_string(), binding);
        self.key_names.insert(action.to_string(), dik_to_key_name(dik).to_string());
    }

    /// Ergänzt fehlende Aktionen mit den Standardwerten.
    fn ensure_defaults(&mut self) {
        let defaults = Self::with_defaults();
        for (action, binding) in defaults.bindings {
            self.bindings.entry(action.clone()).or_insert(binding);
        }
        for (action, name) in defaults.key_names {
            self.key_names.entry(action.clone()).or_insert(name);
        }
    }

    /// Gibt das Windows-Binding für eine Aktion zurück.
    pub fn get(&self, action: &str) -> Option<KeyBinding> {
        self.bindings.get(action).copied()
    }

    /// Gibt den lesbaren Tastennamen für eine Aktion zurück.
    pub fn key_name(&self, action: &str) -> String {
        self.key_names
            .get(action)
            .cloned()
            .unwrap_or_else(|| "?".to_string())
    }

    /// Gibt alle relevanten Bindings als sortierte Liste zurück (fürs Frontend).
    pub fn relevant_bindings(&self) -> Vec<(String, KeyBinding, String)> {
        let relevant = [
            "Tracking Cameras",
            "Driving Cameras",
            "Swingman Camera",
            "Swingman Zoom In",
            "Swingman Zoom Out",
            "Instant Replay",
            "Replay Play",
            "Replay Stop",
            "Replay Slowmotion",
            "Replay Fast Forward",
            "Replay Fast Rewind",
            "Replay Reverse",
        ];
        relevant
            .iter()
            .filter_map(|a| {
                self.bindings.get(*a).map(|b| {
                    (
                        a.to_string(),
                        *b,
                        self.key_names
                            .get(*a)
                            .cloned()
                            .unwrap_or_else(|| "?".to_string()),
                    )
                })
            })
            .collect()
    }
}

/// Parst die Struktur der LMU `keyboard.json`.
#[derive(Debug, Deserialize)]
struct LmuKeyboardFile {
    #[serde(rename = "Input")]
    input: HashMap<String, u16>,
}

/// Übersetzt einen DirectInput-Scancode (DIK) in ein Windows-Binding.
fn dik_to_windows(dik: u16) -> KeyBinding {
    // Extended-Tasten (Home, End, PageUp, PageDown, Insert, Delete)
    if (0xC7..=0xD3).contains(&dik) {
        KeyBinding {
            scan: dik - 0x80,
            extended: true,
        }
    } else {
        KeyBinding {
            scan: dik,
            extended: false,
        }
    }
}

/// Übersetzt einen DirectInput-Scancode (DIK) in einen lesbaren Tastennamen.
fn dik_to_key_name(dik: u16) -> &'static str {
    match dik {
        0x02 => "1",
        0x03 => "2",
        0x04 => "3",
        0x05 => "4",
        0x06 => "5",
        0x07 => "6",
        0x08 => "7",
        0x09 => "8",
        0x0A => "9",
        0x0B => "0",
        0x0E => "Backspace",
        0x0F => "Tab",
        0x10 => "Q",
        0x11 => "W",
        0x12 => "E",
        0x13 => "R",
        0x14 => "T",
        0x15 => "Z",
        0x16 => "U",
        0x17 => "I",
        0x18 => "O",
        0x19 => "P",
        0x1A => "Ü",
        0x1B => "+",
        0x1C => "Enter",
        0x1E => "A",
        0x1F => "S",
        0x20 => "D",
        0x21 => "F",
        0x22 => "G",
        0x23 => "H",
        0x24 => "J",
        0x25 => "K",
        0x26 => "L",
        0x27 => "Ö",
        0x28 => "Ä",
        0x29 => "^",
        0x2B => "#",
        0x2C => "Y",
        0x2D => "X",
        0x2E => "C",
        0x2F => "V",
        0x30 => "B",
        0x31 => "N",
        0x32 => "M",
        0x33 => ",",
        0x34 => ".",
        0x35 => "-",
        0x39 => "Space",
        0x3B => "F1",
        0x3C => "F2",
        0x3D => "F3",
        0x3E => "F4",
        0x3F => "F5",
        0x40 => "F6",
        0x41 => "F7",
        0x42 => "F8",
        0x43 => "F9",
        0x44 => "F10",
        0x45 => "F11", // SCROLL LOCK hat DIK 0x46, F11 = 0x57
        0x47 => "KP7",
        0x48 => "KP8",
        0x49 => "KP9",
        0x4B => "KP4",
        0x4C => "KP5",
        0x4D => "KP6",
        0x4F => "KP1",
        0x50 => "KP2",
        0x51 => "KP3",
        0x52 => "KP0",
        0x53 => "KP.",
        0x57 => "F11",
        0x58 => "F12",
        0xC7 => "Home",
        0xC9 => "PageUp",
        0xCF => "End",
        0xD1 => "PageDown",
        0xD2 => "Insert",
        0xD3 => "Delete",
        _ => "?",
    }
}
