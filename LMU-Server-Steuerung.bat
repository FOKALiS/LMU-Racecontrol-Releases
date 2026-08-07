@echo off
title LMU RACECONTROL - Server Steuerung
color 0A
setlocal enabledelayedexpansion

:: ─────────────────────────────────────────────
::  LMU RACECONTROL - Server Steuerzentrale
::  Startet, stoppt und überwacht den Server
:: ─────────────────────────────────────────────

:: Server-Verzeichnis (wo die lmu-race-control-server.exe liegt)
set "SERVER_DIR=%~dp0server"
set "SERVER_EXE=lmu-race-control-server.exe"
set "SERVER_URL=http://localhost:3000/health"

:: Prüfe ob der Server läuft
:check_server
>nul 2>&1 curl -s --connect-timeout 2 %SERVER_URL%
if %errorlevel%==0 (
    set "SERVER_RUNNING=1"
) else (
    set "SERVER_RUNNING=0"
)

:menu
cls
echo.
echo   ========================================================
echo        🏁  LMU RACECONTROL - Server Steuerzentrale
echo   ========================================================
echo.
if "%SERVER_RUNNING%"=="1" (
    echo   [🟢] Server: LÄUFT  (http://localhost:3000)
) else (
    echo   [🔴] Server: GESTOPPT
)
echo.
echo   ┌─────────────────────────────────────────────┐
echo   │  1.  Server STARTEN                         │
echo   │  2.  Server NEUSTARTEN                      │
echo   │  3.  Server STOPPEN                         │
echo   │  4.  Admin-Dashboard öffnen                 │
echo   │  5.  Status prüfen                          │
echo   │  0.  Beenden                                │
echo   └─────────────────────────────────────────────┘
echo.
set /p choice="   Auswahl: "

if "%choice%"=="1" goto start
if "%choice%"=="2" goto restart
if "%choice%"=="3" goto stop
if "%choice%"=="4" goto dashboard
if "%choice%"=="5" goto check_server
if "%choice%"=="0" goto end
goto menu

:: ────────────────── SERVER STARTEN ──────────────────
:start
cls
echo.
echo   Starte LMU RACECONTROL Server...
echo.
if "%SERVER_RUNNING%"=="1" (
    echo   ⚠️  Server läuft bereits!
    echo.
    pause
    goto check_server
)

if not exist "%SERVER_DIR%\%SERVER_EXE%" (
    echo   ❌ Datei nicht gefunden: %SERVER_DIR%\%SERVER_EXE%
    echo      Bitte lege die Datei in den Ordner "server" neben dieser Datei.
    echo.
    pause
    goto menu
)

:: Server im Hintergrund starten
start "LMU RACECONTROL Server" /MIN cmd /c "cd /d "%SERVER_DIR%" && "%SERVER_EXE%""

:: Warten bis der Server antwortet
echo   Warte auf Server-Start...
set /a attempts=0
:wait_start
>nul 2>&1 curl -s --connect-timeout 1 %SERVER_URL%
if %errorlevel%==0 (
    echo   ✅ Server erfolgreich gestartet!
    echo.
    echo   Dashboard: http://localhost:3000/admin/
    echo.
    set "SERVER_RUNNING=1"
    pause
    goto menu
)
set /a attempts+=1
if %attempts% GEQ 20 (
    echo   ⚠️  Server konnte nicht erreicht werden (Timeout).
    echo      Prüfe ob Port 3000 frei ist.
    echo.
    pause
    goto check_server
)
timeout /t 1 /nobreak >nul
goto wait_start

:: ────────────────── SERVER NEUSTARTEN ──────────────────
:restart
cls
echo.
echo   Starte Server neu...
echo.
if "%SERVER_RUNNING%"=="1" (
    echo   Stoppe alten Server-Prozess...
    taskkill /f /im %SERVER_EXE% >nul 2>&1
    timeout /t 2 /nobreak >nul
    echo   Starte Server neu...
    start "LMU RACECONTROL Server" /MIN cmd /c "cd /d "%SERVER_DIR%" && "%SERVER_EXE%""
    timeout /t 3 /nobreak >nul
    echo   ✅ Server neu gestartet!
) else (
    echo   ⚠️  Server läuft nicht - starte ihn neu...
    start "LMU RACECONTROL Server" /MIN cmd /c "cd /d "%SERVER_DIR%" && "%SERVER_EXE%""
    timeout /t 3 /nobreak >nul
    echo   ✅ Server gestartet!
)
echo.
set "SERVER_RUNNING=1"
pause
goto check_server

:: ────────────────── SERVER STOPPEN ──────────────────
:stop
cls
echo.
echo   Stoppe LMU RACECONTROL Server...
echo.
if "%SERVER_RUNNING%"=="1" (
    taskkill /f /im %SERVER_EXE% >nul 2>&1
    if %errorlevel%==0 (
        echo   ✅ Server gestoppt!
        set "SERVER_RUNNING=0"
    ) else (
        echo   ⚠️  Konnte Prozess nicht beenden - Admin-Rechte nötig?
    )
) else (
    echo   ⚠️  Server läuft nicht.
)
echo.
pause
goto check_server

:: ────────────────── DASHBOARD ÖFFNEN ──────────────────
:dashboard
start "" http://localhost:3000/admin/
goto check_server

:end
endlocal
exit