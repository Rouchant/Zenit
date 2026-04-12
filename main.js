// Disable hardware acceleration to prevent GPU process crashes on unstable drivers/hardware
// This is critical for Kiosk/POS stability on low-end integrated graphics (i3/Ryzen 3)
const { app, BrowserWindow, ipcMain, powerSaveBlocker, dialog, globalShortcut, protocol } = require('electron');
app.disableHardwareAcceleration();

const path = require('path');
const fs = require('fs');
const { exec, spawn } = require('child_process');
const { autoUpdater } = require('electron-updater');

// Auto-updater config (only runs in production builds)
autoUpdater.autoDownload = true;        // Download in background automatically
autoUpdater.autoInstallOnAppQuit = true; // Install when the app quits

autoUpdater.on('update-available', (info) => {
    console.log(`[Updater] Nueva versión disponible: ${info.version}`);
});

autoUpdater.on('update-downloaded', (info) => {
    console.log(`[Updater] Versión ${info.version} descargada. Se instalará al cerrar la app.`);
});

autoUpdater.on('update-not-available', () => {
    console.log('[Updater] La app está actualizada.');
});

autoUpdater.on('error', (err) => {
    console.error('[Updater] Error en auto-updater:', err.message);
});

let mainWindow;
let returnWindow;
let psBlockerId;
let isQuitting = false; // Flag for authorized exit
let minimizeTimeout; // Timer for auto-restoration (minimized or blurred)
let isRestoring = false; // Guard to prevent overlapping focus bombardment

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
        
    // Using exec for better stability during the sensitive boot phase
    exec(`powershell.exe -NoProfile -ExecutionPolicy Bypass -File "${scriptPath}"`, (error, stdout) => {
        if (error) console.error('System setup error:', error);
        else if (stdout) console.log('System setup output:', stdout);
    });
}

function createWindow() {
    const isDev = !app.isPackaged;
    
    // Persistent detection: we use a file marker to know if we've ever successfully started before
    const markerPath = path.join(app.getPath('userData'), '.first-run-done');
    const isFirstLaunch = !fs.existsSync(markerPath) || process.argv.includes('--first-launch');
    
    // DELAY OPTIMIZED: 
    // - 10s ONLY on the absolute first run (to let the installer close).
    // - 1000ms on all subsequent runs for a snappy experience.
    const finalDelay = isFirstLaunch ? 10000 : 1000;

    mainWindow = new BrowserWindow({
        width: 1280,
        height: 720,
        show: false, // Hidden until ready
        backgroundColor: '#0a0a0c', // Dark background to match app
        frame: false,
        alwaysOnTop: false, // Start without alwaysOnTop to avoid conflicting with installer
        autoHideMenuBar: true,
        skipTaskbar: false, // Make it visible in the taskbar
        icon: isDev ? path.join(__dirname, 'public', 'assets', 'logo.ico') : path.join(__dirname, 'dist_app', 'assets', 'logo.ico'),
        webPreferences: {
            nodeIntegration: false,
            contextIsolation: true,
            preload: path.join(__dirname, 'preload.js')
        }
    });

    // Remove menu bar completely for a clean kiosk look
    mainWindow.removeMenu();

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
        
        // Delay kiosk mode: longer on first launch so installer finishes closing
        setTimeout(() => {
            if (mainWindow && !mainWindow.isDestroyed()) {
                mainWindow.setKiosk(true);
                mainWindow.setFullScreen(true);
                mainWindow.focus();
                mainWindow.setAlwaysOnTop(true, 'screen-saver', { relativeLevel: 10 });
                
                // Mark first run as completed so subsequent launches are fast
                try {
                    const markerPath = path.join(app.getPath('userData'), '.first-run-done');
                    if (!fs.existsSync(markerPath)) {
                        fs.writeFileSync(markerPath, 'done');
                    }
                } catch (e) {
                    console.error('Failed to create first-run marker:', e);
                }
            }
        }, finalDelay);
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

    // Auto-restore logic: If focus is lost (blur), wait 5 minutes and return to focus
    // This allows temporary use of the OS (Start Menu, etc) but ensures the app returns
    mainWindow.on('blur', () => {
        if (!isQuitting) {
            // Clear any existing timer to avoid overlaps
            if (minimizeTimeout) clearTimeout(minimizeTimeout);
            
            minimizeTimeout = setTimeout(() => {
                // Double check we are still out of focus or minimized before restoring
                if (mainWindow && (!mainWindow.isFocused() || mainWindow.isMinimized())) {
                    console.log('[Lockdown] Auto-restoring focus after 2m blur/minimize');
                    restoreMainApp();
                }
            }, 120000); // 2 minutes
        }
    });

    // Backup listener for system-level blur
    app.on('browser-window-blur', () => {
        if (mainWindow && !mainWindow.isFocused() && !isQuitting) {
            // If the window loses focus but the timer isn't set, start it
            if (!minimizeTimeout) {
                minimizeTimeout = setTimeout(() => {
                    if (mainWindow && !mainWindow.isFocused()) {
                        restoreMainApp();
                    }
                }, 120000);
            }
        }
    });

    mainWindow.on('focus', () => {
        // If we regained focus manually, stop the auto-restore timer
        if (minimizeTimeout) {
            clearTimeout(minimizeTimeout);
            minimizeTimeout = null;
        }
    });

    app.on('activate', () => {
        if (BrowserWindow.getAllWindows().length === 0) createWindow();
    });

    // Check for updates after startup (only in production)
    if (app.isPackaged) {
        setTimeout(() => {
            autoUpdater.checkForUpdatesAndNotify();
        }, 5000); // 5s delay so the UI loads first
    }
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
        
        // Auto-maximize after 2 minutes (120,000 ms)
        minimizeTimeout = setTimeout(() => {
            if (mainWindow && mainWindow.isMinimized()) {
                restoreMainApp();
            }
        }, 120000);
    }
});

