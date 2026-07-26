# LMU Shared Memory Check
Write-Host "========================================"
Write-Host "  LMU Shared Memory Check"
Write-Host "========================================"
Write-Host ""

$names = @("LMU_Data", "LMU_SharedMemory", "rFactor2_Data", "rFactor2_SharedMemory", "Local\LMU_Data")

foreach ($name in $names) {
    Write-Host "  Suche: '$name'... " -NoNewline
    try {
        $handle = [System.Threading.MemoryMappedFiles.MemoryMappedFile]::OpenExisting($name)
        Write-Host "GEFUNDEN!"
        
        $stream = $handle.CreateViewStream()
        $reader = New-Object System.IO.BinaryReader($stream)
        $bytes = $reader.ReadBytes(256)
        $reader.Close()
        $stream.Close()
        $handle.Dispose()
        
        Write-Host ""
        Write-Host "  Hex-Dump (erste 256 Bytes):"
        Write-Host ""
        for ($row = 0; $row -lt 256; $row += 16) {
            $hex = ""
            $ascii = ""
            for ($i = 0; $i -lt 16; $i++) {
                $hex = $hex + ("{0:X2} " -f $bytes[$row + $i])
                $c = $bytes[$row + $i]
                if ($c -ge 32 -and $c -lt 127) {
                    $ascii = $ascii + [char]$c
                } else {
                    $ascii = $ascii + "."
                }
            }
            Write-Host ("  " + ("{0:X4}" -f $row) + ": " + $hex + " |" + $ascii + "|")
        }
        
        # Jetzt Snapshot 1 komplett lesen
        $handle1 = [System.Threading.MemoryMappedFiles.MemoryMappedFile]::OpenExisting($name)
        $stream1 = $handle1.CreateViewStream()
        $reader1 = New-Object System.IO.BinaryReader($stream1)
        $before = $reader1.ReadBytes(4096)
        $reader1.Close()
        $stream1.Close()
        $handle1.Dispose()
        
        Write-Host ""
        Write-Host "  Jetzt in BCUK eine Kamera-Taste druecken!"
        Write-Host "  Danach hier Enter druecken..."
        $null = Read-Host
        
        # Snapshot 2
        $handle2 = [System.Threading.MemoryMappedFiles.MemoryMappedFile]::OpenExisting($name)
        $stream2 = $handle2.CreateViewStream()
        $reader2 = New-Object System.IO.BinaryReader($stream2)
        $after = $reader2.ReadBytes(4096)
        $reader2.Close()
        $stream2.Close()
        $handle2.Dispose()
        
        # Vergleiche
        $changes = @()
        for ($i = 0; $i -lt 4096; $i++) {
            if ($before[$i] -ne $after[$i]) {
                $changes = $changes + @($i, $before[$i], $after[$i])
            }
        }
        
        if ($changes.Count -eq 0) {
            Write-Host ""
            Write-Host "  Keine Aenderungen gefunden!"
        } else {
            $count = $changes.Count / 3
            Write-Host ""
            Write-Host ("  " + $count + " Bytes geaendert!")
            Write-Host ""
            for ($j = 0; $j -lt $count -and $j -lt 30; $j++) {
                $offset = $changes[$j * 3]
                $oldVal = $changes[$j * 3 + 1]
                $newVal = $changes[$j * 3 + 2]
                $delta = $newVal - $oldVal
                Write-Host ("    0x" + ("{0:X4}" -f $offset) + ": " + $oldVal + " -> " + $newVal + " (Delta " + $delta + ")")
            }
        }
        
        return
    }
    catch {
        Write-Host "nicht gefunden"
    }
}

Write-Host ""
Write-Host "  KEIN Shared Memory gefunden!"
Write-Host ""
Write-Host "  Starte LMU mit Watch-Modus und versuche es erneut."

Write-Host ""
$null = Read-Host "  Enter zum Beenden..."