const { app, BrowserWindow, ipcMain, powerSaveBlocker, dialog, globalShortcut } = require('electron');
const path = require('path');
const isDev = require('electron-is-dev');
const fs = require('fs');
const { exec } = require('child_process');

// Performance Optimizations for Retail Hardware
app.commandLine.appendSwitch('enable-gpu-rasterization');
app.commandLine.appendSwitch('enable-zero-copy');
app.commandLine.appendSwitch('ignore-gpu-blocklist');
app.commandLine.appendSwitch('disable-software-rasterizer');
app.commandLine.appendSwitch('force-gpu-rasterization');

let mainWindow;
let returnWindow;
let psBlockerId;
let isQuitting = false; // Flag for authorized exit

// Single Instance Lock
const gotTheLock = app.requestSingleInstanceLock();

if (!gotTheLock) {
    app.quit();
} else {
    app.on('second-instance', (event, commandLine, workingDirectory) => {
        // Someone tried to run a second instance, we should focus our window.
        if (mainWindow) {
            if (mainWindow.isMinimized()) mainWindow.restore();
            mainWindow.focus();
        }
    });
}

function runSystemSetup() {
    const isDev = !app.isPackaged;
    const scriptPath = isDev 
        ? path.join(__dirname, 'system-setup.ps1') 
        : path.join(process.resourcesPath, 'system-setup.ps1');
        
    exec(`powershell.exe -ExecutionPolicy Bypass -File "${scriptPath}"`, (error, stdout, stderr) => {
        if (error) console.error('System setup error:', error);
        else console.log('System setup success:', stdout);
    });
}

function createWindow() {
    mainWindow = new BrowserWindow({
        width: 1920,
        height: 1080,
        kiosk: true,
        alwaysOnTop: true,
        autoHideMenuBar: true,
        icon: isDev ? path.join(__dirname, 'public', 'assets', 'logo.ico') : path.join(__dirname, 'dist', 'assets', 'logo.ico'),
        webPreferences: {
            nodeIntegration: false,
            contextIsolation: true,
            preload: path.join(__dirname, 'preload.js')
        }
    });

    if (isDev) {
        mainWindow.loadURL('http://localhost:5173');
    } else {
        mainWindow.loadFile(path.join(__dirname, 'dist', 'index.html'));
    }

    // Prevent sleep (redundant with powercfg but extra safety)
    psBlockerId = powerSaveBlocker.start('prevent-display-sleep');

    // Prevent Unauthorized Close (Alt+F4 etc)
    mainWindow.on('close', (e) => {
        if (!isQuitting) {
            e.preventDefault();
        }
    });

    mainWindow.on('closed', () => {
        if (returnWindow && !returnWindow.isDestroyed()) {
            returnWindow.close();
        }
        mainWindow = null;
    });
}

app.whenReady().then(() => {
    runSystemSetup();
    createWindow();
    createReturnWindow(); // Pre-create hidden

    // Register Lockdown Shortcuts
    globalShortcut.register('Alt+Tab', () => {
        console.log('Alt+Tab blocked');
    });
    globalShortcut.register('Alt+F4', () => {
        console.log('Alt+F4 blocked');
    });

    app.on('activate', () => {
        if (BrowserWindow.getAllWindows().length === 0) createWindow();
    });
});

app.on('will-quit', () => {
    globalShortcut.unregisterAll();
});

app.on('window-all-closed', () => {
    if (process.platform !== 'darwin') {
        if (psBlockerId !== undefined) powerSaveBlocker.stop(psBlockerId);
        app.quit();
    }
});

// IPC Handler for Minimizing
ipcMain.handle('minimize-app', (event, store) => {
    if (mainWindow) {
        mainWindow.minimize();
        updateAndShowReturnButton(store);
        
        // Auto-maximize after 5 minutes (300,000 ms)
        setTimeout(() => {
            if (mainWindow && mainWindow.isMinimized()) {
                restoreMainApp();
            }
        }, 300000);
    }
});

// IPC Handler for Restoring
ipcMain.handle('restore-app', () => {
    restoreMainApp();
});

ipcMain.handle('quit-app', () => {
    isQuitting = true;
    app.quit();
});

function createReturnWindow() {
    returnWindow = new BrowserWindow({
        width: 340,
        height: 140,
        frame: false,
        transparent: true,
        alwaysOnTop: true,
        resizable: false,
        show: false, // Start hidden
        skipTaskbar: true,
        icon: isDev ? path.join(__dirname, 'public', 'assets', 'logo.ico') : path.join(__dirname, 'dist', 'assets', 'logo.ico'),
        hasShadow: false,
        webPreferences: {
            nodeIntegration: false,
            contextIsolation: true,
            preload: path.join(__dirname, 'preload.js')
        }
    });

    if (isDev) {
        returnWindow.loadURL(`http://localhost:5173/return.html`);
    } else {
        returnWindow.loadFile(path.join(__dirname, 'dist', 'return.html'));
    }
}

