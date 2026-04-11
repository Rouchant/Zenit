import { defineStore } from 'pinia';
import { reactive, ref } from 'vue';

export const useSpecsStore = defineStore('specs', () => {
  const currentSpecs = ref({});
  const autoDetectedSpecs = ref({});
  const customSpecs = ref(JSON.parse(localStorage.getItem('customSpecs')) || null);
  
  const isVideoMode = ref(false);
  const isModalOpen = ref(false);
  const isLoading = ref(true);
  const theme = ref('default');
  
  const CONFIG = {
    INACTIVITY_LIMIT: 120000,
    PASSWORD: 'rogally',
    THEMES: ['falabella', 'paris', 'ripley', 'default']
  };

  const updateTheme = (storeName) => {
    const s = (storeName || 'none').toLowerCase();
    theme.value = s === 'none' ? 'default' : s;
    document.body.className = `theme-${theme.value}`;
  };

  const saveCustom = async (specs) => {
    if (!specs) return;
    
    // Infer logic
    const inferVendor = (name) => {
      const n = (name || '').toLowerCase();
      if (n.includes('intel')) return 'Intel';
      if (n.includes('amd')) return 'AMD';
      return 'Generic';
    };

    const inferGen = (name) => {
      const n = (name || '').toLowerCase();
      if (n.includes('ultra')) return 'Core Ultra';
      
      const coreMatch = n.match(/core\s+[357]\s+(\d)/);
      if (coreMatch) return `Serie ${coreMatch[1]}`;
      
      const intelMatch = n.match(/i[3579]-(\d{1,2})/);
      if (intelMatch) return intelMatch[1] + 'ª Gen';
      
      const amdMatch = n.match(/ryzen\s+[3579]\s+(\d)/);
      if (amdMatch) return amdMatch[1] + '000 Series';

      if (n.match(/n\d{3}/)) return 'N-Series';
      return '';
    };

    specs.vendor = inferVendor(specs.processor);
    specs.gen = inferGen(specs.processor);
    if (!specs.os) specs.os = 'Windows 11 Home';

    // Merge to avoid losing non-editable fields
    currentSpecs.value = { ...currentSpecs.value, ...specs };
    
    // Multi-persistence: Store in JS and File
    localStorage.setItem('customSpecs', JSON.stringify(currentSpecs.value));
    
    if (window.electronAPI && window.electronAPI.saveConfig) {
      await window.electronAPI.saveConfig(currentSpecs.value);
    }
    
    updateTheme(specs.store);
  };

  const loadSpecs = async () => {
    isLoading.value = true;
    try {
      // 1. Get Physical Backup if exists
      let backupSpecs = null;
      if (window.electronAPI && window.electronAPI.loadConfig) {
        backupSpecs = await window.electronAPI.loadConfig();
      }

      // 2. Get Auto-detected hardware
      if (window.electronAPI) {
        autoDetectedSpecs.value = await window.electronAPI.getSystemSpecs();
      } else {
        autoDetectedSpecs.value = {
          brand: 'PC Generico', processor: 'Procesador Demo', ram: '8GB', storage: '256GB SSD', 
          gpu: 'Graficos', display: '1920x1080', os: 'Windows', cores: 4, threads: 8
        };
      }

      // 3. Merge hierarchy: Auto-detected < LocalStorage < File Backup
      // This ensures File Backup is the ground truth
      const localS = JSON.parse(localStorage.getItem('customSpecs')) || {};
      currentSpecs.value = { 
        ...autoDetectedSpecs.value, 
        ...localS,
        ...(backupSpecs || {}) 
      };
      
      // Default store to 'none' if missing
      if (!currentSpecs.value.store) {
        currentSpecs.value.store = 'none';
      }

      updateTheme(currentSpecs.value.store);
    } catch (err) {
      console.error('Failed to load specs:', err);
    } finally {
      isLoading.value = false;
    }
  };

  const getVideoUrl = (path) => {
    if (!path) return '';
    // Use custom protocol to bypass CORS/WebSecurity issues in dev mode
    const cleanPath = path.replace(/\\/g, '/');
    return `zenit-file:///${cleanPath}`;
  };

  return {
    currentSpecs,
    autoDetectedSpecs,
    customSpecs,
    isVideoMode,
    isModalOpen,
    isLoading,
    theme,
    CONFIG,
    saveCustom,
    loadSpecs,
    updateTheme,
    getVideoUrl
  };
});
