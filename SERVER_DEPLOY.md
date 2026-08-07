# Server Deployment – Schritt-für-Schritt

## 1. Build
Die fertige Server-Exe liegt hier:
```
server\target\release\lmu-race-control-server.exe
```
Größe: ~10 MB (optimiert, keine Debug-Infos)

## 2. Auf den V-Server kopieren
Die **komplette `server/`-Ordnerstruktur** auf den V-Server kopieren:
```
V-Server (z.B. C:\lmu-race-control-server\)
├── lmu-race-control-server.exe
├── lmu-race-control.db (wird automatisch beim ersten Start erstellt)
```

## 3. Server starten
Auf dem V-Server in der PowerShell:
```powershell
cd C:\lmu-race-control-server
.\lmu-race-control-server.exe
```

Der Server läuft dann auf **Port 3000** und ist von außen erreichbar unter:
```
http://V-SERVER-IP:3000
```

## 4. Firewall-Freigabe
Auf dem V-Server Port 3000 in der Windows-Firewall freigeben:
```
Neue eingehende Regel → Port: 3000 → Zulassen
```

## 5. Test
Auf Deinem lokalen Rechner testen (V-SERVER-IP durch echte IP ersetzen):
```powershell
curl http://V-SERVER-IP:3000/health
```
Erwartet: `{"status":"ok","version":"0.1.0"}`

## 6. API-Key
Beim **ersten Start** wird automatisch ein API-Key generiert und im Log ausgegeben:
```
Default Tenant erstellt. API-Key: xxxxxxxx-xxxx-xxxx-xxxx-xxxxxxxxxxxx
```
Diesen Key notieren und in den Client-Einstellungen eintragen.

## 7. Client-Einstellungen
Im Tool → Einstellungen → **Server Verbindung** (nur bei Enterprise-Lizenz sichtbar):
- **Server-URL:** `http://V-SERVER-IP:3000`
- **API-Key:** den generierten Key eintragen

## 8. Als Windows-Dienst (optional)
Damit der Server automatisch startet:
```powershell
New-Service -Name "LMU-Racecontrol-Server" -BinaryPathName "C:\lmu-race-control-server\lmu-race-control-server.exe" -StartupType Automatic
Start-Service -Name "LMU-Racecontrol-Server"