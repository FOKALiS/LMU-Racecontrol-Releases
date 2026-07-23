// Alle Oberflächentexte der App, für Deutsch und Englisch.
// Neue Texte: hier als neuen Schlüssel bei "de" UND "en" eintragen - TypeScript
// meldet einen Fehler, falls einer der beiden vergessen wird.

export type Lang = "de" | "en";

// Bewusst OHNE "as const": so hat jeder Text den Typ "string" statt eines
// exakten Literal-Typs - sonst würde TypeScript beim Bauen einen Fehler
// werfen, sobald sich ein deutscher und englischer Text unterscheiden
// (was ja der Sinn der Sache ist).
const de = {
  // Sidebar
  sidebar_functions: "Functions",
  sidebar_control: "Control",
  sidebar_race_control: "Race Control",
  sidebar_software_infos: "Software Infos",
  connect_to_server: "Connect to Server",
  server_connected: "Server Connected",
  nav_fahrerfeld: "Fahrerfeld",
  nav_vorfaelle: "Vorfälle",
  nav_archiv: "Archiv",
  nav_einstellungen: "Einstellungen",
  nav_hilfe: "Hilfe",
  fcy_active_short: "FCY AKTIV",
  footer_copyright: "Copyright © 2026",
  footer_product: "LMU RACECONTROL",
  footer_version: "Version 0.3",

  // Top-Toolbar (Image/Cam Control)
  toolbar_image_control: "Image Control",
  toolbar_cam_control: "Cam Control",
  toolbar_live: "LIVE Bild",
  toolbar_replay: "Replay",
  cam_tv: "TV",
  cam_helmet: "Helmet",
  cam_front: "Front",
  cam_rear: "Heck",
  cam_top: "Top",
  cam_behind: "Behind",

  // FCY-Overlay
  fcy_overlay_countdown: "FULL COURSE YELLOW in",
  fcy_overlay_active: "FULL COURSE YELLOW AKTIV",
  fcy_overlay_sub: "Alle Fahrzeuge: Pit-Limiter, max. {limit} km/h",

  // Investigation-Modal
  modal_title: "Investigation",
  modal_causing_driver: "Verursachender Fahrer",
  modal_affected_driver: "Geschädigter Fahrer",
  modal_select_driver: "Fahrer auswählen...",
  modal_select_driver_optional: "Fahrer auswählen... (optional)",
  modal_lap: "Runde",
  modal_corner: "Kurve",
  modal_timestamp: "Zeitstempel",
  modal_incident_type: "Vorfall auswählen",
  modal_select_incident_type: "Vorfall auswählen...",
  modal_decision: "Entscheidung der Rennkommission",
  modal_select_decision: "Entscheidung auswählen...",
  modal_reasoning: "Begründung",
  modal_reasoning_placeholder: "Begründung eingeben...",
  modal_cancel: "Abbrechen",
  modal_submit: "Entscheidung absenden",

  // Home
  home_welcome: "Willkommen bei LMU Racecontrol",
  home_instructions:
    'Um das Tool nutzen zu können, joine als Zuschauer auf dem LMU Server und klicke anschließend in der Menüleiste links auf "Connect to Server"',
  home_connected_hint: 'Verbunden – wähle links "Fahrerfeld"',

  // Fahrerfeld
  fahrerfeld_title: "Fahrerfeld",
  col_pos: "Pos",
  col_class: "Class",
  col_number: "#",
  col_driver_name: "Drivers Name",
  col_team: "Team",
  col_car: "Car",
  col_speed: "Speed",
  col_fastest_lap: "Fastest Lap",
  col_incident: "Vorfall",
  col_decision: "Entscheidung",
  col_lap: "Runde",
  col_corner: "Kurve",
  col_timestamp: "Zeitstempel",
  fahrerfeld_no_data: "Keine Live-Daten – warte auf LMU-Verbindung...",
  investigate: "Investigate",

  // Vorfälle
  vorfaelle_title: "Vorfälle",
  replay_control: "Replay Control",
  pre_roll: "Vorlaufzeit",
  post_roll: "Nachlaufzeit",
  seconds_short: "Sek.",
  new_incident: "Neuer Vorfall",
  finished_incidents: "Erledigte Vorfälle",
  full_course_yellow: "Full Course Yellow",
  vorfaelle_empty: "Keine offenen Vorfälle.",

  // Archiv
  archiv_title: "Archiv",
  archiv_empty: "Noch keine entschiedenen Vorfälle im Archiv.",
  decision_penalty: "Strafe",
  decision_nfa: "NFA",

  // Einstellungen
  settings_title: "Einstellungen",
  settings_hint:
    "Diese Werte werden lokal auf diesem Rechner gespeichert und gelten nur für diese Installation. Für alle Kommissars-PCs identisch einzurichten.",
  settings_webhook_label: "Discord-Webhook-URL",
  settings_webhook_hint:
    'Server-Einstellungen → Integrationen → Webhooks → Neuer Webhook → URL kopieren. Wird bei jeder Entscheidung ("Entscheidung absenden") automatisch benachrichtigt.',
  settings_speed_limit: "FCY-Geschwindigkeitslimit (km/h)",
  settings_countdown: "FCY-Countdown (Sekunden)",
  settings_incident_types_label: 'Vorfall-Kategorien ("Vorfall auswählen")',
  settings_incident_types_hint: "Eine Kategorie pro Zeile, in dieser Reihenfolge im Dropdown.",
  settings_decision_types_label: 'Entscheidungs-Optionen ("Entscheidung der Rennkommission")',
  settings_decision_types_hint:
    'Eine Option pro Zeile. Optionen, die das Wort "keine" enthalten, werden im Archiv und bei Discord automatisch grün/NFA markiert - alle anderen rot/Strafe.',
  settings_save: "Einstellungen speichern",
  settings_saved: "Gespeichert",

  // Hilfe-Button (Tooltip)
  help_tooltip: "Hilfe & Übersicht öffnen",

  // Fehlermeldungen
  alert_connect_failed: "Keine Verbindung zu LMU möglich. Läuft das Spiel und bist du auf dem Server?",
  alert_replay_failed: "Replay-Sprung fehlgeschlagen: {error}",
  alert_focus_unavailable: "Fahrzeug-Fokus fehlgeschlagen. Stelle sicher, dass LMU läuft und nicht minimiert ist.",
  alert_camera_unavailable: "Kamerawechsel fehlgeschlagen. Stelle sicher, dass LMU läuft und nicht minimiert ist.",

  // Lizenz
  license_title: "Lizenz aktivieren",
  license_instructions:
    "Diese Version von LMU RACECONTROL benötigt eine gültige Lizenz. Trage deinen Lizenzschlüssel ein, den du nach dem Kauf per E-Mail erhalten hast.",
  license_key_placeholder: "Lizenzschlüssel eingeben...",
  license_activate_button: "Aktivieren",
  license_activating: "Wird geprüft...",
  license_no_key_hint: "Noch keine Lizenz? Ihr findet das Angebot auf unserer Website.",
};

