@echo off
echo ========================================
echo  LMU Camera Endpoint Test
echo ========================================
echo.
echo Testet verschiedene Parameter fuer den
echo CameraController-Endpunkt
echo.
echo VORAUSSETZUNG: LMU muss laufen (Port 6397)
echo ========================================
echo.

:: Pruefe ob LMU erreichbar ist
curl -s -o NUL -w "%%{http_code}" http://localhost:6397/rest/watch/sessionInfo >nul 2>&1
if errorlevel 1 (
    echo FEHLER: LMU nicht erreichbar auf Port 6397
    echo Starte LMU mit Watch-Modus und versuche es erneut.
    pause
    exit /b 1
)

echo LMU ist erreichbar!
echo.
echo ========================================
echo Test 1: CameraController getCameraInfo
echo ========================================
curl -s http://localhost:6397/rest/replay/CameraController/getCameraInfo
echo.

echo.
echo ========================================
echo Test 2: CameraController switchCameraFamily mit "id":4
echo ========================================
curl -s -X POST http://localhost:6397/rest/replay/CameraController/switchCameraFamily -H "Content-Type: application/json" -d "{\"id\":4}"
echo.
echo Aktuelle Kamera nach Test 2:
curl -s http://localhost:6397/rest/replay/CameraController/getCameraInfo
echo.

echo.
echo ========================================
echo Test 3: CameraController switchCameraFamily mit "id":6
echo ========================================
curl -s -X POST http://localhost:6397/rest/replay/CameraController/switchCameraFamily -H "Content-Type: application/json" -d "{\"id\":6}"
echo.
echo Aktuelle Kamera nach Test 3:
curl -s http://localhost:6397/rest/replay/CameraController/getCameraInfo
echo.

echo.
echo ========================================
echo Test 4: CameraController switchCameraFamily mit "group":"TV"
echo ========================================
curl -s -X POST http://localhost:6397/rest/replay/CameraController/switchCameraFamily -H "Content-Type: application/json" -d "{\"group\":\"TV\"}"
echo.
echo Aktuelle Kamera nach Test 4:
curl -s http://localhost:6397/rest/replay/CameraController/getCameraInfo
echo.

echo.
echo ========================================
echo Test 5: CameraController switchCameraFamily mit "group":"Onboard"
echo ========================================
curl -s -X POST http://localhost:6397/rest/replay/CameraController/switchCameraFamily -H "Content-Type: application/json" -d "{\"group\":\"Onboard\"}"
echo.
echo Aktuelle Kamera nach Test 5:
curl -s http://localhost:6397/rest/replay/CameraController/getCameraInfo
echo.

echo.
echo ========================================
echo Test 6: CameraController switchCameraFamily mit "name":"TV"
echo ========================================
curl -s -X POST http://localhost:6397/rest/replay/CameraController/switchCameraFamily -H "Content-Type: application/json" -d "{\"name\":\"TV\"}"
echo.
echo Aktuelle Kamera nach Test 6:
curl -s http://localhost:6397/rest/replay/CameraController/getCameraInfo
echo.

echo.
echo ========================================
echo Test 7: CameraController switchCameraFamily mit "family":"Trackside"
echo ========================================
curl -s -X POST http://localhost:6397/rest/replay/CameraController/switchCameraFamily -H "Content-Type: application/json" -d "{\"family\":\"Trackside\"}"
echo.
echo Aktuelle Kamera nach Test 7:
curl -s http://localhost:6397/rest/replay/CameraController/getCameraInfo
echo.

echo.
echo ========================================
echo Test 8: PUT /rest/watch/focus/TV (alter Endpunkt)
echo ========================================
curl -s -X PUT http://localhost:6397/rest/watch/focus/TV -H "Content-Type: application/json" -d "{}"
echo.
echo Aktuelle Kamera nach Test 8:
curl -s http://localhost:6397/rest/replay/CameraController/getCameraInfo
echo.

echo.
echo ========================================
echo Test 9: PUT /rest/watch/focus/Onboard
echo ========================================
curl -s -X PUT http://localhost:6397/rest/watch/focus/Onboard -H "Content-Type: application/json" -d "{}"
echo.
echo Aktuelle Kamera nach Test 9:
curl -s http://localhost:6397/rest/replay/CameraController/getCameraInfo
echo.

echo.
echo ========================================
echo TEST ABGESCHLOSSEN
echo ========================================
echo.
echo Kopiere das gesamte Terminal-Ergebnis
echo und poste es hier.
echo.
pause