# LMU Broadcast Control - HTTP Sniffer
# Zeigt live alle HTTP-Requests an, die BCUK an LMU sendet
# Kein Fiddler, kein Proxy, keine Zertifikate nötig!

$LMU_PORT = 6397
$logFile = "$env:TEMP\bcuk_requests.log"

Write-Host "=== LMU BCUK HTTP Sniffer ===" -ForegroundColor Cyan
Write-Host "Starte LMU und BCUK, dann klicke auf Kamera-Buttons" -ForegroundColor Yellow
Write-Host "Druecke STRG+C zum Beenden" -ForegroundColor Yellow
Write-Host "Log: $logFile" -ForegroundColor Gray
Write-Host ""

# Lösche altes Log
Remove-Item $logFile -ErrorAction SilentlyContinue

# Starte netsh trace für HTTP
$traceFile = "$env:TEMP\bcuk_trace.etl"
Remove-Item $traceFile -ErrorAction SilentlyContinue

Write-Host "1️⃣  Starte Netzwerk-Trace (Admin-Rechte benoetigt)..." -ForegroundColor Green
Start-Process -FilePath "netsh" -ArgumentList "trace start capture=yes tracefile=$traceFile report=no overwrite=yes correlation=no" -Verb RunAs -Wait

Write-Host "2️⃣  Jetzt in BCUK die Kamera-Buttons klicken!" -ForegroundColor Green
Write-Host "    Klicke JEDEN Kamera-Button einmal:" -ForegroundColor Yellow
Write-Host "    - TV" -ForegroundColor White
Write-Host "    - Onboard" -ForegroundColor White
Write-Host "    - Nose" -ForegroundColor White
Write-Host "    - Heli" -ForegroundColor White
Write-Host "    - Swingman" -ForegroundColor White
Write-Host ""
Write-Host "3️⃣  Wenn Du alle durch hast, druecke EINGABE" -ForegroundColor Green
Read-Host

Write-Host "4️⃣  Stoppe Trace..." -ForegroundColor Green
Start-Process -FilePath "netsh" -ArgumentList "trace stop" -Verb RunAs -Wait

Write-Host "5️⃣  Analysiere Trace..." -ForegroundColor Green

# Extrahiere HTTP-Requests aus dem Trace
$output = netsh trace convert $traceFile 2>&1
$cabFile = $traceFile -replace '\.etl$', '.cab'
if (Test-Path $cabFile) {
    Write-Host "   Entpacke $cabFile ..." -ForegroundColor Gray
    Expand-Archive -Path $cabFile -DestinationPath "$env:TEMP\bcuk_trace" -Force
}

# Suche nach HTTP-Requests in der TXT-Datei
$txtFiles = Get-ChildItem "$env:TEMP\bcuk_trace" -Filter "*.txt" -Recurse
foreach ($txt in $txtFiles) {
    $content = Get-Content $txt.FullName -Raw
    if ($content -match "6397") {
        Write-Host "=== Gefunden in: $($txt.Name) ===" -ForegroundColor Cyan
        # Extrahiere HTTP-Methoden und URLs
        $lines = $content -split "`n"
        foreach ($line in $lines) {
            if ($line -match "(GET|POST|PUT|DELETE)\s+/rest") {
                Write-Host "  $line" -ForegroundColor White
                $line | Out-File -FilePath $logFile -Append
            }
        }
    }
}

Write-Host ""
Write-Host "=== Fertig! ===" -ForegroundColor Cyan
Write-Host "Gefundene Requests stehen in: $logFile" -ForegroundColor Yellow
Write-Host ""
Write-Host "Alternativ: Schick mir den Inhalt von $logFile" -ForegroundColor Yellow