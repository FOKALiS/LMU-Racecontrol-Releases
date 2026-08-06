//! Scannt die VehicleTelemetry-Struktur nach Energie-Offsets.
//! Läuft im User-Kontext (gleicher Zugriff wie die Tauri-App).
//! Schreibt Ergebnis in energy_offsets_found.json.
//!
//! Nutzung: cargo run --bin energy_scanner
//! (LMU muss laufen und im Rennen sein!)

use std::collections::HashMap;
use std::fs;
use std::path::Path;
use std::time::Duration;
use std::thread::sleep;

type HANDLE = isize;
type BOOL = i32;
type DWORD = u32;
type LPCWSTR = *const u16;
type LPVOID = *mut u8;

const FILE_MAP_READ: DWORD = 0x0004;
const FILE_MAP_ALL_ACCESS: DWORD = 0x000F001F;
const SCORING_OFFSET: usize = 1632;
const VEHICLE_TELEMETRY_OFFSET: usize = 128468;
const VEHICLE_TELEMETRY_SIZE: usize = 1888;

extern "system" {
    fn OpenFileMappingW(dw_desired_access: DWORD, b_inherit_handle: BOOL, lp_name: LPCWSTR) -> HANDLE;
    fn MapViewOfFile(h_file_mapping_object: HANDLE, dw_desired_access: DWORD, dw_file_offset_high: DWORD, dw_file_offset_low: DWORD, dw_number_of_bytes_to_map: usize) -> LPVOID;
    fn UnmapViewOfFile(lp_base_address: LPVOID) -> BOOL;
    fn CloseHandle(h_object: HANDLE) -> BOOL;
}

fn open_sm() -> Option<(HANDLE, *mut u8)> {
    unsafe {
        let wide: Vec<u16> = "LMU_Data".encode_utf16().chain(std::iter::once(0)).collect();
        let mapping = OpenFileMappingW(FILE_MAP_ALL_ACCESS, 0, wide.as_ptr());
        let mapping = if mapping != 0 { mapping } else { OpenFileMappingW(FILE_MAP_READ, 0, wide.as_ptr()) };
        if mapping == 0 { return None; }
        let view = MapViewOfFile(mapping, FILE_MAP_ALL_ACCESS, 0, 0, 4 * 1024 * 1024);
        let view = if !view.is_null() { view } else { MapViewOfFile(mapping, FILE_MAP_READ, 0, 0, 4 * 1024 * 1024) };
        if view.is_null() { CloseHandle(mapping); return None; }
        Some((mapping, view))
    }
}

fn read_f32(view: *mut u8, offset: usize) -> f32 {
    unsafe { *(view.add(offset) as *const f32) }
}

fn read_i32(view: *mut u8, offset: usize) -> i32 {
    unsafe { *(view.add(offset) as *const i32) }
}

