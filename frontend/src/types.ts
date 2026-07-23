export type FlagColor = "Red" | "Yellow" | "White" | "None";
export type FcyPhase = "idle" | "countdown" | "active";

export interface Incident {
  id: string;
  incident_number: number;
  created_at: string;
  decided_at: string | null;

  session_time_s: number;
  lap: number;
  corner: string;
  timestamp_label: string;
  track_name: string;

  class_a: string;
  car_number_a: string;
  driver_a: string;

  class_b: string;
  car_number_b: string;
  driver_b: string;

  flag_color: FlagColor;
  slot_id_a: number | null;

  incident_type: string;
  decision: string | null;
  reasoning: string;

  archived: boolean;
}

export interface CarStanding {
  slot_id: number;
  position: number;
  car_number: string;
  team: string;
  driver: string;
  class: string;
  car_model: string;
  class_position: number;
  laps: number;
  gap: string;
  last_lap_s: number;
  best_lap_s: number;
  sector1_s: number;
  sector2_s: number;
  sector3_s: number;
  top_speed_kmh: number;
  speed_kmh: number;
  in_pits: boolean;
}

export interface SessionInfo {
  session_type: string;
  track_name: string;
  time_of_day: string;
  session_time_remaining_s: number;
  num_cars: number;
}

export interface Settings {
  discord_webhook_url: string;
  incident_types: string[];
  decision_types: string[];
  fcy_speed_limit_kmh: number;
  fcy_countdown_seconds: number;
}

export type View = "home" | "fahrerfeld" | "vorfaelle" | "archiv" | "einstellungen";

export interface LicenseData {
  licensed: boolean;
  license_key: string;
  license_id: string;
  fingerprint: string;
  valid: boolean;
  last_validated_at: string | null;
  last_error: string | null;
}

/** Vorbelegung für das Investigation-Modal - kann leer sein ("Neuer Vorfall") */
export interface IncidentDraft {
  id: string | null; // null = wird beim Absenden neu angelegt
  class_a: string;
  car_number_a: string;
  driver_a: string;
  class_b: string;
  car_number_b: string;
  driver_b: string;
  lap: number;
  corner: string;
  timestamp_label: string;
  incident_type: string;
  decision: string;
  reasoning: string;
}