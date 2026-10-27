# 💻 Zenit - Retail Showroom Kiosk (100% Rust Native)

![Language](https://img.shields.io/badge/core-Rust%20Native%202024-orange.svg)
![UI](https://img.shields.io/badge/ui-Slint%20UI%201.18-blueviolet.svg)
![Video](https://img.shields.io/badge/video-libmpv%20GPU%20Decoded-red.svg)
![Architecture](https://img.shields.io/badge/design-Screaming%20Architecture-brightgreen.svg)
![OLED Care](https://img.shields.io/badge/OLED-Anti%20Burn--In%20Protected-brightgreen.svg)
![Scale](https://img.shields.io/badge/display-DPI%20Scale%20Immune-cyan.svg)

Zenit es una solución de **Showroom Kiosk Terminal** de ultra alto rendimiento, compilada **100% de forma nativa en Rust** con **Slint UI** y decodificación de video por aceleración GPU mediante **`libmpv`**.

Diseñada para laptops y estaciones en exhibición en puntos de venta retail (Falabella, Paris, Ripley, ASUS Stores, etc.), sin necesidad de Node.js, npm, WebView2 ni navegadores web en tiempo de ejecución.

---

## 🛡️ Capacidades Principales del Sistema

### 1. 100% Rust Native & Slint UI
- Cero dependencias de navegadores web, Chromium ni WebView2.
- Arranque instantáneo (menos de 500 ms) y huella de memoria ultrabaja (< 45 MB).
- Inmune a la escala DPI de Windows (Full HD, 2.5K, 2.8K OLED, 4K).

### 2. Protección Integral para Pantallas OLED (Anti Burn-In)
- **Pixel Shifting Orbital:** Temporizador orbital suave de 60s (±2px con transición de 1200ms) que rota continuamente los subpíxeles activos de tarjetas y textos sin distorsión visual perceptible.
- **Static Content Dimming:** Atenuación gradual del 20% tras 40s de inactividad en el showroom, retornando de inmediato al 100% al interactuar.
- **Control Lumínico Dinámico (Opción Híbrida):** Máximo brillo (100%) durante la reproducción activa de videos comerciales para captar clientes; preservación lumínica y reducción de calor en pantallas de espera estáticas.
- **Relación de Aspecto Adaptativa (16:9 / 16:10) & Pan-Scan:** Detección automática del panel nativo para eliminar barras negras fijas (letterboxing / pillarboxing), causantes del desgaste irregular en paneles OLED.

### 3. Decodificación de Video por GPU (`libmpv`)
- Decodificación por hardware (NVDEC, QuickSync, D3D11VA, AMF) sin saturar el CPU.
- Renderizado de fotogramas directos en memoria Slint (`SharedPixelBuffer<Rgba8Pixel>`) a 60 FPS con audio silenciado para retail.
- Playlist de inactividad configurable con retorno automático a la vitrina principal.

### 4. Modo "Probar esta PC" y Retorno Inteligente
- Al presionar **PROBAR ESTA PC**, Zenit minimiza el showroom y activa un botón flotante minimalista para que el cliente explore Windows con total libertad.
- Watcher nativo Win32 (`GetLastInputInfo`): Si el equipo queda inactivo por el tiempo configurado (ej: 120s), restaura la vitrina y activa el salvapantallas promocional automáticamente.

### 5. Asistente de Configuración Inicial & Panel de Administración
- Asistente de configuración rápida en el primer arranque para tiendas y promotores.
- **Panel de Administración** protegido por contraseña (`demo` / `z3n1t`), accesible mediante 4 toques en la esquina superior derecha:
  - Personalización de precios tarjeta/efectivo, SKU, logos y especificaciones técnicas.
  - Baúl de videos promocionales integrados.
  - Selector de colores con vista previa en tiempo real.
  - Atajos de emergencia: `Ctrl+Shift+Alt+Z` (cierre de emergencia) y `Ctrl+Shift+Alt+R` (reinicio rápido).

---

## 🏛️ Screaming Architecture (Estructura Orientada al Dominio)

```text
Zenit/
├── Cargo.toml               # Manifiesto puro de Rust (Slint, libloading, chrono, windows-sys)
├── build.rs                 # Compilación Slint nativa (slint_build::compile)
├── libmpv-2.dll             # Biblioteca nativa de decodificación de video por GPU
├── installer.iss            # Script de Inno Setup 7 (generador de Setup.exe)
├── package.ps1              # Script de compilación release, empaquetado portable e instalador
├── assets/                  # Videos MP4 de exhibición, logos vectoriales e iconos
│   ├── videos/              # landing-generic.mp4, promo-generic.mp4, etc.
│   └── logos/               # Logos oficiales de fabricantes y retail
├── icons/                   # Iconos nativos (.ico, .png)
├── ui/                      # Vistas y componentes Slint
│   ├── app.slint            # Ventana principal y gestor de vistas
│   ├── theme.slint          # Tokens de diseño y tipos de datos
│   ├── views/               # ShowroomView, ScreensaverView, AdminView, QuickSetupView
│   └── components/          # SpecCard, FloatingButton, ColorPicker, etc.
├── src/
│   ├── main.rs              # Punto de entrada binario nativo (zenit_showroom::run)
│   ├── lib.rs               # Bucle de eventos, enlace de datos Slint y watcher de inactividad
│   ├── app/                 # Capa de orquestación, timers, hotkeys, sync y handlers
│   └── domains/             # DOMINIOS DEL NEGOCIO (Screaming Architecture)
│       ├── hardware/        # Inspección Win32/WMI (CPU, GPU, RAM, SSD, Batería, Hz, 16:9/16:10)
│       ├── showcase/        # Vitrina comercial, estado de UI y persistencia (%APPDATA%/Zenit)
│       ├── kiosk/           # Vigilancia de inactividad Win32 (GetLastInputInfo) con cero overhead
│       └── media/           # Controlador libmpv + Slint GPU rendering y Baúl de Videos
```

---

## 🚀 Comandos Rápidos

```powershell
# Ejecutar la aplicación en modo desarrollo
cargo run

# Ejecutar la suite completa de pruebas unitarias (45 tests)
cargo test

# Compilar ejecutable binario de producción optimizado
cargo build --release
```

---

## 📦 Empaquetado y Distribución

Para generar la versión portable y el instalador automático de Windows:

```powershell
.\package.ps1
```

Este script automatiza:
1. Compilación ultra optimizada con `cargo build --release`.
2. Generación del paquete **Portable** en `dist\zenit-showroom-portable\` (listo para copiar y ejecutar en cualquier PC sin instalar).
3. Compilación con **Inno Setup** (compatible con v6 y v7) para generar el instalador final:  
   📁 `dist\ZenitShowroom-Setup-v1.0.0.exe` (incluye inicio automático con Windows, configuración de energía/brillo y desinstalador limpio).
