$names = @("LMU_Data", "LMU_SharedMemory", "rFactor2_Data", "rFactor2_SharedMemory", "Local\LMU_Data")

Write-Host "LMU Shared Memory Check" -ForegroundColor Cyan
Write-Host "========================" -ForegroundColor Cyan
Write-Host ""

foreach ($name in $names) {
    Write-Host "Suche: $name ... " -NoNewline
    try {
        $mmf = [System.IO.MemoryMappedFiles.MemoryMappedFile]::OpenExisting($name)
        Write-Host "GEFUNDEN!" -ForegroundColor Green
        
        $stream = $mmf.CreateViewStream()
        $reader = New-Object System.IO.BinaryReader($stream)
        $data = $reader.ReadBytes(4096)
        $reader.Close()
        $stream.Dispose()
        $mmf.Dispose()
        
        Write-Host ""
        Write-Host "Hex-Dump (erste 256 Bytes):" -ForegroundColor Yellow
        Write-Host ""
        
        for ($r = 0; $r -lt 256; $r += 16) {
            $h = ""
            $a = ""
            for ($c = 0; $c -lt 16; $c++) {
                $b = $data[$r + $c]
                $h += "{0:X2} " -f $b
                if ($b -ge 32 -and $b -lt 127) { $a += [char]$b } else { $a += "." }
            }
            Write-Host ("{0:X4}: {1,-48} |{2}|" -f $r, $h, $a)
        }
        
        Write-Host ""
        Write-Host "Jetzt in BCUK Kamera druecken, dann Enter..."
        $null = Read-Host
        
        # Zweiter Durchlauf
        $mmf2 = [System.IO.MemoryMappedFiles.MemoryMappedFile]::OpenExisting($name)
        $stream2 = $mmf2.CreateViewStream()
        $reader2 = New-Object System.IO.BinaryReader($stream2)
        $data2 = $reader2.ReadBytes(4096)
        $reader2.Close()
        $stream2.Dispose()
        $mmf2.Dispose()
        
        # Vergleiche
        $changes = @()
        for ($i = 0; $i -lt 4096; $i++) {
            if ($data[$i] -ne $data2[$i]) {
                $changes += ,@($i, $data[$i], $data2[$i])
            }
        }
        
        Write-Host ""
        if ($changes.Count -eq 0) {
            Write-Host "Keine Aenderungen" -ForegroundColor Red
        } else {
            Write-Host ("$($changes.Count) Bytes geaendert:") -ForegroundColor Green
            $max = [Math]::Min(30, $changes.Count)
            for ($j = 0; $j -lt $max; $j++) {
                $off = $changes[$j][0]
                $old = $changes[$j][1]
                $new = $changes[$j][2]
                Write-Host ("  0x{0:X4}: {1,3} -> {2,3}" -f $off, $old, $new)
            }
        }
        return
    }
    catch {
        Write-Host "nicht gefunden" -ForegroundColor Red
    }
}

Write-Host ""
Write-Host "KEIN Shared Memory gefunden!" -ForegroundColor Red
Write-Host "Starte LMU (Watch-Modus) und versuche es erneut."
Read-Host "Enter zum Beenden"