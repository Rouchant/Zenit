$shortcutPath = "$env:APPDATA\Microsoft\Windows\Start Menu\Programs\Startup\ZenitShowroom.lnk"
$targetPath = "$PSScriptRoot\target\release\zenit-showroom.exe"

if (-not (Test-Path $targetPath)) {
    $targetPath = "$PSScriptRoot\target\debug\zenit-showroom.exe"
}

if (-not (Test-Path $targetPath)) {
    Write-Host "No se encontró el ejecutable en: $targetPath" -ForegroundColor Yellow
    Write-Host "Asegúrate de haber compilado primero con: cargo build --release" -ForegroundColor White
    exit
}

$WshShell = New-Object -ComObject WScript.Shell
$Shortcut = $WshShell.CreateShortcut($shortcutPath)
$Shortcut.TargetPath = $targetPath
$Shortcut.WorkingDirectory = "$PSScriptRoot"
$Shortcut.Save()

Write-Host "Acceso directo creado en la carpeta de Inicio: $shortcutPath" -ForegroundColor Green
