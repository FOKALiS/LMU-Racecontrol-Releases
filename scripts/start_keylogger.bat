@echo off
echo ============================================
echo LMU Key Capture STARTEN
echo ============================================
echo.
echo 1. Dieses Fenster offen lassen
echo 2. In BCUK auf "Nose" klicken
echo 3. Hier im Fenster eine Taste druecken
echo 4. Dann die Log-Datei oeffnen:
echo    Desktop - lmu_keys_logged.txt
echo.
echo ============================================
echo.

REM AHK-Skript starten (Admin)
cd /d "%~dp0"
start "" "%~dp0keylogger.ahk"

echo AHK-Skript gestartet!
echo.
echo JETZT: In BCUK auf "Nose" klicken!
echo Dann eine beliebige Taste hier druecken...
pause >nul

echo.
echo Log-Datei wird geoeffnet...
start notepad "%USERPROFILE%\Desktop\lmu_keys_logged.txt"
echo.
echo Wenn die Datei leer ist, war BCUK schneller.
echo Einfach nochmal in BCUK auf "Nose" klicken,
echo dann hier eine Taste druecken und die Log-Datei neu laden.
pause