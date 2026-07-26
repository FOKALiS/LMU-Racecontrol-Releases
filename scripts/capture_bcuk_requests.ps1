# BCUK HTTP Request Capture
# Starte dies, dann BCUK, dann klicke Kamera-Buttons
# Zeigt ALLE HTTP-Requests an localhost:6397

$port = 6397
$logFile = "$env:TEMP\bcuk_http_log.txt"
Remove-Item $logFile -ErrorAction SilentlyContinue

Write-Host "=== BCUK HTTP Capture ===" -ForegroundColor Cyan
Write-Host "Starte BCUK und klicke Kamera-Buttons" -ForegroundColor Yellow
Write-Host "Druecke STRG+C zum Beenden`n" -ForegroundColor Yellow

# Starte einen einfachen TCP-Listener, der nur mitschneidet
$listener = New-Object System.Net.Sockets.TcpListener ([System.Net.IPAddress]::Loopback, $port+1)
$listener.Start()

Write-Host "Hoere auf Port $($port+1) (redirect von BCUK nicht moeglich)" -ForegroundColor Gray
Write-Host ""

# Besser: Wir loggen einfach alle Prozesse, die auf Port 6397 zugreifen
Write-Host "Starte Netzwerk-Trace mit PowerShell..." -ForegroundColor Green

# Alternative: Verwende NETSTAT um Verbindungen zu sehen
while ($true) {
    $connections = netstat -ano | Select-String "6397"
    if ($connections) {
        $connections | Out-File -FilePath $logFile -Append
        Write-Host "`n=== Verbindungen gefunden ===" -ForegroundColor Cyan
        $connections | ForEach-Object { Write-Host $_ -ForegroundColor White }
    }
    Start-Sleep -Seconds 2
}