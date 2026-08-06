//! Tastatursimulation für die LMU-Kamera-Steuerung und Fahrzeug-Fokus.
//! Verwendet SendInput mit Scancodes (KEIN externer Helper nötig).
//! Die Scancodes werden dynamisch aus der LMU-Tastenbelegung (keyboard.json)
//! geladen, sodass jede benutzerdefinierte Tastenbelegung funktioniert.

use std::thread;
use std::time::Duration;
use std::sync::RwLock;
use crate::keyboard_config::{KeyboardConfig, KeyBinding};

// ─── Globale Konfiguration (kann jederzeit neu geladen werden) ─────────
static KEYBOARD_CONFIG: RwLock<Option<KeyboardConfig>> = RwLock::new(None);

/// Initialisiert die Tastenbelegung aus der LMU keyboard.json.
/// Kann mehrfach aufgerufen werden (z.B. nach Änderung des LMU-Pfads).
pub fn init(config: KeyboardConfig) {
    if let Ok(mut guard) = KEYBOARD_CONFIG.write() {
        *guard = Some(config);
    }
}

/// Gibt die aktuelle Tastenbelegung zurück.
fn config() -> std::sync::RwLockReadGuard<'static, Option<KeyboardConfig>> {
    KEYBOARD_CONFIG.read().expect("keyboard_config RwLock ist vergiftet!")
}

/// Sonder-Scancodes für Tasten, die nicht in der LMU keyboard.json vorkommen
/// (z.B. Esc, Strg, Enter, F für Focus-Tastatur-Fallback).
const SCAN_ESC: u16 = 0x01;
const SCAN_LCONTROL: u16 = 0x1D;
const SCAN_F: u16 = 0x21;
const SCAN_RETURN: u16 = 0x1C;

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
const WM_KEYDOWN: u32 = 0x0100;
const WM_KEYUP: u32 = 0x0101;
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
    fn AttachThreadInput(idAttach: u32, idAttachTo: u32, fAttach: BOOL) -> BOOL;
    fn GetCurrentThreadId() -> u32;
    fn GetWindowThreadProcessId(hWnd: HANDLE, lpdwProcessId: *mut u32) -> u32;
    fn BringWindowToTop(hWnd: HANDLE) -> BOOL;
    fn PostMessageW(hWnd: HANDLE, Msg: u32, wParam: usize, lParam: isize) -> BOOL;
    fn MapVirtualKeyW(uCode: u32, uMapType: u32) -> u32;
}

