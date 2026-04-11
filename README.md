# 💻 Zenit - Showcase App (Vue 3 Edition)

![Version](https://img.shields.io/badge/version-1.0.0-blue.svg)
![Vue 3](https://img.shields.io/badge/framework-Vue%203-42b883.svg)
![Vite](https://img.shields.io/badge/build-Vite-646cff.svg)
![Electron](https://img.shields.io/badge/platform-Electron-47848F.svg)
![Windows](https://img.shields.io/badge/OS-Windows%2010%2F11-0078D4.svg)

Una solución premium de **Showcase Terminal** diseñada específicamente para equipos de exhibición en puntos de venta (Retail). Esta aplicación transforma cualquier laptop en una vitrina digital interactiva, técnica y visualmente impactante.

---

## ✨ Novedades en la Versión 2.0 (Vue 3)

Recientemente migrado a una arquitectura moderna basada en componentes, ofreciendo mayor estabilidad y rendimiento en equipos de entrada.

### 🔍 Telemetría Avanzada
- **Detección Dinámica**: Scripts de **PowerShell** optimizados para reconocer hardware de última generación (Intel Core 5/Ultra, AMD Ryzen 8000).
- **Resolución Real**: Detección de dimensiones físicas ignorando el escalado de Windows para una visualización perfecta.
- **Gestión de Estado**: Integración de **Pinia** para una reactividad instantánea en los specs.

### 🎥 Gestión de Contenido 2.0
- **Persistencia de Video**: Los videos personalizados se copian automáticamente a una carpeta de sistema protegida para evitar errores por borrado accidental.
- **Protocolo Zenit-File**: Protocolo de archivos personalizado (`zenit-file://`) que permite cargar contenido local pesado de forma segura y eficiente.
- **Optimización i3**: Sistema de pausa inteligente para videos de fondo al abrir ventanas de detalles, maximizando la fluidez en hardware de bajos recursos.

### 🎨 Diseño Premium
- **Vue-Driven Components**: Interfaz modular, limpia y fácil de mantener.
- **Aspect Ratio 16:9**: Contenedor de video diseñado para mantener proporciones cinematográficas en todo momento.
- **Transitions Pro**: Botón de retorno instantáneo precargado en segundo plano.

---

## 🛠️ Requisitos del Sistema

- **S.O.**: Windows 10 o Windows 11 (Recomendado).
- **Entorno**: [Node.js](https://nodejs.org/) v18 o superior.
- **PowerShell**: 5.1 o superior.

---

## 🚀 Instalación y Desarrollo

### 1. Clonar e Instalar Dependencias
```powershell
npm install
```

### 2. Ejecutar en Modo Desarrollo
Para trabajar con **Hot Module Replacement (HMR)** integral:

**Terminal A (Servidor de Estilos/Lógica):**
```powershell
npm run dev
```

**Terminal B (App de Electron):**
```powershell
npm start
```

### 3. Configuración de Contenido
Los recursos estáticos (videos originales, logos, fuentes) ahora residen en la carpeta `public/`. Los videos personalizados seleccionados por el administrador se guardarán en `%APPDATA%/zenit/custom_videos/`.

---

## 📁 Nueva Estructura del Proyecto

- **`src/`**: Código fuente de Vue (Main, Components, Stores, Styles).
- **`public/`**: Assets estáticos servidos directamente por Vite.
- **`main.js`**: Proceso principal de Electron (Gestiona ventanas, protocolos e IPC).
- **`preload.js`**: Puente de seguridad entre Electron y Vue.
- **`get-specs.ps1`**: El motor de telemetría de hardware en PowerShell.

---

## 📦 Compilación y Distribución

```powershell
# Para generar el bundle optimizado
npm run build

# Para generar instaladores profesionales (.exe)
npm run dist
```

---

> **Nota para Administradores**: Para editar la información o cambiar videos, usa el "Hotspot" secreto (esquina superior derecha) e ingresa la contraseña.

---
Desarrollado con ❤️ por el equipo de **Zenit**.
