# 💻 Zenit - Retail Showroom Kiosk (100% Rust Native)

![Language](https://img.shields.io/badge/core-Rust%20Native%202024-orange.svg)
![UI](https://img.shields.io/badge/ui-Slint%20UI%201.18-blueviolet.svg)
![Video](https://img.shields.io/badge/video-libmpv%20GPU%20Decoded-red.svg)
![Architecture](https://img.shields.io/badge/design-Screaming%20Architecture-brightgreen.svg)
![Scale](https://img.shields.io/badge/display-DPI%20Scale%20Immune-cyan.svg)

Zenit es una solución de **Showroom Kiosk Terminal** de ultra alto rendimiento, compilada **100% de forma nativa en Rust** con **Slint UI** y decodificación de video por aceleración GPU mediante **`libmpv`**.

Diseñada para laptops en exhibición en puntos de venta retail (Falabella, Paris, Ripley, ASUS Stores, etc.), sin necesidad de Node.js, npm, WebView2 ni navegadores web en tiempo de ejecución.

---

## 🏛️ Screaming Architecture (Estructura Orientada al Dominio)

La estructura del proyecto **grita** su propósito comercial antes que las herramientas con las que fue construido:

```text
Zenit/
├── Cargo.toml               # Manifiesto puro de Rust (Slint, libloading, chrono, windows-sys)
├── build.rs                 # Compilación Slint nativa (slint_build::compile)
├── libmpv-2.dll             # Biblioteca nativa de decodificación de video por GPU
├── assets/                  # Videos MP4 de exhibición, logos vectoriales e iconos
│   ├── videos/              # landing.mp4, promo.mp4, bg.mp4
│   └── logos/               # Logos oficiales de fabricantes y retail
├── icons/                   # Iconos nativos (.ico, .png)
├── ui/
│   └── app.slint            # Definición de interfaz declarativa Slint moderna (3 columnas)
├── src/
│   ├── main.rs              # Punto de entrada binario nativo (zenit_showroom::run)
│   ├── lib.rs               # Bucle de eventos, enlace de datos Slint y watcher de inactividad
│   └── domains/             # DOMINIOS DEL NEGOCIO (Screaming Architecture)
│       ├── hardware/        # Dominio de Inspección de Hardware
│       │   └── mod.rs       # Win32/WMI directo (CPU, GPU, RAM, SSD, Batería, Hz panel)
│       │
│       ├── showcase/        # Dominio de Vitrina y Presentación Comercial
│       │   ├── state.rs     # Agregador de estado de UI y normalización de specs
│       │   ├── config.rs    # Persistencia en %APPDATA%/Zenit/config.json y credenciales
│       │   └── mod.rs       # Exportación pública del dominio vitrina
│       │
│       ├── kiosk/           # Dominio de Kiosco y Protección de Vitrina
│       │   ├── inactivity.rs# Watcher nativo Win32 (GetLastInputInfo) con cero overhead
│       │   └── mod.rs       # Salvapantallas, Modo Prueba PC y auto-restauración (120s)
│       │
│       └── media/           # Dominio de Contenido Multimedia
│           ├── mpv.rs       # Controlador libmpv + Slint SharedPixelBuffer GPU rendering
│           ├── vault.rs     # Baúl Seguro de Videos (%APPDATA%/Zenit/vault)
│           └── mod.rs       # Aislamiento contra borrados accidentales de usuarios
└── scripts/
    └── download-mpv.ps1     # Script de utilidad para libmpv
```

---

## 🛡️ Capacidades del Sistema

### 1. 100% Rust Native & Slint UI

- Cero dependencias de navegadores web, Chromium ni WebView2.
- Arranque instantáneo (menos de 500 ms) y consumo de memoria inferior a 40 MB.

### 2. Decodificación de Video por GPU (`libmpv`)

- Decodificación por hardware (NVDEC, QuickSync, D3D11VA, AMF) sin saturar el CPU.
- Renderizado directo en fotogramas de Slint (`SharedPixelBuffer<Rgba8Pixel>`) con bucle infinito suave.
- Fallback visual automático elegante si la DLL de mpv aún no ha sido descargada.

### 3. Asistente de Configuración Rápida (Primer Inicio)

- Detección automática en el primer arranque (`setup_completed: false`).
- Permite configurar en menos de 1 minuto: marca y modelo auto-detectados, precios con tarjeta y efectivo, SKU, video central y tienda retail.

### 4. Modo "Prueba esta PC" y Retorno Automático

- Al hacer clic en "PROBAR ESTA PC", Zenit minimiza la ventana para permitir explorar el equipo libremente.
- Si el usuario abandona el equipo por más de 120 segundos sin actividad, Zenit se restaura automáticamente.

### 5. Atajos de Emergencia y Mantenimiento

- `4 toques en esquina superior derecha`: Acceso al Panel de Administración protegido por contraseña (`demo` / `z3n1t`).
- Opciones de salida inmediata a Windows y reinicio de la aplicación desde el panel de control.

---

## 🚀 Comandos Rápidos

```powershell
# Ejecutar la aplicación en modo desarrollo
cargo run

# Ejecutar las pruebas unitarias de todos los dominios (15 tests)
cargo test --lib

# Compilar ejecutable binario de producción optimizado
cargo build --release
```
