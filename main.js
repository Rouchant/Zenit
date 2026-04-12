// Disable hardware acceleration to prevent GPU process crashes on unstable drivers/hardware
// This is critical for Kiosk/POS stability on low-end integrated graphics (i3/Ryzen 3)
const { app, BrowserWindow, ipcMain, powerSaveBlocker, dialog, globalShortcut, protocol } = require('electron');
app.disableHardwareAcceleration();

const path = require('path');
const fs = require('fs');
const { exec } = require('child_process');

let mainWindow;
let returnWindow;
let psBlockerId;
let isQuitting = false; // Flag for authorized exit
let minimizeTimeout; // Timer for auto-restoration

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
    const isDev = !app.isPackaged;
    mainWindow = new BrowserWindow({
        width: 1280,
        height: 720,
        show: false, // Hidden until ready
        backgroundColor: '#0a0a0c', // Dark background to match app
        frame: false,
        alwaysOnTop: true,
        autoHideMenuBar: true,
        skipTaskbar: false, // Make it visible in the taskbar
        icon: isDev ? path.join(__dirname, 'public', 'assets', 'logo.ico') : path.join(__dirname, 'dist', 'assets', 'logo.ico'),
        webPreferences: {
            nodeIntegration: false,
            contextIsolation: true,
            preload: path.join(__dirname, 'preload.js')
        }
    });

    // Remove menu bar completely for a clean kiosk look
    mainWindow.removeMenu();
    
    // Ensure it always stays on top of everything
    mainWindow.setAlwaysOnTop(true, 'screen-saver', { relativeLevel: 10 });

    if (isDev) {
        mainWindow.loadURL('http://localhost:5173');
        // Open DevTools automatically in development mode as requested
        // mainWindow.webContents.openDevTools();
    } else {
        mainWindow.loadFile(path.join(__dirname, 'dist_app', 'index.html'));
    }

    // Graceful Kiosk activation
    mainWindow.once('ready-to-show', () => {
        mainWindow.show();
        mainWindow.maximize();
        
        // Start Kiosk mode with a delay to ensure everything is rendered
        setTimeout(() => {
            if (mainWindow && !mainWindow.isDestroyed()) {
                mainWindow.setKiosk(true);
                mainWindow.setFullScreen(true);
                mainWindow.focus();
                mainWindow.setAlwaysOnTop(true, 'screen-saver', { relativeLevel: 10 });
            }
        }, 2000);
    });

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
    // Register custom protocol for local files (allows loading custom videos in dev mode)
    protocol.registerFileProtocol('zenit-file', (request, callback) => {
        let url = request.url.replace('zenit-file://', '');
        // On Windows, URLs like /C:/Users... need the leading slash removed
        if (url.startsWith('/')) {
            url = url.substring(1);
        }
        try {
            return callback(decodeURIComponent(url));
        } catch (error) {
            console.error('Failed to register protocol', error);
        }
    });

    runSystemSetup();
    createWindow();
    createReturnWindow(); // Pre-create hidden

    // Register Lockdown Shortcuts safely
    const safeRegister = (acc, cb) => {
        try {
            globalShortcut.register(acc, cb);
        } catch (e) {
            console.error(`Failed to register shortcut: ${acc}`, e);
        }
    };

    safeRegister('Alt+Tab', () => { console.log('Alt+Tab blocked'); });
    safeRegister('Alt+F4', () => { console.log('Alt+F4 blocked'); });
    safeRegister('CommandOrControl+Esc', () => { console.log('Start Menu blocked'); });
    safeRegister('Alt+Esc', () => { console.log('Alt+Esc blocked'); });
    
    ['D', 'R', 'E', 'L', 'X', 'I', 'S'].forEach(key => {
        safeRegister(`Meta+${key}`, () => { console.log(`Win+${key} blocked`); });
    });

    // Auto-refocus if blur (lockdown) - Skip if minimized to prevent focus-fighting
    mainWindow.on('blur', () => {
        if (!isQuitting && !mainWindow.isMinimized()) {
            mainWindow.setAlwaysOnTop(true, 'screen-saver', { relativeLevel: 10 });
            mainWindow.focus();
        }
    });

    // Optimized Focus Lock (Kiosk Guard)
    // Throttled to 2500ms and only acts if focus is actually lost and NOT minimized
    setInterval(() => {
        if (mainWindow && !mainWindow.isFocused() && !isQuitting && !mainWindow.isMinimized()) {
            mainWindow.setAlwaysOnTop(true, 'screen-saver', { relativeLevel: 10 });
            mainWindow.focus();
        }
    }, 2500);

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
        // Clear any existing timeout before starting a new one
        if (minimizeTimeout) clearTimeout(minimizeTimeout);

        mainWindow.minimize();
        updateAndShowReturnButton(store);
        
        // Auto-maximize after 5 minutes (300,000 ms)
        minimizeTimeout = setTimeout(() => {
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
    const isDev = !app.isPackaged;
    returnWindow = new BrowserWindow({
        width: 400,
        height: 160,
        frame: false,
        transparent: true,
        alwaysOnTop: true,
        resizable: false,
        movable: false,
        show: false, // Start hidden
        skipTaskbar: false, // Make it visible in the taskbar
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
        returnWindow.loadFile(path.join(__dirname, 'dist_app', 'return.html'));
    }
}

function updateAndShowReturnButton(store) {
    const isDev = !app.isPackaged;
    if (!returnWindow || returnWindow.isDestroyed()) {
        createReturnWindow();
    }
    
    // Position at top right
    const { screen } = require('electron');
    const primaryDisplay = screen.getPrimaryDisplay();
    const { width } = primaryDisplay.workAreaSize;
    returnWindow.setPosition(width - 420, 40);

    // Only reload if the store has changed to avoid latency
    const currentUrl = returnWindow.webContents.getURL();
    const targetQuery = `store=${store || 'none'}`;
    
    if (!currentUrl.includes(targetQuery)) {
        if (isDev) {
            returnWindow.loadURL(`http://localhost:5173/return.html?${targetQuery}`);
        } else {
            returnWindow.loadFile(path.join(__dirname, 'dist_app', 'return.html'), { query: { store: store || 'none' } });
        }
    }
    
    returnWindow.show();
    returnWindow.setAlwaysOnTop(true, 'screen-saver', { relativeLevel: 11 }); // Slightly above main app
}

function restoreMainApp() {
    // Clear auto-maximize timer if manual restore happens
    if (minimizeTimeout) {
        clearTimeout(minimizeTimeout);
        minimizeTimeout = null;
    }

    if (mainWindow) {
        if (mainWindow.isMinimized()) {
            mainWindow.restore();
        }
        mainWindow.show(); // Ensure it's shown
        mainWindow.setAlwaysOnTop(true, 'screen-saver', { relativeLevel: 10 });
        mainWindow.maximize();
        mainWindow.setFullScreen(true);
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

// Robust File Persistence (for config and specs)
const configPath = path.join(app.getPath('userData'), 'config.json');

ipcMain.handle('save-config', (event, configData) => {
    try {
        fs.writeFileSync(configPath, JSON.stringify(configData, null, 2));
        return { success: true };
    } catch (error) {
        console.error('Failed to save config:', error);
        return { success: false, error: error.message };
    }
});

ipcMain.handle('load-config', () => {
    try {
        if (fs.existsSync(configPath)) {
            const data = fs.readFileSync(configPath, 'utf8');
            return JSON.parse(data);
        }
    } catch (error) {
        console.error('Failed to load config:', error);
    }
    return null;
});

// Helper for PowerShell with Timeout
function execPowerShell(command, timeoutMs = 15000) {
    return new Promise((resolve, reject) => {
        const child = exec(command, (error, stdout, stderr) => {
            if (error) {
                if (error.killed) reject(new Error('Process timed out'));
                else reject(error);
            } else {
                resolve(stdout);
            }
        });

        if (timeoutMs > 0) {
            setTimeout(() => {
                child.kill();
            }, timeoutMs);
        }
    });
}

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
    const isDev = !app.isPackaged;
    const scriptPath = isDev 
        ? path.join(__dirname, 'get-specs.ps1') 
        : path.join(process.resourcesPath, 'get-specs.ps1');
        
    try {
        const stdout = await execPowerShell(`powershell.exe -ExecutionPolicy Bypass -File "${scriptPath}"`, 12000);
        const start = stdout.indexOf('{');
        const end = stdout.lastIndexOf('}');
        if (start === -1 || end === -1) {
            throw new Error('Valid JSON block not found in output');
        }
        const cleanJson = stdout.substring(start, end + 1);
        return JSON.parse(cleanJson);
    } catch (error) {
        console.error('Spec detection failed or timed out:', error);
        return null; // Frontend will handle null by loading generic fallbacks
    }
});
