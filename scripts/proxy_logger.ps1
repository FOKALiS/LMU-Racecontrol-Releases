# LMU REST API Proxy Logger
# Starte DIESES Skript, dann BCUK mit Port 6398 verbinden
# Zeigt ALLE HTTP-Requests zwischen BCUK und LMU

$LMU_PORT = 6397
$PROXY_PORT = 6398
$logFile = "$env:TEMP\lmu_proxy_log.txt"
Remove-Item $logFile -ErrorAction SilentlyContinue

Write-Host "=== LMU REST API Proxy Logger ===" -ForegroundColor Cyan
Write-Host "Starte Proxy auf Port $PROXY_PORT -> LMU Port $LMU_PORT" -ForegroundColor Yellow
Write-Host "`n1. LMU laufen lassen (Port $LMU_PORT)" -ForegroundColor Green
Write-Host "2. DIESES Skript laufen lassen" -ForegroundColor Green
Write-Host "3. BCUK starten und in den Einstellungen Port auf $PROXY_PORT aendern" -ForegroundColor Green
Write-Host "4. In BCUK Kamera-Buttons klicken" -ForegroundColor Green
Write-Host "5. Hier erscheinen die Requests live" -ForegroundColor Green
Write-Host "`nDruecke STRG+C zum Beenden`n" -ForegroundColor Yellow

# Erstelle einen Listener auf dem Proxy-Port
$listener = New-Object System.Net.Sockets.TcpListener ([System.Net.IPAddress]::Loopback, $PROXY_PORT)
$listener.Start()

try {
    while ($true) {
        # Warte auf Verbindung von BCUK
        $client = $listener.AcceptTcpClient()
        $stream = $client.GetStream()
        
        # Lese den HTTP-Request
        $reader = New-Object System.IO.StreamReader($stream)
        $request = $reader.ReadToEnd()
        
        if ($request -match "^(GET|POST|PUT|DELETE) (.+?) HTTP") {
            $method = $matches[1]
            $path = $matches[2]
            $timestamp = Get-Date -Format "HH:mm:ss.fff"
            
            Write-Host "[$timestamp] $method $path" -ForegroundColor White
            
            # Logge den Request
            "$timestamp $method $path" | Out-File -FilePath $logFile -Append
            
            # Extrahiere Body (nach den Headern)
            $bodyStart = $request.IndexOf("`r`n`r`n") + 4
            if ($bodyStart -gt 4 -and $bodyStart -lt $request.Length) {
                $body = $request.Substring($bodyStart)
                if ($body.Trim().Length -gt 0) {
                    Write-Host "  Body: $body" -ForegroundColor Gray
                    "  Body: $body" | Out-File -FilePath $logFile -Append
                }
            }
        }
        
        # Leite an LMU weiter
        $lmuClient = New-Object System.Net.Sockets.TcpClient("127.0.0.1", $LMU_PORT)
        $lmuStream = $lmuClient.GetStream()
        
        # Sende den Request an LMU
        $writer = New-Object System.IO.StreamWriter($lmuStream)
        $writer.Write($request)
        $writer.Flush()
        
        # Lese die Response von LMU
        $lmuReader = New-Object System.IO.StreamReader($lmuStream)
        $response = $lmuReader.ReadToEnd()
        
        # Sende Response zurück an BCUK
        $responseWriter = New-Object System.IO.StreamWriter($stream)
        $responseWriter.Write($response)
        $responseWriter.Flush()
        
        # Aufräumen
        $lmuClient.Close()
        $client.Close()
    }
}
finally {
    $listener.Stop()
    Write-Host "`nProxy gestoppt. Log: $logFile" -ForegroundColor Cyan
}