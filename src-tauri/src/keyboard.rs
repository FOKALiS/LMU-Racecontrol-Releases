//! Tastatursimulation für die LMU-Kamera-Steuerung und Fahrzeug-Fokus.
//!
//! LMU (Le Mans Ultimate) / rFactor2 verwendet standardmäßig die F1-F6 Tasten
//! zum Umschalten zwischen Kameraperspektiven. Da die LMU-REST-API KEINEN
//! Endpunkt für die Kamera-Steuerung bietet, simulieren wir die entsprechenden
//! Tastendrücke via Win32 `SendInput` API (deutlich zuverlässiger als das
//! enigo-Crate, da es direkt mit hardware-nahen Input-Ereignissen arbeitet
//! und KEIN sichtbares Terminal-Fenster öffnet).
//!
//! ## Tastenbelegung (rFactor2/LMU-Standard)
//! - F1 = TV/Broadcast Cam
//! - F2 = Helmet Cam (Bord/Onboard)
//! - F3 = Front (Bumper) Cam ("Fährt"-Modus)
//! - F4 = Rear (Chase) Cam ("Fährt"-Modus, schaltet durch: Heck → seitlich hinten → Heck)
//! - F5 = Top/Bonnet Cam
//! - F6 = Behind/Free Cam
//!
//! ## Fahrzeug-Fokus
//! Strg+F öffnet den Fahrzeug-Fokus-Dialog, dann wird die Fahrzeugnummer
//! eingegeben und mit Enter bestätigt.
//!
//! ## Kamera-Kontext (LMU-spezifisch)
//! LMU hat drei Haupt-Kameramodi, die jeweils eigene Unteransichten durchschalten:
//! - "Streckenrand anpassen" (Trackside/Broadcast) → F1 (TV) – mehrfach drücken
//!   schaltet durch verschiedene Kamerapositionen an der Strecke
//! - "Fährt" → TV-Chase-Kamera, schaltet mit F3/F4/F5/F6 durch Ansichten
//! - "Bord" (Onboard) → F2 (Helmet) – schaltet durch Cockpit-/Helmet-Perspektiven
//!
//! WICHTIG: Ein erneuter Druck derselben Taste (z.B. F4) schaltet innerhalb
//! dieses Kameramodus weiter (z.B. Heck → seitlich hinten → Heck).

use std::mem;
use std::ptr;
use std::sync::mpsc;
use std::thread;
use std::time::Duration;

// ─── Win32-Typdefinitionen ─────────────────────────────────────────────
// Wir definieren nur die benötigten Typen, um die Abhängigkeit von großen
// Crates wie `windows-sys` zu vermeiden.

type BOOL = i32;
type HWND = isize;
type LPCWSTR = *const u16;
type UINT = u32;
type WORD = u16;
type DWORD = u32;
type LONG = i32;

const SW_RESTORE: i32 = 9;
const PM_REMOVE: UINT = 1;

// ─── Win32-Strukturen ──────────────────────────────────────────────────

#[repr(C)]
#[derive(Clone, Copy)]
struct POINT {
    x: LONG,
    y: LONG,
}

#[repr(C)]
#[derive(Clone, Copy)]
struct MSG {
    hwnd: HWND,
    message: UINT,
    w_param: usize,
    l_param: isize,
    time: DWORD,
    pt: POINT,
}

#[repr(C)]
#[derive(Clone, Copy)]
struct KEYBDINPUT {
    w_vk: WORD,
    w_scan: WORD,
    dw_flags: DWORD,
    time: DWORD,
    dw_extra_info: usize,
}

// Für die Größenberechnung der Union
#[repr(C)]
#[derive(Clone, Copy)]
struct MOUSEINPUT {
    dx: LONG,
    dy: LONG,
    mouse_data: DWORD,
    dw_flags: DWORD,
    time: DWORD,
    dw_extra_info: usize,
}

#[repr(C)]
#[derive(Clone, Copy)]
struct HARDWAREINPUT {
    u_msg: DWORD,
    w_param_l: WORD,
    w_param_h: WORD,
}

// Die Union muss die Größe des größten Members haben: MOUSEINPUT = 32 Bytes
// (4 + 4 + 4 + 4 + 4 + 8 = 28, aligned auf 8 → 32)
#[repr(C)]
#[derive(Clone, Copy)]
union INPUT_UNION {
    mi: MOUSEINPUT,
    ki: KEYBDINPUT,
    hi: HARDWAREINPUT,
    // Padding auf 32 Bytes (4 × u64 = 32)
    padding: [u64; 4],
}

