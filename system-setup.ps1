# System Setup for Zenit Showroom (Modo Kiosco / Exhibición)
# Configura energía, suspensión a NUNCA en TODOS los planes, batería y brillo máximo

Write-Host "==========================================================" -ForegroundColor Cyan
Write-Host " [Zenit Setup] Configurando energía y pantalla de la PC..." -ForegroundColor Cyan
Write-Host "==========================================================" -ForegroundColor Cyan

# 1. Configurar suspensión, hibernación y apagado de pantalla a NUNCA en TODOS los planes de energía existentes
Write-Host "[1/4] Configurando suspensión a NUNCA en TODOS los planes de energía..." -ForegroundColor Yellow

try {
    $planLines = powercfg /list | Select-String -Pattern '([0-9a-fA-F]{8}-[0-9a-fA-F]{4}-[0-9a-fA-F]{4}-[0-9a-fA-F]{4}-[0-9a-fA-F]{12})'
    foreach ($line in $planLines) {
        $guid = $line.Matches[0].Groups[1].Value
        if ($guid) {
            Write-Host " -> Aplicando 'Nunca suspender' al plan: $guid" -ForegroundColor Gray
            
            # Suspensión en corriente (AC) y batería (DC) = 0 (Nunca)
            powercfg /setacvalueindex $guid SUB_SLEEP STANDBYIDLE 0 2>$null
            powercfg /setdcvalueindex $guid SUB_SLEEP STANDBYIDLE 0 2>$null
            
            # Hibernación en corriente (AC) y batería (DC) = 0 (Nunca)
            powercfg /setacvalueindex $guid SUB_SLEEP HIBERNATEIDLE 0 2>$null
            powercfg /setdcvalueindex $guid SUB_SLEEP HIBERNATEIDLE 0 2>$null
            
            # Apagado de pantalla en corriente (AC) y batería (DC) = 0 (Nunca)
            powercfg /setacvalueindex $guid SUB_VIDEO VIDEOIDLE 0 2>$null
            powercfg /setdcvalueindex $guid SUB_VIDEO VIDEOIDLE 0 2>$null
            
            # Desactivar atenuación automática de pantalla
            powercfg /setacvalueindex $guid SUB_VIDEO VIDEODIM 0 2>$null
            powercfg /setdcvalueindex $guid SUB_VIDEO VIDEODIM 0 2>$null
            
            # Desactivar brillo adaptativo
            powercfg /setacvalueindex $guid SUB_VIDEO ADAPTBRIGHT 0 2>$null
            powercfg /setdcvalueindex $guid SUB_VIDEO ADAPTBRIGHT 0 2>$null
        }
    }
} catch {
    Write-Warning "Error al iterar planes de energía: $_"
}

# Aplicar también directamente al plan actualmente activo
powercfg /change standby-timeout-ac 0 2>$null
powercfg /change standby-timeout-dc 0 2>$null
powercfg /change monitor-timeout-ac 0 2>$null
powercfg /change monitor-timeout-dc 0 2>$null
powercfg /change hibernate-timeout-ac 0 2>$null
powercfg /change hibernate-timeout-dc 0 2>$null

# Desactivar hibernación a nivel global del sistema
powercfg /hibernate off 2>$null

# 2. Establecer plan de Alto Rendimiento si está disponible
Write-Host "[2/4] Seleccionando plan de energía de máximo rendimiento..." -ForegroundColor Yellow
try {
    $highPerf = powercfg /l | Select-String "Máximo rendimiento|Alto rendimiento|High performance|Ultimate Performance"
    if ($highPerf) {
        $firstMatch = $highPerf | Select-Object -First 1
        $guidMatch = [regex]::Match($firstMatch.ToString(), '([0-9a-fA-F]{8}-[0-9a-fA-F]{4}-[0-9a-fA-F]{4}-[0-9a-fA-F]{4}-[0-9a-fA-F]{12})')
        if ($guidMatch.Success) {
            $prefGuid = $guidMatch.Groups[1].Value
            powercfg /s $prefGuid 2>$null
            Write-Host " -> Plan activo configurado a: $prefGuid" -ForegroundColor Green
        }
    }
} catch {
    Write-Warning "No se pudo cambiar el plan de energía: $_"
}

# Actualizar el plan activo
powercfg /setactive SCHEME_CURRENT 2>$null

# 3. Establecer brillo de pantalla al 100% (Laptop / Pantallas compatibles WMI)
Write-Host "[3/4] Ajustando brillo de pantalla al 100%..." -ForegroundColor Yellow
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
            Write-Host " -> Pantalla no compatible con WMI (monitor externo o sin sensor de brillo)." -ForegroundColor Gray
        }
    }
} catch {
    Write-Warning "Fallo al ajustar brillo por WMI: $_"
}

Write-Host "`n[OK] Configuración del sistema de exhibición completada con éxito." -ForegroundColor Green
