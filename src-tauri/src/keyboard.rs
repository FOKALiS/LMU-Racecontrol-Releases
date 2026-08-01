//! Tastatursimulation für die LMU-Kamera-Steuerung und Fahrzeug-Fokus.
//! Verwendet SendInput mit Scancodes (KEIN externer Helper nötig).

use std::thread;
use std::time::Duration;

// ─── Scancodes (Set 1) ──────────────────────────────────────────────────
// Benutzer-Tastenbelegung:
// TV = PG_DN (Page Down), Hinten = PG_UP (Page Up), Bord = INSERT
const SCAN_INSERT: u16 = 0x52;    // INSERT = Bordkamera
const SCAN_PAGEUP: u16 = 0x49;    // PG_UP = Heck/Rear
const SCAN_PAGEDOWN: u16 = 0x51;  // PG_DN = TV
const SCAN_END: u16 = 0x4F;       // END (nicht belegt, aber als Fallback)
const SCAN_KP7: u16 = 0x47;       // KP 7 = Zoom In (ohne extended)
const SCAN_KP9: u16 = 0x49;       // KP 9 = Zoom Out (ohne extended – gleicher Scancode wie PageUp, aber extended=false)
const SCAN_LCONTROL: u16 = 0x1D;
const SCAN_F: u16 = 0x21;
const SCAN_RETURN: u16 = 0x1C;
const SCAN_R: u16 = 0x13;       // R = Sofortwiederholung (Replay)
const SCAN_F6: u16 = 0x40;      // F6 = Stop
const SCAN_F7: u16 = 0x41;      // F7 = Zurückspulen
const SCAN_F8: u16 = 0x42;      // F8 = Schnell zurück
const SCAN_F9: u16 = 0x43;      // F9 = Vorspulen
const SCAN_F10: u16 = 0x44;     // F10 = Slow-Motion
const SCAN_F11: u16 = 0x57;     // F11 = Play/Pause

// ─── Win32 Typen ──────────────────────────────────────────────────────
type HANDLE = isize;
type BOOL = i32;
type DWORD = u32;
type WORD = u16;
type LPVOID = *mut std::ffi::c_void;
type LPCVOID = *const std::ffi::c_void;
type LPCWSTR = *const u16;

const INPUT_KEYBOARD: DWORD = 1;
const KEYEVENTF_KEYUP: DWORD = 0x0002;
const KEYEVENTF_SCANCODE: DWORD = 0x0008;
const KEYEVENTF_EXTENDEDKEY: DWORD = 0x0001;
const FALSE: BOOL = 0;

#[repr(C)]
#[derive(Copy, Clone)]
struct KEYBDINPUT {
    wVk: WORD,
    wScan: WORD,
    dwFlags: DWORD,
    time: DWORD,
    dwExtraInfo: usize,
}

#[repr(C)]
union INPUT_UNION {
    ki: std::mem::ManuallyDrop<KEYBDINPUT>,
    padding: [u8; 28],
}

#[repr(C)]
struct INPUT {
    type_: DWORD,
    u: INPUT_UNION,
}

#[link(name = "user32")]
extern "system" {
    fn SendInput(cInputs: DWORD, pInputs: *mut INPUT, cbSize: i32) -> DWORD;
    fn SetForegroundWindow(hWnd: HANDLE) -> BOOL;
    fn ShowWindow(hWnd: HANDLE, nCmdShow: i32) -> BOOL;
    fn GetForegroundWindow() -> HANDLE;
    fn GetWindowTextW(hWnd: HANDLE, lpString: *mut u16, nMaxCount: i32) -> i32;
    fn EnumWindows(lpEnumFunc: Option<unsafe extern "system" fn(HANDLE, LPVOID) -> BOOL>, lParam: LPVOID) -> BOOL;
    fn IsWindowVisible(hWnd: HANDLE) -> BOOL;
}

