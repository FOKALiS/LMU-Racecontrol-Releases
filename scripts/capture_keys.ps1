# PowerShell Key Capture für LMU
# Starte dieses Skript, klick in BCUK auf "Nose", dann schau in die Log-Datei

Add-Type -AssemblyName System.Windows.Forms

$logFile = "$env:USERPROFILE\Desktop\lmu_keys_logged.txt"
"Key Capture gestartet um $(Get-Date -Format 'HH:mm:ss')" | Out-File $logFile

Write-Host "============================================"
Write-Host "LMU Key Capture läuft!"
Write-Host "JETZT in BCUK auf Nose klicken!"
Write-Host "Dann schliess dieses Fenster mit STRG+C"
Write-Host "============================================"
Write-Host ""

$count = 0
while ($count -lt 50) {
    Start-Sleep -Milliseconds 50
    
    # Prüfe alle relevanten Tasten
    $keys = @(
        @{key=[System.Windows.Forms.Keys]::F1; name="F1"},
        @{key=[System.Windows.Forms.Keys]::F2; name="F2"},
        @{key=[System.Windows.Forms.Keys]::F3; name="F3"},
        @{key=[System.Windows.Forms.Keys]::F4; name="F4"},
        @{key=[System.Windows.Forms.Keys]::F5; name="F5"},
        @{key=[System.Windows.Forms.Keys]::F6; name="F6"},
        @{key=[System.Windows.Forms.Keys]::F7; name="F7"},
        @{key=[System.Windows.Forms.Keys]::F8; name="F8"},
        @{key=[System.Windows.Forms.Keys]::F9; name="F9"},
        @{key=[System.Windows.Forms.Keys]::F10; name="F10"},
        @{key=[System.Windows.Forms.Keys]::F11; name="F11"},
        @{key=[System.Windows.Forms.Keys]::F12; name="F12"},
        @{key=[System.Windows.Forms.Keys]::Next; name="PageDown"},
        @{key=[System.Windows.Forms.Keys]::Prior; name="PageUp"},
        @{key=[System.Windows.Forms.Keys]::Home; name="Home"},
        @{key=[System.Windows.Forms.Keys]::End; name="End"},
        @{key=[System.Windows.Forms.Keys]::Left; name="Left"},
        @{key=[System.Windows.Forms.Keys]::Right; name="Right"},
        @{key=[System.Windows.Forms.Keys]::Up; name="Up"},
        @{key=[System.Windows.Forms.Keys]::Down; name="Down"},
        @{key=[System.Windows.Forms.Keys]::Insert; name="Insert"},
        @{key=[System.Windows.Forms.Keys]::Delete; name="Delete"},
        @{key=[System.Windows.Forms.Keys]::NumPad0; name="Num0"},
        @{key=[System.Windows.Forms.Keys]::NumPad1; name="Num1"},
        @{key=[System.Windows.Forms.Keys]::NumPad2; name="Num2"},
        @{key=[System.Windows.Forms.Keys]::NumPad3; name="Num3"},
        @{key=[System.Windows.Forms.Keys]::NumPad4; name="Num4"},
        @{key=[System.Windows.Forms.Keys]::NumPad5; name="Num5"},
        @{key=[System.Windows.Forms.Keys]::NumPad6; name="Num6"},
        @{key=[System.Windows.Forms.Keys]::NumPad7; name="Num7"},
        @{key=[System.Windows.Forms.Keys]::NumPad8; name="Num8"},
        @{key=[System.Windows.Forms.Keys]::NumPad9; name="Num9"},
        @{key=[System.Windows.Forms.Keys]::Space; name="Space"},
        @{key=[System.Windows.Forms.Keys]::Tab; name="Tab"},
        @{key=[System.Windows.Forms.Keys]::Return; name="Enter"},
        @{key=[System.Windows.Forms.Keys]::Escape; name="Esc"},
        @{key=[System.Windows.Forms.Keys]::Back; name="Backspace"},
        @{key=[System.Windows.Forms.Keys]::A; name="A"},
        @{key=[System.Windows.Forms.Keys]::B; name="B"},
        @{key=[System.Windows.Forms.Keys]::C; name="C"},
        @{key=[System.Windows.Forms.Keys]::D; name="D"},
        @{key=[System.Windows.Forms.Keys]::E; name="E"},
        @{key=[System.Windows.Forms.Keys]::F; name="F"},
        @{key=[System.Windows.Forms.Keys]::G; name="G"},
        @{key=[System.Windows.Forms.Keys]::H; name="H"},
        @{key=[System.Windows.Forms.Keys]::I; name="I"},
        @{key=[System.Windows.Forms.Keys]::J; name="J"},
        @{key=[System.Windows.Forms.Keys]::K; name="K"},
        @{key=[System.Windows.Forms.Keys]::L; name="L"},
        @{key=[System.Windows.Forms.Keys]::M; name="M"},
        @{key=[System.Windows.Forms.Keys]::N; name="N"},
        @{key=[System.Windows.Forms.Keys]::O; name="O"},
        @{key=[System.Windows.Forms.Keys]::P; name="P"},
        @{key=[System.Windows.Forms.Keys]::Q; name="Q"},
        @{key=[System.Windows.Forms.Keys]::R; name="R"},
        @{key=[System.Windows.Forms.Keys]::S; name="S"},
        @{key=[System.Windows.Forms.Keys]::T; name="T"},
        @{key=[System.Windows.Forms.Keys]::U; name="U"},
        @{key=[System.Windows.Forms.Keys]::V; name="V"},
        @{key=[System.Windows.Forms.Keys]::W; name="W"},
        @{key=[System.Windows.Forms.Keys]::X; name="X"},
        @{key=[System.Windows.Forms.Keys]::Y; name="Y"},
        @{key=[System.Windows.Forms.Keys]::Z; name="Z"},
        @{key=[System.Windows.Forms.Keys]::D0; name="0"},
        @{key=[System.Windows.Forms.Keys]::D1; name="1"},
        @{key=[System.Windows.Forms.Keys]::D2; name="2"},
        @{key=[System.Windows.Forms.Keys]::D3; name="3"},
        @{key=[System.Windows.Forms.Keys]::D4; name="4"},
        @{key=[System.Windows.Forms.Keys]::D5; name="5"},
        @{key=[System.Windows.Forms.Keys]::D6; name="6"},
        @{key=[System.Windows.Forms.Keys]::D7; name="7"},
        @{key=[System.Windows.Forms.Keys]::D8; name="8"},
        @{key=[System.Windows.Forms.Keys]::D9; name="9"},
        @{key=[System.Windows.Forms.Keys]::OemPeriod; name="."},
        @{key=[System.Windows.Forms.Keys]::Oemcomma; name=","},
        @{key=[System.Windows.Forms.Keys]::OemMinus; name="-"},
        @{key=[System.Windows.Forms.Keys]::Oemplus; name="+"},
        @{key=[System.Windows.Forms.Keys]::LControlKey; name="LCtrl"},
        @{key=[System.Windows.Forms.Keys]::RControlKey; name="RCtrl"},
        @{key=[System.Windows.Forms.Keys]::LMenu; name="LAlt"},
        @{key=[System.Windows.Forms.Keys]::RMenu; name="RAlt"},
        @{key=[System.Windows.Forms.Keys]::LShiftKey; name="LShift"},
        @{key=[System.Windows.Forms.Keys]::RShiftKey; name="RShift"},
        @{key=[System.Windows.Forms.Keys]::LWin; name="LWin"},
        @{key=[System.Windows.Forms.Keys]::RWin; name="RWin"}
    )
    
    foreach ($k in $keys) {
        if ([System.Windows.Forms.Control]::IsKeyLocked($k.key)) {
            # Skip lock keys
        }
        $state = [System.Windows.Forms.Control]::IsKeyLocked($k.key)  # not what we want
    }
    
    # Use GetAsyncKeyState via Win32 API
    $method = [System.Windows.Forms.Control].GetMethod('IsKeyLocked', [System.Reflection.BindingFlags]'NonPublic,Static')
    if (-not $method) {
        # Fallback: use raw P/Invoke
        $source = @'
using System;
using System.Runtime.InteropServices;
public class KeyChecker {
    [DllImport("user32.dll")]
    public static extern short GetAsyncKeyState(int vKey);
    
    public static bool IsPressed(int vKey) {
        return (GetAsyncKeyState(vKey) & 0x8000) != 0;
    }
}
'@
        Add-Type -TypeDefinition $source -ReferencedAssemblies "System.Runtime.InteropServices" 2>$null
    }
}

Write-Host "Fertig - $count Tasten erfasst!"
Write-Host "Log-Datei: $logFile"