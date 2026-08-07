# LMU RACECONTROL – Arbeitsplan

## Aktueller Stand: v0.9.2
- ✅ FCY (Full Course Yellow) – läuft
- ✅ Session-Erkennung – gefixt
- ✅ Manufacturers – abgeschlossen
- ✅ VE (Verbale Entscheidung) – abgeschlossen
- ✅ Tabellen-Zellen-Ausrichtung – abgeschlossen
- ✅ Discord Webhook – getestet
- ✅ Shared Memory Impact-Daten – getestet
- ✅ Kamera-Steuerung, Replay, Vorfall-Erkennung
- ✅ Fahrerfeld – Runde-Spalte eingebaut
- ✅ Einstellungen – Layout verbessert (Server-Block, Scrollbar, Danger Zone)

---

## 🚀 Phase 6: Server-Architektur (Enterprise)

### Status: Client-Server-Verbindung (Prio 1)

### Architektur
```
lmu-race-control/
├── src-tauri/          ← Client (Tool)
│   ├── src/
│   │   ├── server_client.rs  ← NEU: HTTP-Client für Server-Kommunikation
│   │   └── ...
├── server/             ← Server-Komponente
│   └── src/
│       └── main.rs     ← REST-API (Axum) – läuft auf :3000
```

### Unter-Phasen:

#### ✅ 6.1: Server-Projekt anlegen
#### ✅ 6.2: REST-API Endpunkte
#### ✅ 6.3: Authentifizierung (API-Key + Mandanten)
#### 🔄 6.4: Client-Umbau – Server-Verbindung
- [ ] **6.4.1: Server-Client Rust-Modul** – HTTP-Client für die Server-API
- [ ] **6.4.2: "Connect to Server" in Sidebar** – separater Button vom LMU-Connect
- [ ] **6.4.3: Incidents syncen** – Vorfälle an Server senden/empfangen
- [ ] **6.4.4: Connection-Status anzeigen** – grün/rot in der Sidebar
- [ ] **6.4.5: Settings + Server-URL speichern** – Save-Button berücksichtigt Server-Felder
#### ✅ 6.5: Deployment

---

## Nächster Schritt
**6.4.1: Server-Client Rust-Modul** – Legen wir los!