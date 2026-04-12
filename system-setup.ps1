# System Setup for POS Showcase
# 1. Set Brightness to 100%
Write-Output "Setting brightness to 100%..."
try {
    $monitor = Get-WmiObject -Namespace root/WMI -Class WmiMonitorBrightnessMethods
    if ($monitor) {
        $monitor.WmiSetBrightness(1,100)
    } else {
        Write-Output "No WMI-compatible monitor found for brightness adjustment."
    }
} catch {
    Write-Output "Failed to set brightness (might not be a laptop/supported monitor): $_"
}

# 2. Disable Sleep/Hibernate
Write-Output "Configuring power settings (No Sleep)..."
powercfg /x -hibernate-timeout-ac 0
powercfg /x -standby-timeout-ac 0
powercfg /x -monitor-timeout-ac 0 # Keep monitor always on
powercfg /hibernate off

# 3. Set High Performance Plan
Write-Output "Setting High Performance power plan..."
$highPerf = powercfg /l | Select-String "High performance"
if ($highPerf) {
    $guid = $highPerf.ToString().Split(' ')[3]
    powercfg /s $guid
}

Write-Output "System setup complete."
