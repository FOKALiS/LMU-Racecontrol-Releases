//! Zugriff auf den rFactor 2 / LMU Shared Memory.
//!
//! LMU (Le Mans Ultimate) basiert auf rFactor 2 und verwendet dieselbe
//! Shared-Memory-Schnittstelle. Der Shared Memory Block wird von LMU
//! beim Start erstellt und mit Live-Daten (u.a. Kamera-Status, Fahrzeug-
//! informationen) gefüllt.
//!
//! Durch direktes Schreiben in den Shared Memory können wir:
//! - Die Kamera umschalten (Kamera-Gruppe + Kamera-Index)
//! - Den Fahrzeug-Fokus setzen (Ziel-Fahrzeug-Slot)
//! - Die aktuelle Kamera-Konfiguration auslesen
//!
//! ## Vorteil gegenüber Tastatursimulation
//! Shared Memory ist der "Königsweg" für LMU/rFactor 2 Tools:
//! - Kein Fenster-Fokus nötig (funktioniert im Hintergrund)
//! - Kein Terminal-Flash
//! - Kein SendInput/PostMessage
//! - Sofortige Reaktion (kein Thread-Sleep)
//! - Auch von anderen Tools wie Broadcast Control UK verwendet

use std::ptr;
use std::sync::Mutex;

// ─── Win32 Shared Memory API ──────────────────────────────────────────

type HANDLE = isize;
type BOOL = i32;
type DWORD = u32;
type LPCWSTR = *const u16;
type LPVOID = *mut u8;

const FILE_MAP_ALL_ACCESS: DWORD = 0x000F001F;

extern "system" {
    fn OpenFileMappingW(
        dw_desired_access: DWORD,
        b_inherit_handle: BOOL,
        lp_name: LPCWSTR,
    ) -> HANDLE;
    fn MapViewOfFile(
        h_file_mapping_object: HANDLE,
        dw_desired_access: DWORD,
        dw_file_offset_high: DWORD,
        dw_file_offset_low: DWORD,
        dw_number_of_bytes_to_map: usize,
    ) -> LPVOID;
    fn UnmapViewOfFile(lp_base_address: LPVOID) -> BOOL;
    fn CloseHandle(h_object: HANDLE) -> BOOL;
    fn OpenMutexW(dw_desired_access: DWORD, b_inherit_handle: BOOL, lp_name: LPCWSTR) -> HANDLE;
    fn ReleaseMutex(h_mutex: HANDLE) -> BOOL;
    fn WaitForSingleObject(h_handle: HANDLE, dw_milliseconds: DWORD) -> DWORD;
}

// ─── Shared Memory Layout (rFactor 2) ─────────────────────────────────
//
// Offset  | Größe | Typ    | Feld
// --------|-------|--------|------------------------------------------
// 0x0000  | 4     | DWORD  | mVersion
// 0x0004  | 4     | DWORD  | mBuildVersion
// 0x0008  | 4     | DWORD  | mStatus
// 0x000C  | 4     | DWORD  | mSession
// 0x0010  | 4     | float  | mCurrentTime
// 0x0014  | 4     | float  | mStartTime
// 0x0018  | 4     | float  | mEndTime
// 0x001C  | 4     | DWORD  | mSessionLength
// 0x0020  | 4     | float  | mCompletedLaps
// 0x0024  | 4     | DWORD  | mCameraGroup    ← Kamera-Gruppe
// 0x0028  | 4     | DWORD  | mCurrentCamera  ← Kamera-ID
// 0x002C  | 4     | DWORD  | mTargetVehicle  ← Ziel-Fahrzeug

const OFFSET_CAMERA_GROUP: usize = 0x24;
const OFFSET_CURRENT_CAMERA: usize = 0x28;
const OFFSET_TARGET_VEHICLE: usize = 0x2C;

const SHARED_MEMORY_NAME: &str = "Local\\rFactor2SharedMemory";
const MUTEX_NAME: &str = "Local\\rFactor2SharedMemoryMutex";

// ─── Kamera-Mapping ───────────────────────────────────────────────────

struct CameraMapping {
    group: u32,
    camera: u32,
}

fn get_camera_mapping(cam_id: &str) -> Option<CameraMapping> {
    match cam_id {
        "TV" => Some(CameraMapping { group: 0, camera: 0 }),
        "Helmet" => Some(CameraMapping { group: 2, camera: 0 }),
        "Front" => Some(CameraMapping { group: 3, camera: 0 }),
        "Heck" | "Rear" => Some(CameraMapping { group: 4, camera: 0 }),
        "Top" => Some(CameraMapping { group: 5, camera: 0 }),
        "Behind" => Some(CameraMapping { group: 6, camera: 0 }),
        _ => None,
    }
}

// ─── Wrapper für Send/Sync (da *mut u8 weder Send noch Sync ist) ──────

struct SharedMemoryView {
    view: *mut u8,
    view_size: usize,
    mapping: HANDLE,
}

