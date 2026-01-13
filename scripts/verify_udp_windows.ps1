param(
  [Parameter(Mandatory=$true)][string]$PS5_IP,
  [int]$HB_SECONDS = 10,
  [int]$CAPTURE_EXTRA = 5
)

Write-Host "Starting UDP heartbeat test for $PS5_IP..."

$pktmon = Get-Command pktmon -ErrorAction SilentlyContinue
if ($pktmon) {
  try {
    pktmon filter remove | Out-Null
  } catch {}
  pktmon filter add -p 33739 | Out-Null
  pktmon filter add -p 33740 | Out-Null
  pktmon start --capture --pkt-size 0 | Out-Null
  Start-Sleep -Seconds 1
  Write-Host "pktmon capture started (requires Administrator)."
} else {
  Write-Host "pktmon not found; run Wireshark manually for capture."
}

$client = New-Object System.Net.Sockets.UdpClient
for ($i = 0; $i -lt $HB_SECONDS; $i++) {
  [void]$client.Send([byte[]](0x41), 1, $PS5_IP, 33739)
  Start-Sleep -Seconds 1
}
Write-Host "heartbeat done"
Start-Sleep -Seconds $CAPTURE_EXTRA

if ($pktmon) {
  pktmon stop | Out-Null
  pktmon format pktmon.etl -o pktmon.txt | Out-Null
  Write-Host "pktmon output (filtered for 33739/33740):"
  Get-Content pktmon.txt | Select-String "33739|33740"
  Remove-Item pktmon.etl -ErrorAction SilentlyContinue
}
