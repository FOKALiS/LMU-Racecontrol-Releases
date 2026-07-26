# LMU Shared Memory Dumper (PowerShell)
# Liest LMU_Data aus und zeigt Aenderungen nach BCUK-Klick

Add-Type -TypeDefinition @"
using System;
using System.IO.MemoryMappedFiles;
using System.Runtime.InteropServices;

public class LmuSmDumper {
    private MemoryMappedFile _mmf;
    private MemoryMappedViewAccessor _accessor;
    
    public bool Open() {
        try {
            _mmf = MemoryMappedFile.OpenExisting("LMU_Data");
            _accessor = _mmf.CreateViewAccessor(0, 4096);
            return true;
        } catch {
            return false;
        }
    }
    
    public void Close() {
        if (_accessor != null) _accessor.Dispose();
        if (_mmf != null) _mmf.Dispose();
    }
    
    public byte[] ReadBytes(int offset, int count) {
        byte[] buffer = new byte[count];
        _accessor.ReadArray(offset, buffer, 0, count);
        return buffer;
    }
    
    public uint ReadU32(int offset) {
        return _accessor.ReadUInt32(offset);
    }
    
    public float ReadF32(int offset) {
        return _accessor.ReadSingle(offset);
    }
    
    public string HexDump(int offset, int length) {
        byte[] data = ReadBytes(offset, length);
        string result = "";
        for (int i = 0; i < data.Length; i += 16) {
            result += String.Format("  {0:X4}: ", offset + i);
            for (int j = 0; j < 16 && i + j < data.Length; j++) {
                result += String.Format("{0:X2} ", data[i + j]);
            }
            result += "\n";
        }
        return result;
    }
    
    public string Diff(byte[] a, byte[] b, int maxChanges) {
        string result = "";
        int changes = 0;
        for (int i = 0; i < Math.Min(a.Length, b.Length) && changes < maxChanges; i++) {
            if (a[i] != b[i]) {
                int aligned = i & ~3;
                uint u32Old = BitConverter.ToUInt32(a, aligned);
                uint u32New = BitConverter.ToUInt32(b, aligned);
                float f32Old = BitConverter.ToSingle(a, aligned);
                float f32New = BitConverter.ToSingle(b, aligned);
                result += String.Format("  0x{0:X4}: {1,3} -> {2,3}  (u32: {3} -> {4}, f32: {5:F4} -> {6:F4})\n",
                    i, a[i], b[i], u32Old, u32New, f32Old, f32New);
                changes++;
            }
        }
        if (changes == 0) result = "  Keine Aenderungen gefunden.\n";
        return result;
    }
}
"@

$dumper = New-Object LmuSmDumper

Write-Host "============================================================"
Write-Host "LMU Shared Memory Dumper"
Write-Host "============================================================"

if (-not $dumper.Open()) {
    Write-Host "FEHLER: LMU_Data Shared Memory NICHT gefunden!"
    Write-Host "Stelle sicher, dass LMU laeuft."
    exit 1
}

Write-Host "OK - LMU_Data Shared Memory geoeffnet!"
Write-Host ""

# Hex-Dump
Write-Host "--- HEX-DUMP (erste 256 Bytes) ---"
Write-Host ($dumper.HexDump(0, 256))
Write-Host ""

# Werte
Write-Host "--- WERTE (erste 256 Bytes, nur nicht-Null) ---"
for ($offset = 0; $offset -lt 256; $offset += 4) {
    $u32 = $dumper.ReadU32($offset)
    $f32 = $dumper.ReadF32($offset)
    if ($u32 -ne 0 -or [Math]::Abs($f32) -gt 0.001) {
        Write-Host ("  Offset 0x{0:X4}: u32={1,10}  f32={2,10:F4}" -f $offset, $u32, $f32)
    }
}
Write-Host ""

# Snapshot 1
$snap1 = $dumper.ReadBytes(0, 4096)

Write-Host "============================================================"
Write-Host "JETZT in Broadcast Control auf 'Nose' klicken!"
Write-Host "Dann ENTER druecken fuer zweiten Snapshot..."
Write-Host "============================================================"
Read-Host

# Snapshot 2
$snap2 = $dumper.ReadBytes(0, 4096)

# Unterschiede
Write-Host ""
Write-Host "============================================================"
$diff = $dumper.Diff($snap1, $snap2, 50)
$changeCount = [regex]::Matches($diff, "0x[0-9A-F]{4}:").Count
Write-Host ("AENDERUNGEN: {0} Bytes geaendert" -f $changeCount)
Write-Host "============================================================"
Write-Host ""
Write-Host $diff

$dumper.Close()
Write-Host ""
Write-Host "Fertig!"