// ─── LMU-Fenster finden ──────────────────────────────────────────────
fn find_lmu_window() -> Option<HANDLE> {
    unsafe {
        let mut result: HANDLE = 0;
        
        extern "system" fn enum_cb(hwnd: HANDLE, lparam: LPVOID) -> BOOL {
            unsafe {
                if IsWindowVisible(hwnd) == FALSE { return 1; }
                let mut buf = [0u16; 512];
                let len = GetWindowTextW(hwnd, buf.as_mut_ptr(), 512);
                if len <= 0 { return 1; }
                let title = String::from_utf16_lossy(&buf[..len as usize]);
                if title.starts_with("Le Mans Ultimate v") {
                    *(lparam as *mut HANDLE) = hwnd;
                    return 0;
                }
            }
            1
        }

        EnumWindows(Some(enum_cb), &mut result as *mut _ as LPVOID);
        if result != 0 { Some(result) } else { None }
    }
}

// ─── Scancode senden (mit extended-Flag) ─────────────────────────────
fn send_scancode(scan: u16, extended: bool) {
    unsafe {
        let hwnd = find_lmu_window();
        if hwnd.is_none() { return; }
        let hwnd = hwnd.unwrap();

        let prev = GetForegroundWindow();

        ShowWindow(hwnd, 9);
        thread::sleep(Duration::from_millis(50));
        SetForegroundWindow(hwnd);
        thread::sleep(Duration::from_millis(200));

        let mut flags_down: DWORD = KEYEVENTF_SCANCODE;
        let mut flags_up: DWORD = KEYEVENTF_SCANCODE | KEYEVENTF_KEYUP;
        if extended {
            flags_down |= KEYEVENTF_EXTENDEDKEY;
            flags_up |= KEYEVENTF_EXTENDEDKEY;
        }

        let ki_down = KEYBDINPUT { wVk: 0, wScan: scan, dwFlags: flags_down, time: 0, dwExtraInfo: 0 };
        let mut input_down = INPUT { type_: INPUT_KEYBOARD, u: INPUT_UNION { ki: std::mem::ManuallyDrop::new(ki_down) } };
        let ki_up = KEYBDINPUT { wVk: 0, wScan: scan, dwFlags: flags_up, time: 0, dwExtraInfo: 0 };
        let mut input_up = INPUT { type_: INPUT_KEYBOARD, u: INPUT_UNION { ki: std::mem::ManuallyDrop::new(ki_up) } };

        // Nur 1x senden (3x war zu viel und hat 2 Kameras übersprungen)
        SendInput(1, &mut input_down, std::mem::size_of::<INPUT>() as i32);
        thread::sleep(Duration::from_millis(30));
        SendInput(1, &mut input_up, std::mem::size_of::<INPUT>() as i32);
        thread::sleep(Duration::from_millis(50));

        if prev != 0 && prev != hwnd {
            thread::sleep(Duration::from_millis(100));
            SetForegroundWindow(prev);
        }
    }
}

// ─── Öffentliche API ─────────────────────────────────────────────────
pub fn switch_camera(cam_id: &str) -> Result<(), String> {
    // Deine Tastenbelegung:
    // TV = PG_DN (Page Down), Hinten = PG_UP (Page Up), Bord = INSERT
    // Alle 3 Tasten sind extended (Insert, PageUp, PageDown)
    let (scan, extended) = match cam_id {
        "TV" => (SCAN_PAGEDOWN, true),      // PG_DN = TV
        "Bord" | "Helmet" | "Onboard" => (SCAN_INSERT, true), // INSERT = Bordkamera
        "Heck" | "Rear" => (SCAN_PAGEUP, true), // PG_UP = Heck
        _ => return Err(format!("Unbekannte Kamera-ID: {}", cam_id)),
    };

    thread::spawn(move || {
        send_scancode(scan, extended);
    });

    Ok(())
}

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use once_cell::sync::Lazy;

static ZOOM_ACTIVE: Lazy<Arc<AtomicBool>> = Lazy::new(|| Arc::new(AtomicBool::new(false)));

