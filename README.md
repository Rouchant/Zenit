# 💻 Zenit - Retail Showroom Kiosk Architecture

![Platform](https://img.shields.io/badge/platform-Tauri%20v2-blue.svg)
![Language](https://img.shields.io/badge/core-Rust%20Native-orange.svg)
![Architecture](https://img.shields.io/badge/design-Screaming%20Architecture-brightgreen.svg)
![UI](https://img.shields.io/badge/ui-Glassmorphism%20%26%20DPI%20Immune-cyan.svg)

Zenit es una solución de **Showroom Kiosk Terminal** de alto rendimiento diseñada específicamente para laptops en exhibición en puntos de venta retail (Falabella, Paris, Ripley, ASUS Stores, etc.).

---

## 🏛️ Screaming Architecture (Estructura Orientada al Dominio)

La arquitectura del proyecto **grita** su propósito comercial antes que las herramientas con las que fue construido:

```
src-tauri/src/
├── main.rs                   # Entry point con inmunidad a escala y banderas Chromium
├── lib.rs                    # Orquestador del sistema, IPC handlers y ciclo de vida Tauri
└── domains/                  # DOMINIOS DEL NEGOCIO (Screaming Architecture)
    ├── hardware/             # Dominio de Inspección de Hardware
    │   └── mod.rs            # Win32/WMI directo (CPU, GPU, RAM, SSD, Batería, Hz panel)
    │
    ├── showcase/             # Dominio de Vitrina y Presentación Comercial
    │   ├── state.rs          # Agregador de estado de UI y normalización de specs
    │   ├── config.rs         # Persistencia en %APPDATA%/Zenit/config.json y credenciales
    │   └── mod.rs            # Exportación pública del dominio vitrina
    │
    ├── kiosk/                # Dominio de Kiosco y Protección de Vitrina
    │   ├── inactivity.rs     # Watcher nativo Win32 (GetLastInputInfo) con cero overhead JS
    │   └── mod.rs            # Salvapantallas, Modo Prueba PC y auto-restauración (120s)
    │
    └── media/                # Dominio de Contenido Multimedia
        ├── vault.rs          # Baúl Seguro de Videos (%APPDATA%/Zenit/vault)
        └── mod.rs            # Aislamiento contra borrados accidentales de usuarios
```

---

## 🖥️ Capa de Presentación ("Front Tonto, Rust Inteligente")

El frontend reside en `public/` y es consumido directamente por Tauri v2 sin servidores Node ni bundlers pesados en tiempo de ejecución:

```
public/
├── index.html                # Cuadrícula de 3 columnas (Specs + Hero Video + Precios/CTA)
├── return.html               # Botón flotante siempre visible en "Modo Prueba de PC"
├── css/
│   └── glassmorphism.css     # Design System Glassmorphism con ambient glow e inmunidad DPI
└── js/
    └── app.js                # Cliente ligero: renderizado de estado y conexión IPC
```

---

## 🛡️ Capacidades del Sistema

### 1. Inmunidad a Escala de Windows (DPI Scale Immunity)
- Neutralización automática de escalas 125%, 150%, 175% y 200%.
- Bloqueo de zoom por rueda (`Ctrl + Wheel`), gestos multitáctiles (pellizco) y atajos de teclado (`Ctrl +/-/0`).

### 2. Asistente de Configuración Rápida (Primer Inicio)
- Detección automática en el primer arranque (`setup_completed: false`).
- Permite configurar en menos de 1 minuto: marca y modelo auto-detectados, precios con tarjeta y efectivo, SKU, video central y tienda retail.

### 3. Modo "Prueba esta PC" y Retorno Automático
- Al hacer clic en "Prueba esta PC", Zenit se minimiza y deja una pequeña barra flotante translúcida en el borde derecho: *"Ver detalles / Volver a Vitrina"*.
- Si el usuario abandona el equipo por más de 120 segundos sin actividad, Zenit se restaura automáticamente a pantalla completa.

### 4. Baúl Seguro de Videos (Video Vault)
- Los videos importados se copian y protegen en `%APPDATA%\Zenit\vault\` aislándolos de la carpeta de descargas o temporales.

### 5. Atajos de Emergencia
- `Ctrl + Shift + Alt + Z`: Cierre forzado de emergencia.
- `Ctrl + Shift + Alt + R`: Reinicio automático de la aplicación.
- `4 toques en esquina superior derecha`: Acceso al Panel de Administración protegido por contraseña (`demo` / `z3n1t`).

---

## 🚀 Comandos Rápidos

```bash
# Desarrollo con recarga automática
npm run dev
# o directamente con Tauri
cargo tauri dev

# Compilar ejecutable binario de producción
cargo tauri build
```
