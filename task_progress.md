# Aufgabenfortschritt - Kamera-Steuerung, Bild-Steuerung & Fahrer-Fokus

## Abgeschlossene Änderungen

### 1. `src-tauri/src/keyboard.rs` - Komplett neu geschrieben
- **`enigo`-Abhängigkeit entfernt**: Ersetzt durch direkte Win32 `SendInput`-API
- **Kein PowerShell-Fenster mehr**: `powershell.exe AppActivate` durch Win32 `FindWindowW`/`SetForegroundWindow` ersetzt - kein aufblitzendes Terminal
- **Hintergrund-Thread-Architektur**: Tastatursimulation läuft in einem dedizierten Thread mit eigener Windows-Nachrichtenschleife, verhindert Blockierung des async-Runtimes
- **Korrekte INPUT-Struktur**: Für x64 Windows korrekt dimensioniert (40 Bytes laut Win32-API-Spezifikation)
- **Eingabe-Puffer leeren**: Verwendet `PeekMessageW` zum Leeren des Eingabepuffers nach dem Senden von Tasten
- **Fensterzustand**: Prüft `IsIconic` und ruft `ShowWindow(SW_RESTORE)` auf, falls LMU minimiert ist

### 2. `src-tauri/Cargo.toml` - `enigo` entfernt
- `enigo = { version = "0.2", features = ["serde"] }` vollständig entfernt
- Cargo.lock wird beim nächsten Build automatisch aktualisiert

### 3. `src-tauri/src/main.rs` - Async-Korrektur
- `focus_driver` verwendet jetzt `tokio::time::sleep` statt `std::thread::sleep`
- Kommentare zur nicht-blockierenden Natur der keyboard-Funktionen hinzugefügt

### 4. `frontend/src/App.tsx` - Zentraler Kamera-State
- Neuer `selectedCam`-State (initial "TV") für view-übergreifende Kamera-Auswahl
- `selectCamera` aktualisiert jetzt sowohl den lokalen State als auch ruft das Backend auf
- `selectedCam` wird an alle Views als Prop übergeben

### 5. `frontend/src/views/FahrerfeldView.tsx` - Sortierung + Kamera
- **Sortierung nach Position**: Die Fahrerliste wird jetzt via `useMemo` nach `car.position` sortiert (1., 2., 3., ...)
- `selectedCam`-Prop ins Interface aufgenommen und an TopToolbar weitergereicht
- `handleCamSelect` ruft nur noch `onCamSelect` auf (State wird zentral in App.tsx verwaltet)

### 6. `frontend/src/views/VorfaelleView.tsx` - Kamera-Prop
- `selectedCam`-Prop ins Interface aufgenommen
- Wird an TopToolbar weitergereicht

### 7. `frontend/src/views/ArchivView.tsx` - Kamera-Prop
- `selectedCam`-Prop ins Interface aufgenommen
- Wird an TopToolbar weitergereicht

## Technische Verbesserungen

| Problem | Vorher | Nachher |
|---------|--------|---------|
| Terminal-Fenster blitzt auf | PowerShell `AppActivate` erzeugt sichtbares Fenster | Win32 `FindWindowW`/`SetForegroundWindow` - kein Fenster |
| Tastatur-Zuverlässigkeit | `enigo`-Crate v0.2 (kann bei Spiel-Fenstern versagen) | Direkte `SendInput`-API mit Hardware-nähen Eingaben |
| Async-Blockierung | `std::thread::sleep` in async-Kommandos | Hintergrund-Thread mit Channel-basierter Kommunikation |
| Fenster-Fokus | PowerShell COM-Objekt | Native Win32-API |
| Eingabe-Puffer | Keine Bereinigung | `PeekMessageW`-Flush nach jedem Befehl |
| Sortierung Fahrerfeld | Keine explizite Sortierung | Sortierung nach Position via `useMemo` |
| Kamera-Auswahl aktiv | `selectedCam` wurde nirgendwo gespeichert | Zentraler State in `App.tsx`, view-übergreifend |

## Wichtiger Hinweis zu Admin-Rechten
Wenn LMU mit Administrator-Rechten läuft und die Tauri-App als normaler Benutzer, kann Windows UIPI (User Interface Privilege Isolation) die `SendInput`-API blockieren. In diesem Fall sollte die Tauri-App ebenfalls als Administrator ausgeführt werden, ODER das `KEYEVENTF_SCANCODE`-Flag sollte anstelle von virtuellen Tastencodes verwendet werden (Spiele verwenden oft Scancodes). Die aktuelle Implementierung verwendet virtuelle Tastencodes als Standard-Ansatz.

## Nächste Schritte
- [ ] Build testen (Cargo ist auf diesem System nicht installiert)
- [ ] Funktion im Live-Betrieb mit LMU testen
- [ ] Ggf. Tastatur-Scancodes statt virtueller Codes verwenden, falls Probleme auftreten