const en: typeof de = {
  sidebar_functions: "Functions",
  sidebar_control: "Control",
  sidebar_race_control: "Race Control",
  sidebar_software_infos: "Software Info",
  connect_to_server: "Connect to Server",
  server_connected: "Server Connected",
  nav_fahrerfeld: "Driver Field",
  nav_vorfaelle: "Incidents",
  nav_archiv: "Archive",
  nav_einstellungen: "Settings",
  nav_hilfe: "Help",
  fcy_active_short: "FCY ACTIVE",
  footer_copyright: "Copyright © 2026",
  footer_product: "LMU RACECONTROL",
  footer_version: "Version 0.3",

  toolbar_image_control: "Image Control",
  toolbar_cam_control: "Cam Control",
  toolbar_live: "Live Feed",
  toolbar_replay: "Replay",
  cam_tv: "TV",
  cam_helmet: "Helmet",
  cam_front: "Front",
  cam_rear: "Rear",
  cam_top: "Top",
  cam_behind: "Behind",

  fcy_overlay_countdown: "FULL COURSE YELLOW in",
  fcy_overlay_active: "FULL COURSE YELLOW ACTIVE",
  fcy_overlay_sub: "All cars: pit limiter, max. {limit} km/h",

  modal_title: "Investigation",
  modal_causing_driver: "Causing Driver",
  modal_affected_driver: "Affected Driver",
  modal_select_driver: "Select driver...",
  modal_select_driver_optional: "Select driver... (optional)",
  modal_lap: "Lap",
  modal_corner: "Corner",
  modal_timestamp: "Timestamp",
  modal_incident_type: "Incident Type",
  modal_select_incident_type: "Select incident type...",
  modal_decision: "Stewards' Decision",
  modal_select_decision: "Select decision...",
  modal_reasoning: "Reasoning",
  modal_reasoning_placeholder: "Enter reasoning...",
  modal_cancel: "Cancel",
  modal_submit: "Submit Decision",

  home_welcome: "Welcome to LMU Racecontrol",
  home_instructions:
    'To use this tool, join as a spectator on the LMU server and then click "Connect to Server" in the menu on the left.',
  home_connected_hint: 'Connected – select "Driver Field" on the left',

  fahrerfeld_title: "Driver Field",
  col_pos: "Pos",
  col_class: "Class",
  col_number: "#",
  col_driver_name: "Driver Name",
  col_team: "Team",
  col_car: "Car",
  col_speed: "Speed",
  col_fastest_lap: "Fastest Lap",
  col_incident: "Incident",
  col_decision: "Decision",
  col_lap: "Lap",
  col_corner: "Corner",
  col_timestamp: "Timestamp",
  fahrerfeld_no_data: "No live data – waiting for LMU connection...",
  investigate: "Investigate",

  vorfaelle_title: "Incidents",
  replay_control: "Replay Control",
  pre_roll: "Pre-roll",
  post_roll: "Post-roll",
  seconds_short: "sec.",
  new_incident: "New Incident",
  finished_incidents: "Finished Incidents",
  full_course_yellow: "Full Course Yellow",
  vorfaelle_empty: "No open incidents.",

  archiv_title: "Archive",
  archiv_empty: "No decided incidents in the archive yet.",
  decision_penalty: "Penalty",
  decision_nfa: "NFA",

  settings_title: "Settings",
  settings_hint:
    "These values are stored locally on this computer and only apply to this installation. Set them up identically on every steward's PC.",
  settings_webhook_label: "Discord Webhook URL",
  settings_webhook_hint:
    'Server settings → Integrations → Webhooks → New Webhook → copy URL. A notification is sent automatically for every decision ("Submit Decision").',
  settings_speed_limit: "FCY Speed Limit (km/h)",
  settings_countdown: "FCY Countdown (seconds)",
  settings_incident_types_label: 'Incident Categories ("Incident Type")',
  settings_incident_types_hint: "One category per line, shown in this order in the dropdown.",
  settings_decision_types_label: 'Decision Options ("Stewards\' Decision")',
  settings_decision_types_hint:
    'One option per line. Options containing the word "no" are automatically shown green/NFA in the archive and Discord - everything else red/Penalty.',
  settings_save: "Save Settings",
  settings_saved: "Saved",

  help_tooltip: "Open help & overview",

  alert_connect_failed: "Could not connect to LMU. Is the game running and are you on the server?",
  alert_replay_failed: "Replay jump failed: {error}",
  alert_focus_unavailable: "Vehicle focus failed. Make sure LMU is running and not minimized.",
  alert_camera_unavailable: "Camera switch failed. Make sure LMU is running and not minimized.",

  license_title: "Activate License",
  license_instructions:
    "This version of LMU RACECONTROL requires a valid license. Enter the license key you received by email after purchase.",
  license_key_placeholder: "Enter license key...",
  license_activate_button: "Activate",
  license_activating: "Checking...",
  license_no_key_hint: "Don't have a license yet? Find our offer on our website.",
};

export const translations: Record<Lang, typeof de> = { de, en };
export type TranslationKey = keyof typeof de;
