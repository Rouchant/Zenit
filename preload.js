const { contextBridge, ipcRenderer } = require('electron');

contextBridge.exposeInMainWorld('electronAPI', {
    getSystemSpecs: () => ipcRenderer.invoke('get-system-specs'),
    getVideoPath: () => ipcRenderer.invoke('get-video-path'),
    minimizeApp: (store) => ipcRenderer.invoke('minimize-app', store),
    restoreApp: () => ipcRenderer.invoke('restore-app'),
    selectVideo: () => ipcRenderer.invoke('select-video'),
    saveCustomVideo: (sourcePath) => ipcRenderer.invoke('save-custom-video', sourcePath),
    checkFileExists: (filePath) => ipcRenderer.invoke('check-file-exists', filePath),
    setupAutostart: () => ipcRenderer.invoke('setup-autostart'),
    removeAutostart: () => ipcRenderer.invoke('remove-autostart'),
    saveConfig: (configData) => ipcRenderer.invoke('save-config', configData),
    loadConfig: () => ipcRenderer.invoke('load-config'),
    quitApp: () => ipcRenderer.invoke('quit-app')
});