/// Sendet einen Scancode OHNE Fenster-Wechsel (für Dauer-Zoom).
/// LMU muss bereits im Vordergrund sein.
fn send_scancode_fast(scan: u16, extended: bool) {
    unsafe {
        let mut flags_down: DWORD = KEYEVENTF_SCANCODE;
        let mut flags_up: DWORD = KEYEVENTF_SCANCODE | KEYEVENTF_KEYUP;
        if extended {
            flags_down |= KEYEVENTF_EXTENDEDKEY;
            flags_up |= KEYEVENTF_EXTENDEDKEY;
        }

        let ki_down = KEYBDINPUT { wVk: 0, wScan: scan, dwFlags: flags_down, time: 0, dwExtraInfo: 0 };
        let mut input_down = INPUT { type_: INPUT_KEYBOARD, u: INPUT_UNION { ki: std::mem::ManuallyDrop::new(ki_down) } };
        let ki_up = KEYBDINPUT { wVk: 0, wScan: scan, dwFlags: flags_up, time: 0, dwExtraInfo: 0 };
        let mut input_up = INPUT { type_: INPUT_KEYBOARD, u: INPUT_UNION { ki: std::mem::ManuallyDrop::new(ki_up) } };

        SendInput(1, &mut input_down, std::mem::size_of::<INPUT>() as i32);
        thread::sleep(Duration::from_millis(2));
        SendInput(1, &mut input_up, std::mem::size_of::<INPUT>() as i32);
        thread::sleep(Duration::from_millis(2));
    }
}

/// Startet Dauer-Zoom. Sendet die Taste alle 15ms, bis `zoom_stop()` aufgerufen wird.
/// Holt LMU 1x in den Vordergrund und bleibt dort bis zum Stop.
pub fn zoom_start(direction: &str) -> Result<(), String> {
    if ZOOM_ACTIVE.load(Ordering::SeqCst) {
        return Ok(());
    }
    ZOOM_ACTIVE.store(true, Ordering::SeqCst);

    let (scan, extended) = match direction {
        "in" => (SCAN_KP7, false),
        "out" => (SCAN_KP9, false),
        _ => return Err(format!("Unbekannte Zoom-Richtung: {}", direction)),
    };

    let active = ZOOM_ACTIVE.clone();
    thread::spawn(move || {
        // LMU 1x in den Vordergrund holen
        let hwnd = find_lmu_window();
        if hwnd.is_none() { 
            active.store(false, Ordering::SeqCst);
            return; 
        }
        let hwnd = hwnd.unwrap();
        let prev = unsafe { GetForegroundWindow() };
        
        unsafe {
            ShowWindow(hwnd, 9);
            thread::sleep(Duration::from_millis(50));
            SetForegroundWindow(hwnd);
            thread::sleep(Duration::from_millis(50)); // reduziert von 200ms auf 50ms
        }

        // Turbo-Zoom: KeyDown+KeyUp so schnell wie möglich senden
        let mut flags_down: DWORD = KEYEVENTF_SCANCODE;
        let mut flags_up: DWORD = KEYEVENTF_SCANCODE | KEYEVENTF_KEYUP;
        if extended {
            flags_down |= KEYEVENTF_EXTENDEDKEY;
            flags_up |= KEYEVENTF_EXTENDEDKEY;
        }

        // Turbo-Zoom (bewährte Methode): KeyDown und KeyUp getrennt senden
        // mit minimaler Pause zwischen den Events
        while active.load(Ordering::SeqCst) {
            unsafe {
                let ki_down = KEYBDINPUT { wVk: 0, wScan: scan, dwFlags: flags_down, time: 0, dwExtraInfo: 0 };
                let mut input_down = INPUT { type_: INPUT_KEYBOARD, u: INPUT_UNION { ki: std::mem::ManuallyDrop::new(ki_down) } };
                SendInput(1, &mut input_down, std::mem::size_of::<INPUT>() as i32);
                
                thread::sleep(Duration::from_millis(1)); // 1ms genug für KeyDown
                
                let ki_up = KEYBDINPUT { wVk: 0, wScan: scan, dwFlags: flags_up, time: 0, dwExtraInfo: 0 };
                let mut input_up = INPUT { type_: INPUT_KEYBOARD, u: INPUT_UNION { ki: std::mem::ManuallyDrop::new(ki_up) } };
                SendInput(1, &mut input_up, std::mem::size_of::<INPUT>() as i32);
                
                thread::sleep(Duration::from_millis(1)); // 1ms Pause zwischen Zoom-Stufen (halbiert von 2ms!)
            }
        }

        // Kurz warten und zurück zum vorherigen Fenster
        thread::sleep(Duration::from_millis(100));
        if prev != 0 && prev != hwnd {
            unsafe { SetForegroundWindow(prev); }
        }
    });

    Ok(())
}