#[repr(C)]
#[derive(Clone, Copy)]
struct INPUT {
    type_: DWORD,
    // 4 Bytes Padding für 8-Byte-Ausrichtung der Union
    _padding: DWORD,
    u: INPUT_UNION,
}

const INPUT_KEYBOARD: DWORD = 1;
const KEYEVENTF_KEYUP: DWORD = 0x0002;

// ─── Virtuelle Tastencodes (Windows) ───────────────────────────────────
const VK_F1: WORD = 0x70;
const VK_F2: WORD = 0x71;
const VK_F3: WORD = 0x72;
const VK_F4: WORD = 0x73;
const VK_F5: WORD = 0x74;
const VK_F6: WORD = 0x75;
const VK_CONTROL: WORD = 0x11;
const VK_RETURN: WORD = 0x0D;
const VK_F: WORD = 0x46; // 'F'

// ─── Win32-Funktionen (extern "system") ────────────────────────────────

extern "system" {
    fn FindWindowW(lp_class_name: LPCWSTR, lp_window_name: LPCWSTR) -> HWND;
    fn SetForegroundWindow(h_wnd: HWND) -> BOOL;
    fn ShowWindow(h_wnd: HWND, n_cmd_show: i32) -> BOOL;
    fn IsIconic(h_wnd: HWND) -> BOOL;
    fn SendInput(c_inputs: UINT, p_inputs: *const INPUT, cb_size: i32) -> UINT;
    fn Sleep(dw_milliseconds: DWORD);
    fn PeekMessageW(lp_msg: *mut MSG, h_wnd: HWND, w_msg_filter_min: UINT, w_msg_filter_max: UINT, w_remove_msg: UINT) -> BOOL;
}

// ─── Nachrichten für den Hintergrund-Thread ────────────────────────────

enum KeyCommand {
    SwitchCamera { vk: WORD },
    FocusCar { car_number: String },
    Shutdown,
}

// ─── Hintergrund-Thread für Tastatursimulation ────────────────────────
// Dieser Thread läuft dauerhaft im Hintergrund und hat eine eigene
// Windows-Nachrichtenschleife. Das ist wichtig, weil:
// 1. `SendInput` aus einem Thread mit Message-Queue zuverlässiger ist
// 2. `SetForegroundWindow` eine Message-Queue auf dem aufrufenden Thread benötigt
// 3. Keine Blockierung des async-Tokio-Runtimes

struct KeyboardThread {
    sender: mpsc::Sender<KeyCommand>,
}

impl KeyboardThread {
    fn new() -> Self {
        let (tx, rx) = mpsc::channel::<KeyCommand>();

        thread::spawn(move || {
            // Message-Queue für diesen Thread erstellen (wichtig für SetForegroundWindow!)
            unsafe {
                let mut msg: MSG = mem::zeroed();
                PeekMessageW(&mut msg, 0, 0, 0, PM_REMOVE);
            }

            loop {
                match rx.recv() {
                    Ok(cmd) => match cmd {
                        KeyCommand::SwitchCamera { vk } => {
                            Self::focus_lmu();
                            Self::flush_input();
                            Self::send_key(vk);
                            Self::flush_input();
                        }
                        KeyCommand::FocusCar { car_number } => {
                            Self::focus_lmu();
                            Self::flush_input();
                            // Strg+F
                            Self::send_key_with_modifier(VK_CONTROL, VK_F);
                            thread::sleep(Duration::from_millis(500));
                            // Ziffern eingeben
                            for c in car_number.chars() {
                                Self::send_char(c);
                                thread::sleep(Duration::from_millis(50));
                            }
                            thread::sleep(Duration::from_millis(100));
                            // Enter
                            Self::send_key(VK_RETURN);
                            Self::flush_input();
                        }
                        KeyCommand::Shutdown => break,
                    },
                    Err(_) => break,
                }
            }
        });

        KeyboardThread { sender: tx }
    }

