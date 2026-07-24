//! Zugriff auf den rFactor 2 / LMU Shared Memory (optional).
//!
//! WARNUNG: Dieser Code wird beim Start NICHT ausgeführt. Der Shared Memory
//! wird nur bei Bedarf geöffnet (lazy). Wenn LMU nicht läuft, wird einfach
//! `None` zurückgegeben - kein Blockieren, kein Absturz.
//!
//! ## Verwendung
//! ```rust
//! if let Some(sm) = shared_memory::try_open() {
//!     sm.set_camera("TV").ok();
//! }
//! ```

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

// ─── Shared Memory Layout ─────────────────────────────────────────────

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

// ─── Shared Memory View ───────────────────────────────────────────────

pub struct SharedMemoryView {
    view: *mut u8,
    view_size: usize,
    mapping: HANDLE,
}

unsafe impl Send for SharedMemoryView {}
unsafe impl Sync for SharedMemoryView {}

impl SharedMemoryView {
    /// Öffnet den Shared Memory. Gibt `None` zurück, wenn LMU nicht läuft.
    pub fn open() -> Option<Self> {
        unsafe {
            let name: Vec<u16> = SHARED_MEMORY_NAME
                .encode_utf16()
                .chain(std::iter::once(0))
                .collect();

            let mapping = OpenFileMappingW(FILE_MAP_ALL_ACCESS, 0, name.as_ptr());
            if mapping == 0 {
                return None;
            }

            let view_size: usize = 4096;
            let view = MapViewOfFile(mapping, FILE_MAP_ALL_ACCESS, 0, 0, view_size);
            if view.is_null() {
                CloseHandle(mapping);
                return None;
            }

            Some(SharedMemoryView { view, view_size, mapping })
        }
    }

    pub fn set_camera(&self, cam_id: &str) -> Result<(), String> {
        let mapping = get_camera_mapping(cam_id)
            .ok_or_else(|| format!("Unbekannte Kamera-ID: {}", cam_id))?;

        let mutex_name: Vec<u16> = MUTEX_NAME
            .encode_utf16()
            .chain(std::iter::once(0))
            .collect();

        unsafe {
            let mutex = OpenMutexW(0x1F0001, 0, mutex_name.as_ptr());
            if mutex != 0 {
                WaitForSingleObject(mutex, 100);
            }

            let ptr_group = self.view.add(OFFSET_CAMERA_GROUP) as *mut u32;
            ptr::write_unaligned(ptr_group, mapping.group);

            let ptr_cam = self.view.add(OFFSET_CURRENT_CAMERA) as *mut u32;
            ptr::write_unaligned(ptr_cam, mapping.camera);

            if mutex != 0 {
                ReleaseMutex(mutex);
                CloseHandle(mutex);
            }
        }

        println!(
            "[shared_memory] Kamera gewechselt: {} (Gruppe={}, Kamera={})",
            cam_id, mapping.group, mapping.camera
        );
        Ok(())
    }

    pub fn focus_vehicle(&self, target_slot: u32) -> Result<(), String> {
        let mutex_name: Vec<u16> = MUTEX_NAME
            .encode_utf16()
            .chain(std::iter::once(0))
            .collect();

        unsafe {
            let mutex = OpenMutexW(0x1F0001, 0, mutex_name.as_ptr());
            if mutex != 0 {
                WaitForSingleObject(mutex, 100);
            }

            let ptr = self.view.add(OFFSET_TARGET_VEHICLE) as *mut u32;
            ptr::write_unaligned(ptr, target_slot);

            if mutex != 0 {
                ReleaseMutex(mutex);
                CloseHandle(mutex);
            }
        }

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
    }
}

// ─── Öffentliche API ───────────────────────────────────────────────────

/// Versucht, den Shared Memory zu öffnen. Gibt `None` zurück, wenn LMU
/// nicht läuft. Blockiert NICHT beim Start.
pub fn try_open() -> Option<SharedMemoryView> {
    SharedMemoryView::open()
}