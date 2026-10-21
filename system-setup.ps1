# System Setup for Zenit Showroom (Modo Kiosco / Exhibición)
# Configura energía, suspensión, batería y brillo máximo

Write-Host "==========================================================" -ForegroundColor Cyan
Write-Host " [Zenit Setup] Configurando energía y pantalla de la PC..." -ForegroundColor Cyan
Write-Host "==========================================================" -ForegroundColor Cyan

# 1. Desactivar suspensión, hibernación y apagado de pantalla (tanto en Corriente AC como en Batería DC)
Write-Host "[1/4] Desactivando suspensión e hibernación (AC y Batería)..." -ForegroundColor Yellow
powercfg /x -hibernate-timeout-ac 0
powercfg /x -hibernate-timeout-dc 0
powercfg /x -standby-timeout-ac 0
powercfg /x -standby-timeout-dc 0
powercfg /x -monitor-timeout-ac 0
powercfg /x -monitor-timeout-dc 0
powercfg /hibernate off

# 2. Desactivar brillo adaptativo y atenuación de video
Write-Host "[2/4] Desactivando brillo adaptativo y atenuación automática..." -ForegroundColor Yellow
powercfg /setacvalueindex SCHEME_CURRENT SUB_VIDEO ADAPTBRIGHT 0 2>$null
powercfg /setdcvalueindex SCHEME_CURRENT SUB_VIDEO ADAPTBRIGHT 0 2>$null
powercfg /setacvalueindex SCHEME_CURRENT SUB_VIDEO VIDEODIM 0 2>$null
powercfg /setdcvalueindex SCHEME_CURRENT SUB_VIDEO VIDEODIM 0 2>$null
powercfg /setactive SCHEME_CURRENT 2>$null

# 3. Establecer plan de Alto Rendimiento si está disponible
Write-Host "[3/4] Aplicando plan de energía de máximo rendimiento..." -ForegroundColor Yellow
try {
    $highPerf = powercfg /l | Select-String "Alto rendimiento|High performance|Ultimate Performance"
    if ($highPerf) {
        $guid = ($highPerf.ToString().Trim() -split '\s+')[3]
        if ($guid -match '^[0-9a-fA-F-]+$') {
            powercfg /s $guid
        }
    }
} catch {
    Write-Warning "No se pudo cambiar el plan de energía: $_"
}

# 4. Establecer brillo de pantalla al 100% (Laptop / Pantallas compatibles WMI)
Write-Host "[4/4] Ajustando brillo de pantalla al 100%..." -ForegroundColor Yellow
try {
    $monitor = Get-CimInstance -Namespace root/WMI -ClassName WmiMonitorBrightnessMethods -ErrorAction SilentlyContinue
    if ($monitor) {
        $monitor | Invoke-CimMethod -MethodName WmiSetBrightness -Arguments @{ Timeout = 1; Brightness = 100 } | Out-Null
        Write-Host " -> Brillo configurado al 100% mediante CIM." -ForegroundColor Green
    } else {
        $wmi = Get-WmiObject -Namespace root/WMI -Class WmiMonitorBrightnessMethods -ErrorAction SilentlyContinue
        if ($wmi) {
            $wmi.WmiSetBrightness(1, 100) | Out-Null
            Write-Host " -> Brillo configurado al 100% mediante WMI." -ForegroundColor Green
        } else {
            Write-Host " -> Pantalla no compatible con WMI (monitor externo de escritorio o sin sensor de brillo)." -ForegroundColor Gray
        }
    }
} catch {
    Write-Warning "Fallo al ajustar brillo por WMI: $_"
}

Write-Host "`n[OK] Configuración del sistema de exhibición completada." -ForegroundColor Green
