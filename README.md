# 💻 Zenit - Showcase App

![Version](https://img.shields.io/badge/version-1.0.0-blue.svg)
![Electron](https://img.shields.io/badge/platform-Electron-47848F.svg)
![Windows](https://img.shields.io/badge/OS-Windows%2010%2F11-0078D4.svg)

Una solución premium de **Showcase Terminal** diseñada específicamente para equipos de exhibición en puntos de venta (Retail). Esta aplicación transforma cualquier laptop en una vitrina digital interactiva y técnica.

---

## ✨ Características Principales

### 🔍 Detección Inteligente de Hardware
Utiliza scripts de **PowerShell** optimizados para detectar automáticamente:
- **Marca y Modelo** exacto del equipo.
- **CPU**: Arquitectura y núcleos.
- **RAM**: Capacidad y velocidad.
- **GPU**: Especificaciones de video dedicadas e integradas.
- **Almacenamiento**: Tipo (SSD/HDD) y capacidad total.

### 🎭 Interfaz Adaptativa y Premium
- **Diseño Glassmorphism**: Estética moderna inspirada en Windows 11.
- micro-animaciones**: Transiciones fluidas entre el modo de información y el modo video.
- dark Mode Optimizado**: Contraste elegante que resalta los colores del hardware.

### 🎥 Gestión de Contenido Automática
- **Loop de Video Inactivo**: La aplicación detecta la inactividad y activa automáticamente videos promocionales.
- **Transición Fluida**: Regresa instantáneamente a la ficha técnica cuando se detecta actividad del usuario.

### ⚙️ Control de Sistema (Always Awake)
- **Prevención de Suspensión**: Configura automáticamente el sistema para que no entre en modo de espera mientras la aplicación esté activa.
- **Autostart**: Incluye scripts para que el equipo inicie directamente en modo exhibición al encenderse.

---

## 🛠️ Requisitos del Sistema

- **S.O.**: Windows 10 o Windows 11 (Recomendado).
- **Entorno**: [Node.js](https://nodejs.org/) (solo para desarrollo/construcción).
- **PowerShell**: 5.1 o superior (incluido en Windows).

---

## 🚀 Instalación y Desarrollo

### 1. Clonar e Instalar Dependencias
```powershell
# Instalar dependencias necesarias
npm install
```

### 2. Configuración de Contenido
Coloca tus videos promocionales en la carpeta `assets/videos/`. El sistema buscará archivos `.mp4` para reproducir durante el tiempo de espera.

### 3. Ejecutar en Modo Desarrollo
```powershell
npm start
```

---

## 📦 Compilación y Distribución

El proyecto está configurado para generar instaladores profesionales y versiones portables.

### Generar Instalador (.exe)
Este proceso crea un instalador `NSIS` en la carpeta `dist/installer/`.
```powershell
npm run dist
```

### Versión Portable (Packaged)
Para una copia rápida sin instalación:
```powershell
npm run build
```

---

## 📁 Estructura del Proyecto

- `main.js`: Corazón de la aplicación Electron, maneja la ventana y permisos de sistema.
- `app.js`: Lógica del frontend, gestión de cronómetros y renderizado dinámico.
- `index.html`: Estructura semántica de la interfaz.
- `css/`: Directorio de estilos (componentes, modales, animaciones).
- `get-specs.ps1`: Script PowerShell encargado de la telemetría de hardware.
- `setup-autostart.ps1`: Utilidad de configuración para inicio automático.

---

## 🧪 Pruebas y Validación

La aplicación ha sido probada para:
1. **Detección correcta** en marcas principales (HP, Dell, ASUS, Lenovo, Apple/Bootcamp).
2. **Estabilidad 24/7** para entornos de tienda.
3. **Bloqueo de suspensión** exitoso en múltiples perfiles de energía.

---

> **Nota**: Para editar manualmente la información de un equipo detectado, haz clic en el icono de engranaje ⚙️ ubicado en la esquina superior derecha.

---
Desarrollado con ❤️ por el equipo de **Zenit**.