function updateAndShowReturnButton(store) {
    if (!returnWindow || returnWindow.isDestroyed()) {
        createReturnWindow();
    }
    
    // Position at top right
    const { screen } = require('electron');
    const primaryDisplay = screen.getPrimaryDisplay();
    const { width } = primaryDisplay.workAreaSize;
    returnWindow.setPosition(width - 360, 40);

    // Send the store info via IPC instead of query params if possible, 
    // but query params are fine for a simple static page. 
    // We'll just refresh the URL with the store.
    if (isDev) {
        returnWindow.loadURL(`http://localhost:5173/return.html?store=${store || 'none'}`);
    } else {
        returnWindow.loadFile(path.join(__dirname, 'dist', 'return.html'), { query: { store: store || 'none' } });
    }
    
    returnWindow.show();
}

function restoreMainApp() {
    if (mainWindow) {
        mainWindow.restore();
        mainWindow.maximize();
        mainWindow.setFullScreen(true);
        mainWindow.setAlwaysOnTop(true, 'screen-saver', { relativeLevel: 1 });
        mainWindow.focus();
    }
    if (returnWindow && !returnWindow.isDestroyed()) {
        returnWindow.hide();
    }
}

ipcMain.handle('get-video-path', () => {
    return app.getAppPath();
});

// IPC Handler for Selecting Video
ipcMain.handle('select-video', async () => {
    const wasAlwaysOnTop = mainWindow.isAlwaysOnTop();
    if (wasAlwaysOnTop) {
        mainWindow.setAlwaysOnTop(false);
    }

    const result = await dialog.showOpenDialog(mainWindow, {
        properties: ['openFile'],
        filters: [
            { name: 'Videos', extensions: ['mp4', 'webm', 'ogg'] }
        ]
    });
    
    if (wasAlwaysOnTop) {
        mainWindow.setAlwaysOnTop(true, 'screen-saver', { relativeLevel: 1 });
    }

    if (result.canceled) return null;
    return result.filePaths[0];
});

// New: Safe Video Persistence
const customVideosDir = path.join(app.getPath('userData'), 'custom_videos');
if (!fs.existsSync(customVideosDir)) {
    fs.mkdirSync(customVideosDir, { recursive: true });
}

ipcMain.handle('save-custom-video', async (event, sourcePath) => {
    try {
        if (!sourcePath || !fs.existsSync(sourcePath)) return null;
        
        const fileName = path.basename(sourcePath);
        // Use timestamp to avoid name collisions
        const newFileName = `${Date.now()}_${fileName}`;
        const destPath = path.join(customVideosDir, newFileName);
        
        fs.copyFileSync(sourcePath, destPath);
        return destPath;
    } catch (e) {
        console.error('Failed to copy custom video:', e);
        return null;
    }
});

ipcMain.handle('check-file-exists', (event, filePath) => {
    if (!filePath) return false;
    return fs.existsSync(filePath);
});

// Autostart Handlers
ipcMain.handle('setup-autostart', () => {
    return new Promise((resolve, reject) => {
        const isDev = !app.isPackaged;
        const scriptPath = isDev 
            ? path.join(__dirname, 'setup-autostart.ps1') 
            : path.join(process.resourcesPath, 'setup-autostart.ps1');
            
        exec(`powershell.exe -ExecutionPolicy Bypass -File "${scriptPath}"`, (error, stdout, stderr) => {
            if (error) reject(error);
            else resolve(stdout);
        });
    });
});

ipcMain.handle('remove-autostart', () => {
    try {
        const shortcutPath = path.join(process.env.APPDATA, 'Microsoft', 'Windows', 'Start Menu', 'Programs', 'Startup', 'Zenit.lnk');
        if (fs.existsSync(shortcutPath)) {
            fs.unlinkSync(shortcutPath);
            return { success: true };
        }
        return { success: false, message: 'Shortcut not found' };
    } catch (e) {
        return { success: false, error: e.message };
    }
});

// IPC Handler for System Specs
ipcMain.handle('get-system-specs', async () => {
    return new Promise((resolve, reject) => {
        const isDev = !app.isPackaged;
        const scriptPath = isDev 
            ? path.join(__dirname, 'get-specs.ps1') 
            : path.join(process.resourcesPath, 'get-specs.ps1');
            
        exec(`powershell.exe -ExecutionPolicy Bypass -File "${scriptPath}"`, (error, stdout, stderr) => {
            if (error) {
                console.error(`Error: ${error.message}`);
                reject(error);
                return;
            }
            try {
                const start = stdout.indexOf('{');
                const end = stdout.lastIndexOf('}');
                if (start === -1 || end === -1) {
                    throw new Error('Valid JSON block not found in output');
                }
                const cleanJson = stdout.substring(start, end + 1);
                const specs = JSON.parse(cleanJson);
                resolve(specs);
            } catch (e) {
                console.error('Failed to parse PS output:', stdout);
                reject(e);
            }
        });
    });
});
