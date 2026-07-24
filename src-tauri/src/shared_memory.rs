//! Zugriff auf den LMU Shared Memory.
//!
//! Broadcast Control UK hat gezeigt: Der korrekte Shared Memory Name
//! in LMU ist **LMU_Data** (nicht `rFactor2SharedMemory`!).
//!
//! ## Verwendung
//! ```rust
//! if let Some(sm) = shared_memory::try_open() {
//!     sm.set_camera("TV").ok();
//! }
//! ```

use std::ptr;

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
}

// ─── Shared Memory Name (LMU-spezifisch!) ─────────────────────────────
//
// Broadcast Control UK verwendet "LMU_Data". Das ist der korrekte Name
// für Le Mans Ultimate (abweichend von rFactor 2).

const LMU_DATA_NAME: &str = "LMU_Data";

// ─── Kamera-Offsets (gleiche Positionen wie rFactor 2 Shared Memory) ───

const OFFSET_CAMERA_GROUP: usize = 0x24;
const OFFSET_CURRENT_CAMERA: usize = 0x28;

// ─── Kamera-Mapping ───────────────────────────────────────────────────
//
// Broadcast Control UK verwendet:
//   TV Cycle  -> Gruppe 4 (Trackside-Zyklus)
//   Onboard   -> Gruppe 6 (Onboard-Kameras)
// Wir nutzen die Standard-rFactor2-Gruppen als Basis:

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
    mapping: HANDLE,
}

unsafe impl Send for SharedMemoryView {}
unsafe impl Sync for SharedMemoryView {}

impl SharedMemoryView {
    /// Öffnet den LMU Shared Memory. Gibt `None` zurück, wenn LMU nicht läuft.
    pub fn open() -> Option<Self> {
        unsafe {
            let wide: Vec<u16> = LMU_DATA_NAME
                .encode_utf16()
                .chain(std::iter::once(0))
                .collect();

            let mapping = OpenFileMappingW(FILE_MAP_ALL_ACCESS, 0, wide.as_ptr());
            if mapping == 0 {
                return None;
            }

            let view = MapViewOfFile(mapping, FILE_MAP_ALL_ACCESS, 0, 0, 4096);
            if view.is_null() {
                CloseHandle(mapping);
                return None;
            }

            println!("[shared_memory] LMU_Data geöffnet");
            Some(SharedMemoryView { view: view as *mut u8, mapping })
        }
    }

    pub fn set_camera(&self, cam_id: &str) -> Result<(), String> {
        let mapping = get_camera_mapping(cam_id)
            .ok_or_else(|| format!("Unbekannte Kamera-ID: {}", cam_id))?;

        unsafe {
            let ptr_group = self.view.add(OFFSET_CAMERA_GROUP) as *mut u32;
            ptr::write_unaligned(ptr_group, mapping.group);

            let ptr_cam = self.view.add(OFFSET_CURRENT_CAMERA) as *mut u32;
            ptr::write_unaligned(ptr_cam, mapping.camera);
        }

        println!(
            "[shared_memory] Kamera gewechselt: {} (Gruppe={}, Kamera={})",
            cam_id, mapping.group, mapping.camera
        );
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

/// Versucht, den Shared Memory zu öffnen. Gibt `None` zurück, wenn LMU
/// nicht läuft. Blockiert NICHT beim Start.
pub fn try_open() -> Option<SharedMemoryView> {
    SharedMemoryView::open()
}