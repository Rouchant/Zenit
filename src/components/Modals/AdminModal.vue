<script setup>
import { ref, reactive, onMounted } from 'vue';
import { useSpecsStore } from '../../store/specs';

const emit = defineEmits(['close']);
const store = useSpecsStore();

const activeTab = ref('colors'); // 'colors' | 'hardware' | 'pricing' | 'videos'

// Form state cloned from current store
const formConfig = reactive({
  primary_color: store.appConfig.primary_color || '#00f2ff',
  secondary_color: store.appConfig.secondary_color || '#0072ff',
  accent_color: store.appConfig.accent_color || '#00e5ff',
  store_theme: store.appConfig.store_theme || 'default',

  price_card: store.appConfig.price_card || '',
  price_all_methods: store.appConfig.price_all_methods || '',
  sku: store.appConfig.sku || '',

  brand_override: store.currentSpecs.brand || '',
  model_override: store.currentSpecs.model || '',
  processor_override: store.currentSpecs.processor || '',
  ram_override: store.currentSpecs.ram || '',
  storage_override: store.currentSpecs.storage || '',
  display_override: store.currentSpecs.display || '',
  gpu_override: store.currentSpecs.gpu || '',
  os_override: store.currentSpecs.os || '',
  battery_override: store.currentSpecs.battery || '',
  other_specs: store.appConfig.other_specs || store.currentSpecs.other_specs || '',

  central_video_path: store.appConfig.central_video_path || 'assets/videos/landing.mp4',
  bg_video_path: store.appConfig.bg_video_path || 'assets/images/fallback-bg/bg.mp4',
  inactivity_video_path: store.appConfig.inactivity_video_path || 'assets/videos/promo.mp4',
  inactivity_timeout_sec: store.appConfig.inactivity_timeout_sec || 120,
});

// Color Presets
const presets = [
  { name: 'Zenit Cyan', primary: '#00f2ff', secondary: '#0072ff', accent: '#00e5ff' },
  { name: 'Falabella Lime', primary: '#B9D40D', secondary: '#8da10a', accent: '#d4f20d' },
  { name: 'Paris Blue', primary: '#009DE0', secondary: '#007bb0', accent: '#33bbee' },
  { name: 'Ripley Violet', primary: '#9932cc', secondary: '#7928a1', accent: '#ba55d3' },
  { name: 'ROG Cyberpunk', primary: '#ff0055', secondary: '#7b00ff', accent: '#00ffff' },
  { name: 'Matrix Emerald', primary: '#00ff66', secondary: '#008833', accent: '#66ff99' },
];

const applyPreset = (preset) => {
  formConfig.primary_color = preset.primary;
  formConfig.secondary_color = preset.secondary;
  formConfig.accent_color = preset.accent;
};

// Restore individual fields to auto-detected values
const restoreField = (field) => {
  const auto = store.autoDetectedSpecs;
  if (!auto) return;
  switch (field) {
    case 'brand':
      formConfig.brand_override = auto.brand || '';
      break;
    case 'model':
      formConfig.model_override = auto.model || '';
      break;
    case 'processor':
      formConfig.processor_override = auto.processor || '';
      break;
    case 'ram':
      formConfig.ram_override = auto.ram || '';
      break;
    case 'storage':
      formConfig.storage_override = auto.storage || '';
      break;
    case 'display':
      formConfig.display_override = auto.display || '';
      break;
    case 'gpu':
      formConfig.gpu_override = auto.gpu || '';
      break;
    case 'os':
      formConfig.os_override = auto.os || '';
      break;
    case 'battery':
      formConfig.battery_override = auto.battery || '';
      break;
  }
};

// Video Vault Handling
const isImporting = ref(false);

const handleImportVideo = async () => {
  isImporting.value = true;
  try {
    const selectedFile = await store.pickVideoFile();
    if (selectedFile) {
      const imported = await store.importVideo(selectedFile);
      if (imported) {
        // Automatically assign as central video by default
        formConfig.central_video_path = imported.path;
      }
    }
  } finally {
    isImporting.value = false;
  }
};

const assignVideo = (path, target) => {
  if (target === 'central') formConfig.central_video_path = path;
  if (target === 'bg') formConfig.bg_video_path = path;
  if (target === 'inactivity') formConfig.inactivity_video_path = path;
};

