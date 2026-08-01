# LMU Race Control

Tool für die LIVE-Rennkommission in **Le Mans Ultimate**: erkennt und dokumentiert
Vorfälle (Crashs, Kontakte, Auffälligkeiten) während eines Endurance-Rennens und
erlaubt dem Kommissar einen Ein-Klick-Sprung ins In-Game-Instant-Replay.

Tech-Stack: **Tauri 2 (Rust) + React/TypeScript**. Läuft als eigenständige
Windows-Anwendung, keine zusätzliche Streaming-Software (kein OBS) nötig.

---

## ⚠️ Wichtig, bevor du loslegst: Status dieses Projekts

Dieses Repository ist ein **funktionsfähiges, aber ungetestetes Erstgerüst**.
Ich (Claude) konnte es in meiner Sandbox-Umgebung **nicht kompilieren**, da dort
kein Internetzugriff zum Laden der Cargo-/npm-Pakete besteht. Der Code ist nach
bestem Wissen syntaktisch korrekt geschrieben, aber du solltest vor dem
produktiven Renneinsatz unbedingt:

1. Einmal lokal bauen (`cargo tauri build`, siehe unten) und alle Compiler-Fehler
   beheben, die durch Versionsunterschiede in den Crates entstehen können.
2. Die **JSON-Feldnamen der LMU-REST-API gegen deine echte Spielinstanz
   verifizieren** (siehe Abschnitt "Bekannte Lücken" unten) - das ist der
   einzige Teil, den ich nicht aus offiziell dokumentierten Quellen bestätigen
   konnte.
3. Das Tool in einer Testsession (nicht live) durchspielen, bevor es bei einem
   echten Rennen zum Einsatz kommt.

---

## Architektur-Entscheidungen (und warum)

| Thema | Entscheidung | Begründung |
|---|---|---|
| Live-Daten | Offizielle LMU REST-API (`localhost:6397`) | Im Spiel fest eingebaut, kein Shared-Memory-Reverse-Engineering nötig |
| Replay-Sprung | `/rest/watch/replaytime/{sekunden}` | Offizieller REST-Endpunkt, von mehreren Community-Tools bestätigt genutzt - kein Tastatur-Simulations-Hack nötig |
| Video-Aufzeichnung | Keine eigene - nutzt das ohnehin permanent laufende LMU-Instant-Replay | Spart Komplexität, keine Abhängigkeit von OBS |
| Vorfall-Erkennung | Heuristik (Pace-/Positionsanomalien) + manueller Marker | Kein bestätigter Kontakt/Schaden-Endpunkt für gegnerische Fahrzeuge auffindbar |
| Desktop-Framework | Tauri 2 (Rust) statt Electron | Kleinerer Installer, geringerer RAM-Verbrauch auf dem Kommissars-PC |

## Sprache, Schrift, Icon, Hilfe (Version 0.3)

### Deutsch/Englisch
Die App erkennt beim ersten Start automatisch die Sprache des Betriebssystems
(Windows auf Deutsch → App startet auf Deutsch, alles andere → Englisch). Oben
in der Sidebar, neben dem Logo, kann jederzeit manuell zwischen DE/EN
umgeschaltet werden - die Wahl wird gespeichert (auch nach Neustart der App).

Alle Oberflächentexte liegen zentral in `frontend/src/i18n/translations.ts`.
Neue Texte oder Änderungen: dort als Schlüssel bei **beiden** Sprachen (`de`
und `en`) eintragen.

### Hilfe-Text ändern
Der Inhalt des Hilfe-Fensters (Klick auf "Hilfe" in der Sidebar) liegt in
**`frontend/src/content/helpContent.ts`** - dort ausführlich kommentiert, wie
man Abschnitte/Absätze hinzufügt oder ändert, ganz ohne Programmierkenntnisse.

### Schriftart (Michroma/Inter)
Wird beim Bauen automatisch von Google Fonts heruntergeladen und **fest in die
App eingebaut** (kein Internet zur Laufzeit nötig) - siehe Schritt
"Schriftarten für Offline-Nutzung herunterladen" in
`.github/workflows/build-windows.yml`. Grund für die vorherige Arial-Anzeige:
die Sicherheitsrichtlinie (CSP) der App blockierte den externen Google-Fonts-Link.

### App-Icon ändern
`src-tauri/icons/icon-source.png` durch ein eigenes quadratisches PNG mit
transparentem Hintergrund ersetzen (mind. 512×512px). Alle benötigten Formate
(.ico, .icns, verschiedene PNG-Größen) werden beim Bauen automatisch erzeugt.


---

## Neu in Version 0.2: Wo trage ich was ein?

