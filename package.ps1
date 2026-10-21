# Script de empaquetado para Zenit Showroom
$ErrorActionPreference = "Stop"

Write-Host "=========================================" -ForegroundColor Cyan
Write-Host " 1. Compilando en modo Release (Optimizada)" -ForegroundColor Cyan
Write-Host "=========================================" -ForegroundColor Cyan
cargo build --release

$releaseExe = "$PSScriptRoot\target\release\zenit-showroom.exe"
if (-not (Test-Path $releaseExe)) {
    Write-Host "Error: No se encontró el ejecutable release." -ForegroundColor Red
    exit 1
}

$distDir = "$PSScriptRoot\dist"
$portableDir = "$distDir\zenit-showroom-portable"

Write-Host "=========================================" -ForegroundColor Cyan
Write-Host " 2. Generando carpeta Portable en: dist\" -ForegroundColor Cyan
Write-Host "=========================================" -ForegroundColor Cyan
if (Test-Path $portableDir) {
    Remove-Item -Path $portableDir -Recurse -Force
}
New-Item -ItemType Directory -Path $portableDir -Force | Out-Null

Copy-Item -Path $releaseExe -Destination $portableDir
Copy-Item -Path "$PSScriptRoot\libmpv-2.dll" -Destination $portableDir
Copy-Item -Path "$PSScriptRoot\assets" -Destination $portableDir -Recurse
if (Test-Path "$PSScriptRoot\get-specs.ps1") {
    Copy-Item -Path "$PSScriptRoot\get-specs.ps1" -Destination $portableDir
}

Write-Host "[OK] Paquete Portable creado en: $portableDir" -ForegroundColor Green

# 3. Verificar si Inno Setup Compiler está instalado para compilar el Setup .exe
$isccPaths = @(
    "C:\Program Files (x86)\Inno Setup 6\ISCC.exe",
    "C:\Program Files\Inno Setup 6\ISCC.exe",
    (Get-Command iscc.exe -ErrorAction SilentlyContinue).Source
) | Where-Object { $_ -and (Test-Path $_) }

if ($isccPaths.Count -gt 0) {
    $iscc = $isccPaths[0]
    Write-Host "=========================================" -ForegroundColor Cyan
    Write-Host " 3. Compilando instalador Setup .exe con Inno Setup" -ForegroundColor Cyan
    Write-Host "=========================================" -ForegroundColor Cyan
    & $iscc "$PSScriptRoot\installer.iss"
    Write-Host "[OK] Instalador generado exitosamente en: $distDir" -ForegroundColor Green
} else {
    Write-Host "`nNota: Para generar el instalador automático Setup.exe, instala Inno Setup (gratuito) desde https://jrsoftware.org/isdl.php y vuelve a ejecutar este script." -ForegroundColor Yellow
}
