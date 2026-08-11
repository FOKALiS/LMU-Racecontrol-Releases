# Changelog

All notable changes to LMU Race Control are documented here.
Format based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
versioning follows [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.9.5] - 2026-08-10 (Release Repo, Update Fix, Discord Notification, Server Backup, License Deactivation)
### Added
- **Release repo created:** `FOKALiS/LMU-Racecontrol-Releases` – release artifacts only (no source code)
- **Discord release notification:** Workflow `.github/workflows/discord-notify.yml` – automatic on release with formatted changelog
- **Server backup script:** `scripts/backup-server.ps1` – automatic (Task Scheduler) + manual
- **License deactivation:** User can deactivate license (machine change) – in Settings
- **Help system:** New detailed help pages (License, API Key, Server, Discord)

### Changed
- **Update URL migrated:** From old main repo to `FOKALiS/LMU-Racecontrol-Releases`
- **Tauri signing key renewed:** New key stored as secret in release repo
- **Keygen API token renewed:** For license system

### Fixed
- **Update fix:** Old v0.9.4 installations need a one-time manual update (changed signing key + URL)

## [0.9.4] - 2026-08-07 (API Key Lookup, Server Cleanup)
### Added
- **Server endpoint `POST /api/lookup-api-key`:** Validates license key live at Keygen, auto-creates tenant + API key with correct tier (Enterprise L/XL/HOWE)
- **Button "Request API Key":** In Settings → Server Connection – requests API key for license key (licensed Enterprise only)
- **Version in license heading:** "License Information – Version: Enterprise XL (Server)"