fn main() {
    println!("{}", "=".repeat(70));
    println!("  LMU ENERGY OFFSET SCANNER");
    println!("{}", "=".repeat(70));
    println!("  LMU muss laufen und im Rennen sein!");
    println!();

    let (mapping, view) = match open_sm() {
        Some(v) => v,
        None => { println!("❌ LMU_Data nicht gefunden"); return; }
    };

    let num_vehicles = read_i32(view, SCORING_OFFSET + 24);
    println!("✅ LMU_Data geöffnet! Fahrzeuge: {}", num_vehicles);
    println!();

    if num_vehicles <= 0 {
        println!("❌ Keine Fahrzeuge");
        unsafe { UnmapViewOfFile(view); CloseHandle(mapping); }
        return;
    }

    // 1. Alle Float-Werte 0.0-1.0 pro Fahrzeug
    println!("--- Scan 1: Float-Werte 0.0-1.0 ---");
    for veh in 0..num_vehicles.min(8) {
        let base = VEHICLE_TELEMETRY_OFFSET + veh as usize * VEHICLE_TELEMETRY_SIZE;
        let slot_id = read_i32(view, base);
        println!("  Slot {}:", slot_id);
        for off in (12..400).step_by(4) {
            let val = read_f32(view, base + off);
            if val >= 0.0 && val <= 1.0 {
                println!("    +0x{:03X} = {:.4}", off, val);
            }
        }
    }

    // 2. Live-Überwachung: 5 Ticks
    println!();
    println!("--- Scan 2: Live-Überwachung (5s) ---");
    let mut ticks: Vec<HashMap<i32, HashMap<usize, f32>>> = Vec::new();
    for tick in 0..5 {
        let mut tick_data: HashMap<i32, HashMap<usize, f32>> = HashMap::new();
        for veh in 0..num_vehicles.min(32) {
            let base = VEHICLE_TELEMETRY_OFFSET + veh as usize * VEHICLE_TELEMETRY_SIZE;
            let slot_id = read_i32(view, base);
            let mut vals = HashMap::new();
            for off in (12..400).step_by(4) {
                let val = read_f32(view, base + off);
                if val >= 0.0 && val <= 1.0 { vals.insert(off, val); }
            }
            if !vals.is_empty() { tick_data.insert(slot_id, vals); }
        }
        println!("  Tick {}: {} Fahrzeuge", tick + 1, tick_data.len());
        ticks.push(tick_data);
        if tick < 4 { sleep(Duration::from_secs(1)); }
    }

    // 3. Analyse
    println!();
    println!("--- Analyse ---");
    let mut variable_offsets: HashMap<usize, Vec<f32>> = HashMap::new();
    let mut constant_offsets: HashMap<usize, Vec<f32>> = HashMap::new();

    if let Some(first_slot) = ticks[0].keys().next() {
        let slot_id = *first_slot;
        println!("  Analysiere Slot {}", slot_id);
        for off in (12..400).step_by(4) {
            let mut values = Vec::new();
            for tick in &ticks {
                if let Some(slot_data) = tick.get(&slot_id) {
                    if let Some(val) = slot_data.get(&off) { values.push(*val); }
                }
            }
            if values.len() >= 3 {
                let min_val = values.iter().cloned().fold(f32::MAX, f32::min);
                let max_val = values.iter().cloned().fold(f32::MIN, f32::max);
                let range = max_val - min_val;
                if range > 0.01 {
                    println!("  🔄 +0x{:03X}: variabel range={:.4} {:?}", off, range, values);
                    variable_offsets.insert(off, values);
                } else if range > 0.0 {
                    println!("  📊 +0x{:03X}: konstant range={:.6} {:?}", off, range, values);
                    constant_offsets.insert(off, values);
                }
            }
        }
    }

    // 4. Ergebnisse
    println!();
    println!("{}", "=".repeat(70));
    println!("  ERGEBNISSE");
    println!("{}", "=".repeat(70));
    println!();
    println!("  Variable Offsets (mVirtualEnergy):");
    let mut sorted_vars: Vec<_> = variable_offsets.iter().collect();
    sorted_vars.sort_by_key(|(off, _)| **off);
    for (off, vals) in &sorted_vars {
        println!("    +0x{:03X}: {:?}", off, vals);
    }
    println!();
    println!("  Konstante Offsets (mFuelFraction, mBatteryChargeFraction):");
    let mut sorted_const: Vec<_> = constant_offsets.iter().collect();
    sorted_const.sort_by_key(|(off, _)| **off);
    for (off, vals) in &sorted_const {
        println!("    +0x{:03X}: {:?}", off, vals);
    }

    // 5. Empfehlung
    println!();
    println!("{}", "=".repeat(70));
    println!("  EMPFEHLUNG für shared_memory.rs");
    println!("{}", "=".repeat(70));
    println!();
    if let Some((best_off, _)) = sorted_vars.first() {
        println!("    const VIRTUAL_ENERGY_OFFSET: usize = 0x{:03X};", best_off);
    }
    for (i, (off, vals)) in sorted_const.iter().enumerate().take(2) {
        let name = if i == 0 { "FUEL_FRACTION" } else { "BATTERY_CHARGE" };
        println!("    const {}_OFFSET: usize = 0x{:03X};  // {:.4}", name, off, vals[0]);
    }
    println!();

    // 6. JSON-Export
    let result = serde_json::json!({
        "num_vehicles": num_vehicles,
        "variable_offsets": variable_offsets.iter().map(|(k, v)| {
            (format!("0x{:03X}", k), v.iter().map(|x| format!("{:.4}", x)).collect::<Vec<_>>())
        }).collect::<HashMap<_, _>>(),
        "constant_offsets": constant_offsets.iter().map(|(k, v)| {
            (format!("0x{:03X}", k), v.iter().map(|x| format!("{:.4}", x)).collect::<Vec<_>>())
        }).collect::<HashMap<_, _>>(),
    });
    let out_path = Path::new("energy_offsets_found.json");
    fs::write(out_path, serde_json::to_string_pretty(&result).unwrap()).ok();
    println!("  ✅ energy_offsets_found.json geschrieben");

    unsafe { UnmapViewOfFile(view); CloseHandle(mapping); }
    println!("✅ Fertig!");
}