### Vorfall-Kategorien, Entscheidungs-Optionen, Discord-Webhook

Sidebar → **Einstellungen** (unter "Software Infos"). Dort:
- **Discord-Webhook-URL**: Discord-Server → Kanal-Einstellungen → Integrationen
  → Webhooks → "Neuer Webhook" → URL kopieren, hier einfügen. Bei jeder
  Entscheidung ("Entscheidung absenden") wird automatisch eine Embed-Nachricht
  gepostet.
- **Vorfall-Kategorien** / **Entscheidungs-Optionen**: je eine pro Zeile,
  erscheinen in dieser Reihenfolge in den Dropdowns des Investigation-Fensters.
- Wird lokal als `settings.json` im App-Datenverzeichnis gespeichert
  (`%APPDATA%/com.yourteam.lmuracecontrol/settings.json` unter Windows) -
  muss auf jedem Kommissars-Rechner einmal gepflegt werden.

Dieser Einstellungen-Bereich war nicht Teil eures Figma-Mockups, ist aber
notwendig, damit ihr die Dropdown-Inhalte selbst pflegen könnt, ohne dass
jedes Mal der Code angefasst werden muss.

### Full Course Yellow (FCY)

Klick auf den gelben FCY-Button startet einen konfigurierbaren Countdown
(Default 10s, einstellbar). Bei 0 wechselt die App in den Zustand "Aktiv":
Ab da wird die Live-Geschwindigkeit (`speed_kmh`) aller Fahrzeuge gegen das
konfigurierte Limit (Default 60 km/h) geprüft. Jedes Fahrzeug, das während
der aktiven FCY-Phase schneller fährt, wird einmalig automatisch als Vorfall
markiert. Erneuter Klick auf den FCY-Button beendet die Phase wieder.

**Wichtig:** Die App kann als Zuschauer/Spectator kein echtes FCY auf dem
LMU-Server auslösen (dafür bräuchte es Server-Admin-/RCON-Zugriff). Der
Button ist bewusst als reines Kommissions-internes Werkzeug gebaut:
Countdown anzeigen, Verstöße protokollieren. Die tatsächliche FCY-Ansage an
die Fahrer muss weiterhin über euren gewohnten Kanal erfolgen.

## Bekannte Lücken (bitte vor Renneinsatz prüfen)

### 1. Exakte Feldnamen der Live-Timing-JSON

`src-tauri/src/lmu_client.rs` parst `/rest/watch/standings` tolerant über mehrere
mögliche Feldnamen (`slotId`/`slotID`/`id`, `position`/`place`/`pos`, ...). So
prüfst du die echten Feldnamen:

1. LMU starten, in eine Session gehen (Practice reicht).
2. Im Browser `http://localhost:6397/rest/watch/standings` öffnen.
3. Die JSON-Struktur mit den `field_*`-Aufrufen in `parse_standings()` abgleichen
   und bei Abweichungen die Schlüssel-Listen ergänzen/korrigieren.
4. Dasselbe für `http://localhost:6397/rest/watch/sessionInfo` - insbesondere
   das Feld für "verstrichene Session-/Replay-Zeit in Sekunden" wird für exakte
   Replay-Sprünge gebraucht. Aktuell nutzt die App eine Client-seitige
   Annäherung (verstrichene Echtzeit seit Verbindungsaufbau), was bei Pausen,
   Session-Wechseln o.ä. ungenau werden kann.

### 2. Automatische Vorfallserkennung ist eine Heuristik, kein Crash-Sensor

Es gibt (Stand meiner Recherche) keinen dokumentierten REST-Endpunkt, der
Kontakt/Schaden für gegnerische Fahrzeuge liefert (nur `/rest/garage/...`
für das eigene Auto in der Box). Die App markiert deshalb:

- ungewöhnlich langsame Runden (>25 % über dem eigenen Schnitt),
- plötzliche Positionsverluste (≥3 Plätze ohne Boxenstopp)

als **Verdachtsfälle**, die der Kommissar bestätigen oder verwerfen muss.
Der manuelle Marker-Button bleibt der zuverlässige Hauptweg.

Falls ihr Zugriff auf eine bessere Datenquelle habt (z.B. ein bestätigtes
Schadens-Feld, das ihr selbst in eurer LMU-Version gefunden habt), lässt sich
das leicht in `src-tauri/src/incidents.rs` ergänzen.

### 3. Zwei weitere Felder aus Version 0.2 zu verifizieren

- **`speed_kmh`** (Momentangeschwindigkeit) - Basis der FCY-Verstoßerkennung,
  unbedingt vor dem ersten FCY-Einsatz gegen echte Daten prüfen (siehe oben,
  Feldnamen in `lmu_client.rs`, Funktion `parse_standings`).
