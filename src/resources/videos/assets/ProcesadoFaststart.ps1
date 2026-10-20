# ====================================================
#     OPTIMIZADOR DE VIDEOS PARA KIOSCO (ZENIT)
# ====================================================
Clear-Host
Write-Host "====================================================" -ForegroundColor Yellow
Write-Host "   OPTIMIZADOR DE VIDEOS PARA KIOSCO (ZENIT 24/7)" -ForegroundColor Yellow
Write-Host "====================================================" -ForegroundColor Yellow
Write-Host ""

# 1. Intentar detectar FFmpeg en el PATH o usar la ruta fija
$FFMPEG_PATH = "C:\Users\jmema\AppData\Local\Microsoft\WinGet\Links\ffmpeg.exe"

if (-not (Test-Path -Path $FFMPEG_PATH)) {
    if (Get-Command "ffmpeg" -ErrorAction SilentlyContinue) {
        $FFMPEG_PATH = "ffmpeg"
    } else {
        Write-Host "[-] ERROR: No se encontró FFmpeg en $FFMPEG_PATH ni en el PATH del sistema." -ForegroundColor Red
        Read-Host "Presione Enter para salir..."
        exit
    }
}

$OUTPUT_FOLDER = ".\videosprocesados"
$videos = Get-ChildItem -File | Where-Object { $_.Extension -in '.mp4', '.mov', '.avi', '.mkv' }

if ($videos.Count -eq 0) {
    Write-Host "[-] No se encontraron archivos de video (.mp4, .mov, .avi, .mkv) en este directorio." -ForegroundColor Red
} else {
    Write-Host "[+] Encontrados $($videos.Count) video(s). Iniciando optimización...`n" -ForegroundColor Cyan

    if (-not (Test-Path -Path $OUTPUT_FOLDER)) {
        New-Item -ItemType Directory -Path $OUTPUT_FOLDER | Out-Null
        Write-Host "[+] Carpeta '$OUTPUT_FOLDER' creada con éxito.`n" -ForegroundColor Green
    }

    foreach ($video in $videos) {
        $fileName = $video.Name
        # Forzamos extensión .mp4 en la salida para mayor compatibilidad
        $outputFileName = [System.IO.Path]::ChangeExtension($fileName, ".mp4")
        $outputPath = Join-Path -Path $OUTPUT_FOLDER -ChildPath $outputFileName

        Write-Host "[Procesando] `"$fileName`" ..." -ForegroundColor White

        # Parametros FFmpeg diseñados para máxima compatibilidad 24/7 en gama baja:
        # -vf "scale='min(1920,iw)':-2,fps=30": Escala máximo a 1080p manteniendo aspecto y limita a 30fps
        # -vcodec libx264 -preset slow -crf 23: Balance entre calidad visual y bajo tamaño
        # -profile:v main -level 4.0: Asegura decodificación ligera por hardware en iGPUs viejas
        # -pix_fmt yuv420p: Formato de píxeles universal para web/reproductores
        # -an: Elimina la pista de audio
        # -movflags +faststart: Mueve la metadata al inicio del archivo
        
        $ffmpegArgs = @(
            "-y",
            "-i", "`"$($video.FullName)`"",
            "-an",
            "-vcodec", "libx264",
            "-profile:v", "main",
            "-level", "4.0",
            "-pix_fmt", "yuv420p",
            "-preset", "slow",
            "-crf", "23",
            "-vf", "scale='min(1920,iw)':-2,fps=30",
            "-movflags", "+faststart",
            "`"$outputPath`""
        ) -join " "

        Start-Process -FilePath $FFMPEG_PATH -ArgumentList $ffmpegArgs -NoNewWindow -Wait

        Write-Host "[ OK ] `"$outputFileName`" listo en '$OUTPUT_FOLDER'." -ForegroundColor Green
        Write-Host "----------------------------------------------------" -ForegroundColor DarkGray
    }

    Write-Host "`n====================================================" -ForegroundColor Green
    Write-Host "      ¡TODOS LOS VIDEOS FUERON OPTIMIZADOS!" -ForegroundColor Green
    Write-Host "====================================================" -ForegroundColor Green
    Write-Host "Ubicación: $OUTPUT_FOLDER`n"
}

Read-Host "Presione Enter para salir..."