const deleteVideo = async (filename) => {
  if (confirm(`¿Eliminar este video del baúl seguro?`)) {
    await store.removeVideoFromVault(filename);
  }
};

const handleSave = async () => {
  await store.saveConfig(formConfig);
  emit('close');
};

onMounted(async () => {
  await store.loadVault();
});
</script>

<template>
  <div id="admin-modal" class="modal active">
    <div class="modal-content admin-modal-wide">
      <!-- Modal Header -->
      <div class="modal-header">
        <div class="modal-title-row">
          <div class="title-badge">ZENIT NATIVE</div>
          <h2>Panel de Configuración de Vitrina</h2>
        </div>
        <button class="close-x-btn" @click="emit('close')">&times;</button>
      </div>

      <!-- Navigation Tabs -->
      <div class="admin-tabs">
        <button 
          class="tab-btn" 
          :class="{ active: activeTab === 'colors' }"
          @click="activeTab = 'colors'"
        >
          🎨 Colores & Estilo
        </button>
        <button 
          class="tab-btn" 
          :class="{ active: activeTab === 'hardware' }"
          @click="activeTab = 'hardware'"
        >
          💻 Hardware & Specs
        </button>
        <button 
          class="tab-btn" 
          :class="{ active: activeTab === 'pricing' }"
          @click="activeTab = 'pricing'"
        >
          🏷️ Precios & SKU
        </button>
        <button 
          class="tab-btn" 
          :class="{ active: activeTab === 'videos' }"
          @click="activeTab = 'videos'"
        >
          🎬 Baúl de Videos
        </button>
      </div>

      <!-- Modal Body Tab Views -->
      <div class="modal-body-scroll">
        <!-- TAB 1: COLORES -->
        <div v-show="activeTab === 'colors'" class="tab-pane">
          <h3 class="pane-title">Paleta de Colores de la Interfaz</h3>
          <p class="pane-subtitle">Configura los tonos visuales que iluminan la vitrina en tiempo real.</p>

          <div class="colors-grid">
            <div class="color-control-card">
              <label>Color Primario (Títulos & Glow)</label>
              <div class="color-input-row">
                <input type="color" v-model="formConfig.primary_color" class="color-picker-input">
                <input type="text" v-model="formConfig.primary_color" class="color-hex-input">
              </div>
            </div>

            <div class="color-control-card">
              <label>Color Secundario (Botones & Gradiente)</label>
              <div class="color-input-row">
                <input type="color" v-model="formConfig.secondary_color" class="color-picker-input">
                <input type="text" v-model="formConfig.secondary_color" class="color-hex-input">
              </div>
            </div>

            <div class="color-control-card">
              <label>Color de Acento (Bordes & Badges)</label>
              <div class="color-input-row">
                <input type="color" v-model="formConfig.accent_color" class="color-picker-input">
                <input type="text" v-model="formConfig.accent_color" class="color-hex-input">
              </div>
            </div>
          </div>

          <h4 class="subhead">Temas Predefinidos de Retail</h4>
          <div class="presets-row">
            <button 
              v-for="p in presets" 
              :key="p.name"
              class="preset-chip"
              @click="applyPreset(p)"
            >
              <span class="preset-dot" :style="{ backgroundColor: p.primary }"></span>
              {{ p.name }}
            </button>
          </div>
        </div>

        <!-- TAB 2: HARDWARE -->
        <div v-show="activeTab === 'hardware'" class="tab-pane">
          <h3 class="pane-title">Especificaciones de la Laptop</h3>
          <p class="pane-subtitle">Detectadas automáticamente mediante el motor nativo de Rust. Puedes sobreescribir cualquier campo o restaurar el valor original con ↺.</p>

          <div class="hardware-fields-grid">
            <div class="field-item">
              <label>Marca</label>
              <div class="input-with-action">
                <input type="text" v-model="formConfig.brand_override" placeholder="HP, Lenovo, Asus...">
                <button class="restore-btn" @click="restoreField('brand')" title="Restaurar detectado">↺</button>
              </div>
            </div>

            <div class="field-item">
              <label>Modelo</label>
              <div class="input-with-action">
                <input type="text" v-model="formConfig.model_override" placeholder="Victus 15, Vivobook 16...">
                <button class="restore-btn" @click="restoreField('model')" title="Restaurar detectado">↺</button>
              </div>
            </div>

            <div class="field-item">
              <label>Procesador</label>
              <div class="input-with-action">
                <input type="text" v-model="formConfig.processor_override" placeholder="Intel Core Ultra 7 / AMD Ryzen 7...">
                <button class="restore-btn" @click="restoreField('processor')" title="Restaurar detectado">↺</button>
              </div>
            </div>

            <div class="field-item">
              <label>Memoria RAM</label>
              <div class="input-with-action">
                <input type="text" v-model="formConfig.ram_override" placeholder="16 GB DDR5...">
                <button class="restore-btn" @click="restoreField('ram')" title="Restaurar detectado">↺</button>
              </div>
            </div>

            <div class="field-item">
              <label>Almacenamiento</label>
              <div class="input-with-action">
                <input type="text" v-model="formConfig.storage_override" placeholder="512GB SSD / 1TB NVMe...">
                <button class="restore-btn" @click="restoreField('storage')" title="Restaurar detectado">↺</button>
              </div>
            </div>

            <div class="field-item">
              <label>Pantalla (Resolución & Hz)</label>
              <div class="input-with-action">
                <input type="text" v-model="formConfig.display_override" placeholder="1920 x 1080 (FHD) @ 144Hz...">
                <button class="restore-btn" @click="restoreField('display')" title="Restaurar detectado">↺</button>
              </div>
            </div>

            <div class="field-item">
              <label>Tarjeta Gráfica (GPU)</label>
              <div class="input-with-action">
                <input type="text" v-model="formConfig.gpu_override" placeholder="NVIDIA RTX 4060 / Intel Arc...">
                <button class="restore-btn" @click="restoreField('gpu')" title="Restaurar detectado">↺</button>
              </div>
            </div>

            <div class="field-item">
              <label>Sistema Operativo</label>
              <div class="input-with-action">
                <input type="text" v-model="formConfig.os_override" placeholder="Windows 11 Home...">
                <button class="restore-btn" @click="restoreField('os')" title="Restaurar detectado">↺</button>
              </div>
            </div>

            <div class="field-item">
              <label>Batería</label>
              <div class="input-with-action">
                <input type="text" v-model="formConfig.battery_override" placeholder="85% (~6h 30m)...">
                <button class="restore-btn" @click="restoreField('battery')" title="Restaurar detectado">↺</button>
              </div>
            </div>

            <div class="field-item full-width">
              <label>Otros (Características Destacadas)</label>
              <input type="text" v-model="formConfig.other_specs" placeholder="Wi-Fi 6E / Teclado Iluminado / Audio Harman Kardon...">
            </div>
          </div>
        </div>

        <!-- TAB 3: PRECIOS & SKU -->
        <div v-show="activeTab === 'pricing'" class="tab-pane">
          <h3 class="pane-title">Precios de Venta & Código SKU</h3>
          <p class="pane-subtitle">Campos <strong>completamente opcionales</strong>. Si se dejan vacíos, no aparecerá el cartel de precios en la columna central.</p>

          <div class="pricing-inputs-grid">
            <div class="field-item">
              <label>Precio con Tarjeta (Destacado)</label>
              <input type="text" v-model="formConfig.price_card" placeholder="Ej: $599.990 (CMR, Ripley, Cencosud)">
            </div>

            <div class="field-item">
              <label>Precio Todo Medio de Pago</label>
              <input type="text" v-model="formConfig.price_all_methods" placeholder="Ej: $649.990">
            </div>

            <div class="field-item">
              <label>Código SKU de la Tienda</label>
              <input type="text" v-model="formConfig.sku" placeholder="Ej: SKU-892184-HP">
            </div>
          </div>
        </div>

        <!-- TAB 4: BAUL DE VIDEOS -->
        <div v-show="activeTab === 'videos'" class="tab-pane">
          <div class="vault-header-row">
            <div>
              <h3 class="pane-title">Baúl Seguro de Videos</h3>
              <p class="pane-subtitle">Los videos importados se copian a una zona protegida del sistema (%APPDATA%/Zenit/vault), impidiendo que el público o usuarios de vitrina los borren por accidente.</p>
            </div>
            <button class="btn primary import-btn" :disabled="isImporting" @click="handleImportVideo">
              {{ isImporting ? 'Copiando al Baúl...' : '+ Importar Video al Baúl' }}
            </button>
          </div>

          <!-- Video Assignments -->
          <div class="video-assignment-cards">
            <div class="assignment-card">
              <h4>Video Central (Columna Media)</h4>
              <p class="active-path-text">{{ formConfig.central_video_path }}</p>
              <div class="assignment-buttons">
                <button class="pill-btn" @click="formConfig.central_video_path = 'assets/videos/landing.mp4'">Predeterminado 1</button>
                <button class="pill-btn" @click="formConfig.central_video_path = 'assets/videos/landing2.mp4'">Predeterminado 2</button>
              </div>
            </div>

            <div class="assignment-card">
              <h4>Video de Fondo</h4>
              <p class="active-path-text">{{ formConfig.bg_video_path }}</p>
              <div class="assignment-buttons">
                <button class="pill-btn" @click="formConfig.bg_video_path = 'assets/images/fallback-bg/bg.mp4'">Fondo Predeterminado</button>
                <button class="pill-btn" @click="formConfig.bg_video_path = 'assets/videos/bg3.mp4'">Fondo Aurora</button>
              </div>
            </div>

            <div class="assignment-card">
              <h4>Salvapantallas (Inactividad)</h4>
              <p class="active-path-text">{{ formConfig.inactivity_video_path }}</p>
              <div class="timeout-picker">
                <label>Tiempo inactividad:</label>
                <select v-model="formConfig.inactivity_timeout_sec" class="custom-select-small">
                  <option :value="30">30 segundos</option>
                  <option :value="60">1 minuto</option>
                  <option :value="120">2 minutos</option>
                  <option :value="180">3 minutos</option>
                  <option :value="300">5 minutos</option>
                </select>
              </div>
            </div>
          </div>

          <!-- Vault List -->
          <h4 class="subhead">Archivos en el Baúl Seguro</h4>
          <div v-if="store.vaultVideos.length === 0" class="empty-vault-box">
            Aún no has importado videos personalizados al baúl. Pulsa <strong>+ Importar Video al Baúl</strong> para agregar videos MP4 o WebM protegidos.
          </div>
          <div v-else class="vault-items-list">
            <div v-for="v in store.vaultVideos" :key="v.filename" class="vault-item-row">
              <div class="vault-info">
                <span class="vault-name">🎬 {{ v.name }}</span>
                <span class="vault-size">{{ v.size_mb }} MB</span>
              </div>
              <div class="vault-actions">
                <button class="mini-btn" @click="assignVideo(v.path, 'central')">Usar Central</button>
                <button class="mini-btn" @click="assignVideo(v.path, 'bg')">Usar Fondo</button>
                <button class="mini-btn" @click="assignVideo(v.path, 'inactivity')">Usar Inactividad</button>
                <button class="mini-btn delete-btn" @click="deleteVideo(v.filename)">🗑️</button>
              </div>
            </div>
          </div>
        </div>
      </div>

      <!-- Modal Footer -->
      <div class="modal-footer">
        <button class="btn secondary" @click="emit('close')">Cancelar</button>
        <button class="btn primary save-btn" @click="handleSave">Guardar Configuración</button>
      </div>
    </div>
  </div>