- **`car_model`** (Fahrzeugmodell für die "Car"-Spalte im Fahrerfeld).

---

## Voraussetzungen zum Bauen

- [Rust](https://rustup.rs/) (stable)
- [Node.js](https://nodejs.org/) 18+
- Tauri CLI: `cargo install tauri-cli --version "^2"`
- Windows: "Desktop development with C++" Workload aus dem Visual Studio
  Build Tools Installer (für den MSVC-Linker)

## Bauen & Starten (Entwicklung)

```powershell
cd frontend
npm install
cd ..
cargo tauri dev
```

Das startet die App mit Live-Reload. LMU sollte parallel laufen, damit die
REST-API erreichbar ist.

## Installer bauen (Produktion)

```powershell
cargo tauri build
```

Ergebnis liegt danach unter:
`src-tauri/target/release/bundle/nsis/LMU RACECONTROL_0.7.0_x64-setup.exe`
(und/oder `.../msi/...msi`)

Diese Installer-Datei ist es, die du an die Rechner der Rennkommissare
verteilst. Version, Icon, Publisher etc. sind in `src-tauri/tauri.conf.json`
und `src-tauri/Cargo.toml` zentral gepflegt - für neue Versionen dort die
`version`-Felder erhöhen und `CHANGELOG.md` ergänzen.

## Projektstruktur

```
lmu-race-control/
├── src-tauri/              Rust-Backend
│   ├── src/
│   │   ├── main.rs         Tauri-Setup, Commands, Polling-Loop, FCY-Countdown
│   │   ├── keyboard.rs     Tastatur-Simulation via Win32 SendInput (Scancodes)
│   │   ├── lmu_client.rs   LMU REST-API Client (Standings, Sessioninfo, Replay-Sprung)
│   │   ├── incidents.rs    Automatische Verdachtserkennung (rot/gelb/weiß) + FCY-Verstöße
│   │   ├── db.rs           SQLite-Persistenz (Vorfälle, Verursacher/Geschädigter)
│   │   ├── settings.rs     Dropdown-Listen, Discord-Webhook, FCY-Parameter (settings.json)
│   │   ├── license.rs      Keygen-Lizenzprüfung (Offline-Kulanz 14 Tage)
│   │   └── discord.rs      Discord-Webhook-Benachrichtigung bei Entscheidung
│   ├── icons/               App-Icons (aus eurem Logo generiert)
│   ├── capabilities/         Tauri-2-Berechtigungen
│   ├── tauri.conf.json      App-/Installer-Konfiguration, Version
│   └── Cargo.toml
├── frontend/                React/TypeScript-UI
│   ├── public/logo.png      Euer LMU-Racecontrol-Logo
│   └── src/
│       ├── App.tsx
│       ├── components/      Sidebar, TopToolbar, FcyOverlay, InvestigationModal
│       └── views/            Home, Fahrerfeld, Vorfälle, Archiv, Einstellungen
├── CHANGELOG.md
└── README.md
```

## Figma-Design – Umsetzungsstand

Das UI ist auf Basis eurer sechs Referenz-Screenshots umgesetzt (Sidebar,
Fahrerfeld, Vorfälle, Archiv, Investigation-Modal). Der Figma-Dev-Mode-Link
selbst war programmatisch nicht auslesbar (Bot-Schutz), daher sind Farben,
Abstände und Schriftart aus den Screenshots abgeleitet - **nicht pixelgenau
aus Figma übernommen**. Für exakte Werte:

1. Im Figma Dev Mode ein Element anklicken → rechtes Panel zeigt Hex-Farben,
   Abstände (px), Schriftgröße/-familie exakt an.
2. Die Werte in `frontend/src/styles.css` unter `:root` (Farben) bzw. direkt
   an den jeweiligen Klassen (Abstände, Schriftgrößen) übertragen.
3. Falls eine andere Schriftart als "Montserrat"/"Inter" verwendet wurde:
   Google-Fonts-Import in Zeile 1 von `styles.css` austauschen.

Icons: `src-tauri/icons/icon-source.png` wurde bereits aus eurem Logo
(`LMU_RC_Logo_-_hell.png`) generiert. Für ein Update:
`cargo tauri icon pfad/zu/neuem-logo.png` ausführen - generiert automatisch
alle benötigten Formate (.ico, .icns, PNG-Größen).

## Lizenz / Nutzung

Internes Tool für eure Rennkommission - keine Lizenzvorgabe hinterlegt, bitte
selbst ergänzen falls relevant (z.B. bei Weitergabe an andere Ligen).
