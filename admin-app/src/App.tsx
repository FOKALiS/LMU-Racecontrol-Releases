import { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import "./styles.css";

const API = "http://localhost:3000";

interface Stats {
  status: string; version: string; uptime_seconds: number;
  total_incidents: number; total_tenants: number; total_api_keys: number;
}
interface Tenant { id: string; name: string; tier: string; max_users: number; created_at: string; }
interface ApiKey { id: string; tenant_id: string; key: string; label: string | null; created_at: string; }
interface Incident { id: string; incident_number: number; car_number_a: string; flag_color: string; incident_type: string; session_type: string; lap_number: number; timestamp: string; decision: string | null; }

function App() {
  const [stats, setStats] = useState<Stats | null>(null);
  const [tenants, setTenants] = useState<Tenant[]>([]);
  const [apiKeys, setApiKeys] = useState<ApiKey[]>([]);
  const [incidents, setIncidents] = useState<Incident[]>([]);
  const [online, setOnline] = useState(false);
  const [msg, setMsg] = useState("");
  const [loading, setLoading] = useState(false);
  const [tab, setTab] = useState("status");
  const [tenantName, setTenantName] = useState("");
  const [tenantTier, setTenantTier] = useState("enterprise_l");
  const [keyTenant, setKeyTenant] = useState("");
  const [keyLabel, setKeyLabel] = useState("");
  const [syncKey, setSyncKey] = useState("");
  const [syncTier, setSyncTier] = useState("enterprise_xl");
  const [syncName, setSyncName] = useState("");
  const [backupPath, setBackupPath] = useState("");
  const [backupMsg, setBackupMsg] = useState("");
  const [backupLoading, setBackupLoading] = useState(false);
  const [configLoading, setConfigLoading] = useState(false);

  async function api(url: string, method = "GET", body?: string): Promise<any> {
    const r: any = await invoke("api_request", { url, method, body: body || null });
    if (r && r.success && r.data) return JSON.parse(r.data);
    throw new Error(r?.error || "API-Fehler");
  }
  async function apiAuth(url: string, key: string, method = "GET", body?: string): Promise<any> {
    const r: any = await invoke("api_request_with_key", { url, method, apiKey: key, body: body || null });
    if (r && r.success && r.data) return JSON.parse(r.data);
    throw new Error(r?.error || "API-Fehler");
  }
  async function getKey(): Promise<string | null> {
    try { const d = await api(`${API}/api-key`); return d.api_key || null; } catch { return null; }
  }

  async function refresh() {
    try {
      await api(`${API}/health`);
      setOnline(true);
      const key = await getKey();
      if (!key) return;
      const [s, t, k, i] = await Promise.all([
        apiAuth(`${API}/api/stats`, key).catch(() => null),
        apiAuth(`${API}/api/tenants`, key).catch(() => []),
        apiAuth(`${API}/api/keys`, key).catch(() => []),
        apiAuth(`${API}/api/incidents`, key).catch(() => []),
      ]);
      if (s) setStats(s);
      if (Array.isArray(t)) setTenants(t);
      if (Array.isArray(k)) setApiKeys(k);
      if (Array.isArray(i)) setIncidents(i);
    } catch { setOnline(false); }
  }

  useEffect(() => { refresh(); const i = setInterval(refresh, 5000); return () => clearInterval(i); }, []);

  async function stopServer() {
    if (!confirm("Server wirklich stoppen?")) return;
    setLoading(true); setMsg("Stoppe Server...");
    try { const key = await getKey(); if (!key) { setMsg("Kein API-Key!"); setLoading(false); return; }
      await apiAuth(`${API}/api/stop`, key, "POST"); setMsg("Server gestoppt!"); setOnline(false);
    } catch { setMsg("Server gestoppt!"); setOnline(false); } setLoading(false);
  }
  async function restartServer() {
    setLoading(true); setMsg("Starte Server neu...");
    try { const key = await getKey(); if (!key) { setMsg("Kein API-Key!"); setLoading(false); return; }
      await apiAuth(`${API}/api/restart`, key, "POST"); setMsg("Neustart ausgelöst...");
      for (let i = 0; i < 15; i++) {
        await new Promise(r => setTimeout(r, 2000));
        try { await api(`${API}/health`); setMsg("Server neu gestartet!"); setOnline(true); break; } catch {}
      }
    } catch { setMsg("Neustart fehlgeschlagen!"); } setLoading(false);
  }
  async function createTenant() {
    if (!tenantName.trim()) { showMsg("Bitte Namen eingeben!"); return; }
    try { const key = await getKey(); if (!key) return;
      await apiAuth(`${API}/api/tenants`, key, "POST", JSON.stringify({ name: tenantName, tier: tenantTier }));
      showMsg("Mandant erstellt!"); setTenantName(""); refresh();
    } catch { showMsg("Fehler!"); }
  }
  async function upgradeTenant(id: string) {
    try { const key = await getKey(); if (!key) return;
      const newTier = prompt("Neues Tier (enterprise_l, enterprise_xl, enterprise_howe):", "enterprise_xl");
      if (!newTier) return;
      await apiAuth(`${API}/api/tenants/${id}`, key, "PATCH", JSON.stringify({ tier: newTier }));
      showMsg(`Mandant auf ${newTier} upgegradet!`); refresh();
    } catch { showMsg("Fehler beim Upgrade!"); }
  }
  async function deleteTenant(id: string, name: string) {
    if (!confirm(`Mandant "${name}" wirklich löschen?`)) return;
    try { const key = await getKey(); if (!key) return;
      await apiAuth(`${API}/api/tenants/${id}`, key, "DELETE");
      showMsg(`Mandant "${name}" gelöscht!`); refresh();
    } catch { showMsg("Fehler beim Löschen!"); }
  }
  async function createApiKey() {
    if (!keyTenant) { showMsg("Bitte Mandant wählen!"); return; }
    try { const key = await getKey(); if (!key) return;
      const d = await apiAuth(`${API}/api/keys`, key, "POST", JSON.stringify({ tenant_id: keyTenant, label: keyLabel || null }));
      showMsg(`API-Key: ${d.key}`); setKeyLabel(""); refresh();
    } catch { showMsg("Fehler!"); }
  }
  async function syncLicense() {
    if (!syncKey.trim()) { showMsg("Bitte Lizenz-Key eingeben!"); return; }
    try { const key = await getKey(); if (!key) return;
      const d = await apiAuth(`${API}/api/sync-license`, key, "POST", JSON.stringify({
        license_key: syncKey, tier: syncTier, name: syncName || "Importierter Kunde"
      }));
      showMsg(`Sync OK: ${d.status} – ${d.tenant_name} (${d.tier})`); refresh();
    } catch (e: any) { showMsg(`Sync-Fehler: ${e.message}`); }
  }
  async function loadConfig() {
    try { const key = await getKey(); if (!key) return;
      const cfgArr = await apiAuth(`${API}/api/config`, key);
      if (Array.isArray(cfgArr)) {
        const cfgMap: Record<string, string> = {};
        cfgArr.forEach((c: any) => { cfgMap[c.key] = c.value ?? ""; });
        setBackupPath(cfgMap["backup_path"] ?? "");
      }
    } catch { /* leer */ }
  }
  async function saveConfig() {
    setConfigLoading(true);
    try { const key = await getKey(); if (!key) { setConfigLoading(false); return; }
      await apiAuth(`${API}/api/config`, key, "POST", JSON.stringify({ key: "backup_path", value: backupPath }));
      showMsg("Konfiguration gespeichert!"); setConfigLoading(false);
    } catch (e: any) { showMsg(`Fehler: ${e.message}`); setConfigLoading(false); }
  }
  async function runBackup() {
    if (!confirm("Jetzt ein Backup erstellen?")) return;
    setBackupLoading(true); setBackupMsg("");
    try { const key = await getKey(); if (!key) { setBackupLoading(false); setBackupMsg("Kein API-Key!"); return; }
      const r = await apiAuth(`${API}/api/backup`, key, "POST");
      setBackupMsg(r?.message || "Backup erfolgreich!"); setBackupLoading(false);
    } catch (e: any) { setBackupMsg(`Backup-Fehler: ${e.message}`); setBackupLoading(false); }
    setTimeout(() => setBackupMsg(""), 5000);
  }
  function showMsg(t: string) { setMsg(t); setTimeout(() => setMsg(""), 4000); }
  const fmt = (s: number) => `${Math.floor(s/3600)}h ${Math.floor((s%3600)/60)}m ${s%60}s`;

  return (
    <div className="app">
      <aside className="sidebar">
        <div className="logo"><h1>🏁 LMU RC</h1><span>Admin Tool</span></div>
        <nav>
          <button className={tab==="status"?"active":""} onClick={()=>setTab("status")}>📊 Status</button>
          <button className={tab==="kunden"?"active":""} onClick={()=>setTab("kunden")}>🏢 Kunden</button>
          <button className={tab==="keys"?"active":""} onClick={()=>setTab("keys")}>🔑 API-Keys</button>
          <button className={tab==="incidents"?"active":""} onClick={()=>setTab("incidents")}>🚨 Vorfälle</button>
          <button className={tab==="sync"?"active":""} onClick={()=>setTab("sync")}>🔄 Keygen Sync</button>
          <button className={tab==="backup"?"active":""} onClick={()=>{setTab("backup"); loadConfig();}}>💾 Backup</button>
        </nav>
        <div className={`server-status ${online?"online":"offline"}`}>
          {online ? "🟢 Server online" : "🔴 Server offline"}
        </div>
      </aside>
      <main className="main">
        {msg && <div className="message">{msg}</div>}
        {tab === "status" && (
          <>
            <h2>Server-Status</h2>
            <div className="stats-grid">
              <div className="card"><div className="label">Status</div><div className="value">{online ? "🟢 Online" : "🔴 Offline"}</div></div>
              <div className="card"><div className="label">Version</div><div className="value">{stats?.version ?? "–"}</div></div>
              <div className="card"><div className="label">Uptime</div><div className="value">{stats ? fmt(stats.uptime_seconds) : "–"}</div></div>
              <div className="card"><div className="label">Vorfälle</div><div className="value">{stats?.total_incidents ?? "–"}</div></div>
              <div className="card"><div className="label">Kunden</div><div className="value">{stats?.total_tenants ?? "–"}</div></div>
              <div className="card"><div className="label">API-Keys</div><div className="value">{stats?.total_api_keys ?? "–"}</div></div>
            </div>
            <div className="actions">
              <button className="btn btn-restart" onClick={restartServer} disabled={loading || !online}>🔄 Server neustarten</button>
              <button className="btn btn-stop" onClick={stopServer} disabled={loading || !online}>⏹️ Server stoppen</button>
              <button className="btn btn-refresh" onClick={refresh} disabled={loading}>🔄 Aktualisieren</button>
            </div>
          </>
        )}
        {tab === "kunden" && (
          <>
            <h2>Kunden</h2>
            <div className="panel">
              <h3>Neuer Kunde</h3>
              <div className="form-row">
                <input type="text" placeholder="Name" value={tenantName} onChange={e => setTenantName(e.target.value)} />
                <select value={tenantTier} onChange={e => setTenantTier(e.target.value)}>
                  <option value="enterprise_l">Enterprise L (3 User)</option>
                  <option value="enterprise_xl">Enterprise XL (10 User)</option>
                  <option value="enterprise_howe">Enterprise HOWE (25 User)</option>
                </select>
                <button className="btn btn-primary" onClick={createTenant}>🏢 Anlegen</button>
              </div>
            </div>
            <div className="list">
              {tenants.map(t => (
                <div className="list-item" key={t.id}>
                  <div><strong>{t.name}</strong><div className="muted">{t.tier} · {t.max_users} User · ID: {t.id}</div></div>
                  <div style={{display:"flex",gap:"6px",alignItems:"center"}}>
                    <span className="badge">{t.tier}</span>
                    <button className="btn btn-small" onClick={() => upgradeTenant(t.id)}>⬆ Upgrade</button>
                    <button className="btn btn-small btn-danger" onClick={() => deleteTenant(t.id, t.name)}>🗑 Löschen</button>
                  </div>
                </div>
              ))}
            </div>
          </>
        )}
        {tab === "keys" && (
          <>
            <h2>API-Keys</h2>
            <div className="panel">
              <h3>Neuer API-Key</h3>
              <div className="form-row">
                <select value={keyTenant} onChange={e => setKeyTenant(e.target.value)}>
                  <option value="">Mandant wählen...</option>
                  {tenants.map(t => <option key={t.id} value={t.id}>{t.name}</option>)}
                </select>
                <input type="text" placeholder="Bezeichnung" value={keyLabel} onChange={e => setKeyLabel(e.target.value)} />
                <button className="btn btn-primary" onClick={createApiKey}>🔑 Erstellen</button>
              </div>
            </div>
            <div className="list">
              {apiKeys.map(k => (
                <div className="list-item" key={k.id}>
                  <div><code>{k.key}</code><div className="muted">{k.label ?? "–"} · {k.tenant_id}</div></div>
                </div>
              ))}
            </div>
          </>
        )}
        {tab === "incidents" && (
          <>
            <h2>Vorfälle</h2>
            <button className="btn btn-refresh" onClick={refresh}>🔄 Aktualisieren</button>
            <div className="table-wrap">
              <table>
                <thead><tr><th>#</th><th>Flagge</th><th>Auto</th><th>Typ</th><th>Sitzung</th><th>Runde</th><th>Entscheidung</th></tr></thead>
                <tbody>
                  {incidents.map(inc => (
                    <tr key={inc.id}>
                      <td>{inc.incident_number}</td>
                      <td><span className={`flag flag-${inc.flag_color.toLowerCase()}`} />{inc.flag_color}</td>
                      <td>{inc.car_number_a}</td><td>{inc.incident_type}</td><td>{inc.session_type}</td>
                      <td>{inc.lap_number}</td><td>{inc.decision ?? "–"}</td>
                    </tr>
                  ))}
                </tbody>
              </table>
            </div>
          </>
        )}
        {tab === "backup" && (
          <>
            <h2>💾 Backup & Konfiguration</h2>
            <div className="panel">
              <h3>Server-Konfiguration</h3>
              <div className="form-row" style={{flexDirection:"column",gap:"8px"}}>
                <div style={{display:"flex",gap:"8px",alignItems:"center"}}>
                  <label style={{minWidth:"120px",color:"#9aa0b8"}}>Backup-Pfad:</label>
                  <input type="text" placeholder="z.B. C:\lmu-race-control-server\backups" value={backupPath} onChange={e => setBackupPath(e.target.value)} style={{flex:1}} />
                </div>
                <button className="btn btn-primary" onClick={saveConfig} disabled={configLoading} style={{alignSelf:"flex-start",marginTop:"4px"}}>
                  {configLoading ? "⏳ Speichern..." : "💾 Konfiguration speichern"}
                </button>
              </div>
            </div>
            <div className="panel" style={{marginTop:"12px"}}>
              <h3>Manuelles Backup</h3>
              <p style={{color:"#9aa0b8",fontSize:"13px",marginBottom:"12px"}}>
                Ein Backup erstellt eine Kopie der Datenbank und der Konfiguration am eingestellten Backup-Pfad.
              </p>
              <button className="btn btn-primary" onClick={runBackup} disabled={backupLoading || !online} style={{fontSize:"15px",padding:"12px 24px"}}>
                {backupLoading ? "⏳ Backup läuft..." : "💾 Jetzt Backup erstellen"}
              </button>
              {backupMsg && <div style={{marginTop:"8px",color:backupMsg.includes("Fehler")?"#e57373":"#81c784",fontSize:"13px"}}>{backupMsg}</div>}
            </div>
          </>
        )}
        {tab === "sync" && (
          <>
            <h2>Keygen Lizenz-Sync</h2>
            <div className="panel">
              <h3>Bestehende Lizenz importieren</h3>
              <p style={{color:"#9aa0b8",fontSize:"13px",marginBottom:"12px"}}>
                Gib Deinen Keygen-Lizenz-Key ein, um einen Mandanten mit API-Key anzulegen.
              </p>
              <div className="form-row">
                <input type="text" placeholder="Lizenz-Key" value={syncKey} onChange={e => setSyncKey(e.target.value)} style={{minWidth:"200px"}} />
                <input type="text" placeholder="Kundenname" value={syncName} onChange={e => setSyncName(e.target.value)} style={{minWidth:"150px"}} />
                <select value={syncTier} onChange={e => setSyncTier(e.target.value)}>
                  <option value="enterprise_l">Enterprise L (3 User)</option>
                  <option value="enterprise_xl">Enterprise XL (10 User)</option>
                  <option value="enterprise_howe">Enterprise HOWE (25 User)</option>
                </select>
                <button className="btn btn-primary" onClick={syncLicense}>🔄 Sync</button>
              </div>
            </div>
            <div className="panel" style={{marginTop:"12px"}}>
              <h3>Info</h3>
              <p style={{color:"#9aa0b8",fontSize:"13px"}}>
                Der Keygen-Webhook unter <code>/api/webhook/keygen</code> erstellt automatisch 
                einen Mandanten + API-Key, wenn eine neue Lizenz erstellt wird. 
                Das Tier wird aus den Metadaten der Lizenz ausgelesen.
              </p>
            </div>
          </>
        )}
      </main>
    </div>
  );
}
export default App;