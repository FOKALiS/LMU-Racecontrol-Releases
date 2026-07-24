//! Tastatursimulation für die LMU-Kamera-Steuerung und Fahrzeug-Fokus.
//!
//! LMU (Le Mans Ultimate) / rFactor2 verwendet standardmäßig die F1-F6 Tasten
//! zum Umschalten zwischen Kameraperspektiven. Da die LMU-REST-API KEINEN
//! Endpunkt für die Kamera-Steuerung bietet, simulieren wir die entsprechenden
//! Tastendrücke.
//!
//! ## Wichtig: Fenster-Fokus
//! Spiele wie LMU/rFactor2 verwenden DirectInput/Raw Input für die Tastatur,
//! nicht die Windows Message Queue. Daher funktioniert `PostMessageW` mit
//! WM_KEYDOWN/WM_KEYUP NICHT - die Nachrichten landen in der Windows-Message-
//! Queue, aber das Spiel liest dort nicht.
//!
//! Stattdessen verwenden wir `SendInput` mit `KEYEVENTF_SCANCODE`, das in den
//! systemweiten Raw Input Stream injiziert. Damit das Spiel die Tasten
//! empfängt, MUSS es im Vordergrund sein. Dafür verwenden wir:
//! - `SwitchToThisWindow` (aggressiver als SetForegroundWindow)
//! - `AttachThreadInput` (umgeht Windows-UIPI)

use std::mem;
use std::ptr;
use std::sync::mpsc;
use std::thread;
use std::time::Duration;

// ─── Win32-Typdefinitionen ─────────────────────────────────────────────

type BOOL = i32;
type HWND = isize;
type LPCWSTR = *const u16;
type UINT = u32;
type WORD = u16;
type DWORD = u32;
type LONG = i32;

const SW_RESTORE: i32 = 9;
const SW_SHOW: i32 = 5;

// ─── Win32-Strukturen ──────────────────────────────────────────────────

#[repr(C)]
#[derive(Clone, Copy)]
struct KEYBDINPUT {
    w_vk: WORD,
    w_scan: WORD,
    dw_flags: DWORD,
    time: DWORD,
    dw_extra_info: usize,
}

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

#[repr(C)]
#[derive(Clone, Copy)]
union INPUT_UNION {
    mi: MOUSEINPUT,
    ki: KEYBDINPUT,
    hi: HARDWAREINPUT,
    padding: [u64; 4],
}

#[repr(C)]
#[derive(Clone, Copy)]
struct INPUT {
    type_: DWORD,
    _padding: DWORD,
    u: INPUT_UNION,
}

const INPUT_KEYBOARD: DWORD = 1;
const KEYEVENTF_KEYUP: DWORD = 0x0002;
const KEYEVENTF_SCANCODE: DWORD = 0x0008;

// ─── Scancodes (IBM PC/AT) ─────────────────────────────────────────────

const SC_F1: WORD = 0x3B;
const SC_F2: WORD = 0x3C;
const SC_F3: WORD = 0x3D;
const SC_F4: WORD = 0x3E;
const SC_F5: WORD = 0x3F;
const SC_F6: WORD = 0x40;
const SC_ENTER: WORD = 0x1C;
const SC_LCONTROL: WORD = 0x1D;
const SC_F_KEY: WORD = 0x21;
const SC_0: WORD = 0x0B;
const SC_1: WORD = 0x02;
const SC_2: WORD = 0x03;
const SC_3: WORD = 0x04;
const SC_4: WORD = 0x05;
const SC_5: WORD = 0x06;
const SC_6: WORD = 0x07;
const SC_7: WORD = 0x08;
const SC_8: WORD = 0x09;
const SC_9: WORD = 0x0A;

// ─── Win32-Funktionen ──────────────────────────────────────────────────

extern "system" {
    fn FindWindowW(lp_class_name: LPCWSTR, lp_window_name: LPCWSTR) -> HWND;
    fn SetForegroundWindow(h_wnd: HWND) -> BOOL;
    fn ShowWindow(h_wnd: HWND, n_cmd_show: i32) -> BOOL;
    fn IsIconic(h_wnd: HWND) -> BOOL;
    fn SendInput(c_inputs: UINT, p_inputs: *const INPUT, cb_size: i32) -> UINT;
    fn Sleep(dw_milliseconds: DWORD);
    fn AttachThreadInput(id_attach: DWORD, id_attach_to: DWORD, f_attach: BOOL) -> BOOL;
    fn GetWindowThreadProcessId(h_wnd: HWND, lpdw_process_id: *mut DWORD) -> DWORD;
    fn GetCurrentThreadId() -> DWORD;
    fn BringWindowToTop(h_wnd: HWND) -> BOOL;
    fn SwitchToThisWindow(h_wnd: HWND, f_unknown: BOOL);
}

// ─── Nachrichten für den Hintergrund-Thread ────────────────────────────