// IPC Handler for Restoring
ipcMain.handle('restore-app', async () => {
    await restoreMainApp();
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
        icon: isDev ? path.join(__dirname, 'public', 'assets', 'logo.ico') : path.join(__dirname, 'dist_app', 'assets', 'logo.ico'),
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

function sendEscapeKey() {
    return new Promise((resolve) => {
        // Use PowerShell to simulate Escape key press
        // This dismisses the Start Menu/Search overlay before we reclaim focus
        const ps = spawn('powershell.exe', [
            '-NoProfile',
            '-ExecutionPolicy', 'Bypass',
            '-Command', 'Add-Type -AssemblyName System.Windows.Forms; [System.Windows.Forms.SendKeys]::SendWait(\'{ESC}\')'
        ]);

        ps.on('close', () => resolve());
        ps.on('error', (err) => {
            console.error('[Lockdown] Failed to spawn PowerShell for Escape key:', err);
            resolve(); // Resolve anyway to not hang the app
        });
        
        // Safety timeout to ensure we don't hang if PowerShell fails to close
        setTimeout(() => resolve(), 2000);
    });
}

async function restoreMainApp() {
    // Safety check to prevent overlapping restoration cycles
    if (isRestoring || isQuitting) return;
    isRestoring = true;

    // Clear auto-maximize timer if manual restore happens
    if (minimizeTimeout) {
        clearTimeout(minimizeTimeout);
        minimizeTimeout = null;
    }

    if (mainWindow) {
        console.log('[Lockdown] Forcing app restoration to foreground (Aggressive Sync)');
        
        // Step 1: Clear the path by sending Escape (Awaiting completion)
        await sendEscapeKey();

        // Step 2: Reset window state to force OS to re-evaluate Z-order
        if (!mainWindow || mainWindow.isDestroyed()) {
            isRestoring = false;
            return;
        }

        mainWindow.setKiosk(false);
        mainWindow.setAlwaysOnTop(false);
        
        if (mainWindow.isMinimized()) {
            mainWindow.restore();
        }
        
        mainWindow.show();
        
        // Use a very high relative level to stay above system bars
        mainWindow.setAlwaysOnTop(true, 'screen-saver', { relativeLevel: 25 });
        mainWindow.maximize();
        mainWindow.setFullScreen(true);
        mainWindow.setKiosk(true);
        
        // Ensure it's absolutely on top
        mainWindow.moveTop();
        mainWindow.focus();
        
        // Step 3: Z-order "Bombardment"
        // This keeps pushing the window to the top for 3 seconds
        let bombardmentCount = 0;
        const bombardmentInterval = setInterval(() => {
            if (mainWindow && !mainWindow.isDestroyed()) {
                mainWindow.setAlwaysOnTop(true, 'screen-saver', { relativeLevel: 25 });
                mainWindow.moveTop();
                if (bombardmentCount % 2 === 0) mainWindow.focus();
            }
            bombardmentCount++;
            if (bombardmentCount > 12) {
                clearInterval(bombardmentInterval);
                isRestoring = false; // Open the guard again
            }
        }, 250);
        
        // Final focus attempt after a small tick
        setTimeout(() => {
            if (mainWindow && !mainWindow.isDestroyed()) {
                mainWindow.focus();
            }
        }, 500);
    } else {
        isRestoring = false;
    }

    if (returnWindow && !returnWindow.isDestroyed()) {
        returnWindow.hide();
    }
}

ipcMain.handle('get-video-path', () => {
    return app.getAppPath();
});

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

// Helper for PowerShell with Timeout (using exec for stability)
function execPowerShell(command, timeoutMs = 15000) {
    return new Promise((resolve, reject) => {
        // We must explicitly call powershell.exe since exec uses cmd.exe by default
        const fullCommand = `powershell.exe -NoProfile -ExecutionPolicy Bypass -Command "${command.replace(/"/g, '\"')}"`;
        exec(fullCommand, { timeout: timeoutMs }, (error, stdout) => {
            if (error) reject(error);
            else resolve(stdout);
        });
    });
}

// Autostart Handlers
ipcMain.handle('setup-autostart', async () => {
    const isDev = !app.isPackaged;
    const scriptPath = isDev 
        ? path.join(__dirname, 'setup-autostart.ps1') 
        : path.join(process.resourcesPath, 'setup-autostart.ps1');
        
    try {
        const stdout = await execPowerShell(`& '${scriptPath}'`);
        return stdout;
    } catch (err) {
        throw err;
    }
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
        const stdout = await execPowerShell(`& '${scriptPath}'`, 12000);
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