    fn focus_lmu() {
        let titles = ["Le Mans Ultimate", "LMU", "rFactor 2"];
        for title in &titles {
            let wide: Vec<u16> = title.encode_utf16().chain(std::iter::once(0)).collect();
            unsafe {
                let hwnd = FindWindowW(ptr::null(), wide.as_ptr());
                if hwnd != 0 {
                    if IsIconic(hwnd) != 0 {
                        ShowWindow(hwnd, SW_RESTORE);
                    }
                    SetForegroundWindow(hwnd);
                    Sleep(150);
                    return;
                }
            }
        }
    }

    fn send_key(vk: WORD) {
        unsafe {
            let input = INPUT {
                type_: INPUT_KEYBOARD,
                _padding: 0,
                u: INPUT_UNION {
                    ki: KEYBDINPUT {
                        w_vk: vk,
                        w_scan: 0,
                        dw_flags: 0,
                        time: 0,
                        dw_extra_info: 0,
                    },
                },
            };
            SendInput(1, &input, mem::size_of::<INPUT>() as i32);
            Sleep(30);

            let input_up = INPUT {
                type_: INPUT_KEYBOARD,
                _padding: 0,
                u: INPUT_UNION {
                    ki: KEYBDINPUT {
                        w_vk: vk,
                        w_scan: 0,
                        dw_flags: KEYEVENTF_KEYUP,
                        time: 0,
                        dw_extra_info: 0,
                    },
                },
            };
            SendInput(1, &input_up, mem::size_of::<INPUT>() as i32);
        }
    }

    fn send_key_with_modifier(mod_vk: WORD, key_vk: WORD) {
        unsafe {
            let mod_down = INPUT {
                type_: INPUT_KEYBOARD,
                _padding: 0,
                u: INPUT_UNION {
                    ki: KEYBDINPUT {
                        w_vk: mod_vk,
                        w_scan: 0,
                        dw_flags: 0,
                        time: 0,
                        dw_extra_info: 0,
                    },
                },
            };
            SendInput(1, &mod_down, mem::size_of::<INPUT>() as i32);
            Sleep(30);
            Self::send_key(key_vk);
            Sleep(30);
            let mod_up = INPUT {
                type_: INPUT_KEYBOARD,
                _padding: 0,
                u: INPUT_UNION {
                    ki: KEYBDINPUT {
                        w_vk: mod_vk,
                        w_scan: 0,
                        dw_flags: KEYEVENTF_KEYUP,
                        time: 0,
                        dw_extra_info: 0,
                    },
                },
            };
            SendInput(1, &mod_up, mem::size_of::<INPUT>() as i32);
        }
    }

    fn send_char(c: char) {
        if let Some(d) = c.to_digit(10) {
            Self::send_key(0x30 + d as WORD);
        } else {
            let upper = c.to_ascii_uppercase();
            if upper >= 'A' && upper <= 'Z' {
                Self::send_key(0x41 + (upper as u32 - 'A' as u32) as WORD);
            }
        }
    }

    fn flush_input() {
        unsafe {
            let mut msg: MSG = mem::zeroed();
            while PeekMessageW(&mut msg, 0, 0, 0, PM_REMOVE) != 0 {}
        }
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
///
/// Verfügbare Kameras: TV, Helmet, Front, Heck/Rear, Top, Behind
/// Ein erneuter Aufruf mit derselben Kamera-ID schaltet innerhalb des
/// Kameramodus weiter (z.B. Heck → seitlich hinten → Heck).
pub fn switch_camera(cam_id: &str) -> Result<(), String> {
    let vk = match cam_id {
        "TV" => VK_F1,
        "Helmet" => VK_F2,
        "Front" => VK_F3,
        "Heck" | "Rear" => VK_F4,
        "Top" => VK_F5,
        "Behind" => VK_F6,
        _ => return Err(format!(
            "Unbekannte Kamera-ID: {}. Gültig: TV, Helmet, Front, Heck, Top, Behind",
            cam_id
        )),
    };

    let thread = keyboard_thread();
    thread
        .sender
        .send(KeyCommand::SwitchCamera { vk })
        .map_err(|e| format!("Keyboard-Thread nicht verfügbar: {}", e))?;

    Ok(())
}

/// Fokussiert die Kamera auf ein bestimmtes Fahrzeug.
///
/// Verwendet Strg+F → Fahrzeugnummer eingeben → Enter.
/// Funktioniert nur, wenn LMU das aktive Fenster ist.
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