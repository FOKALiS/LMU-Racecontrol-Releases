//! Zugriff auf den LMU Shared Memory (LMU_Data).
//!
//! Broadcast Control UK hat bestätigt: Der korrekte Shared Memory Name
//! in LMU ist **LMU_Data**.
//!
//! WICHTIG: Der LMU Shared Memory wird NICHT für die Kamera-Steuerung
//! verwendet! Die Kamera-Steuerung erfolgt über die LMU REST-API
//! (PUT /rest/watch/focus/{type}/{group}/{advance}). Die rFactor2-Offsets
//! 0x24/0x28 sind für LMU ungültig - LMU hat eine komplett andere
//! Shared Memory Struktur als rFactor2.
//!
//! ## Verwendung (nur Read-Access)
//! ```rust
//! if let Some(sm) = shared_memory::try_open() {
//!     // Nur Lesen von Shared Memory Daten, KEINE Kamera-Steuerung!
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

const LMU_DATA_NAME: &str = "LMU_Data";

// ─── Shared Memory View (nur lesen) ───────────────────────────────────

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

    /// Liest einen u32-Wert an einem bestimmten Offset.
    pub fn read_u32(&self, offset: usize) -> u32 {
        unsafe {
            let ptr = self.view.add(offset) as *mut u32;
            ptr::read_unaligned(ptr)
        }
    }

    /// Liest einen f32-Wert an einem bestimmten Offset.
    pub fn read_f32(&self, offset: usize) -> f32 {
        unsafe {
            let ptr = self.view.add(offset) as *mut f32;
            ptr::read_unaligned(ptr)
        }
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