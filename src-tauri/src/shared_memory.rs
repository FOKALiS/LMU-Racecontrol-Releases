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
use std::collections::HashMap;

// ─── Win32 Shared Memory API ──────────────────────────────────────────

type HANDLE = isize;
type BOOL = i32;
type DWORD = u32;
type LPCWSTR = *const u16;
type LPVOID = *mut u8;

const FILE_MAP_READ: DWORD = 0x0004;
const FILE_MAP_ALL_ACCESS: DWORD = 0x000F001F;

// ─── LMU Shared Memory Offsets (aus SM Bridge Log bestätigt) ──────────
// Siehe bridge_2026-07-27.log:
//   Scoring=1632, VehScoring=2192, Telemetry=128464, VehTelem=128468
//   VehicleTelemetrySize=1888

const SCORING_OFFSET: usize = 1632;
const VEHICLE_TELEMETRY_OFFSET: usize = 128468;
const VEHICLE_TELEMETRY_SIZE: usize = 1888;

/// Liest für alle Fahrzeuge die Impact-Daten (impactET, impactMag) aus dem Shared Memory.
/// Gibt eine HashMap<slot_id, (impact_et, impact_mag)> zurück.
pub fn read_impact_data() -> HashMap<i64, (f64, f64)> {
    let mut result = HashMap::new();
    
    unsafe {
        let wide: Vec<u16> = "LMU_Data"
            .encode_utf16()
            .chain(std::iter::once(0))
            .collect();
        
        let mapping = OpenFileMappingW(FILE_MAP_READ, 0, wide.as_ptr());
        if mapping == 0 {
            return result;
        }
        
        let view = MapViewOfFile(mapping, FILE_MAP_READ, 0, 0, 4 * 1024 * 1024);
        if view.is_null() {
            CloseHandle(mapping);
            return result;
        }
        
        // Anzahl Fahrzeuge aus Scoring-Header
        let num_vehicles = *(view.add(SCORING_OFFSET + 24) as *const i32);
        let max_vehicles = num_vehicles.min(64).max(0) as usize;
        
        for i in 0..max_vehicles {
            let base = VEHICLE_TELEMETRY_OFFSET + i * VEHICLE_TELEMETRY_SIZE;
            // slot_id bei Offset 0 (int32)
            let slot_id = *(view.add(base) as *const i32) as i64;
            // impactET bei Offset 4 (float)
            let impact_et = *(view.add(base + 4) as *const f32) as f64;
            // impactMag bei Offset 8 (float)
            let impact_mag = *(view.add(base + 8) as *const f32) as f64;
            
            if slot_id >= 0 {
                result.insert(slot_id, (impact_et, impact_mag));
            }
        }
        
        UnmapViewOfFile(view);
        CloseHandle(mapping);
    }
    
    result
}

/// Liest für alle Fahrzeuge die Energie-Daten aus dem Shared Memory.
/// Gibt eine HashMap<slot_id, (virtual_energy, battery_charge_fraction, fuel_fraction)> zurück.
///
/// Offsets (VehicleTelemetry, 1888 Bytes pro Fahrzeug):
///   0x000: slotID (i32)
///   0x004: impactET (f32)
///   0x008: impactMag (f32)
///   0x???: mVirtualEnergy (f32) – WERT NOCH NICHT BESTIMMT!
///   0x???: mBatteryChargeFraction (f32) – WERT NOCH NICHT BESTIMMT!
///   0x???: mFuelFraction (f32) – WERT NOCH NICHT BESTIMMT!
///
/// TODO: Diese Offsets müssen mit `scripts/find_energy_offset.py` bei laufendem LMU
///       ermittelt werden! Aktuell sind es Platzhalter.
const VIRTUAL_ENERGY_OFFSET: usize = 0x100;   // TODO: echten Offset ermitteln!
const BATTERY_CHARGE_OFFSET: usize = 0x0F0;   // TODO: echten Offset ermitteln!
const FUEL_FRACTION_OFFSET: usize = 0x0EC;    // TODO: echten Offset ermitteln!