enum KeyCommand {
    SwitchCamera { scancode: WORD },
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
                        KeyCommand::SwitchCamera { scancode } => {
                            if let Some(hwnd) = Self::find_lmu() {
                                Self::force_foreground(hwnd);
                                Self::send_scancode(scancode);
                            }
                        }
                        KeyCommand::FocusCar { car_number } => {
                            if let Some(hwnd) = Self::find_lmu() {
                                Self::force_foreground(hwnd);
                                Self::send_scancode_with_modifier(SC_LCONTROL, SC_F_KEY);
                                thread::sleep(Duration::from_millis(500));
                                for c in car_number.chars() {
                                    if let Some(sc) = char_to_scancode(c) {
                                        Self::send_scancode(sc);
                                        thread::sleep(Duration::from_millis(50));
                                    }
                                }
                                thread::sleep(Duration::from_millis(100));
                                Self::send_scancode(SC_ENTER);
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
        // LMU Fenster-Titel: "Le Mans Ultimate" (genau so heißt das Fenster)
        // Zusätzlich suchen wir nach Teilstrings, falls der Titel abweicht
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

    /// Versucht, LMU in den Vordergrund zu bringen.
    /// Verwendet SetForegroundWindow (sanfter) + minimale Verzögerung.
    /// SwitchToThisWindow + AttachThreadInput wurde entfernt, da es
    /// das LMU-Fenster aggressiv aufpoppen lässt.
    fn force_foreground(hwnd: HWND) {
        unsafe {
            if IsIconic(hwnd) != 0 {
                ShowWindow(hwnd, SW_RESTORE);
            }

            // Sanftere Methode: nur SetForegroundWindow + BringWindowToTop
            // ohne AttachThreadInput/SwitchToThisWindow, um das
            // unerwünschte Aufpoppen des LMU-Fensters zu vermeiden.
            SetForegroundWindow(hwnd);
            BringWindowToTop(hwnd);

            // Minimale Verzögerung für den Fokus-Wechsel
            Sleep(50);
        }
    }

    fn send_scancode(scancode: WORD) {
        unsafe {
            let input_down = INPUT {
                type_: INPUT_KEYBOARD,
                _padding: 0,
                u: INPUT_UNION {
                    ki: KEYBDINPUT {
                        w_vk: 0,
                        w_scan: scancode,
                        dw_flags: KEYEVENTF_SCANCODE,
                        time: 0,
                        dw_extra_info: 0,
                    },
                },
            };
            SendInput(1, &input_down, mem::size_of::<INPUT>() as i32);
            Sleep(50);

            let input_up = INPUT {
                type_: INPUT_KEYBOARD,
                _padding: 0,
                u: INPUT_UNION {
                    ki: KEYBDINPUT {
                        w_vk: 0,
                        w_scan: scancode,
                        dw_flags: KEYEVENTF_SCANCODE | KEYEVENTF_KEYUP,
                        time: 0,
                        dw_extra_info: 0,
                    },
                },
            };
            SendInput(1, &input_up, mem::size_of::<INPUT>() as i32);
        }
    }

    fn send_scancode_with_modifier(mod_scancode: WORD, key_scancode: WORD) {
        unsafe {
            let mod_down = INPUT {
                type_: INPUT_KEYBOARD,
                _padding: 0,
                u: INPUT_UNION {
                    ki: KEYBDINPUT {
                        w_vk: 0,
                        w_scan: mod_scancode,
                        dw_flags: KEYEVENTF_SCANCODE,
                        time: 0,
                        dw_extra_info: 0,
                    },
                },
            };
            SendInput(1, &mod_down, mem::size_of::<INPUT>() as i32);
            Sleep(50);

            let key_down = INPUT {
                type_: INPUT_KEYBOARD,
                _padding: 0,
                u: INPUT_UNION {
                    ki: KEYBDINPUT {
                        w_vk: 0,
                        w_scan: key_scancode,
                        dw_flags: KEYEVENTF_SCANCODE,
                        time: 0,
                        dw_extra_info: 0,
                    },
                },
            };
            SendInput(1, &key_down, mem::size_of::<INPUT>() as i32);
            Sleep(50);

            let key_up = INPUT {
                type_: INPUT_KEYBOARD,
                _padding: 0,
                u: INPUT_UNION {
                    ki: KEYBDINPUT {
                        w_vk: 0,
                        w_scan: key_scancode,
                        dw_flags: KEYEVENTF_SCANCODE | KEYEVENTF_KEYUP,
                        time: 0,
                        dw_extra_info: 0,
                    },
                },
            };
            SendInput(1, &key_up, mem::size_of::<INPUT>() as i32);
            Sleep(50);

            let mod_up = INPUT {
                type_: INPUT_KEYBOARD,
                _padding: 0,
                u: INPUT_UNION {
                    ki: KEYBDINPUT {
                        w_vk: 0,
                        w_scan: mod_scancode,
                        dw_flags: KEYEVENTF_SCANCODE | KEYEVENTF_KEYUP,
                        time: 0,
                        dw_extra_info: 0,
                    },
                },
            };
            SendInput(1, &mod_up, mem::size_of::<INPUT>() as i32);
        }
    }
}

// ─── Hilfsfunktion: Zeichen → Scancode ─────────────────────────────────

fn char_to_scancode(c: char) -> Option<WORD> {
    match c {
        '0' => Some(SC_0),
        '1' => Some(SC_1),
        '2' => Some(SC_2),
        '3' => Some(SC_3),
        '4' => Some(SC_4),
        '5' => Some(SC_5),
        '6' => Some(SC_6),
        '7' => Some(SC_7),
        '8' => Some(SC_8),
        '9' => Some(SC_9),
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
pub fn switch_camera(cam_id: &str) -> Result<(), String> {
    let scancode = match cam_id {
        "TV" => SC_F1,
        "Helmet" => SC_F2,
        "Front" => SC_F3,
        "Heck" | "Rear" => SC_F4,
        "Top" => SC_F5,
        "Behind" => SC_F6,
        _ => return Err(format!(
            "Unbekannte Kamera-ID: {}. Gültig: TV, Helmet, Front, Heck, Top, Behind",
            cam_id
        )),
    };

    let thread = keyboard_thread();
    thread
        .sender
        .send(KeyCommand::SwitchCamera { scancode })
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