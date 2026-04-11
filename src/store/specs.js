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

  const saveCustom = (specs) => {
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

    // Merge to avoid losing non-editable fields like cores/threads
    currentSpecs.value = { ...currentSpecs.value, ...specs };
    localStorage.setItem('customSpecs', JSON.stringify(currentSpecs.value));
    updateTheme(specs.store);
  };

  const loadSpecs = async () => {
    isLoading.value = true;
    try {
      if (window.electronAPI) {
        autoDetectedSpecs.value = await window.electronAPI.getSystemSpecs();
      } else {
        autoDetectedSpecs.value = {
          brand: 'Asus', processor: 'AMD Ryzen 7', ram: '16GB', storage: '512GB SSD', 
          gpu: 'Radeon Graphics', display: '1920x1080', os: 'Windows 11', cores: 8, threads: 16
        };
      }

      // Fallback for brand
      const detectedBrand = (autoDetectedSpecs.value.brand || '').toLowerCase();
      if (!detectedBrand || detectedBrand === 'system manufacturer' || detectedBrand.includes('to be filled')) {
        autoDetectedSpecs.value.brand = 'Asus';
      }

      currentSpecs.value = { ...autoDetectedSpecs.value, ...(customSpecs.value || {}) };
      updateTheme(currentSpecs.value.store);
    } catch (err) {
      console.error('Failed to load specs:', err);
    } finally {
      isLoading.value = false;
    }
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
    updateTheme
  };
});
