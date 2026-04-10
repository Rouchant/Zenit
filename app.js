// --- Configuration & Constants ---
const CONFIG = {
    INACTIVITY_LIMIT: 120000, // 2 minutes
    PASSWORD: 'rogally',
    THEMES: ['falabella', 'paris', 'ripley', 'default']
};

// --- State Management ---
const State = {
    currentSpecs: {},
    autoDetectedSpecs: {},
    customSpecs: JSON.parse(localStorage.getItem('customSpecs')) || null,
    inactivityTimer: null,
    isVideoMode: false,
    cycleInterval: null
};

// --- DOM Elements ---
const UI = {
    // Views
    views: {
        info: document.getElementById('info-view'),
        video: document.getElementById('video-view')
    },
    // Media
    media: {
        promoVideo: document.getElementById('promo-video'),
        landingVideo: document.getElementById('landing-video')
    },
    // Modals
    modals: {
        custom: document.getElementById('custom-modal'),
        password: document.getElementById('password-modal'),
        specs: document.getElementById('specs-modal')
    },
    // Display Elements (Main Screen)
    display: {
        brand: document.getElementById('display-brand'),
        processor: document.getElementById('display-processor'),
        gen: document.getElementById('display-gen'),
        cores: document.getElementById('display-cores'),
        ram: document.getElementById('display-ram'),
        ramType: document.getElementById('display-ram-type'),
        price: document.getElementById('display-price'),
        storage: document.getElementById('display-storage'),
        gpu: document.getElementById('display-gpu'),
        screen: document.getElementById('display-screen'),
        os: document.getElementById('display-os')
    },
    // Input Elements (Customization)
    inputs: {
        brand: document.getElementById('input-brand'),
        processor: document.getElementById('input-processor'),
        ram: document.getElementById('input-ram'),
        storage: document.getElementById('input-storage'),
        gpu: document.getElementById('input-gpu'),
        screen: document.getElementById('input-screen'),
        os: document.getElementById('input-os'),
        store: document.getElementById('input-store'),
        ramType: document.getElementById('input-ram-type'),
        pricePrimary: document.getElementById('input-price-primary'),
        priceSecondary: document.getElementById('input-price-secondary'),
        priceStrike: document.getElementById('input-price-strike'),
        password: document.getElementById('admin-password')
    },
    // Containers & Branding
    branding: {
        headerLogo: document.getElementById('header-logo-container'),
        storeLogo: document.getElementById('store-logo-container')
    },
    // Error / Status
    status: {
        passwordError: document.getElementById('password-error'),
        vPathLabel: document.getElementById('selected-video-path'),
        lPathLabel: document.getElementById('selected-landing-video-path'),
        systemStatus: document.getElementById('system-status')
    }
};

// Helper to truncate paths for display (show filename only)
function formatPath(fullPath) {
    if (!fullPath) return 'Sin archivo';
    // Extracts filename from Windows or Unix paths
    const fileName = fullPath.split(/[/\\]/).pop();
    if (fileName.length <= 15) return fileName;
    return fileName.substring(0, 12) + '...';
}

async function init() {
    console.log('Initializing app...');
    resetTimer();

    setupAuthEvents();
    setupConfigEvents();
    setupRestoreEvents();
    setupGlobalEvents();

    // Load Specs
    try {
        if (window.electronAPI) {
            State.autoDetectedSpecs = await window.electronAPI.getSystemSpecs();
        } else {
            console.warn('Electron API not found, using fallbacks.');
            State.autoDetectedSpecs = {
                brand: 'Asus', processor: 'AMD Ryzen 7', ram: '16GB', storage: '512GB SSD', 
                gpu: 'Radeon Graphics', display: '1920x1080', os: 'Windows 11'
            };
        }

        // Apply Asus fallback if brand is unknown
        const detectedBrand = (State.autoDetectedSpecs.brand || '').toLowerCase();
        if (!detectedBrand || detectedBrand === 'system manufacturer' || detectedBrand.includes('to be filled')) {
            State.autoDetectedSpecs.brand = 'Asus';
        }

        State.currentSpecs = { ...State.autoDetectedSpecs, ...(State.customSpecs || {}) };
        await updateUI(State.currentSpecs);
        checkSpecsOverflow();
    } catch (err) {
        console.error('Failed to get specs:', err);
        State.currentSpecs = State.customSpecs || {
            brand: 'Asus', processor: 'AMD Ryzen 7', ram: '16GB', storage: '512GB SSD', 
            gpu: 'Radeon Graphics', display: '1920x1080', os: 'Windows 11', store: 'none',
            videoType: 'default', customVideoPath: '',
            landingVideoType: 'default', customLandingVideoPath: ''
        };
        updateUI(State.currentSpecs);
        checkSpecsOverflow();
    }
}