### Changed
- **Server URL field removed:** Should not be user-editable
- **Server `DELETE /api/incidents`:** Only deletes incidents of own tenant (team-safe)
- **Server `purge_old_incidents`:** Auto-cleanup of old incidents (>26h) every 30 minutes
- **`clear_all_incidents`:** Now also deletes server incidents of own tenant (no other team's data)

## [0.9.1] - 2026-08-06 (White Flag Fix)
### Fixed
- **White flag fix:** White flag (slow vehicle) now also checks `> MIN_STOPPED_SPEED_KMH (0.5 km/h)`, so vehicles at exactly 0.0 km/h (Pipo Derani) are no longer falsely reported as "slow"

## [0.9.0] - 2026-08-06 (Pipo Derani Fix, Discord Formatting, UI Improvements)
### Fixed
- **Pipo Derani fix:** Introduced `MIN_STOPPED_SPEED_KMH = 0.5` – vehicles at 0.0 km/h are no longer detected as stopped

### Changed
- **Discord webhook:** Formatting overhauled (driver, warning/penalty points, "N.A." for empty corner)
- **Settings:** Heading left-aligned (text-align: center → left)
- **Folder icon (📂)** for LMU path selection restored
- **Archive:** Decision badge now opens InvestigationModal
- **Window title:** "LMU RACECONTROL – The tool for race stewards"

## [0.8.8] - 2026-08-05 (Incident Detection Overhaul, Keyboard Config Fix)
### Changed
- **Incident detection overhauled:**
  - RED: Impact >3.0g, lap time >30%, stopped <10 km/h
  - YELLOW: Lap time >15%, position loss ≥3, FCY violation
  - WHITE (NEW): >30s under 50 km/h (timer-based)
- **Session buttons:** No longer clickable, active state from LMU (`session?.session_type`)
- **Keyboard config fixed:**
  - Complete scancode table (missing keys added)
  - "Onboard Cameras" supported
  - OnceLock → RwLock (reload now works multiple times)
- **File browser (📂)** for LMU installation path in Settings
- `@tauri-apps/plugin-dialog` installed

## [0.8.7] - 2026-08-05 (Timer, Splashscreen, Connect Lock)
### Changed
- Timer: replay_pause() via F11 after pre-roll + post-roll
- LIVE button: onSwitchToLive?.() in all 3 views
- Splashscreen: Copyright "FOKALiS - Film & Medienagentur"
- Connect button disabled when not licensed
- Session buttons/Player bar/Filter tabs as separate CSS classes

## [0.8.3] - 2026-08-01 (Figma MCP Integration + Player Bar Icons)
### Added
- **Figma MCP Server** (`figma-developer-mcp` v0.13.2) installed and configured
- **Figma Design "LMU Racecontrol"** imported: 6 screens (Home, Driver Field, Incidents, Archive, Investigation Overlay), 14 components, design tokens
- **Design data** saved in `figma-screens/design-data-complete.json`
- **Logos** from Figma export in `figma-screens/` and `frontend/public/logo.png`

### Changed
- **Player bar icons:** Emoji placeholders (⏮⏪▶⏩⏭) replaced with real PNG icons (Slow Rewind, Rewind, Play, Forward, Slow Forward) in all three views (Driver Field, Incidents, Archive)
- **Icons** from `C:\Users\Administrator\Documents\AI\Software Entwicklung\LMU Racecontrol\Icons` integrated into `frontend/public/icons/`
- **CSS variables** extended with Figma design tokens (`--text-secondary`, `--text-dim-soft`, `--purple`)
- **CSS for player icons** added (20x20 size, hover effects, disabled state)
- **Sidebar duplicate** in Sidebar.tsx fixed
- **Logo height** set to `auto` for correct proportions

### Technical
- **Version:** 0.8.2 → 0.8.3

## [0.7.0] - 2026-07-27 (Keyboard Simulation instead of Camera Helper + Turbo Zoom)
### Changed
- **Camera Helper removed:** The separate `camera-helper` process has been completely removed. Camera control now runs directly via `SendInput` with scancodes from the user's key bindings – no external process needed.
- **Camera button "Onboard" instead of "Helmet":** Matches LMU default key binding (Insert = Onboard Camera). "Helmet" is still recognized as an alias.
- **Zoom function:** New zoom buttons (+ / -) next to camera controls. Hold down = continuous zoom via background thread in Rust (no setInterval). Works on all pages (Driver Field, Incidents, Archive).
- **Zoom speed:** ~500 key presses per second (1ms KeyDown, 1ms Pause).

### Fixed
- **Zoom didn't work on Incidents/Archive pages:** `onZoomStart`/`onZoomEnd` props were not passed to `TopToolbar`. Now passed to `VorfaelleView` and `ArchivView`.

### Technical
- `keyboard.rs`: Completely rewritten – `SendInput` with scancodes, no more `enigo`. `zoom_start`/`zoom_stop` with background thread and AtomicBool flag.
- `src-tauri/camera-helper/` removed (saved over 100MB build artifacts).
- `tauri.conf.json`: `resources` changed from `camera-helper.exe` to empty.

## [0.6.12] - 2026-07-26 (Cam Control + Replay Control Fix)
### Fixed
- **jumpToReplay now also sets camera:** After replay time jump, TV camera is automatically set (via REST API). Previously the replay jumped to the right time but the user only saw the previous camera angle.
- **Replay mode activated before time jump:** LMU requires replay mode for camera commands to work. `switch_to_replay` is now called before the time jump.
- **Longer pauses between commands:** 200ms after mode switch, 500ms after time jump – giving LMU enough time to process commands.
- **focus_driver now also sets TV camera:** After driver focus, TV camera is activated so the user immediately sees the vehicle.
- **Improved debug logging:** All steps are now logged with emoji and timestamp for easier troubleshooting.

### Source
- Analysis of BCUK (Broadcast Control UK) on desktop confirmed: BCUK uses exactly the same REST API approach (`/rest/watch/focus/{name}`) – the difference is the **order**: first replay mode + time jump, then camera.
- BCUK plugin JS (Stream Deck) shows: Camera control runs via `POST /api/control` with actions like `setTv`, `setNose`, `setCockpit`, `setOnboard`, `setOnCockpit`, `setOnDash`, `setOnRear` – which are mapped to `/rest/watch/focus/{name}` internally by LMU.
- Version: 0.6.11 → 0.6.12

## [0.6.11] - 2026-07-25 (REST API PUT Body Fix + Camera Key Fix)
### Fixed
- **All PUT requests failed (HTTP 400):** The LMU REST API requires an empty JSON body `{}` with `Content-Type: application/json` for PUT requests. Without a body, HTTP 400 was returned – affecting: focus, camera, replaytime, switch_to_live/replay
- **Camera control now works reliably via REST API:** `/rest/watch/focus/TV`, `/rest/watch/focus/Onboard`, `/rest/watch/focus/Heli` and others confirmed via curl ✅ The endpoint `/rest/watch/focus/{name}/{group}/{advance}` is NOT supported by LMU (HTTP 400) and has been removed from the code.

### Source
- curl tests against running LMU instance (localhost:6397) during active session
- JSON structure from `/rest/watch/standings` confirmed (all field names correct)
- Version: 0.6.10 → 0.6.11

## [0.6.7] - 2026-07-24 (Shared Memory, Connect/Disconnect, FCY +3 km/h Tolerance)
### Added
- **Shared Memory (rFactor 2/LMU):** Direct access to LMU Shared Memory (`Local\rFactor2SharedMemory`). Camera switching and vehicle focus now work **without window focus, keyboard simulation, or terminal flash** – just like Broadcast Control UK, SimHub and other professional tools
- **Connect/Disconnect hover:** When connected, hovering the mouse shows "Disconnect from Server" (red) – click disconnects

### Fixed
- **FCY monitoring activated:** When exceeding limit + 3 km/h tolerance (e.g. 60+3=63 km/h), an FCY violation incident is automatically created
- **Debug logging:** The API response from `/rest/watch/standings` is now printed to identify the real field names for `speed_kmh`

### Changed
- `shared_memory.rs`: New module – writes camera values directly to LMU Shared Memory (group + camera ID)
- `keyboard.rs`: Only used as fallback when Shared Memory is unavailable
- `lmu_client.rs`: `groundSpeed` added as additional field name for `speed_kmh`, debug logging
- `main.rs`: `set_camera` tries Shared Memory first, then keyboard fallback
- `App.tsx` + `Sidebar.tsx`: Connect/Disconnect with hover effect
- `translations.ts`: New texts `server_disconnect` / `server_disconnected`
- **Version:** 0.6.6 → 0.6.7

## [0.6.6] - 2026-07-24 (Keyboard Control: Scancodes, AttachThreadInput for LMU Focus on Another Monitor)
### Fixed
- **Key presses now reliably reach LMU (not the Tauri app):** Scancodes via `KEYEVENTF_SCANCODE` instead of virtual key codes – games use scancodes for their key bindings
- **LMU focus across multiple monitors:** `AttachThreadInput` bypasses Windows UIPI, allowing focus to be reliably set on LMU
- **Additional focus safety:** `BringWindowToTop` + `SetFocus` after bringing to foreground

### Changed
- `keyboard.rs`: Scancodes (F1=0x3B, F2=0x3C, ...) instead of virtual key codes (VK_F1 etc.)
- `force_foreground()`: `AttachThreadInput` + `BringWindowToTop` + `SetFocus`
- **Version:** 0.6.5 → 0.6.6

## [0.6.5] - 2026-07-24 (Keyboard Control New: Win32 SendInput, No PowerShell Flash, Driver Field Sorting, Camera Selection Active)
### Fixed
- **Terminal window flash when focusing LMU eliminated:** PowerShell `AppActivate` replaced with native Win32 `FindWindowW`/`SetForegroundWindow` – no more flashing terminal
- **Keyboard commands more reliable:** `enigo` crate removed, replaced with direct Win32 `SendInput` API with background thread architecture
- **Async blocking fixed:** `std::thread::sleep` replaced with `tokio::time::sleep` in `focus_driver`

### Changed
- **Driver field now sorted by position** (1st, 2nd, 3rd, ...) via `useMemo`
- **Camera selection shown as active:** Central `selectedCam` state in `App.tsx`, passed across all views
- **`enigo` dependency removed:** `Cargo.toml` cleaned up
- **`keyboard.rs` completely rewritten:** Win32 API, background thread, input buffer flush
- **Version:** 0.6.4 → 0.6.5

## [0.6.4] - 2026-07-24 (Icon Size: Logo Enlarged on Desktop/Taskbar)
### Fixed
- **Icon on desktop and taskbar was too small:** The source image (`icon-source.png`) had too much transparent padding. When scaled to small icon sizes (32x32), the logo appeared tiny. The transparent padding was removed and the logo now fills almost the entire icon area.

### Changed
- **Version:** 0.6.3 → 0.6.4

## [0.6.3] - 2026-07-23 (Icon Fix: Correct Windows Icon Generation)
### Fixed
- **Windows icon (.ico) was not displayed correctly:** The `icon.ico` was faulty and too small for proper display on desktop and taskbar. Regenerated from `icon-source.png` using the Tauri Icon Generator (`npx @tauri-apps/cli icon`) (34,690 bytes).
- **All icons** (32x32.png, 128x128.png, 128x128@2x.png, icon.icns, icon.ico) were regenerated from source.

### Changed
- **Version:** 0.6.2 → 0.6.3

## [0.6.1] - 2026-07-22 (Installer Fix, Local Fonts, Sidebar Control)
### Fixed
- **ERR_CONNECTION_REFUSED on startup:** The main window was missing `"url": "index.html"`, causing the installed app to try loading from the dev server (localhost:1420). The app now starts immediately, without internet connection.
- **Fonts (Michroma/Inter) now embedded locally:** Font files are stored as `.woff2` in `frontend/public/fonts/` and are built into the app – no more Google Fonts network access. The app now works completely offline.

### Changed
- **Sidebar control** adapted to three states:
  - Without license: only "Software Info" (language, help, website, footer)
  - Licensed, not connected: "Connect to Server" + "Software Info"
  - Licensed + connected: all buttons (Driver Field, Incidents, Archive, FCY)
- **`beforeBuildCommand`** changed to `npm --prefix frontend run build` (solves special character issues with umlauts in path)
- **Version:** 0.6.0 → 0.6.1

## [0.6.0] - 2026-07-22 (Camera Control, Vehicle Focus, Splashscreen Design)
### Added
- **Camera control via keyboard simulation:** Camera buttons (TV, Helmet, Front, Rear, Top, Behind) now simulate F1-F6 key presses directly in LMU/rFactor2 – works because the LMU REST API doesn't provide a camera endpoint
- **Vehicle focus via keyboard simulation:** Click on an incident or double-click on a driver jumps to the correct replay position, switches to TV camera and focuses the vehicle via Ctrl+F + vehicle number + Enter
- **Automatic replay jump:** Replay jump to incident position now works reliably via the LMU REST API

### Changed
- **Splashscreen design overhauled:** Logo enlarged (300px → 380px), version number placed below logo, overall layout visually enhanced
- **Backend:** Keyboard simulation switched from `windows` crate to `enigo` crate (solves version conflicts with Tauri 2)
- **Version:** 0.5.4 → 0.6.0 (all version numbers updated)

### Technical
- `keyboard.rs`: New module for keyboard simulation with `enigo` crate
- `main.rs`: `set_camera` and `focus_driver` now use keyboard simulation instead of failing REST API calls
- `Cargo.toml`: `enigo = "0.2"` replaces `windows = "0.58"`

## [0.5.4] - Unreleased (Cam Control Right-Aligned)
### Changed
- Image Control + Cam Control now aligned as one unit to the right (same width as the Race Control row below) – Image Control keeps its size and only moves as a whole to the right, Cam Control now ends exactly at the true right edge instead of having empty space after it

## [0.5.3] - Unreleased (Race Control Buttons Full Width)
### Changed
- "New Incident"/"Resolved Incidents"/"Full Course Yellow" on Incidents page now span the full width of Image Control + Cam Control combined (row above), buttons evenly stretched instead of just naturally right-aligned

## [0.5.2] - Unreleased (Fix: Activation with "Require Fingerprint Scope")
### Fixed
- Your Keygen policy has "Require Fingerprint Scope" enabled (every validation MUST include a device fingerprint). The first activation call previously ran deliberately WITHOUT this fingerprint, which would have immediately failed with this policy setting. From now on, the device fingerprint is always sent from the start.

## [0.5.1] - Unreleased (Keygen Account ID Entered)
### Changed
- `KEYGEN_ACCOUNT` in `src-tauri/src/license.rs` changed from placeholder to the real Keygen account ID – license validation is now actually functional (assuming the Keygen policy is configured as discussed: Node-locked, authentication strategy License/Mixed)

## [0.5.0] - Unreleased (License System)
### Added
- License requirement: without a valid license, only the splash screen (with license key input), "Help" and the website link are usable – all other functions (Driver Field, Incidents, Archive, Settings, Connect to Server, FCY) are locked
- Integration with Keygen License API (https://keygen.sh): per-device activation, regular online re-validation, 14-day offline grace period (so a race weekend with poor internet doesn't lock anyone out)
- Recommended sales channel: existing Wix shop remains the point of sale, Wix Automation automatically creates a license key via the Keygen API on order receipt (setup follows as a separate step once the Keygen account is created)

### Important – before the next build
- Replace the placeholder `KEYGEN_ACCOUNT` in `src-tauri/src/license.rs` with the real Keygen account slug – without this step, every license validation will fail

## [0.4.2] - Unreleased (FCY Highlight, Button Alignment)
### Changed
- When FCY is active/triggered: yellow border around the main area, "FULL COURSE YELLOW ACTIVE" banner now correctly centered over the main area (instead of over the entire window including sidebar)
- Buttons "New Incident"/"Resolved Incidents"/"Full Course Yellow" on Incidents page are now left-aligned with the "Cam Control" row above
- Update strategy: repository remains private for now, auto-update detection will work automatically once the project is set to "Public" – until then, updates continue to be downloaded manually from the Releases page

## [0.4.1] - Unreleased (Visual Polish)
### Changed
- Language switcher (DE/EN) moved from the logo area to the "Software Info" section (was too cramped, got cut off)
- Website line (www.lmu-racecontrol.com) in sidebar AND splashscreen is now clickable and opens the page in the default browser
- Second input fields for "Pre-roll"/"Post-roll" widened so the adjustment arrows no longer overlap the number
- Table headers AND data rows now consistently have rounded outer corners (previously invisible due to a CSS property that blocked rounding on table cells)
- Text on the home page ("Welcome to...") significantly reduced in size
- Splashscreen display duration reduced from 10 to 5 seconds

## [0.4.0] - Unreleased (Splashscreen, Auto-Update, Dynamic Version Display)
### Added
- Splashscreen window on program start (5 seconds, logo, version, website line), then automatic switch to maximized main window
- Built-in auto-updater: Splashscreen checks for new versions in the background; if available, clicking the green bar downloads the update, installs it and restarts the app
- Build workflow now uses the official `tauri-apps/tauri-action`: cryptographically signs updates, automatically creates a versioned GitHub release (e.g. "v0.4.0") including the `latest.json` required by the updater – replaces the previous provisional "latest" release
- New one-time workflow `generate-update-key.yml` for generating the signing key pair (see GUIDE-GETTING-INSTALLER.md, step 0)
- Version display in sidebar is now dynamic (reads the actual app version instead of being hardcoded)

### Important
- Before the next build, the signing key pair must be generated and set up once (guide, step 0) – otherwise the build will fail
- Future updates: increment version number in three files (`tauri.conf.json`, `Cargo.toml`, `package.json`), then upload as usual

## [0.3.0] - Unreleased (Polish: Name, Icon, Font, Multilingual, Help)
### Changed
- App name unified to "LMU RACECONTROL" everywhere (Start menu, window title, uninstall entry)
- Fonts Michroma/Inter are now automatically downloaded during build and EMBEDDED into the app (previously: Google Fonts link was blocked by the app's security policy, so Arial appeared instead of Michroma)
- App icon is now automatically generated from `src-tauri/icons/icon-source.png` during build (all sizes/formats) – just replace this one image to change the icon

### Added
- German/English switchable: automatic detection based on system language on first start, persistent toggle (DE/EN) at the top of the sidebar
- New help window (click "Help" in sidebar) with quick overview of usage – text freely editable in `frontend/src/content/helpContent.ts`
- Automatic, permanent installer download link (GitHub Release) in addition to the previous 90-day artifact

## [0.2.0] - Unreleased (Figma Design Implementation)
### Added
- Complete UI implemented in Figma design: sidebar with logo/navigation, Driver Field, Incidents, Archive, Investigation Modal
- New data model: Causing/Affected driver, lap, corner, timestamp, incident type, decision, reasoning
- Explicit "Connect to Server" instead of auto-connection on startup
- Full Course Yellow workflow: countdown overlay, then automatic speed monitoring with automatic incident marking for violations of the configured speed limit
- Red/Yellow/White status dots in Driver Field (crash suspicion / unusual pace anomaly / slow vehicle) – see "Known Gaps"
- Discord webhook notification for every commission decision
- New "Settings" area (not included in mockup, but necessary): incident categories, decision options, Discord webhook, FCY parameters – stored locally per installation
- App icons generated from the real LMU Racecontrol logo

### Known missing / to verify
- Field for live speed (`speed_kmh`, for FCY monitoring) and vehicle model (`car_model`) not yet verified against a real LMU instance
- Yellow/White markers are heuristics (pace anomaly / slow vehicle in field comparison), no confirmed flag field found in the REST API
- "Timestamp"/lap reference is based on elapsed real time since "Connect to Server", not on a confirmed session time field

## [0.1.0] - Unreleased (Initial Skeleton)
### Added
- Basic skeleton as Tauri 2 app (Rust backend + React/TypeScript frontend)
- Client for the official LMU REST API (`localhost:6397`): Live Timing (`/rest/watch/standings`), Session Info, Replay Time Jump (`/rest/watch/replaytime/{s}`), Camera Focus (`/rest/watch/focus/{slot}`)
- Heuristic automatic incident suspicion detection (lap time and position anomalies)
- Manual incident marker placement by the steward
- SQLite persistence of all incidents including status workflow (Suspicion → Under Review → No Action / Penalty Imposed)
- One-click jump to LMU Instant Replay with configurable pre-/post-roll
- Windows installer (NSIS/MSI) via `cargo tauri build`

### Known missing / to verify
- Exact JSON field for "elapsed time since session/replay start" in `/rest/watch/sessionInfo` not yet verified against a real running LMU instance (see README, section "Known Gaps")
- No confirmed REST endpoint for damage/contact per opposing vehicle found – automatic detection therefore works with pace/position anomalies instead of direct collision detection