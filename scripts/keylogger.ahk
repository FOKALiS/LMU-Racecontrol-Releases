; LMU Key Capture - AutoHotkey v2
; Einfach und direkt

#SingleInstance Force

; Log-Datei auf Desktop
logFile := A_Desktop . "\lmu_keys_log.txt"

; Start-Meldung
FileAppend "=== LMU Key Capture gestartet " FormatTime(, "HH:mm:ss") " ===`n", logFile

; Hotkeys - alle wichtigen Tasten
keys := ["F1","F2","F3","F4","F5","F6","F7","F8","F9","F10","F11","F12",
    "PgDn","PgUp","Home","End","Left","Right","Up","Down","Insert","Delete",
    "Numpad0","Numpad1","Numpad2","Numpad3","Numpad4","Numpad5","Numpad6","Numpad7","Numpad8","Numpad9",
    "Space","Tab","Enter","Escape","Backspace",
    "a","b","c","d","e","f","g","h","i","j","k","l","m",
    "n","o","p","q","r","s","t","u","v","w","x","y","z",
    "0","1","2","3","4","5","6","7","8","9",
    ".",",","-","=",
    "LControl","RControl","LAlt","RAlt","LShift","RShift","LWin","RWin"]

for key in keys {
    Try Hotkey("~*$" key, KeyPressed)
}

KeyPressed(ThisHotkey) {
    time := FormatTime(, "HH:mm:ss")
    keyName := SubStr(ThisHotkey, 4)
    FileAppend time " - " keyName "`n", logFile
}

; Info-Box
TrayTip "LMU Key Capture läuft!", "Jetzt in BCUK auf Nose klicken`nDann Log-Datei oeffnen:", "Iconi"