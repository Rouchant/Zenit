$shortcutPath = "$env:APPDATA\Microsoft\Windows\Start Menu\Programs\Startup\Zenit.lnk"
$targetPath = "$PSScriptRoot\zenit-win32-x64\zenit.exe" # assuming default build path

if (-not (Test-Path $targetPath)) {
    Write-Host "No se encontró el ejecutable en: $targetPath" -ForegroundColor Yellow
    Write-Host "Asegúrate de haber corrido 'npm run build' primero."
    exit
}

$WshShell = New-Object -ComObject WScript.Shell
$Shortcut = $WshShell.CreateShortcut($shortcutPath)
$Shortcut.TargetPath = $targetPath
$Shortcut.WorkingDirectory = "$PSScriptRoot\zenit-win32-x64"
$Shortcut.Save()

Write-Host "Acceso directo creado en la carpeta de Inicio: $shortcutPath" -ForegroundColor Green
