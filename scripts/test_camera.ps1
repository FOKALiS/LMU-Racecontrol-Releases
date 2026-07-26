# Teste alle Kamera-Endpunkte direkt
$base = "http://localhost:6397"
$body = '{}'
$headers = @{"Content-Type"="application/json"}

# Endpunkte aus BCUK's BroadcastService.js
$tests = @(
    # Format: focus/{type}/{trackSideGroup}/{shouldAdvance}
    @{url="/rest/watch/focus/SCV_COCKPIT/0/true"; name="On-board"},
    @{url="/rest/watch/focus/SCV_COCKPIT/1/false"; name="Cockpit"},
    @{url="/rest/watch/focus/SCV_NOSECAM/0/false"; name="Nose"},
    @{url="/rest/watch/focus/SCV_SWINGMAN/0/true"; name="Swingman"},
    @{url="/rest/watch/focus/SCV_TRACKSIDE/0/true"; name="Trackside"},
    @{url="/rest/watch/focus/SCV_SPECTATOR/0/true"; name="Spectator"},
    # Auch ohne trailing slash
    @{url="/rest/watch/focus/SCV_NOSECAM/0/false"; name="Nose (no slash)"},
    # Auch einfache Variante
    @{url="/rest/watch/focus/SCV_NOSECAM"; name="Nose simple"},
    @{url="/rest/watch/focus/Nose"; name="Nose alt"},
    @{url="/rest/watch/focus/Onboard"; name="Onboard alt"},
    @{url="/rest/watch/focus/TV"; name="TV alt"}
)

Write-Host "============================================"
Write-Host "Teste LMU Kamera-Endpunkte"
Write-Host "============================================"
Write-Host ""

foreach ($test in $tests) {
    try {
        $resp = Invoke-WebRequest -Uri ($base + $test.url) -Method PUT -Body $body -ContentType "application/json" -UseBasicParsing -TimeoutSec 2
        Write-Host ("[OK] " + $test.name + " -> " + $test.url + " (" + $resp.StatusCode + ")")
    } catch {
        if ($_.Exception.Response.StatusCode -eq 404) {
            Write-Host ("[404] " + $test.name + " -> " + $test.url)
        } elseif ($_.Exception.Message -match "Timeout") {
            Write-Host ("[TIMEOUT] " + $test.name + " -> " + $test.url)
        } else {
            $code = $_.Exception.Response.StatusCode.value__
            Write-Host ("[" + $code + "] " + $test.name + " -> " + $test.url + " (" + $_.Exception.Message.Substring(0, 60) + ")")
        }
    }
}

Write-Host ""
Write-Host "Fertig!"