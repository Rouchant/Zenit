# 💻 Zenit - Showcase App (Kiosk Lockdown Edition)

![Version](https://img.shields.io/badge/version-1.1.0-blue.svg)
![Vue 3](https://img.shields.io/badge/framework-Vue%203-42b883.svg)
![Electron](https://img.shields.io/badge/platform-Electron-47848F.svg)
![Lockdown](https://img.shields.io/badge/security-Aggressive%20Kiosk-red.svg)

Una solución premium de **Showcase Terminal** diseñada para equipos de exhibición en puntos de venta (Retail). Esta versión incluye un motor de seguridad de bajo nivel para garantizar que la aplicación se mantenga siempre en primer plano, protegiendo la experiencia del cliente de interferencias del sistema operativo.

---

## 🛡️ Gestión de Kiosco "Ultra-Aggressive"

Esta versión introduce el sistema de seguridad **Zenit Lockdown 2.0**, diseñado para entornos de alta rotación donde el cliente puede intentar interactuar con Windows.

### 🔒 Control de Foco y Z-Order
- **Escape Hack**: Simulación inteligente de la tecla `Esc` mediante PowerShell para cerrar automáticamente el Menú Inicio y overlays de búsqueda antes de reclamar el foco.
- **Z-Order Bombardment**: Ciclos de refresco de profundidad (Z-index) cada 250ms durante las transiciones para ganar la prioridad visual frente a la barra de tareas.
- **Auto-Restauración**: Regreso forzado a pantalla completa tras 2 minutos de inactividad o pérdida accidental del foco.

### ⚡ Transiciones Sincronizadas
- **Async Focus Sync**: Coordinación mediante `async/await` entre el proceso de Electron y la interfaz de Vue. El video de promoción solo comienza cuando el sistema confirma que el Menú Inicio ha sido completamente descartado.
- **Zero-Flicker Transition**: Un búfer de sincronización elimina el parpadeo visual entre el sistema operativo y el reproductor de video.

### 🏗️ Estabilización de Instalación
- **Intelligent First-Run**: La aplicación detecta si es su primer arranque tras la instalación. Aplica un retraso de seguridad de **10 segundos** para permitir que el instalador (NSIS) cierre sus procesos antes de bloquear la pantalla.
- **Fast Reboot**: Los arranques posteriores son casi instantáneos (1s), optimizando la operación diaria.

---

## 🔍 Telemetría y Contenido

- **Detección PowerShell**: Diagnóstico de hardware en tiempo real compatible con Intel Core Ultra y AMD Ryzen series 8000.
- **Persistencia de Video**: Los videos seleccionados por el administrador se copian y protegen en `%APPDATA%/zenit/custom_videos/`.
- **Protocolo Zenit-File**: Carga ultra-rápida de archivos locales mediante el protocolo personalizado `zenit-file://`.

---

## 🚀 Comandos Rápidos

### Desarrollo
```powershell
npm install
npm run dev   # Terminal A
npm start     # Terminal B
```

### Distribución
```powershell
# Generar instalador profesional (.exe)
npm run dist
```

---

## 🛠️ Requisitos
- Windows 10/11
- Node.js v18+
- PowerShell 5.1+ con permisos de ejecución

---
Desarrollado con ❤️ para entornos retail de alto rendimiento.