/// Stoppt den Dauer-Zoom.
pub fn zoom_stop() {
    ZOOM_ACTIVE.store(false, Ordering::SeqCst);
}

/// Aktiviert den Replay-Modus über die R-Taste (Sofortwiederholung).
pub fn replay_activate() -> Result<(), String> {
    thread::spawn(move || {
        // R-Taste senden (KEIN extended-Flag)
        send_scancode(SCAN_R, false);
        // Kurz warten, damit LMU den Replay-Modus aktiviert
        thread::sleep(Duration::from_millis(500));
    });
    Ok(())
}

/// Spielt das Replay ab/pausiert es (F11-Toggle).
pub fn replay_play() -> Result<(), String> {
    thread::spawn(move || {
        send_scancode(SCAN_F11, false);
    });
    Ok(())
}

/// Pausiert das Replay (F11 = Play/Pause-Toggle).
/// F6 ist in LMU ein Hold-to-Stop (kein Toggle), F11 ist der richtige Play/Pause-Befehl.
pub fn replay_pause() -> Result<(), String> {
    thread::spawn(move || {
        // LMU in den Vordergrund holen
        let hwnd = find_lmu_window();
        if hwnd.is_none() {
            eprintln!("[replay_pause] LMU-Fenster nicht gefunden, sende F11 trotzdem...");
            send_scancode(SCAN_F11, false);
            return;
        }
        let hwnd = hwnd.unwrap();
        let prev = unsafe { GetForegroundWindow() };
        
        unsafe {
            ShowWindow(hwnd, 9);
            thread::sleep(Duration::from_millis(50));
            SetForegroundWindow(hwnd);
            thread::sleep(Duration::from_millis(100));
        }
        
        // F11 senden (Play/Pause-Toggle)
        send_scancode(SCAN_F11, false);
        thread::sleep(Duration::from_millis(200));
        
        // Zurück zum vorherigen Fenster
        if prev != 0 && prev != hwnd {
            unsafe { SetForegroundWindow(prev); }
        }
        println!("[replay_pause] F11 gesendet, Replay sollte pausiert sein");
    });
    Ok(())
}

/// Verlässt den Replay-Modus (Esc-Taste) – bringt LMU zurück zu Live.
pub fn replay_exit() -> Result<(), String> {
    let scan_esc: u16 = 0x01; // Esc-Scancode
    thread::spawn(move || {
        let hwnd = find_lmu_window();
        if hwnd.is_none() {
            eprintln!("[replay_exit] LMU-Fenster nicht gefunden, sende Esc trotzdem...");
            send_scancode(scan_esc, false);
            return;
        }
        let hwnd = hwnd.unwrap();
        let prev = unsafe { GetForegroundWindow() };
        
        unsafe {
            ShowWindow(hwnd, 9);
            thread::sleep(Duration::from_millis(50));
            SetForegroundWindow(hwnd);
            thread::sleep(Duration::from_millis(100));
        }
        
        // Esc senden (Replay-Modus verlassen)
        send_scancode(scan_esc, false);
        thread::sleep(Duration::from_millis(200));
        
        // Zurück zum vorherigen Fenster
        if prev != 0 && prev != hwnd {
            unsafe { SetForegroundWindow(prev); }
        }
        println!("[replay_exit] Esc gesendet, sollte zurück zu Live sein");
    });
    Ok(())
}

pub fn focus_car(car_number: &str) -> Result<(), String> {
    let car_num = car_number.to_string();
    thread::spawn(move || {
        send_scancode(SCAN_LCONTROL, false);
        send_scancode(SCAN_F, false);
        thread::sleep(Duration::from_millis(500));

        for c in car_num.chars() {
            if let Some(scan) = char_to_scancode(c) {
                send_scancode(scan, false);
                thread::sleep(Duration::from_millis(50));
            }
        }

        send_scancode(SCAN_RETURN, false);
    });

    Ok(())
}

fn char_to_scancode(c: char) -> Option<u16> {
    match c {
        '0' => Some(0x0B), '1' => Some(0x02), '2' => Some(0x03),
        '3' => Some(0x04), '4' => Some(0x05), '5' => Some(0x06),
        '6' => Some(0x07), '7' => Some(0x08), '8' => Some(0x09),
        '9' => Some(0x0A), _ => None,
    }
}