import { defineStore } from 'pinia';
import { ref, reactive } from 'vue';

async function invokeTauri(cmd, args = {}) {
  try {
    if (window.__TAURI__ && window.__TAURI__.core) {
      return await window.__TAURI__.core.invoke(cmd, args);
    }
    const { invoke } = await import('@tauri-apps/api/core');
    return await invoke(cmd, args);
  } catch (err) {
    console.warn(`[Zenit Native] Error ejecutando comando '${cmd}':`, err);
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
  const clean = path.replace(/\\/g, '/');
  return `asset://localhost/${clean}`;
}

export const useSpecsStore = defineStore('specs', () => {
  const currentSpecs = ref({
    brand: 'Laptop Showcase',
    model: 'Serie Demo',
    processor: 'Intel Core Ultra 7 155H',
    cores: 16,
    threads: 22,
    gen: 'Core Ultra',
    vendor: 'Intel',
    ram: '16 GB',
    ram_type: 'LPDDR5X',
    storage: '1TB NVMe SSD',
    display: '2880 x 1800 (2.8K OLED) @ 120Hz',
    refresh_rate: 120,
    gpu: 'Intel Arc Graphics',
    os: 'Windows 11 Home',
    battery: '85% (~6h 30m)',
    battery_percent: 85,
    is_charging: false,
    other_specs: 'Wi-Fi 6E / Teclado Retroiluminado / Chasis de Aluminio'
  });

  const autoDetectedSpecs = ref({});
  const appConfig = ref({
    primary_color: '#00f2ff',
    secondary_color: '#0072ff',
    accent_color: '#00e5ff',
    bg_gradient: 'radial-gradient(circle at center, #111422 0%, #06070a 100%)',
    store_theme: 'default',
    price_card: '',
    price_all_methods: '',
    sku: '',
    brand_override: null,
    model_override: null,
    processor_override: null,
    ram_override: null,
    storage_override: null,
    display_override: null,
    gpu_override: null,
    os_override: null,
    battery_override: null,
    other_specs: 'Wi-Fi 6E / Bluetooth 5.3 / Teclado en Español',
    central_video_path: 'assets/videos/landing.mp4',
    bg_video_path: 'assets/videos/bg.mp4',
    inactivity_video_path: 'assets/videos/promo.mp4',
    inactivity_timeout_sec: 120,
  });

  const vaultVideos = ref([]);
  const isVideoMode = ref(false); // Inactivity screensaver
  const isModalOpen = ref(false);
  const isLoading = ref(true);

  const applyColors = () => {
    const root = document.documentElement;
    if (appConfig.value.primary_color) {
      root.style.setProperty('--primary', appConfig.value.primary_color);
    }
    if (appConfig.value.secondary_color) {
      root.style.setProperty('--secondary', appConfig.value.secondary_color);
    }
    if (appConfig.value.accent_color) {
      root.style.setProperty('--accent', appConfig.value.accent_color);
    }
    if (appConfig.value.bg_gradient) {
      root.style.setProperty('--bg-gradient', appConfig.value.bg_gradient);
    }
  };

  const loadSpecs = async () => {
    isLoading.value = true;
    try {
      // 1. Load Config from Rust
      const cfg = await invokeTauri('get_app_config');
      if (cfg) {
        appConfig.value = { ...appConfig.value, ...cfg };
      }

      // 2. Load Raw and Active Hardware Specs
      const detected = await invokeTauri('get_raw_hardware_specs');
      if (detected) {
        autoDetectedSpecs.value = detected;
      }

      const active = await invokeTauri('get_system_specs');
      if (active) {
        currentSpecs.value = {
          ...currentSpecs.value,
          ...active,
          other_specs: appConfig.value.other_specs || active.other_specs || currentSpecs.value.other_specs,
        };
      }

      // 3. Load Vault Videos
      await loadVault();

      // 4. Apply dynamic colors
      applyColors();
    } catch (err) {
      console.error('Error cargando especificaciones nativas:', err);
    } finally {
      isLoading.value = false;
    }
  };

  const saveConfig = async (newCfg) => {
    appConfig.value = { ...appConfig.value, ...newCfg };
    applyColors();
    await invokeTauri('save_app_config', { config: appConfig.value });
    // Reload specs with new overrides
    const updated = await invokeTauri('get_system_specs');
    if (updated) {
      currentSpecs.value = {
        ...currentSpecs.value,
        ...updated,
        other_specs: appConfig.value.other_specs
      };
    }
  };

  const verifyPassword = async (pass) => {
    const res = await invokeTauri('check_admin_password', { password: pass });
    if (res !== null) return res;
    // Client-side fallback: accepts demo and z3n1t (case-insensitive)
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
    if (list) {
      vaultVideos.value = list;
    }
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
      const selected = await open({
        multiple: false,
        filters: [{ name: 'Videos', extensions: ['mp4', 'webm', 'mov', 'mkv'] }]
      });
      return selected;
    } catch (e) {
      console.warn("Dialog plugin no disponible:", e);
      return null;
    }
  };

  return {
    currentSpecs,
    autoDetectedSpecs,
    appConfig,
    vaultVideos,
    isVideoMode,
    isModalOpen,
    isLoading,
    applyColors,
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
