//! Tastatursimulation für die LMU-Kamera-Steuerung und Fahrzeug-Fokus.
//!
//! Verwendet `PostMessageW` mit WM_KEYDOWN/WM_KEYUP, um Tasten direkt
//! an das LMU-Fenster zu senden – **ohne Fenster-Fokus**.
//!
//! LMU verarbeitet WM_KEYDOWN/WM_KEYUP in seiner `GetMessageW`-Pump,
//! daher funktioniert dies auch, wenn LMU nicht im Vordergrund ist.
//!
//! ## LMU-Standard-Tasten (aus keyboard.json):
//! - Insert = Driving Cameras (TV Cycle)
//! - Home = Onboard Cameras
//! - PageUp = Swingman Camera (Rear/Heck)
//! - PageDown = Tracking Cameras (Trackside/Top)
//! - End = Spectator Cameras (Behind)

use std::ptr;
use std::sync::mpsc;
use std::thread;
use std::time::Duration;

// ─── Win32-Typdefinitionen ─────────────────────────────────────────────

type BOOL = i32;
type HWND = isize;
type LPCWSTR = *const u16;
type UINT = u32;
type DWORD = u32;

// ─── Windows Messages ──────────────────────────────────────────────────

const WM_KEYDOWN: UINT = 0x0100;
const WM_KEYUP: UINT = 0x0101;

// ─── Virtual Key Codes (Windows) ──────────────────────────────────────
// Die keyboard.json verwendet DirectInput-Codes, aber Windows wandelt
// Tastatureingaben in VK-Codes um. LMU verarbeitet VK-Codes in seiner
// Message-Pump.

const VK_INSERT: usize = 0x2D;   // Driving Cameras (TV Cycle)
const VK_HOME: usize = 0x24;     // Onboard Cameras
const VK_PRIOR: usize = 0x21;    // PageUp = Swingman Camera (Rear/Heck)
const VK_NEXT: usize = 0x22;     // PageDown = Tracking Cameras (Trackside/Top)
const VK_END: usize = 0x23;      // End = Spectator Cameras (Behind)
const VK_LCONTROL: usize = 0xA2; // Left Control (für Fahrzeug-Fokus)
const VK_F: usize = 0x46;        // F-Taste (für Fahrzeug-Fokus)
const VK_RETURN: usize = 0x0D;   // Enter
const VK_0: usize = 0x30;
const VK_1: usize = 0x31;
const VK_2: usize = 0x32;
const VK_3: usize = 0x33;
const VK_4: usize = 0x34;
const VK_5: usize = 0x35;
const VK_6: usize = 0x36;
const VK_7: usize = 0x37;
const VK_8: usize = 0x38;
const VK_9: usize = 0x39;

// ─── Win32-Funktionen ──────────────────────────────────────────────────

extern "system" {
    fn FindWindowW(lp_class_name: LPCWSTR, lp_window_name: LPCWSTR) -> HWND;
    fn PostMessageW(h_wnd: HWND, msg: UINT, w_param: usize, l_param: isize) -> BOOL;
    fn Sleep(dw_milliseconds: DWORD);
}

// ─── Nachrichten für den Hintergrund-Thread ────────────────────────────

enum KeyCommand {
    SwitchCamera { vk_code: usize },
    FocusCar { car_number: String },
    Shutdown,
}

// ─── Hintergrund-Thread ───────────────────────────────────────────────

struct KeyboardThread {
    sender: mpsc::Sender<KeyCommand>,
}

impl KeyboardThread {
    fn new() -> Self {
        let (tx, rx) = mpsc::channel::<KeyCommand>();

        thread::spawn(move || {
            loop {
                match rx.recv() {
                    Ok(cmd) => match cmd {
                        KeyCommand::SwitchCamera { vk_code } => {
                            if let Some(hwnd) = Self::find_lmu() {
                                Self::send_key(hwnd, vk_code);
                            }
                        }
                        KeyCommand::FocusCar { car_number } => {
                            if let Some(hwnd) = Self::find_lmu() {
                                Self::send_key(hwnd, VK_LCONTROL);
                                Self::send_key(hwnd, VK_F);
                                thread::sleep(Duration::from_millis(500));
                                for c in car_number.chars() {
                                    if let Some(vk) = char_to_vk(c) {
                                        Self::send_key(hwnd, vk);
                                        thread::sleep(Duration::from_millis(50));
                                    }
                                }
                                thread::sleep(Duration::from_millis(100));
                                Self::send_key(hwnd, VK_RETURN);
                            }
                        }
                        KeyCommand::Shutdown => break,
                    },
                    Err(_) => break,
                }
            }
        });

        KeyboardThread { sender: tx }
    }

