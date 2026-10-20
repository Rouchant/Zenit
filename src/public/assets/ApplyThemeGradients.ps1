# ApplyThemeGradients.ps1
# Script de PowerShell para procesar los videos de fondo (background-asus.mp4 y background-generic.mp4)
# emulando exactamente el gradiente radial translúcido de CSS usando FFmpeg.

$ErrorActionPreference = "Stop"

# Encontrar FFmpeg
$ffmpeg = "ffmpeg"
if (!(Get-Command $ffmpeg -ErrorAction SilentlyContinue)) {
    $wingetFfmpeg = "$env:USERPROFILE\AppData\Local\Microsoft\WinGet\Links\ffmpeg.exe"
    if (Test-Path $wingetFfmpeg) {
        $ffmpeg = $wingetFfmpeg
    } else {
        Write-Error "No se encontró FFmpeg en el PATH ni en la ruta predeterminada de WinGet. Por favor instálalo o añádelo al PATH."
    }
}

# Crear carpeta de salida
$outputDir = ".\zenit_output"
if (!(Test-Path $outputDir)) {
    New-Item -ItemType Directory -Path $outputDir | Out-Null
}

# Obtener videos de fondo en la carpeta actual
$videos = Get-ChildItem -File | Where-Object { $_.Name -match '^background-(asus|generic)\.(mp4|mov|avi|mkv)$' }

if ($videos.Count -eq 0) {
    Write-Host "⚠️ No se encontraron videos 'background-asus' o 'background-generic' en el directorio actual." -ForegroundColor Yellow
    Exit
}

# Fórmulas con Alpha Constante (0.65), Alta Saturación y Esquinas 20% más Oscuras:
# - El centro focal mantiene los picos cromáticos máximos para no perder viveza.
# - Se incrementó el sustraendo dinámico (el valor que resta después del signo menos) 
#   para forzar a que los colores en los bordes exteriores caigan un 20% más hacia la sombra.
$themes = @{
    "default" = @{
        "a" = "0.65"
        "r" = "(0)"
        "g" = "(255 - 155 * min(hypot(X-W*0.5,Y-H*0.2)/(W*0.95),1))"
        "b" = "(235 - 145 * min(hypot(X-W*0.5,Y-H*0.2)/(W*0.95),1))"
    };
    "falabella" = @{
        "a" = "0.65"
        "r" = "(245 - 150 * min(hypot(X-W*0.5,Y-H*0.2)/(W*0.95),1))"
        "g" = "(255 - 155 * min(hypot(X-W*0.5,Y-H*0.2)/(W*0.95),1))"
        "b" = "(0)"
    };
    "paris" = @{
        "a" = "0.65"
        "r" = "(0)"
        "g" = "(240 - 155 * min(hypot(X-W*0.5,Y-H*0.2)/(W*0.95),1))"
        "b" = "(255 - 160 * min(hypot(X-W*0.5,Y-H*0.2)/(W*0.95),1))"
    };
    "ripley" = @{
        "a" = "0.65"
        "r" = "(230 - 150 * min(hypot(X-W*0.5,Y-H*0.2)/(W*0.95),1))"
        "g" = "(30 - 20 * min(hypot(X-W*0.5,Y-H*0.2)/(W*0.95),1))"
        "b" = "(255 - 165 * min(hypot(X-W*0.5,Y-H*0.2)/(W*0.95),1))"
    }
}

foreach ($video in $videos) {
    $name = $video.BaseName
    $ext = $video.Extension
    
    foreach ($themeName in $themes.Keys) {
        $theme = $themes[$themeName]
        $a = $theme["a"]
        $r = $theme["r"]
        $g = $theme["g"]
        $b = $theme["b"]
        
        $outputFile = Join-Path $outputDir "$($name)_$($themeName)$($ext)"
        Write-Host "🛍️ Procesando gradiente radial real ($themeName) para: $($video.Name)" -ForegroundColor Cyan
        
        # Tu filtro original modificado solo con la salida de color
        $filter = "format=rgb24,geq=" +
                  "r='r(X,Y)*(1-$a) + ($r)*$a':" +
                  "g='g(X,Y)*(1-$a) + ($g)*$a':" +
                  "b='b(X,Y)*(1-$a) + ($b)*$a'," +
                  "format=yuv420p"

        # Ejecutar FFmpeg sin escalar, con alta fidelidad (CRF 16), preset slow y Faststart
        & $ffmpeg -y -i $video.FullName -vf $filter -c:v libx264 -crf 12 -preset slow -tune grain -c:a copy -movflags +faststart $outputFile
    }
}

Write-Host "✅ ¡Todos los entornos de retail exportados manteniendo el video de fondo!" -ForegroundColor Green
