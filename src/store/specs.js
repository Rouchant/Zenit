import { defineStore } from 'pinia';
import { ref } from 'vue';

async function invokeTauri(cmd, args = {}) {
  try {
    if (window.__TAURI__ && window.__TAURI__.core) {
      return await window.__TAURI__.core.invoke(cmd, args);
    }
    const { invoke } = await import('@tauri-apps/api/core');
    return await invoke(cmd, args);
  } catch (err) {
    console.warn(`[Zenit Native] Error ejecutando '${cmd}':`, err);
    return null;
  }
}

export function toVideoSrc(path) {
  if (!path) return '';
  if (path.startsWith('http') || path.startsWith('assets/') || path.startsWith('/assets/')) {
    return path;
  }
  try {
    if (window.__TAURI__ && window.__TAURI__.core && window.__TAURI__.core.convertFileSrc) {
      return window.__TAURI__.core.convertFileSrc(path);
    }
  } catch (e) {}
  return `asset://localhost/${path.replace(/\\/g, '/')}`;
}

export const useSpecsStore = defineStore('specs', () => {
  // Pure declarative UI State delivered pre-computed by Rust
  const uiState = ref({
    brand_name: 'Laptop Showcase',
    brand_logo: null,
    store_logo: null,
    left_specs: [],
    right_specs: [],
    center_video: 'assets/videos/landing.mp4',
    bg_video: 'assets/images/fallback-bg/bg.mp4',
    screensaver_video: 'assets/videos/promo.mp4',
    price_card: null,
    price_all_methods: null,
    sku: null,
    has_pricing: false,
    colors: {
      primary: '#00f2ff',
      secondary: '#0072ff',
      accent: '#00e5ff',
      bg_gradient: 'radial-gradient(circle at center, #111422 0%, #06070a 100%)'
    },
    inactivity_timeout_sec: 120,
  });

  const rawHardwareSpecs = ref({});
  const appConfig = ref({});
  const vaultVideos = ref([]);
  const isVideoMode = ref(false); // Controlled natively by Rust
  const isModalOpen = ref(false);
  const isLoading = ref(true);

  const applyColors = () => {
    const root = document.documentElement;
    const c = uiState.value.colors;
    if (c.primary) root.style.setProperty('--primary', c.primary);
    if (c.secondary) root.style.setProperty('--secondary', c.secondary);
    if (c.accent) root.style.setProperty('--accent', c.accent);
    if (c.bg_gradient) root.style.setProperty('--bg-gradient', c.bg_gradient);
  };

  const loadSpecs = async () => {
    isLoading.value = true;
    try {
      // 1. Fetch Complete UI State from Smart Rust
      const state = await invokeTauri('get_zenit_ui_state');
      if (state) {
        uiState.value = state;
        applyColors();
      }

      // 2. Fetch raw specs and config for the Admin Modal editor
      const raw = await invokeTauri('get_raw_hardware_specs');
      if (raw) rawHardwareSpecs.value = raw;

      const cfg = await invokeTauri('get_app_config');
      if (cfg) appConfig.value = cfg;

      await loadVault();

      // 3. Listen to Native Events from Rust
      try {
        const { listen } = await import('@tauri-apps/api/event');
        
        // Listen to state changes broadcasted by Rust
        listen('zenit://state-changed', (event) => {
          if (event.payload) {
            uiState.value = event.payload;
            applyColors();
          }
        });

        // Listen to native Windows inactivity / screensaver toggle
        listen('zenit://screensaver-change', (event) => {
          if (!isModalOpen.value) {
            isVideoMode.value = Boolean(event.payload);
          }
        });
      } catch (e) {
        console.warn("Event listener fallback:", e);
      }
    } finally {
      isLoading.value = false;
    }
  };

  const saveConfig = async (newConfig) => {
    await invokeTauri('save_app_config', { config: newConfig });
    // State will be updated via event broadcast from Rust or direct re-fetch
    const updated = await invokeTauri('get_zenit_ui_state');
    if (updated) {
      uiState.value = updated;
      applyColors();
    }
  };

  const verifyPassword = async (pass) => {
    const res = await invokeTauri('check_admin_password', { password: pass });
    if (res !== null) return res;
    const clean = (pass || '').trim().toLowerCase();
    return clean === 'demo' || clean === 'z3n1t';
  };

  const tryPc = async () => {
    await invokeTauri('try_pc_minimize');
  };

  const restoreApp = async () => {
    await invokeTauri('restore_main_window');
  };

  const emergencyExit = async () => {
    await invokeTauri('emergency_exit');
  };

  const emergencyRestart = async () => {
    await invokeTauri('emergency_restart');
  };

  const loadVault = async () => {
    const list = await invokeTauri('get_vault_videos');
    if (list) vaultVideos.value = list;
  };

  const importVideo = async (sourcePath) => {
    const imported = await invokeTauri('import_video', { sourcePath });
    if (imported) {
      await loadVault();
      return imported;
    }
    return null;
  };

  const removeVideoFromVault = async (filename) => {
    await invokeTauri('remove_vault_video', { filename });
    await loadVault();
  };

  const pickVideoFile = async () => {
    try {
      const { open } = await import('@tauri-apps/plugin-dialog');
      return await open({
        multiple: false,
        filters: [{ name: 'Videos', extensions: ['mp4', 'webm', 'mov', 'mkv'] }]
      });
    } catch (e) {
      return null;
    }
  };

  return {
    uiState,
    rawHardwareSpecs,
    appConfig,
    vaultVideos,
    isVideoMode,
    isModalOpen,
    isLoading,
    loadSpecs,
    saveConfig,
    verifyPassword,
    tryPc,
    restoreApp,
    emergencyExit,
    emergencyRestart,
    loadVault,
    importVideo,
    removeVideoFromVault,
    pickVideoFile,
    toVideoSrc,
  };
});