/// Schreibt die ersten 400 Bytes der VehicleTelemetry in eine JSON-Datei.
/// Wird bei jedem Polling-Durchlauf aufgerufen, damit wir die Float-Werte sehen.
pub fn debug_dump_telemetry() {
    use std::collections::HashMap;
    
    unsafe {
        let wide: Vec<u16> = "LMU_Data"
            .encode_utf16()
            .chain(std::iter::once(0))
            .collect();
        
        let mapping = OpenFileMappingW(FILE_MAP_READ, 0, wide.as_ptr());
        if mapping == 0 {
            return;
        }
        
        let view = MapViewOfFile(mapping, FILE_MAP_READ, 0, 0, 4 * 1024 * 1024);
        if view.is_null() {
            CloseHandle(mapping);
            return;
        }
        
        let num_vehicles = *(view.add(SCORING_OFFSET + 24) as *const i32);
        if num_vehicles <= 0 {
            UnmapViewOfFile(view);
            CloseHandle(mapping);
            return;
        }
        
        // Einmalig: Rohdaten für Slot 0 dumpen
        let base = VEHICLE_TELEMETRY_OFFSET;
        let slot_id = *(view.add(base) as *const i32);
        
        let mut data = HashMap::new();
        for off in (0..400).step_by(4) {
            let val = *(view.add(base + off) as *const f32);
            data.insert(format!("0x{:03X}", off), val);
        }
        
        let result = serde_json::json!({
            "slot_id": slot_id,
            "num_vehicles": num_vehicles,
            "telemetry_raw": data,
        });
        
        let dump_path = "C:\\Users\\Administrator\\energy_offsets_found.json";
        let _ = std::fs::write(dump_path, serde_json::to_string_pretty(&result).unwrap());
        
        UnmapViewOfFile(view);
        CloseHandle(mapping);
    }
}

pub fn read_energy_data() -> HashMap<i64, (f64, f64, f64)> {
    let mut result = HashMap::new();
    
    unsafe {
        let wide: Vec<u16> = "LMU_Data"
            .encode_utf16()
            .chain(std::iter::once(0))
            .collect();
        
        let mapping = OpenFileMappingW(FILE_MAP_READ, 0, wide.as_ptr());
        if mapping == 0 {
            return result;
        }
        
        let view = MapViewOfFile(mapping, FILE_MAP_READ, 0, 0, 4 * 1024 * 1024);
        if view.is_null() {
            CloseHandle(mapping);
            return result;
        }
        
        let num_vehicles = *(view.add(SCORING_OFFSET + 24) as *const i32);
        let max_vehicles = num_vehicles.min(64).max(0) as usize;
        
        for i in 0..max_vehicles {
            let base = VEHICLE_TELEMETRY_OFFSET + i * VEHICLE_TELEMETRY_SIZE;
            let slot_id = *(view.add(base) as *const i32) as i64;
            
            // Lese Energie-Daten (falls Offset gültig, sonst 0.0)
            let virtual_energy = if VIRTUAL_ENERGY_OFFSET + 4 < VEHICLE_TELEMETRY_SIZE {
                *(view.add(base + VIRTUAL_ENERGY_OFFSET) as *const f32) as f64
            } else {
                0.0
            };
            let battery_charge = if BATTERY_CHARGE_OFFSET + 4 < VEHICLE_TELEMETRY_SIZE {
                *(view.add(base + BATTERY_CHARGE_OFFSET) as *const f32) as f64
            } else {
                0.0
            };
            let fuel_fraction = if FUEL_FRACTION_OFFSET + 4 < VEHICLE_TELEMETRY_SIZE {
                *(view.add(base + FUEL_FRACTION_OFFSET) as *const f32) as f64
            } else {
                0.0
            };
            
            if slot_id >= 0 {
                result.insert(slot_id, (virtual_energy, battery_charge, fuel_fraction));
            }
        }
        
        UnmapViewOfFile(view);
        CloseHandle(mapping);
    }
    
    result
}

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