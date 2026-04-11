# Hardware Detection Script for Zenit (Optimized and Robust)
# Silence internal errors to prevent JSON corruption
$ErrorActionPreference = "SilentlyContinue"

try {
    $proc = Get-CimInstance -ClassName Win32_Processor | Select-Object -First 1
    $procName = $proc.Name.Trim()

    $gen = "Desconocida"
    if ($procName -match "i[3579]-(\d+)") { $gen = "$($Matches[1])a Gen" }
    elseif ($procName -match "Core\s+[357]\s+(\d)") { $gen = "Serie $($Matches[1])" }
    elseif ($procName -match "Ultra") { $gen = "Core Ultra" }
    elseif ($procName -match "Ryzen\s+[3579]\s+(\d)") { $gen = "$($Matches[1])000 Series" }
    elseif ($procName -match "N(\d{3})") { $gen = "N-Series" }

    $system = Get-CimInstance -ClassName Win32_ComputerSystem | Select-Object -First 1
    $sysBrand = $system.Manufacturer.Trim()
    
    # If the system brand is generic (common in custom PCs), use the Motherboard Manufacturer
    if ($sysBrand -match "To be filled|System manufacturer|Default string|System Product Name" -or -not $sysBrand) { 
        $baseboard = Get-CimInstance -ClassName Win32_BaseBoard | Select-Object -First 1
        $sysBrand = $baseboard.Manufacturer.Trim()
    }

    # Final clean up for redundant strings
    if ($sysBrand -match "To be filled|Default string" -or -not $sysBrand) {
        $sysBrand = "PC Desktop"
    }

    $brand = $sysBrand
    $model = $system.Model.Trim()
    if ($model -eq "Default string" -or $model -eq "System Product Name") { 
        $baseboard = Get-CimInstance -ClassName Win32_BaseBoard | Select-Object -First 1
        $model = $baseboard.Product.Trim()
    }

    $allVideos = Get-CimInstance -ClassName Win32_VideoController
    $video = $allVideos | Where-Object { $_.Name -match "RTX|GTX|Radeon" } | Select-Object -First 1
    if (-not $video) { $video = $allVideos | Select-Object -First 1 }
    $gpu = $video.Name.Trim()

    # Resolution detection
    $h = $video.CurrentHorizontalResolution
    $v = $video.CurrentVerticalResolution
    if ($video.VideoModeDescription -match "(\d{3,4}) x (\d{3,4})") {
        $h = [int]$Matches[1]
        $v = [int]$Matches[2]
    }

    $resName = switch ($h) {
        1280 { if ($v -eq 720) { "HD" } }
        1366 { if ($v -eq 768) { "HD" } }
        1920 { if ($v -eq 1080) { "Full HD" } }
        2560 { if ($v -eq 1440) { "2K QHD" } }
        3840 { if ($v -eq 2160) { "4K UHD" } }
        default { "" }
    }
    $display = if ($h -and $v) { if ($resName) { "$h x $v ($resName)" } else { "$h x $v" } } else { "1920 x 1080" }

    $os = Get-CimInstance -ClassName Win32_OperatingSystem | Select-Object -First 1
    $osName = $os.Caption -replace "Microsoft ", ""

    $memSticks = Get-CimInstance -ClassName Win32_PhysicalMemory
    $ramSize = [math]::Round(($memSticks | Measure-Object -Property Capacity -Sum).Sum / 1GB, 0)
    $ramTypeRaw = ($memSticks | Select-Object -First 1).SMBIOSMemoryType
    $ramType = switch ($ramTypeRaw) { 26 { "DDR4" } 34 { "DDR5" } 35 { "LPDDR5" } default { "DDR4" } }

    $drives = Get-CimInstance -ClassName Win32_DiskDrive | Where-Object { $_.MediaType -match 'Fixed' }
    $totalGB = ($drives | Measure-Object -Property Size -Sum).Sum / 1000000000
    $roundedGB = [math]::Round($totalGB / 128) * 128
    if ($roundedGB -eq 0) { $roundedGB = [math]::Round($totalGB) }
    $storage = if ($roundedGB -ge 1024) { "$([math]::Round($roundedGB / 1024, 0))TB SSD" } else { "$($roundedGB)GB SSD" }

    $fullModel = if ($brand -and $model -and $model -ne "PC Desktop" -and $model -ne "System Product Name") { 
        if ($model.StartsWith($brand)) { $model } else { "$brand $model" }
    } else { 
        $brand 
    }

    $obj = [PSCustomObject]@{
        brand = $fullModel.Trim()
        model = "PC Desktop" # Set as generic to avoid duplication in Header.vue
        processor = $procName
        cores = [int]$proc.NumberOfCores
        threads = [int]$proc.NumberOfLogicalProcessors
        gen = $gen
        vendor = if ($procName -match "Intel") { "Intel" } elseif ($procName -match "AMD") { "AMD" } else { "Generic" }
        ram = "$ramSize GB"
        ramType = $ramType
        gpu = $gpu
        storage = $storage
        display = $display
        os = $osName
    }

    Write-Output "---JSON_START---"
    $obj | ConvertTo-Json -Compress
    Write-Output "---JSON_END---"

} catch {
    Write-Output '---JSON_START---{"brand":"PC Generico","processor":"Procesador","ram":"8 GB","storage":"256GB SSD"}---JSON_END---'
}