function setupAuthEvents() {
    let settingsClicks = 0;
    let exitClicks = 0;
    let lastSettingsClick = 0;
    let lastExitClick = 0;

    const settingsHotspot = document.getElementById('settings-hotspot');
    const exitHotspot = document.getElementById('exit-hotspot');

    settingsHotspot.onclick = () => {
        const now = Date.now();
        if (now - lastSettingsClick < 600) settingsClicks++;
        else settingsClicks = 1;
        lastSettingsClick = now;

        if (settingsClicks >= 4) {
            settingsClicks = 0;
            UI.modals.password.dataset.mode = 'settings';
            UI.modals.password.classList.add('active');
            UI.status.passwordError.style.display = 'none';
            UI.inputs.password.value = '';
            UI.inputs.password.focus();
        }
    };

    exitHotspot.onclick = () => {
        const now = Date.now();
        if (now - lastExitClick < 600) exitClicks++;
        else exitClicks = 1;
        lastExitClick = now;

        if (exitClicks >= 4) {
            exitClicks = 0;
            UI.modals.password.dataset.mode = 'exit';
            UI.modals.password.classList.add('active');
            UI.status.passwordError.style.display = 'none';
            UI.inputs.password.value = '';
            UI.inputs.password.focus();
        }
    };

    const verifyPassword = () => {
        const pass = UI.inputs.password.value;
        const mode = UI.modals.password.dataset.mode;

        if (pass === CONFIG.PASSWORD) {
            UI.modals.password.classList.remove('active');
            UI.inputs.password.value = '';
            
            if (mode === 'exit') {
                window.electronAPI.quitApp();
            } else {
                openModal();
            }
        } else {
            UI.status.passwordError.style.display = 'block';
            UI.inputs.password.value = '';
            UI.inputs.password.focus();
        }
    };

    document.getElementById('verify-password').onclick = verifyPassword;
    UI.inputs.password.onkeydown = (e) => { if (e.key === 'Enter') verifyPassword(); };
    document.getElementById('close-password').onclick = () => UI.modals.password.classList.remove('active');
}