    fn find_lmu() -> Option<HWND> {
        let titles = [
            "Le Mans Ultimate",
            "LMU",
            "rFactor 2",
            "LMU -",
            "Le Mans Ultimate -",
        ];
        for title in &titles {
            let wide: Vec<u16> = title.encode_utf16().chain(std::iter::once(0)).collect();
            unsafe {
                let hwnd = FindWindowW(ptr::null(), wide.as_ptr());
                if hwnd != 0 {
                    println!("[keyboard] LMU-Fenster gefunden: '{}' (HWND={})", title, hwnd);
                    return Some(hwnd);
                }
            }
        }
        println!("[keyboard] LMU-Fenster NICHT gefunden! Gesucht: {:?}", titles);
        None
    }

    /// Sendet WM_KEYDOWN + WM_KEYUP direkt an das LMU-Fenster.
    /// KEIN Fenster-Fokus nötig! LMU verarbeitet es in seiner Message-Pump.
    fn send_key(hwnd: HWND, vk_code: usize) {
        unsafe {
            PostMessageW(hwnd, WM_KEYDOWN, vk_code, 0);
            Sleep(50);
            PostMessageW(hwnd, WM_KEYUP, vk_code, 0);
        }
    }
}

// ─── Hilfsfunktion: Zeichen → VK-Code ──────────────────────────────────

fn char_to_vk(c: char) -> Option<usize> {
    match c {
        '0' => Some(VK_0),
        '1' => Some(VK_1),
        '2' => Some(VK_2),
        '3' => Some(VK_3),
        '4' => Some(VK_4),
        '5' => Some(VK_5),
        '6' => Some(VK_6),
        '7' => Some(VK_7),
        '8' => Some(VK_8),
        '9' => Some(VK_9),
        _ => None,
    }
}

// ─── Globaler Keyboard-Thread (Singleton) ──────────────────────────────

use std::sync::OnceLock;

fn keyboard_thread() -> &'static KeyboardThread {
    static INSTANCE: OnceLock<KeyboardThread> = OnceLock::new();
    INSTANCE.get_or_init(|| {
        println!("[keyboard] Hintergrund-Thread für Tastatursimulation gestartet");
        KeyboardThread::new()
    })
}

// ─── Öffentliche API ───────────────────────────────────────────────────

/// Schaltet die LMU-Kamera auf die angegebene Kamera-ID um.
/// Verwendet `PostMessageW` mit den LMU-Standard-Tasten:
/// - Insert = Driving Cameras (TV Cycle)
/// - Home = Onboard Cameras
/// - PageUp = Swingman Camera (Rear/Heck)
/// - PageDown = Tracking Cameras (Trackside/Top)
/// - End = Spectator Cameras (Behind)
pub fn switch_camera(cam_id: &str) -> Result<(), String> {
    let vk_code = match cam_id {
        "TV" => VK_INSERT,
        "Helmet" | "Onboard" => VK_HOME,
        "Front" | "Nose" => VK_INSERT,
        "Heck" | "Rear" => VK_PRIOR,
        "Top" | "Trackside" => VK_NEXT,
        "Behind" => VK_END,
        _ => return Err(format!(
            "Unbekannte Kamera-ID: {}. Gültig: TV, Helmet, Front, Heck, Top, Behind",
            cam_id
        )),
    };

    let thread = keyboard_thread();
    thread
        .sender
        .send(KeyCommand::SwitchCamera { vk_code })
        .map_err(|e| format!("Keyboard-Thread nicht verfügbar: {}", e))?;

    Ok(())
}

/// Fokussiert die Kamera auf ein bestimmtes Fahrzeug.
pub fn focus_car(car_number: &str) -> Result<(), String> {
    let thread = keyboard_thread();
    thread
        .sender
        .send(KeyCommand::FocusCar {
            car_number: car_number.to_string(),
        })
        .map_err(|e| format!("Keyboard-Thread nicht verfügbar: {}", e))?;

    Ok(())
}