# PowerShell Script to download and install official pre-built libmpv for Windows
# Usage: .\scripts\download-mpv.ps1

Write-Host "=========================================" -ForegroundColor Cyan
Write-Host "  Zenit Showroom - Descargador de libmpv" -ForegroundColor Cyan
Write-Host "=========================================" -ForegroundColor Cyan

$destDir = "$PSScriptRoot\.."
$targetDebugDir = "$PSScriptRoot\..\target\debug"

if (-not (Test-Path $destDir)) {
    New-Item -ItemType Directory -Path $destDir -Force | Out-Null
}
if (-not (Test-Path $targetDebugDir)) {
    New-Item -ItemType Directory -Path $targetDebugDir -Force | Out-Null
}

Write-Host "Descargando la última versión de libmpv-2.dll para Windows x64..." -ForegroundColor Yellow

$releaseApiUrl = "https://api.github.com/repos/shinchiro/mpv-winbuild-cmake/releases/latest"
try {
    $releaseJson = Invoke-RestMethod -Uri $releaseApiUrl -Headers @{ "User-Agent" = "PowerShell" }
    $asset = $releaseJson.assets | Where-Object { $_.name -like "mpv-dev-x86_64-*.7z" -or $_.name -like "mpv-dev-x86_64-*.zip" } | Select-Object -First 1

    if ($asset) {
        $downloadUrl = $asset.browser_download_url
        $tempFile = "$env:TEMP\" + $asset.name
        Write-Host "Descargando desde: $downloadUrl" -ForegroundColor Green
        Invoke-WebRequest -Uri $downloadUrl -OutFile $tempFile

        Write-Host "Extrayendo archivo en $destDir..." -ForegroundColor Green
        if ($asset.name.EndsWith(".zip")) {
            Expand-Archive -Path $tempFile -DestinationPath $destDir -Force
        } else {
            # Use 7z or tar if available
            & tar -xf $tempFile -C $destDir
        }

        # Copy mpv-2.dll to target\debug as well
        if (Test-Path "$destDir\mpv-2.dll") {
            Copy-Item "$destDir\mpv-2.dll" "$targetDebugDir\mpv-2.dll" -Force
            Write-Host "¡mpv-2.dll instalado exitosamente en $destDir y $targetDebugDir!" -ForegroundColor Green
        }
    } else {
        Write-Host "No se encontró el asset mpv-dev. Descárgalo manualmente desde:" -ForegroundColor Yellow
        Write-Host "https://sourceforge.net/projects/mpv-player-windows/files/libmpv/" -ForegroundColor White
    }
} catch {
    Write-Host "Nota: Puedes descargar manualmente 'mpv-dev-x86_64.7z' o 'mpv-2.dll' desde:" -ForegroundColor Yellow
    Write-Host "https://sourceforge.net/projects/mpv-player-windows/files/libmpv/" -ForegroundColor Cyan
    Write-Host "Y colocar 'mpv-2.dll' en la carpeta 'src-tauri\' o 'target\debug\'." -ForegroundColor White
}
