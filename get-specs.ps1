$proc = Get-CimInstance -ClassName Win32_Processor | Select-Object -First 1
$procName = $proc.Name.Trim()

$gen = "Desconocida"
# Improved CPU Generation/Series Detection
if ($procName -match "i[3579]-(\d+)") { 
    $gen = "$($Matches[1])a Gen" 
}
elseif ($procName -match "Core\s+[357]\s+(\d)") { 
    $gen = "Serie $($Matches[1])" 
}
elseif ($procName -match "Ultra") { 
    $gen = "Core Ultra" 
}
elseif ($procName -match "Ryzen\s+[3579]\s+(\d)") { 
    $gen = "$($Matches[1])000 Series" 
}
elseif ($procName -match "N(\d{3})") {
    $gen = "N-Series"
}

$system = Get-CimInstance -ClassName Win32_ComputerSystem | Select-Object -First 1
$brand = "$($system.Manufacturer) $($system.Model)".Trim()

$allVideos = Get-CimInstance -ClassName Win32_VideoController
$video = $allVideos | Where-Object { $_.Name -match "RTX" } | Select-Object -First 1
if (-not $video) {
    $video = $allVideos | Where-Object { $_.CurrentHorizontalResolution -gt 0 } | Select-Object -First 1
    if (-not $video) {
        $video = $allVideos | Select-Object -First 1
    }
}

$gpu = $video.Name.Trim()

# Resolution detection (Physical focus)
$h = $video.CurrentHorizontalResolution
$v = $video.CurrentVerticalResolution

# If horizontal/vertical from VideoController is logical (due to scaling), 
# we try to parse from VideoModeDescription which usually holds the physical mode
if ($video.VideoModeDescription -match "(\d{3,4}) x (\d{3,4})") {
    $h = [int]$Matches[1]
    $v = [int]$Matches[2]
}

# Fallback for display if CIM fails
if (-not $h -or $h -eq 0) {
    try {
        Add-Type -AssemblyName System.Windows.Forms
        $h = [System.Windows.Forms.Screen]::PrimaryScreen.Bounds.Width
        $v = [System.Windows.Forms.Screen]::PrimaryScreen.Bounds.Height
    } catch {}
}

$resName = switch ($h) {
    1280 { if ($v -eq 720) { "HD" } }
    1366 { if ($v -eq 768) { "HD" } }
    1600 { if ($v -eq 900) { "HD+" } }
    1920 { if ($v -eq 1080) { "Full HD" } elseif ($v -eq 1200) { "WUXGA" } }
    2560 { if ($v -eq 1440) { "2K QHD" } elseif ($v -eq 1600) { "WQXGA" } }
    2880 { if ($v -eq 1800) { "2.8K" } }
    3840 { if ($v -eq 2160) { "4K UHD" } }
    default { "" }
}

$display = if ($h -and $v) { 
    if ($resName) { "$h x $v ($resName)" } else { "$h x $v" }
} else { 
    "No detectada" 
}

$os = Get-CimInstance -ClassName Win32_OperatingSystem | Select-Object -First 1
$osName = $os.Caption -replace "Microsoft ", ""

$memSticks = Get-CimInstance -ClassName Win32_PhysicalMemory
$memory = $memSticks | Measure-Object -Property Capacity -Sum
$ramSize = [math]::Round($memory.Sum / 1GB, 0)
$ram = "$($ramSize) GB"

$ramTypeRaw = ($memSticks | Select-Object -First 1).SMBIOSMemoryType
$ramType = switch ($ramTypeRaw) {
    20 { "DDR" }
    21 { "DDR2" }
    24 { "DDR3" }
    26 { "DDR4" }
    30 { "LPDDR4" }
    34 { "DDR5" }
    35 { "LPDDR5" }
    default { "" }
}

$drives = Get-CimInstance -ClassName Win32_DiskDrive | Where-Object { $_.MediaType -match 'Fixed' }
$totalBytes = ($drives | Measure-Object -Property Size -Sum).Sum
# Convert to GB (using 10^9 as base for retail marketing sizes)
$totalGB = $totalBytes / 1000000000
# Round to nearest 128 standard (128, 256, 512, 1024...)
$roundedGB = [math]::Round($totalGB / 128) * 128

if ($roundedGB -eq 0) { $roundedGB = [math]::Round($totalGB) }

$storage = if ($roundedGB -ge 1024) { 
    "$([math]::Round($roundedGB / 1024, 0))TB SSD" 
} else { 
    "$($roundedGB)GB SSD" 
}

$obj = [PSCustomObject]@{
    brand = $brand
    processor = $procName
    cores = [int]$proc.NumberOfCores
    threads = [int]$proc.NumberOfLogicalProcessors
    gen = $gen
    vendor = if ($procName -match "Intel") { "Intel" } elseif ($procName -match "AMD") { "AMD" } else { "Generic" }
    ram = $ram
    ramType = $ramType
    gpu = $gpu
    storage = $storage
    display = $display
    os = $osName
}

$obj | ConvertTo-Json -Compress