/// Erzwingt den Fokus zuverlässig (Windows blockiert SetForegroundWindow sonst manchmal).
fn force_foreground(hwnd: HANDLE) {
    unsafe {
        let current_thread = GetCurrentThreadId();
        let target_thread = GetWindowThreadProcessId(hwnd, std::ptr::null_mut());
        AttachThreadInput(current_thread, target_thread, 1);
        BringWindowToTop(hwnd);
        SetForegroundWindow(hwnd);
        AttachThreadInput(current_thread, target_thread, 0);
    }
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

/// Hilfsfunktion: Holt ein Binding aus der Konfiguration.
fn get_binding(action: &str) -> Result<KeyBinding, String> {
    let guard = config();
    guard
        .as_ref()
        .and_then(|cfg| cfg.get(action))
        .ok_or_else(|| format!("Taste '{}' nicht in LMU-Tastenbelegung gefunden", action))
}

// ─── Öffentliche API ─────────────────────────────────────────────────
pub fn switch_camera(cam_id: &str) -> Result<(), String> {
    // LMU-Aktion pro Kamera-Button (basierend auf der LMU keyboard.json):
    // "TV" → "Tracking Cameras" (Verfolgerkamera, z.B. PG_DN)
    // "Bord" → "Driving Cameras" (Fahrkameras, z.B. Insert)
    // "Heck" → "Swingman Camera" (Schwenkkopfkamera, z.B. PG_UP)
    let action = match cam_id {
        "TV" | "Tracking" => "Tracking Cameras",
        "Bord" | "Helmet" | "Onboard" | "Driving" => "Driving Cameras",
        "Heck" | "Rear" | "Swingman" => "Swingman Camera",
        _ => return Err(format!("Unbekannte Kamera-ID: {}", cam_id)),
    };

    let binding = get_binding(action)?;

    thread::spawn(move || {
        send_scancode(binding.scan, binding.extended);
    });

    Ok(())
}

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use once_cell::sync::Lazy;

static ZOOM_ACTIVE: Lazy<Arc<AtomicBool>> = Lazy::new(|| Arc::new(AtomicBool::new(false)));

/// Startet Dauer-Zoom. Sendet die Taste alle 15ms, bis `zoom_stop()` aufgerufen wird.
/// Holt LMU 1x in den Vordergrund und bleibt dort bis zum Stop.
pub fn zoom_start(direction: &str) -> Result<(), String> {
    if ZOOM_ACTIVE.load(Ordering::SeqCst) {
        return Ok(());
    }
    ZOOM_ACTIVE.store(true, Ordering::SeqCst);

    let action = match direction {
        "in" => "Swingman Zoom In",
        "out" => "Swingman Zoom Out",
        _ => return Err(format!("Unbekannte Zoom-Richtung: {}", direction)),
    };

    let binding = get_binding(action)?;

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
            thread::sleep(Duration::from_millis(50));
        }

        let mut flags_down: DWORD = KEYEVENTF_SCANCODE;
        let mut flags_up: DWORD = KEYEVENTF_SCANCODE | KEYEVENTF_KEYUP;
        if binding.extended {
            flags_down |= KEYEVENTF_EXTENDEDKEY;
            flags_up |= KEYEVENTF_EXTENDEDKEY;
        }

        while active.load(Ordering::SeqCst) {
            unsafe {
                let ki_down = KEYBDINPUT { wVk: 0, wScan: binding.scan, dwFlags: flags_down, time: 0, dwExtraInfo: 0 };
                let mut input_down = INPUT { type_: INPUT_KEYBOARD, u: INPUT_UNION { ki: std::mem::ManuallyDrop::new(ki_down) } };
                SendInput(1, &mut input_down, std::mem::size_of::<INPUT>() as i32);

                thread::sleep(Duration::from_millis(1));

                let ki_up = KEYBDINPUT { wVk: 0, wScan: binding.scan, dwFlags: flags_up, time: 0, dwExtraInfo: 0 };
                let mut input_up = INPUT { type_: INPUT_KEYBOARD, u: INPUT_UNION { ki: std::mem::ManuallyDrop::new(ki_up) } };
                SendInput(1, &mut input_up, std::mem::size_of::<INPUT>() as i32);

                thread::sleep(Duration::from_millis(1));
            }
        }

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

/// Aktiviert den Replay-Modus über die Instant Replay-Taste (Standard: R).
pub fn replay_activate() -> Result<(), String> {
    let binding = get_binding("Instant Replay")?;

    thread::spawn(move || {
        send_scancode(binding.scan, binding.extended);
        // Kurz warten, damit LMU den Replay-Modus aktiviert
        thread::sleep(Duration::from_millis(500));
    });
    Ok(())
}

/// Spielt das Replay ab/pausiert es (Replay Play-Taste, Standard: F11).
pub fn replay_play() -> Result<(), String> {
    let binding = get_binding("Replay Play")?;

    thread::spawn(move || {
        send_scancode(binding.scan, binding.extended);
    });
    Ok(())
}

/// Pausiert das Replay (Replay Play = Play/Pause-Toggle).
/// SCHNELL: Minimaler Fokus-Wechsel + direktes SendInput (keine 430ms Wartezeit).
pub fn replay_pause() -> Result<(), String> {
    let binding = get_binding("Replay Play")?;

    thread::spawn(move || {
        let hwnd = find_lmu_window();
        if hwnd.is_none() {
            eprintln!("[replay_pause] LMU-Fenster nicht gefunden, sende trotzdem...");
            send_scancode(binding.scan, binding.extended);
            return;
        }
        let hwnd = hwnd.unwrap();
        let prev = unsafe { GetForegroundWindow() };

        // LMU schnell in den Vordergrund (force_foreground = ~10ms)
        force_foreground(hwnd);
        thread::sleep(Duration::from_millis(15));

        let mut flags_down: DWORD = KEYEVENTF_SCANCODE;
        let mut flags_up: DWORD = KEYEVENTF_SCANCODE | KEYEVENTF_KEYUP;
        if binding.extended {
            flags_down |= KEYEVENTF_EXTENDEDKEY;
            flags_up |= KEYEVENTF_EXTENDEDKEY;
        }

        unsafe {
            let ki = KEYBDINPUT { wVk: 0, wScan: binding.scan, dwFlags: flags_down, time: 0, dwExtraInfo: 0 };
            let mut inp = INPUT { type_: INPUT_KEYBOARD, u: INPUT_UNION { ki: std::mem::ManuallyDrop::new(ki) } };
            SendInput(1, &mut inp, std::mem::size_of::<INPUT>() as i32);
            thread::sleep(Duration::from_millis(10));
            let ki_up = KEYBDINPUT { wVk: 0, wScan: binding.scan, dwFlags: flags_up, time: 0, dwExtraInfo: 0 };
            let mut inp_up = INPUT { type_: INPUT_KEYBOARD, u: INPUT_UNION { ki: std::mem::ManuallyDrop::new(ki_up) } };
            SendInput(1, &mut inp_up, std::mem::size_of::<INPUT>() as i32);
        }

        // Zurück zur Racecontrol-App
        if prev != 0 && prev != hwnd {
            unsafe { SetForegroundWindow(prev); }
        }
        println!("[replay_pause] Play/Pause gesendet (schnell)");
    });
    Ok(())
}

/// Pausiert das Replay via PostMessageW – sendet WM_KEYDOWN/WM_KEYUP
/// DIREKT an das LMU-Fenster, OHNE Fenster-Fokus-Wechsel!
/// PostMessageW funktioniert auch dann, wenn LMU NICHT im Vordergrund ist,
/// weil es die Message direkt in die Message-Queue des Ziel-Fensters einreiht.
pub fn replay_pause_postmessage() -> Result<(), String> {
    let binding = get_binding("Replay Play")?;

    let hwnd = find_lmu_window();
    if hwnd.is_none() {
        eprintln!("[replay_pause_postmessage] LMU-Fenster nicht gefunden – versuche SendInput...");
        return replay_pause_simple_fallback(&binding);
    }
    let hwnd = hwnd.unwrap();

    // Virtual-Key-Code aus dem Scancode ermitteln (MapVirtualKeyW)
    let vk = unsafe { MapVirtualKeyW(binding.scan as u32, 1) };

    // lParam zusammensetzen:
    //   Bits 0-15: Scancode
    //   Bit 16: extended flag
    //   Bit 30: previous key state (1 = wurde bereits gedrückt)
    //   Bit 31: transition state (0 = key down, 1 = key up)
    let scan = binding.scan as isize;
    let extended_bit = if binding.extended { 0x0100_0000 } else { 0 };
    let lparam_down = scan | extended_bit;
    let lparam_up = scan | extended_bit | 0xC000_0000; // Bit 30 + Bit 31 = key up + previous key state

    unsafe {
        // WM_KEYDOWN mit VK und lParam senden
        PostMessageW(hwnd, WM_KEYDOWN, vk as usize, lparam_down);
        thread::sleep(Duration::from_millis(10));
        // WM_KEYUP mit VK und lParam senden
        PostMessageW(hwnd, WM_KEYUP, vk as usize, lparam_up);
    }

    println!("[replay_pause_postmessage] F11 via PostMessageW an LMU gesendet (HWND={:?})", hwnd);
    Ok(())
}

/// Fallback: SendInput ohne Fokus-Wechsel (falls LMU-Fenster nicht gefunden wurde).
fn replay_pause_simple_fallback(binding: &KeyBinding) -> Result<(), String> {
    let mut flags_down: DWORD = KEYEVENTF_SCANCODE;
    let mut flags_up: DWORD = KEYEVENTF_SCANCODE | KEYEVENTF_KEYUP;
    if binding.extended {
        flags_down |= KEYEVENTF_EXTENDEDKEY;
        flags_up |= KEYEVENTF_EXTENDEDKEY;
    }

    unsafe {
        let ki = KEYBDINPUT { wVk: 0, wScan: binding.scan, dwFlags: flags_down, time: 0, dwExtraInfo: 0 };
        let mut inp = INPUT { type_: INPUT_KEYBOARD, u: INPUT_UNION { ki: std::mem::ManuallyDrop::new(ki) } };
        SendInput(1, &mut inp, std::mem::size_of::<INPUT>() as i32);
        thread::sleep(Duration::from_millis(10));
        let ki_up = KEYBDINPUT { wVk: 0, wScan: binding.scan, dwFlags: flags_up, time: 0, dwExtraInfo: 0 };
        let mut inp_up = INPUT { type_: INPUT_KEYBOARD, u: INPUT_UNION { ki: std::mem::ManuallyDrop::new(ki_up) } };
        SendInput(1, &mut inp_up, std::mem::size_of::<INPUT>() as i32);
    }

    println!("[replay_pause_simple_fallback] Play/Pause via SendInput gesendet");
    Ok(())
}

/// Generisches Hold-Key-System für Tasten, die gedrückt gehalten werden müssen.
/// Startet einen Hintergrund-Thread, der die Taste alle 50ms sendet, bis `hold_stop()` aufgerufen wird.
static HOLD_ACTIVE: Lazy<Arc<AtomicBool>> = Lazy::new(|| Arc::new(AtomicBool::new(false)));

fn hold_start(action: &str) -> Result<(), String> {
    if HOLD_ACTIVE.load(Ordering::SeqCst) {
        return Ok(());
    }
    HOLD_ACTIVE.store(true, Ordering::SeqCst);

    let binding = get_binding(action)?;
    let play_binding = get_binding("Replay Play").ok();
    let active = HOLD_ACTIVE.clone();
    let prev = unsafe { GetForegroundWindow() };
    thread::spawn(move || {
        // LMU 1x in den Vordergrund holen (wenn nötig)
        let hwnd = find_lmu_window();
        if hwnd.is_none() { 
            active.store(false, Ordering::SeqCst);
            return; 
        }
        let hwnd = hwnd.unwrap();

        unsafe {
            ShowWindow(hwnd, 9);
            thread::sleep(Duration::from_millis(30));
            SetForegroundWindow(hwnd);
            thread::sleep(Duration::from_millis(30));
        }

        let mut flags_down: DWORD = KEYEVENTF_SCANCODE;
        let mut flags_up: DWORD = KEYEVENTF_SCANCODE | KEYEVENTF_KEYUP;
        if binding.extended {
            flags_down |= KEYEVENTF_EXTENDEDKEY;
            flags_up |= KEYEVENTF_EXTENDEDKEY;
        }

        // Taste EINMAL drücken und HALTEN (KEYDOWN ohne KEYUP) – wie LMU selbst
        unsafe {
            let ki = KEYBDINPUT { wVk: 0, wScan: binding.scan, dwFlags: flags_down, time: 0, dwExtraInfo: 0 };
            let mut inp = INPUT { type_: INPUT_KEYBOARD, u: INPUT_UNION { ki: std::mem::ManuallyDrop::new(ki) } };
            SendInput(1, &mut inp, std::mem::size_of::<INPUT>() as i32);
        }

        // Warten bis Stop – Taste bleibt gedrückt
        while active.load(Ordering::SeqCst) {
            thread::sleep(Duration::from_millis(10));
        }

        // Taste loslassen (KEYUP)
        unsafe {
            let ki = KEYBDINPUT { wVk: 0, wScan: binding.scan, dwFlags: flags_up, time: 0, dwExtraInfo: 0 };
            let mut inp = INPUT { type_: INPUT_KEYBOARD, u: INPUT_UNION { ki: std::mem::ManuallyDrop::new(ki) } };
            SendInput(1, &mut inp, std::mem::size_of::<INPUT>() as i32);
        }

        // Automatisch Play (F11) senden wie LMU
        if let Some(pb) = play_binding {
            let mut pf_down: DWORD = KEYEVENTF_SCANCODE;
            let mut pf_up: DWORD = KEYEVENTF_SCANCODE | KEYEVENTF_KEYUP;
            if pb.extended {
                pf_down |= KEYEVENTF_EXTENDEDKEY;
                pf_up |= KEYEVENTF_EXTENDEDKEY;
            }
            unsafe {
                let ki = KEYBDINPUT { wVk: 0, wScan: pb.scan, dwFlags: pf_down, time: 0, dwExtraInfo: 0 };
                let mut inp = INPUT { type_: INPUT_KEYBOARD, u: INPUT_UNION { ki: std::mem::ManuallyDrop::new(ki) } };
                SendInput(1, &mut inp, std::mem::size_of::<INPUT>() as i32);
                thread::sleep(Duration::from_millis(1));
                let ki_up = KEYBDINPUT { wVk: 0, wScan: pb.scan, dwFlags: pf_up, time: 0, dwExtraInfo: 0 };
                let mut inp_up = INPUT { type_: INPUT_KEYBOARD, u: INPUT_UNION { ki: std::mem::ManuallyDrop::new(ki_up) } };
                SendInput(1, &mut inp_up, std::mem::size_of::<INPUT>() as i32);
            }
        }

        // Vorheriges Fenster wiederherstellen (nur wenn nicht LMU)
        if prev != 0 && prev != hwnd {
            thread::sleep(Duration::from_millis(50));
            unsafe { SetForegroundWindow(prev); }
        }
    });
    Ok(())
}

pub fn hold_stop() {
    HOLD_ACTIVE.store(false, Ordering::SeqCst);
}

/// Replay Slow-Motion (F10) – muss gedrückt gehalten werden
pub fn replay_slow() -> Result<(), String> {
    hold_start("Replay Slowmotion")
}

/// Replay Vorspulen (F9) – muss gedrückt gehalten werden
pub fn replay_forward() -> Result<(), String> {
    hold_start("Replay Fast Forward")
}

/// Replay schnell Zurückspulen (F8) – LMU-Kombi: F7 (Reverse) halten + F8 (Fast Rewind) senden
/// F7 wird gedrückt GEHALTEN (KEYDOWN ohne KEYUP), F8 wird alle 15ms press+release gesendet.
/// Beim Stop: F7 loslassen (KEYUP) + F11 (Play) senden.
pub fn rewind_fast() -> Result<(), String> {
    if HOLD_ACTIVE.load(Ordering::SeqCst) {
        return Ok(());
    }
    HOLD_ACTIVE.store(true, Ordering::SeqCst);

    let f7_binding = get_binding("Replay Reverse")?;
    let f8_binding = get_binding("Replay Fast Rewind")?;
    let play_binding = get_binding("Replay Play").ok();
    let active = HOLD_ACTIVE.clone();
    let prev = unsafe { GetForegroundWindow() };
    thread::spawn(move || {
        let hwnd = find_lmu_window();
        if hwnd.is_none() { 
            active.store(false, Ordering::SeqCst);
            return; 
        }
        let hwnd = hwnd.unwrap();

        unsafe {
            ShowWindow(hwnd, 9);
            thread::sleep(Duration::from_millis(30));
            SetForegroundWindow(hwnd);
            thread::sleep(Duration::from_millis(30));
        }

        // F7 Scancode-Flags
        let mut f7_down: DWORD = KEYEVENTF_SCANCODE;
        let mut f7_up: DWORD = KEYEVENTF_SCANCODE | KEYEVENTF_KEYUP;
        if f7_binding.extended {
            f7_down |= KEYEVENTF_EXTENDEDKEY;
            f7_up |= KEYEVENTF_EXTENDEDKEY;
        }
        // F8 Scancode-Flags
        let mut f8_down: DWORD = KEYEVENTF_SCANCODE;
        let mut f8_up: DWORD = KEYEVENTF_SCANCODE | KEYEVENTF_KEYUP;
        if f8_binding.extended {
            f8_down |= KEYEVENTF_EXTENDEDKEY;
            f8_up |= KEYEVENTF_EXTENDEDKEY;
        }

        // F7 drücken und HALTEN (nur KEYDOWN, kein KEYUP)
        unsafe {
            let ki = KEYBDINPUT { wVk: 0, wScan: f7_binding.scan, dwFlags: f7_down, time: 0, dwExtraInfo: 0 };
            let mut inp = INPUT { type_: INPUT_KEYBOARD, u: INPUT_UNION { ki: std::mem::ManuallyDrop::new(ki) } };
            SendInput(1, &mut inp, std::mem::size_of::<INPUT>() as i32);
        }

        // F8 drücken und HALTEN (nur KEYDOWN, kein KEYUP) – wie F7
        unsafe {
            let ki = KEYBDINPUT { wVk: 0, wScan: f8_binding.scan, dwFlags: f8_down, time: 0, dwExtraInfo: 0 };
            let mut inp = INPUT { type_: INPUT_KEYBOARD, u: INPUT_UNION { ki: std::mem::ManuallyDrop::new(ki) } };
            SendInput(1, &mut inp, std::mem::size_of::<INPUT>() as i32);
        }

        // Kurz warten, damit LMU die Kombination registriert
        thread::sleep(Duration::from_millis(50));

        // Warten bis Stop – beide Tasten bleiben gedrückt
        while active.load(Ordering::SeqCst) {
            thread::sleep(Duration::from_millis(10));
        }

        // F8 loslassen (KEYUP)
        unsafe {
            let ki = KEYBDINPUT { wVk: 0, wScan: f8_binding.scan, dwFlags: f8_up, time: 0, dwExtraInfo: 0 };
            let mut inp = INPUT { type_: INPUT_KEYBOARD, u: INPUT_UNION { ki: std::mem::ManuallyDrop::new(ki) } };
            SendInput(1, &mut inp, std::mem::size_of::<INPUT>() as i32);
        }

        // F7 loslassen (KEYUP)
        unsafe {
            let ki = KEYBDINPUT { wVk: 0, wScan: f7_binding.scan, dwFlags: f7_up, time: 0, dwExtraInfo: 0 };
            let mut inp = INPUT { type_: INPUT_KEYBOARD, u: INPUT_UNION { ki: std::mem::ManuallyDrop::new(ki) } };
            SendInput(1, &mut inp, std::mem::size_of::<INPUT>() as i32);
        }

        // Automatisch Play (F11) senden wie LMU
        if let Some(pb) = play_binding {
            let mut pf_down: DWORD = KEYEVENTF_SCANCODE;
            let mut pf_up: DWORD = KEYEVENTF_SCANCODE | KEYEVENTF_KEYUP;
            if pb.extended {
                pf_down |= KEYEVENTF_EXTENDEDKEY;
                pf_up |= KEYEVENTF_EXTENDEDKEY;
            }
            unsafe {
                let ki = KEYBDINPUT { wVk: 0, wScan: pb.scan, dwFlags: pf_down, time: 0, dwExtraInfo: 0 };
                let mut inp = INPUT { type_: INPUT_KEYBOARD, u: INPUT_UNION { ki: std::mem::ManuallyDrop::new(ki) } };
                SendInput(1, &mut inp, std::mem::size_of::<INPUT>() as i32);
                thread::sleep(Duration::from_millis(1));
                let ki_up = KEYBDINPUT { wVk: 0, wScan: pb.scan, dwFlags: pf_up, time: 0, dwExtraInfo: 0 };
                let mut inp_up = INPUT { type_: INPUT_KEYBOARD, u: INPUT_UNION { ki: std::mem::ManuallyDrop::new(ki_up) } };
                SendInput(1, &mut inp_up, std::mem::size_of::<INPUT>() as i32);
            }
        }

        // Vorheriges Fenster wiederherstellen
        if prev != 0 && prev != hwnd {
            thread::sleep(Duration::from_millis(50));
            unsafe { SetForegroundWindow(prev); }
        }
    });
    Ok(())
}

/// Replay Rückwärts (F7) – muss gedrückt gehalten werden
pub fn replay_reverse() -> Result<(), String> {
    hold_start("Replay Reverse")
}

/// Verlässt den Replay-Modus (Esc-Taste) – bringt LMU zurück zu Live.
pub fn replay_exit() -> Result<(), String> {
    thread::spawn(move || {
        let hwnd = find_lmu_window();
        if hwnd.is_none() {
            eprintln!("[replay_exit] LMU-Fenster nicht gefunden, sende Esc trotzdem...");
            send_scancode(SCAN_ESC, false);
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

        send_scancode(SCAN_ESC, false);
        thread::sleep(Duration::from_millis(200));

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

/// Gibt die relevanten Tastenbelegungen für das Frontend zurück.
/// Format: Vec<(action, scan, extended, key_name)> als einfache Struktur.
pub fn get_relevant_bindings() -> Vec<KeyboardMappingEntryFrontend> {
    let guard = config();
    let bindings = guard
        .as_ref()
        .map(|cfg| cfg.relevant_bindings())
        .unwrap_or_default();
    bindings
        .into_iter()
        .map(|(action, binding, key_name)| KeyboardMappingEntryFrontend {
            action,
            key_name,
            scan: binding.scan,
            extended: binding.extended,
        })
        .collect()
}

/// Vereinfachte Struktur fürs Frontend.
#[derive(Debug, Clone, serde::Serialize)]
pub struct KeyboardMappingEntryFrontend {
    pub action: String,
    pub key_name: String,
    pub scan: u16,
    pub extended: bool,
}