function setupConfigEvents() {
    document.getElementById('save-custom').onclick = () => {
        const specs = {
            brand: UI.inputs.brand.value,
            processor: UI.inputs.processor.value,
            ram: UI.inputs.ram.value,
            storage: UI.inputs.storage.value,
            gpu: UI.inputs.gpu.value,
            display: UI.inputs.screen.value,
            os: UI.inputs.os.value,
            store: UI.inputs.store.value,
            ramType: UI.inputs.ramType.value,
            pricePrimary: UI.inputs.pricePrimary.value || '',
            priceSecondary: UI.inputs.priceSecondary.value || '',
            priceStrike: UI.inputs.priceStrike.checked,
            videoType: document.querySelector('input[name="video-source"]:checked').value,
            customVideoPath: UI.status.vPathLabel.dataset.fullPath || '',
            landingVideoType: document.querySelector('input[name="landing-video-source"]:checked').value,
            customLandingVideoPath: UI.status.lPathLabel.dataset.fullPath || ''
        };
        saveCustom(specs);
        UI.modals.custom.classList.remove('active');
    };

    document.getElementById('view-pc').onclick = () => window.electronAPI.minimizeApp(State.currentSpecs.store);
    document.getElementById('close-modal').onclick = () => UI.modals.custom.classList.remove('active');
    
    // Detailed Specs Modal
    const toggleBtn = document.getElementById('toggle-specs');
    if (toggleBtn) {
        toggleBtn.onclick = () => {
            updateModalSpecs();
            UI.modals.specs.classList.add('active');
        };
    }
    document.getElementById('close-specs-modal').onclick = () => UI.modals.specs.classList.remove('active');

    // Inactivity Video Options
    const videoSourceRadios = document.querySelectorAll('input[name="video-source"]');
    const selectVideoBtn = document.getElementById('select-video-btn');
    videoSourceRadios.forEach(radio => {
        radio.onchange = () => {
            selectVideoBtn.style.display = radio.value === 'custom' ? 'block' : 'none';
        };
    });

    document.getElementById('select-video-btn').onclick = async () => {
        const path = await window.electronAPI.selectVideo();
        if (path) {
            const safePath = await window.electronAPI.saveCustomVideo(path);
            if (safePath) {
                UI.status.vPathLabel.textContent = formatPath(safePath);
                UI.status.vPathLabel.dataset.fullPath = safePath;
                document.getElementById('video-custom').checked = true;
                selectVideoBtn.style.display = 'block';
            }
        }
    };

    // Landing Video Options
    const landingVideoSourceRadios = document.querySelectorAll('input[name="landing-video-source"]');
    const selectLandingBtn = document.getElementById('select-landing-video-btn');
    landingVideoSourceRadios.forEach(radio => {
        radio.onchange = () => {
            selectLandingBtn.style.display = radio.value === 'custom' ? 'block' : 'none';
        };
    });

    document.getElementById('select-landing-video-btn').onclick = async () => {
        const path = await window.electronAPI.selectVideo();
        if (path) {
            const safePath = await window.electronAPI.saveCustomVideo(path);
            if (safePath) {
                UI.status.lPathLabel.textContent = formatPath(safePath);
                UI.status.lPathLabel.dataset.fullPath = safePath;
                document.getElementById('landing-video-custom').checked = true;
                selectLandingBtn.style.display = 'block';
            }
        }
    };

    // System Autostart Handlers
    document.getElementById('enable-autostart').onclick = async () => {
        UI.status.systemStatus.textContent = 'Configurando...';
        try {
            const result = await window.electronAPI.setupAutostart();
            UI.status.systemStatus.textContent = 'Autostart activado correctamente.';
            setTimeout(() => UI.status.systemStatus.textContent = '', 3000);
        } catch (e) {
            UI.status.systemStatus.textContent = 'Error al configurar autostart.';
        }
    };

    document.getElementById('disable-autostart').onclick = async () => {
        UI.status.systemStatus.textContent = 'Quitando...';
        try {
            const result = await window.electronAPI.removeAutostart();
            if (result.success) {
                UI.status.systemStatus.textContent = 'Autostart desactivado.';
            } else {
                UI.status.systemStatus.textContent = result.message || 'No se encontró el autostart.';
            }
            setTimeout(() => UI.status.systemStatus.textContent = '', 3000);
        } catch (e) {
            UI.status.systemStatus.textContent = 'Error al desactivar autostart.';
        }
    };
}

function setupRestoreEvents() {
    document.querySelectorAll('.restore-btn').forEach(btn => {
        btn.onclick = () => {
            const field = btn.getAttribute('data-field');
            const autoValue = State.autoDetectedSpecs[field] || '';
            
            // Map data-field to input ID
            let inputId = `input-${field}`;
            if (field === 'display') inputId = 'input-screen';
            if (field === 'ramType') inputId = 'input-ram-type';
            
            const input = document.getElementById(inputId);
            if (input) {
                input.value = autoValue;
                
                // Visual feedback
                input.style.borderColor = 'var(--primary)';
                input.style.boxShadow = '0 0 10px var(--primary)';
                setTimeout(() => {
                    input.style.borderColor = '';
                    input.style.boxShadow = '';
                }, 400);
            }
        };
    });
}

function setupGlobalEvents() {
    window.addEventListener('resize', checkSpecsOverflow);
    window.addEventListener('mousemove', resetTimer);
    window.addEventListener('keydown', resetTimer);
    window.addEventListener('mousedown', resetTimer);
    window.addEventListener('load', () => {
        setTimeout(checkSpecsOverflow, 100);
    });
}