</template>

<style scoped>
.admin-modal-wide {
  width: 90vw;
  max-width: 980px;
  max-height: 88vh;
  display: flex;
  flex-direction: column;
  background: #0d111a;
  border: 1px solid rgba(255, 255, 255, 0.15);
  border-radius: 20px;
  box-shadow: 0 25px 60px rgba(0, 0, 0, 0.8), 0 0 30px rgba(0, 242, 255, 0.15);
  overflow: hidden;
  color: #fff;
}

.modal-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 20px 28px 16px;
  border-bottom: 1px solid rgba(255, 255, 255, 0.08);
}

.title-badge {
  font-size: 0.7rem;
  font-weight: 800;
  letter-spacing: 2px;
  color: var(--primary, #00f2ff);
  text-transform: uppercase;
}

.modal-title-row h2 {
  font-size: 1.4rem;
  margin-top: 2px;
  font-weight: 700;
}

.close-x-btn {
  background: transparent;
  border: none;
  color: rgba(255, 255, 255, 0.6);
  font-size: 1.8rem;
  cursor: pointer;
  line-height: 1;
}

.close-x-btn:hover {
  color: #fff;
}

/* Tabs */
.admin-tabs {
  display: flex;
  background: rgba(0, 0, 0, 0.35);
  padding: 6px 28px;
  gap: 12px;
  border-bottom: 1px solid rgba(255, 255, 255, 0.06);
}

.tab-btn {
  background: transparent;
  border: none;
  color: rgba(255, 255, 255, 0.6);
  padding: 10px 18px;
  font-size: 0.95rem;
  font-weight: 600;
  border-radius: 10px;
  cursor: pointer;
  transition: all 0.2s ease;
}

.tab-btn:hover {
  color: #fff;
  background: rgba(255, 255, 255, 0.05);
}

.tab-btn.active {
  color: #000;
  background: var(--primary, #00f2ff);
  box-shadow: 0 0 15px rgba(0, 242, 255, 0.3);
}

/* Modal Body */
.modal-body-scroll {
  flex: 1;
  overflow-y: auto;
  padding: 24px 28px;
}

.pane-title {
  font-size: 1.2rem;
  font-weight: 700;
  margin-bottom: 4px;
}

.pane-subtitle {
  font-size: 0.88rem;
  color: rgba(255, 255, 255, 0.6);
  margin-bottom: 20px;
}

.subhead {
  font-size: 1rem;
  margin-top: 24px;
  margin-bottom: 12px;
  font-weight: 600;
  color: rgba(255, 255, 255, 0.9);
}

/* Tab 1: Colors */
.colors-grid {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(260px, 1fr));
  gap: 16px;
}

.color-control-card {
  background: rgba(255, 255, 255, 0.03);
  border: 1px solid rgba(255, 255, 255, 0.08);
  border-radius: 12px;
  padding: 14px;
}

.color-control-card label {
  display: block;
  font-size: 0.82rem;
  color: rgba(255, 255, 255, 0.7);
  margin-bottom: 8px;
}

.color-input-row {
  display: flex;
  align-items: center;
  gap: 12px;
}

.color-picker-input {
  width: 44px;
  height: 40px;
  border: none;
  border-radius: 8px;
  cursor: pointer;
  background: transparent;
}

.color-hex-input {
  flex: 1;
  background: rgba(0, 0, 0, 0.5);
  border: 1px solid rgba(255, 255, 255, 0.15);
  border-radius: 8px;
  padding: 8px 12px;
  color: #fff;
  font-family: monospace;
}

.presets-row {
  display: flex;
  flex-wrap: wrap;
  gap: 10px;
}

.preset-chip {
  display: flex;
  align-items: center;
  gap: 8px;
  background: rgba(255, 255, 255, 0.06);
  border: 1px solid rgba(255, 255, 255, 0.12);
  border-radius: 20px;
  padding: 8px 16px;
  color: #fff;
  cursor: pointer;
  font-size: 0.85rem;
  transition: all 0.2s ease;
}

.preset-chip:hover {
  background: rgba(255, 255, 255, 0.12);
  border-color: #fff;
}

.preset-dot {
  width: 12px;
  height: 12px;
  border-radius: 50%;
}

/* Tab 2: Hardware */
.hardware-fields-grid {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 14px;
}

.field-item {
  display: flex;
  flex-direction: column;
  gap: 6px;
}

.field-item.full-width {
  grid-column: 1 / -1;
}

.field-item label {
  font-size: 0.82rem;
  color: rgba(255, 255, 255, 0.7);
}

.input-with-action {
  display: flex;
  gap: 8px;
}

.input-with-action input,
.field-item input {
  flex: 1;
  background: rgba(0, 0, 0, 0.4);
  border: 1px solid rgba(255, 255, 255, 0.15);
  border-radius: 8px;
  padding: 9px 12px;
  color: #fff;
  font-size: 0.9rem;
}

.input-with-action input:focus,
.field-item input:focus {
  border-color: var(--primary, #00f2ff);
  outline: none;
}

.restore-btn {
  background: rgba(255, 255, 255, 0.08);
  border: 1px solid rgba(255, 255, 255, 0.15);
  color: #fff;
  border-radius: 8px;
  padding: 0 12px;
  cursor: pointer;
  font-size: 1rem;
}

.restore-btn:hover {
  background: var(--primary, #00f2ff);
  color: #000;
}

/* Tab 3: Pricing */
.pricing-inputs-grid {
  display: grid;
  grid-template-columns: 1fr 1fr 1fr;
  gap: 16px;
}

/* Tab 4: Vault */
.vault-header-row {
  display: flex;
  justify-content: space-between;
  align-items: flex-start;
  gap: 20px;
}

.import-btn {
  white-space: nowrap;
}

.video-assignment-cards {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(260px, 1fr));
  gap: 14px;
  margin-top: 16px;
}

.assignment-card {
  background: rgba(255, 255, 255, 0.03);
  border: 1px solid rgba(255, 255, 255, 0.08);
  border-radius: 12px;
  padding: 14px;
}

.assignment-card h4 {
  font-size: 0.9rem;
  margin-bottom: 4px;
}

.active-path-text {
  font-size: 0.75rem;
  color: var(--primary, #00f2ff);
  word-break: break-all;
  margin-bottom: 10px;
}

.assignment-buttons {
  display: flex;
  gap: 8px;
}

.pill-btn {
  background: rgba(255, 255, 255, 0.08);
  border: 1px solid rgba(255, 255, 255, 0.15);
  color: #fff;
  border-radius: 12px;
  padding: 4px 10px;
  font-size: 0.75rem;
  cursor: pointer;
}

.pill-btn:hover {
  background: rgba(255, 255, 255, 0.2);
}

.timeout-picker {
  display: flex;
  align-items: center;
  gap: 8px;
  margin-top: 8px;
}

.timeout-picker label {
  font-size: 0.8rem;
  color: rgba(255, 255, 255, 0.7);
}

.custom-select-small {
  background: rgba(0, 0, 0, 0.5);
  border: 1px solid rgba(255, 255, 255, 0.2);
  color: #fff;
  border-radius: 6px;
  padding: 4px 8px;
  font-size: 0.82rem;
}

.empty-vault-box {
  background: rgba(255, 255, 255, 0.02);
  border: 1px dashed rgba(255, 255, 255, 0.15);
  border-radius: 12px;
  padding: 24px;
  text-align: center;
  font-size: 0.88rem;
  color: rgba(255, 255, 255, 0.6);
}

.vault-items-list {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.vault-item-row {
  display: flex;
  justify-content: space-between;
  align-items: center;
  background: rgba(255, 255, 255, 0.04);
  border: 1px solid rgba(255, 255, 255, 0.08);
  border-radius: 10px;
  padding: 10px 14px;
}

.vault-info {
  display: flex;
  align-items: center;
  gap: 12px;
}

.vault-name {
  font-size: 0.9rem;
  font-weight: 600;
}

.vault-size {
  font-size: 0.75rem;
  color: rgba(255, 255, 255, 0.5);
}

.vault-actions {
  display: flex;
  gap: 6px;
}

.mini-btn {
  background: rgba(255, 255, 255, 0.08);
  border: 1px solid rgba(255, 255, 255, 0.15);
  color: #fff;
  border-radius: 6px;
  padding: 4px 8px;
  font-size: 0.75rem;
  cursor: pointer;
}

.mini-btn:hover {
  background: var(--primary, #00f2ff);
  color: #000;
}

.mini-btn.delete-btn:hover {
  background: #ff3355;
  color: #fff;
}

/* Modal Footer */
.modal-footer {
  display: flex;
  justify-content: flex-end;
  gap: 14px;
  padding: 16px 28px;
  border-top: 1px solid rgba(255, 255, 255, 0.08);
  background: rgba(0, 0, 0, 0.2);
}

.save-btn {
  padding: 10px 24px;
  font-weight: 700;
}
</style>
