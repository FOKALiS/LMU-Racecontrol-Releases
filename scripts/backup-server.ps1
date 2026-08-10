# LMU RACECONTROL Server Backup Script
# Auf dem V-Server per Task Scheduler automatisierbar
# Beispiel: Täglich um 3:00 Uhr
# 
# Einrichtung:
#   powershell.exe -File "C:\lmu-race-control-server\backup-server.ps1"
#
# Oder über das Admin-Dashboard (POST /api/backup)

param(
    [string]$BackupPath = "C:\lmu-race-control-server\backups",
    [string]$DbPath = "lmu-race-control.db",
    [string]$KeepDays = 30
)

$timestamp = Get-Date -Format "yyyyMMdd_HHmmss"
$backupDir = $BackupPath

# Verzeichnis erstellen falls nicht vorhanden
if (-not (Test-Path $backupDir)) {
    New-Item -ItemType Directory -Path $backupDir -Force | Out-Null
    Write-Host "📁 Backup-Verzeichnis erstellt: $backupDir"
}

# Datenbank kopieren
$backupFile = Join-Path $backupDir "lmu-race-control-backup-$timestamp.db"
if (Test-Path $DbPath) {
    Copy-Item $DbPath $backupFile -Force
    Write-Host "✅ Backup erstellt: $backupFile"
    
    # Alte Backups (> KeepDays) löschen
    $cutoff = (Get-Date).AddDays(-$KeepDays)
    Get-ChildItem $backupDir -Filter "lmu-race-control-backup-*.db" | Where-Object {
        $_.LastWriteTime -lt $cutoff
    } | ForEach-Object {
        Remove-Item $_.FullName -Force
        Write-Host "🧹 Altes Backup gelöscht: $($_.Name)"
    }
} else {
    Write-Host "❌ Datenbank nicht gefunden: $DbPath"
    exit 1
}

Write-Host "✅ Backup abgeschlossen!"