async function updateUI(specs) {
    if (!specs) return;

    // Fallback logic for missing files
    let specsChanged = false;
    if (specs.videoType === 'custom' && specs.customVideoPath) {
        const exists = await window.electronAPI.checkFileExists(specs.customVideoPath);
        if (!exists) {
            console.warn('Custom inactivity video missing, reverting to default.');
            specs.videoType = 'default';
            specsChanged = true;
        }
    }
    if (specs.landingVideoType === 'custom' && specs.customLandingVideoPath) {
        const exists = await window.electronAPI.checkFileExists(specs.customLandingVideoPath);
        if (!exists) {
            console.warn('Custom landing video missing, reverting to default.');
            specs.landingVideoType = 'default';
            specsChanged = true;
        }
    }
    
    if (specsChanged) {
        saveCustom(specs);
    }

    State.currentSpecs = specs;
    UI.display.brand.innerText = specs.brand || 'Desconocido';
    UI.display.processor.innerText = specs.processor || 'Desconocido';
    UI.display.gen.innerText = specs.gen || '';
    UI.display.ramType.innerText = specs.ramType || '';
    UI.display.cores.innerText = specs.cores ? `${specs.cores} Núcleos / ${specs.threads} Hilos` : '';
    UI.display.ram.innerText = specs.ram || 'Desconocido';
    UI.display.storage.innerText = specs.storage || 'Desconocido';
    UI.display.gpu.innerText = specs.gpu || 'Desconocido';
    UI.display.screen.innerText = specs.display || 'Desconocido';
    
    // Dual Price Rendering
    if (specs.pricePrimary || specs.priceSecondary) {
        let priceHtml = '';
        if (specs.priceSecondary) {
            priceHtml += `<div class="price-secondary ${specs.priceStrike ? 'strike' : ''}">${specs.priceSecondary}</div>`;
        }
        if (specs.pricePrimary) {
            priceHtml += `<div class="price-primary">${specs.pricePrimary}</div>`;
        }
        UI.display.price.innerHTML = priceHtml;
    } else {
        UI.display.price.innerHTML = '';
    }
    
    if (UI.display.os) UI.display.os.innerText = specs.os || 'Desconocido';

    // Update inputs
    UI.inputs.brand.value = specs.brand || '';
    UI.inputs.processor.value = specs.processor || '';
    UI.inputs.ram.value = specs.ram || '';
    UI.inputs.storage.value = specs.storage || '';
    UI.inputs.gpu.value = specs.gpu || '';
    UI.inputs.screen.value = specs.display || '';
    if (UI.inputs.os) UI.inputs.os.value = specs.os || '';
    if (UI.inputs.store) UI.inputs.store.value = specs.store || 'none';
    if (UI.inputs.ramType) UI.inputs.ramType.value = specs.ramType || '';
    if (UI.inputs.pricePrimary) UI.inputs.pricePrimary.value = specs.pricePrimary || '';
    if (UI.inputs.priceSecondary) UI.inputs.priceSecondary.value = specs.priceSecondary || '';
    if (UI.inputs.priceStrike) UI.inputs.priceStrike.checked = specs.priceStrike || false;

    // Theme switching
    const store = (specs.store || 'none').toLowerCase();
    document.body.classList.remove(...CONFIG.THEMES.map(t => `theme-${t}`));
    document.body.classList.add(`theme-${store === 'none' ? 'default' : store}`);

    // Update video settings in modal
    const videoType = specs.videoType || 'default';
    const radio = document.querySelector(`input[name="video-source"][value="${videoType}"]`);
    if (radio) radio.checked = true;
    
    UI.status.vPathLabel.textContent = formatPath(specs.customVideoPath);

    // Update landing video settings in modal
    const lVideoType = specs.landingVideoType || 'default';
    const lRadio = document.querySelector(`input[name="landing-video-source"][value="${lVideoType}"]`);
    if (lRadio) lRadio.checked = true;
    
    UI.status.lPathLabel.textContent = formatPath(specs.customLandingVideoPath);

    applyLandingVideo(specs);
    updateLogo(specs.brand);
    updateStoreLogo(specs.store);
    updateProcessorLogo(specs.vendor);
    updateOSLogo(specs.os);
}