// `*mut u8` ist weder Send noch Sync, aber wir schützen den Zugriff
// durch einen Mutex. Daher ist es sicher, Send/Sync zu implementieren.
unsafe impl Send for SharedMemoryView {}
unsafe impl Sync for SharedMemoryView {}

impl SharedMemoryView {
    fn open() -> Option<Self> {
        unsafe {
            let name: Vec<u16> = SHARED_MEMORY_NAME
                .encode_utf16()
                .chain(std::iter::once(0))
                .collect();

            let mapping = OpenFileMappingW(FILE_MAP_ALL_ACCESS, 0, name.as_ptr());
            if mapping == 0 {
                println!("[shared_memory] LMU Shared Memory nicht gefunden (läuft LMU?)");
                return None;
            }

            let view_size: usize = 4096;
            let view = MapViewOfFile(mapping, FILE_MAP_ALL_ACCESS, 0, 0, view_size);
            if view.is_null() {
                println!("[shared_memory] MapViewOfFile fehlgeschlagen");
                CloseHandle(mapping);
                return None;
            }

            println!("[shared_memory] LMU Shared Memory geöffnet");
            Some(SharedMemoryView { view, view_size, mapping })
        }
    }

    fn read_u32(&self, offset: usize) -> Option<u32> {
        if offset + 4 > self.view_size {
            return None;
        }
        unsafe {
            let ptr = self.view.add(offset) as *const u32;
            Some(ptr::read_unaligned(ptr))
        }
    }

    fn write_u32(&self, offset: usize, value: u32) -> bool {
        if offset + 4 > self.view_size {
            return false;
        }

        let mutex_name: Vec<u16> = MUTEX_NAME
            .encode_utf16()
            .chain(std::iter::once(0))
            .collect();

        unsafe {
            let mutex = OpenMutexW(0x1F0001, 0, mutex_name.as_ptr());
            if mutex != 0 {
                WaitForSingleObject(mutex, 1000);
            }

            let ptr = self.view.add(offset) as *mut u32;
            ptr::write_unaligned(ptr, value);

            if mutex != 0 {
                ReleaseMutex(mutex);
                CloseHandle(mutex);
            }
        }
        true
    }

    fn set_camera(&self, cam_id: &str) -> Result<(), String> {
        let mapping = get_camera_mapping(cam_id)
            .ok_or_else(|| format!("Unbekannte Kamera-ID: {}", cam_id))?;

        self.write_u32(OFFSET_CAMERA_GROUP, mapping.group);
        self.write_u32(OFFSET_CURRENT_CAMERA, mapping.camera);

        println!(
            "[shared_memory] Kamera gewechselt: {} (Gruppe={}, Kamera={})",
            cam_id, mapping.group, mapping.camera
        );
        Ok(())
    }

    fn focus_vehicle(&self, target_slot: u32) -> Result<(), String> {
        self.write_u32(OFFSET_TARGET_VEHICLE, target_slot);
        println!("[shared_memory] Fahrzeug-Fokus auf Slot {} gesetzt", target_slot);
        Ok(())
    }
}

impl Drop for SharedMemoryView {
    fn drop(&mut self) {
        unsafe {
            if !self.view.is_null() {
                UnmapViewOfFile(self.view);
            }
            if self.mapping != 0 {
                CloseHandle(self.mapping);
            }
        }
        println!("[shared_memory] LMU Shared Memory geschlossen");
    }
}

// ─── Globaler Singleton (thread-safe via Mutex) ───────────────────────

use std::sync::OnceLock;

static SHARED_MEMORY_INSTANCE: OnceLock<Mutex<Option<SharedMemoryView>>> = OnceLock::new();

fn get_shared_memory() -> &'static Mutex<Option<SharedMemoryView>> {
    SHARED_MEMORY_INSTANCE.get_or_init(|| {
        println!("[shared_memory] Versuche, LMU Shared Memory zu öffnen...");
        Mutex::new(SharedMemoryView::open())
    })
}

/// Prüft, ob Shared Memory verfügbar ist (LMU läuft?).
pub fn is_available() -> bool {
    get_shared_memory()
        .lock()
        .unwrap()
        .is_some()
}

/// Schaltet die LMU-Kamera über Shared Memory um.
/// Gibt einen Fehler zurück, wenn Shared Memory nicht verfügbar ist.
pub fn switch_camera_sm(cam_id: &str) -> Result<(), String> {
    let guard = get_shared_memory().lock().unwrap();
    match guard.as_ref() {
        Some(sm) => sm.set_camera(cam_id),
        None => Err("Shared Memory nicht verfügbar (LMU läuft nicht?)".to_string()),
    }
}

/// Fokussiert ein Fahrzeug über Shared Memory.
/// `slot_id` ist die LMU-Slot-ID (nicht die Startnummer!).
pub fn focus_vehicle_sm(slot_id: u32) -> Result<(), String> {
    let guard = get_shared_memory().lock().unwrap();
    match guard.as_ref() {
        Some(sm) => sm.focus_vehicle(slot_id),
        None => Err("Shared Memory nicht verfügbar (LMU läuft nicht?)".to_string()),
    }
}