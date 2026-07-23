import { useEffect, useState, useCallback } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import type {
  Incident,
  CarStanding,
  SessionInfo,
  Settings,
  View,
  FcyPhase,
  IncidentDraft,
  LicenseData,
} from "./types";
import Sidebar from "./components/Sidebar";
import FcyOverlay from "./components/FcyOverlay";
import InvestigationModal from "./components/InvestigationModal";
import HomeView from "./views/HomeView";
import LicenseGateView from "./views/LicenseGateView";
import FahrerfeldView from "./views/FahrerfeldView";
import VorfaelleView from "./views/VorfaelleView";
import ArchivView from "./views/ArchivView";
import EinstellungenView from "./views/EinstellungenView";
import { useLanguage } from "./i18n/LanguageContext";

const DEFAULT_SETTINGS: Settings = {
  discord_webhook_url: "",
  incident_types: [],
  decision_types: [],
  fcy_speed_limit_kmh: 60,
  fcy_countdown_seconds: 10,
};

export default function App() {
  const { t } = useLanguage();
  const [view, setView] = useState<View>("home");
  const [connected, setConnected] = useState(false);
  const [standings, setStandings] = useState<CarStanding[]>([]);
  const [session, setSession] = useState<SessionInfo | null>(null);
  const [sessionTimeS, setSessionTimeS] = useState(0);
  const [pendingIncidents, setPendingIncidents] = useState<Incident[]>([]);
  const [archivedIncidents, setArchivedIncidents] = useState<Incident[]>([]);
  const [settings, setSettings] = useState<Settings>(DEFAULT_SETTINGS);

  const [fcyPhase, setFcyPhase] = useState<FcyPhase>("idle");
  const [fcyRemaining, setFcyRemaining] = useState(0);

  const [preRoll, setPreRoll] = useState(20);
  const [postRoll, setPostRoll] = useState(20);

  const [modalDraft, setModalDraft] = useState<IncidentDraft | null>(null);

  // Lizenzprüfung: solange "license" null ist, wird noch geladen (kurz
  // beim Start) - erst danach wissen wir, ob die App freigeschaltet ist.
  const [license, setLicense] = useState<LicenseData | null>(null);
  const [licenseError, setLicenseError] = useState<string | null>(null);

  useEffect(() => {
    invoke<LicenseData>("get_license_status")
      .then((data) => {
        setLicense(data);
        setLicenseError(data.last_error);
      })
      .catch((err) => {
        console.error(err);
        setLicense({
          licensed: false,
          license_key: "",
          license_id: "",
          fingerprint: "",
          valid: false,
          last_validated_at: null,
          last_error: String(err),
        });
      });
  }, []);

  async function handleActivateLicense(key: string) {
    setLicenseError(null);
    try {
      const data = await invoke<LicenseData>("activate_license", { licenseKey: key });
      setLicense(data);
      if (!data.licensed) {
        setLicenseError(data.last_error);
      }
    } catch (err) {
      setLicenseError(String(err));
    }
  }

  const refreshIncidents = useCallback(() => {
    invoke<Incident[]>("list_pending_incidents").then(setPendingIncidents).catch(console.error);
    invoke<Incident[]>("list_archived_incidents").then(setArchivedIncidents).catch(console.error);
  }, []);

  useEffect(() => {
    refreshIncidents();
    invoke<Settings>("get_settings").then(setSettings).catch(console.error);

    const unlistenConn = listen<{ connected: boolean }>("connection-status", (e) => {
      setConnected(e.payload.connected);
    });
    const unlistenStandings = listen<{
      standings: CarStanding[];
      session: SessionInfo;
      session_time_s: number;
    }>("standings-update", (e) => {
      setStandings(e.payload.standings);
      setSession(e.payload.session);
      setSessionTimeS(e.payload.session_time_s);
    });
    const unlistenIncident = listen<{ incident: Incident }>("new-incident", (e) => {
      setPendingIncidents((prev) => [e.payload.incident, ...prev]);
    });
    const unlistenFcyCountdown = listen<{ remaining: number }>("fcy-countdown", (e) => {
      setFcyPhase("countdown");
      setFcyRemaining(e.payload.remaining);
    });
    const unlistenFcyPhase = listen<{ phase: FcyPhase }>("fcy-phase", (e) => {
      setFcyPhase(e.payload.phase);
    });

    return () => {
      unlistenConn.then((f) => f());
      unlistenStandings.then((f) => f());
      unlistenIncident.then((f) => f());
      unlistenFcyCountdown.then((f) => f());
      unlistenFcyPhase.then((f) => f());
    };
  }, [refreshIncidents]);

  async function handleConnect() {
    const ok = await invoke<boolean>("connect_to_server");
    setConnected(ok);
    if (ok) setView("fahrerfeld");
    else alert(t("alert_connect_failed"));
  }

  async function handleFcyClick() {
    if (fcyPhase === "idle") {
      await invoke("start_fcy");
    } else {
      await invoke("clear_fcy");
    }
  }

  function openNewIncidentModal() {
    setModalDraft({
      id: null,
      class_a: "",
      car_number_a: "",
      driver_a: "",
      class_b: "",
      car_number_b: "",
      driver_b: "",
      lap: standings[0]?.laps ?? 0,
      corner: "",
      timestamp_label: formatTimestamp(sessionTimeS),
      incident_type: "",
      decision: "",
      reasoning: "",
    });
  }

  function openInvestigateModal(incident: Incident) {
    setModalDraft({
      id: incident.id,
      class_a: incident.class_a,
      car_number_a: incident.car_number_a,
      driver_a: incident.driver_a,
      class_b: incident.class_b,
      car_number_b: incident.car_number_b,
      driver_b: incident.driver_b,
      lap: incident.lap,
      corner: incident.corner,
      timestamp_label: incident.timestamp_label,
      incident_type: incident.incident_type,
      decision: incident.decision ?? "",
      reasoning: incident.reasoning,
    });
  }

  async function submitDraft(draft: IncidentDraft) {
    await invoke<Incident>("submit_incident_decision", {
      id: draft.id,
      classA: draft.class_a,
      carNumberA: draft.car_number_a,
      driverA: draft.driver_a,
      classB: draft.class_b,
      carNumberB: draft.car_number_b,
      driverB: draft.driver_b,
      lap: draft.lap,
      corner: draft.corner,
      timestampLabel: draft.timestamp_label,
      trackName: session?.track_name ?? "",
      incidentType: draft.incident_type,
      decision: draft.decision,
      reasoning: draft.reasoning,
    });
    setModalDraft(null);
    refreshIncidents();
  }

  async function jumpToReplay(incident: Incident) {
    try {
      await invoke("jump_to_incident_replay", {
        sessionTimeS: incident.session_time_s,
        preRollSeconds: preRoll,
      });
      const targetCar = incident.car_number_a || incident.car_number_b;
      if (targetCar) {
        await new Promise(r => setTimeout(r, 300));
        await invoke("focus_driver", { carNumber: targetCar });
      }
    } catch (err) {
      alert(t("alert_replay_failed", { error: String(err) }));
    }
  }

  async function focusDriver(carNumber: string) {
    try {
      await invoke("focus_driver", { carNumber });
    } catch (err) {
      console.error("Fahrzeug-Fokus fehlgeschlagen:", err);
    }
  }

  async function selectCamera(cam: string) {
    try {
      await invoke("set_camera", { camId: cam });
    } catch (err) {
      console.error("Kamerawechsel fehlgeschlagen:", err);
      alert(t("alert_camera_unavailable"));
    }
  }

  async function saveSettings(updated: Settings) {
    await invoke("save_settings", { settings: updated });
    setSettings(updated);
  }

  return (
    <div className="app">
      <Sidebar
        view={view}
        onNavigate={setView}
        connected={connected}
        onConnect={handleConnect}
        fcyPhase={fcyPhase}
        fcyRemaining={fcyRemaining}
        onFcyClick={handleFcyClick}
        licensed={license?.licensed ?? false}
      />

      <main className={`main-content ${fcyPhase !== "idle" ? "fcy-frame" : ""}`}>
        {!license?.licensed && (
          <LicenseGateView error={licenseError} onActivate={handleActivateLicense} />
        )}

        {license?.licensed && view === "home" && (
          <HomeView connected={connected} onConnect={handleConnect} />
        )}

        {license?.licensed && view === "fahrerfeld" && (
          <FahrerfeldView
            standings={standings}
            pendingIncidents={pendingIncidents}
            onInvestigate={openInvestigateModal}
            onFocusDriver={focusDriver}
            onCamSelect={selectCamera}
            onReplay={jumpToReplay}
          />
        )}

        {license?.licensed && view === "vorfaelle" && (
          <VorfaelleView
            incidents={pendingIncidents}
            preRoll={preRoll}
            postRoll={postRoll}
            onSaveReplaySettings={(pre, post) => {
              setPreRoll(pre);
              setPostRoll(post);
            }}
            onNewIncident={openNewIncidentModal}
            onInvestigate={openInvestigateModal}
            onGoToArchiv={() => setView("archiv")}
            onFcyClick={handleFcyClick}
            onCamSelect={selectCamera}
          />
        )}

        {license?.licensed && view === "archiv" && (
          <ArchivView incidents={archivedIncidents} onReplay={jumpToReplay} onCamSelect={selectCamera} />
        )}

        {license?.licensed && view === "einstellungen" && (
          <EinstellungenView settings={settings} onSave={saveSettings} />
        )}
      </main>

      <FcyOverlay phase={fcyPhase} remaining={fcyRemaining} speedLimit={settings.fcy_speed_limit_kmh} />

      {modalDraft && (
        <InvestigationModal
          draft={modalDraft}
          standings={standings}
          settings={settings}
          onClose={() => setModalDraft(null)}
          onSubmit={submitDraft}
        />
      )}
    </div>
  );
}

function formatTimestamp(seconds: number): string {
  if (seconds <= 0) return "0:00.000";
  const m = Math.floor(seconds / 60);
  const s = (seconds % 60).toFixed(3);
  return `${m}:${s.padStart(6, "0")}`;
}