async function applyLandingVideo(specs) {
    if (!UI.media.landingVideo) return;
    
    try {
        let videoUrl = (specs.landingVideoType === 'custom' && specs.customLandingVideoPath)
            ? `file:///${specs.customLandingVideoPath}`.replace(/\\/g, '/')
            : 'assets/videos/landing.mp4';
        
        if (!UI.media.landingVideo.src.endsWith(videoUrl)) {
            UI.media.landingVideo.src = videoUrl;
            UI.media.landingVideo.load();
            UI.media.landingVideo.play().catch(() => {});
        }
    } catch (e) {
         console.error('Failed to set landing video:', e);
    }
}

function updateStoreLogo(store) {
    if (!UI.branding.storeLogo) return;
    const s = (store || 'none').toLowerCase();
    const separator = document.querySelector('.logo-separator');
    
    if (s === 'none') {
        UI.branding.storeLogo.innerHTML = '';
        if (separator) separator.style.display = 'none';
    } else {
        const ext = s === 'paris' ? 'png' : 'svg';
        UI.branding.storeLogo.innerHTML = `<img src="assets/logos/${s}.${ext}" alt="${s}">`;
        if (separator) separator.style.display = 'block';
    }
}

function updateOSLogo(os) {
    const container = document.getElementById('os-logo');
    if (container) container.innerHTML = `<img src="assets/ui/windows-11.svg" alt="Windows 11">`;
}

function updateProcessorLogo(vendor) {
    const container = document.getElementById('proc-vendor-logo');
    if (!container) return;
    
    const v = (vendor || '').toLowerCase();
    const folder = (v === 'intel' || v === 'amd') ? 'logos' : 'ui';
    const icon = (v === 'intel' || v === 'amd') ? v : 'cpu';
    container.innerHTML = `<img src="assets/${folder}/${icon}.svg" alt="${v}">`;
}

function updateLogo(brandName) {
    const brand = (brandName || '').toLowerCase();
    const knownBrands = ['asus', 'hp', 'samsung', 'acer', 'lenovo'];
    const matched = knownBrands.find(b => brand.includes(b));

    if (matched) {
        UI.branding.headerLogo.innerHTML = `<img src="assets/logos/${matched}.svg" alt="${matched}">`;
    } else {
        UI.branding.headerLogo.innerHTML = `<img src="assets/logo.png" alt="Zenit" class="zenit-logo">`;
    }
}

function resetTimer() {
    clearTimeout(State.inactivityTimer);
    if (State.isVideoMode) exitVideoMode();
    State.inactivityTimer = setTimeout(enterVideoMode, CONFIG.INACTIVITY_LIMIT);
}

async function enterVideoMode() {
    State.isVideoMode = true;
    try {
        let videoUrl = (State.currentSpecs.videoType === 'custom' && State.currentSpecs.customVideoPath)
            ? `file:///${State.currentSpecs.customVideoPath}`.replace(/\\/g, '/')
            : 'assets/videos/promo.mp4';
        
        if (!UI.media.promoVideo.src.endsWith(videoUrl)) {
            UI.media.promoVideo.src = videoUrl;
            UI.media.promoVideo.load();
        }
        UI.media.promoVideo.play().catch(e => console.warn('Inactivity video failed to play: ', e));
    } catch (e) {
        console.error('Failed to enter video mode:', e);
    }
    UI.views.info.classList.remove('active');
    UI.views.video.classList.add('active');
    startCycle();
}

function exitVideoMode() {
    State.isVideoMode = false;
    stopCycle();
    UI.media.promoVideo.pause();
    UI.media.promoVideo.currentTime = 0;
    UI.views.video.classList.remove('active');
    UI.views.info.classList.add('active');
}

function startCycle() {
    stopCycle();
    State.cycleInterval = setInterval(() => {
        if (State.isVideoMode) {
            UI.views.video.classList.remove('active');
            UI.views.info.classList.add('active');
            setTimeout(() => {
                if (State.isVideoMode) {
                    UI.views.info.classList.remove('active');
                    UI.views.video.classList.add('active');
                }
            }, 60000);
        }
    }, 300000);
}

function stopCycle() {
    clearInterval(State.cycleInterval);
}

function openModal() {
    // Hardware
    UI.inputs.brand.value = State.currentSpecs.brand || '';
    UI.inputs.processor.value = State.currentSpecs.processor || '';
    UI.inputs.ram.value = State.currentSpecs.ram || '';
    UI.inputs.storage.value = State.currentSpecs.storage || '';
    UI.inputs.gpu.value = State.currentSpecs.gpu || '';
    UI.inputs.screen.value = State.currentSpecs.display || '';
    UI.inputs.os.value = State.currentSpecs.os || '';
    UI.inputs.store.value = State.currentSpecs.store || 'none';
    UI.inputs.ramType.value = State.currentSpecs.ramType || '';
    UI.inputs.pricePrimary.value = State.currentSpecs.pricePrimary || '';
    UI.inputs.priceSecondary.value = State.currentSpecs.priceSecondary || '';
    UI.inputs.priceStrike.checked = State.currentSpecs.priceStrike || false;

    // Inactivity Video
    const isCustom = State.currentSpecs.videoType === 'custom';
    document.getElementById('video-custom').checked = isCustom;
    document.getElementById('video-default').checked = !isCustom;
    UI.status.vPathLabel.textContent = formatPath(State.currentSpecs.customVideoPath);
    UI.status.vPathLabel.dataset.fullPath = State.currentSpecs.customVideoPath || '';
    document.getElementById('select-video-btn').style.display = isCustom ? 'block' : 'none';

    // Landing Video
    const isLandingCustom = State.currentSpecs.landingVideoType === 'custom';
    document.getElementById('landing-video-custom').checked = isLandingCustom;
    document.getElementById('landing-video-default').checked = !isLandingCustom;
    UI.status.lPathLabel.textContent = formatPath(State.currentSpecs.customLandingVideoPath);
    UI.status.lPathLabel.dataset.fullPath = State.currentSpecs.customLandingVideoPath || '';
    document.getElementById('select-landing-video-btn').style.display = isLandingCustom ? 'block' : 'none';

    UI.modals.custom.classList.add('active');
}

function saveCustom(specs) {
    if (!specs) return;
    specs.vendor = inferVendor(specs.processor);
    specs.gen = inferGen(specs.processor);
    if (!specs.os) specs.os = 'Windows 11 Home'; 

    State.currentSpecs = { ...specs };
    localStorage.setItem('customSpecs', JSON.stringify(State.currentSpecs));
    updateUI(State.currentSpecs);
}

function inferVendor(name) {
    const n = name.toLowerCase();
    if (n.includes('intel')) return 'Intel';
    if (n.includes('amd')) return 'AMD';
    return 'Generic';
}

function inferGen(name) {
    const n = (name || '').toLowerCase();
    
    // Intel Core Ultra
    if (n.includes('ultra')) return 'Core Ultra';

    // New Intel Core naming (Core 3, 5, 7)
    // Matches "Core 5 123H" or "Core 5-123" -> captures the first digit of the model as Series
    const coreMatch = n.match(/core\s+[357]\s+(\d)/);
    if (coreMatch) return `Series ${coreMatch[1]}`;

    // Traditional Core i-series (i3-12100 -> 12a Gen)
    const intelMatch = n.match(/i[3579]-(\d{1,2})/);
    if (intelMatch) return intelMatch[1] + 'ª Gen';
    
    // AMD Ryzen
    const amdMatch = n.match(/ryzen\s+[3579]\s+(\d)/);
    if (amdMatch) return amdMatch[1] + '000 Series';
    
    return '';
}

function checkSpecsOverflow() {
    // Disabled: user requested all specs to be always visible
    return;
}

function updateModalSpecs() {
    document.getElementById('modal-cpu').textContent = UI.display.processor.textContent;
    document.getElementById('modal-ram').textContent = UI.display.ram.textContent;
    document.getElementById('modal-storage').textContent = UI.display.storage.textContent;
    document.getElementById('modal-gpu').textContent = UI.display.gpu.textContent;
    document.getElementById('modal-screen').textContent = UI.display.screen.textContent;
    document.getElementById('modal-os').textContent = UI.display.os.textContent;
}

init();
