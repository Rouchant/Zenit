<script setup>
import { ref, reactive, onMounted } from 'vue';
import { useSpecsStore } from '../../store/specs';

const emit = defineEmits(['close']);
const store = useSpecsStore();

const editableSpecs = reactive({ ...store.currentSpecs });
const systemStatus = ref('');

const formatPath = (fullPath) => {
    if (!fullPath) return 'Sin archivo';
    const fileName = fullPath.split(/[/\\]/).pop();
    if (fileName.length <= 15) return fileName;
    return fileName.substring(0, 12) + '...';
};

const save = () => {
    store.saveCustom(editableSpecs);
    emit('close');
};

const restoreField = (field) => {
    editableSpecs[field] = store.autoDetectedSpecs[field] || '';
};

const selectVideo = async (type) => {
    const path = await window.electronAPI.selectVideo();
    if (path) {
        const safePath = await window.electronAPI.saveCustomVideo(path);
        if (safePath) {
            if (type === 'inactivity') {
                editableSpecs.customVideoPath = safePath;
                editableSpecs.videoType = 'custom';
            } else {
                editableSpecs.customLandingVideoPath = safePath;
                editableSpecs.landingVideoType = 'custom';
            }
        }
    }
};

const setupAutostart = async (enable) => {
    systemStatus.value = 'Procesando...';
    try {
        if (enable) {
            await window.electronAPI.setupAutostart();
            systemStatus.value = 'Autostart activado correctamente.';
        } else {
            const result = await window.electronAPI.removeAutostart();
            systemStatus.value = result.success ? 'Autostart desactivado.' : (result.message || 'No se encontró el autostart.');
        }
        setTimeout(() => systemStatus.value = '', 3000);
    } catch (e) {
        systemStatus.value = 'Error en la operación de sistema.';
    }
};
</script>

<template>
  <div id="custom-modal" class="modal active">
    <div class="modal-content wide-modal">
        <div class="modal-header-main">
            <div class="header-title-row">
                <h2>Personalizar Zenit</h2>
            </div>
        </div>

        <div class="modal-body-scroll">
            <div class="settings-grid">
                <!-- Column 1: Hardware -->
                <div class="modal-pane-left">
                    <div class="hardware-grid">
                        <div class="input-group">
                            <label>Marca</label>
                            <div class="input-with-action">
                                <input type="text" v-model="editableSpecs.brand">
                                <button class="restore-btn" @click="restoreField('brand')" title="Restaurar">↺</button>
                            </div>
                        </div>
                        <div class="input-group">
                            <label>Procesador</label>
                            <div class="input-with-action">
                                <input type="text" v-model="editableSpecs.processor">
                                <button class="restore-btn" @click="restoreField('processor')" title="Restaurar">↺</button>
                            </div>
                        </div>
                        <div class="input-group">
                            <label>RAM (Capacidad)</label>
                            <div class="input-with-action">
                                <input type="text" v-model="editableSpecs.ram">
                                <button class="restore-btn" @click="restoreField('ram')" title="Restaurar">↺</button>
                            </div>
                        </div>
                        <div class="input-group">
                            <label>Tipo RAM (DDR4/5)</label>
                            <div class="input-with-action">
                                <input type="text" v-model="editableSpecs.ramType">
                                <button class="restore-btn" @click="restoreField('ramType')" title="Restaurar">↺</button>
                            </div>
                        </div>
                        <div class="input-group">
                            <label>Almacenamiento</label>
                            <div class="input-with-action">
                                <input type="text" v-model="editableSpecs.storage">
                                <button class="restore-btn" @click="restoreField('storage')" title="Restaurar">↺</button>
                            </div>
                        </div>
                        <div class="input-group">
                            <label>Gráficos</label>
                            <div class="input-with-action">
                                <input type="text" v-model="editableSpecs.gpu">
                                <button class="restore-btn" @click="restoreField('gpu')" title="Restaurar">↺</button>
                            </div>
                        </div>
                        <div class="input-group">
                            <label>Pantalla</label>
                            <div class="input-with-action">
                                <input type="text" v-model="editableSpecs.display">
                                <button class="restore-btn" @click="restoreField('display')" title="Restaurar">↺</button>
                            </div>
                        </div>
                        <div class="input-group">
                            <label>Retail</label>
                            <select v-model="editableSpecs.store" class="custom-select">
                                <option value="none">Ninguna</option>
                                <option value="falabella">Falabella</option>
                                <option value="paris">Paris</option>
                                <option value="ripley">Ripley</option>
                            </select>
                        </div>
                        <div class="input-group">
                            <label>Sistema Operativo</label>
                            <div class="input-with-action">
                                <input type="text" v-model="editableSpecs.os">
                                <button class="restore-btn" @click="restoreField('os')" title="Restaurar">↺</button>
                            </div>
                        </div>
                        <div class="input-group">
                            <label>Precio Primario (Oferta)</label>
                            <div class="input-with-action">
                                <input type="text" v-model="editableSpecs.pricePrimary" placeholder="Ej: $899.990">
                            </div>
                        </div>
                        <div class="input-group">
                            <label>Precio Secundario (Normal)</label>
                            <div class="input-with-action">
                                <input type="text" v-model="editableSpecs.priceSecondary" placeholder="Ej: $1.099.990">
                            </div>
                        </div>
                        <div class="input-group checkbox-group">
                            <label>&nbsp;</label>
                            <label class="checkbox-container">
                                <input type="checkbox" v-model="editableSpecs.priceStrike">
                                <span class="checkmark"></span>
                                Tachar precio secundario
                            </label>
                        </div>
                    </div>
                </div>

                <div class="modal-pane-divider"></div>

                <!-- Column 2: Video -->
                <div class="modal-pane-right">
                    <div class="video-settings-grid">
                        <div class="video-section">
                            <h3>Video Inactividad (Ad)</h3>
                            <div class="video-control-row">
                                <div class="video-control-toggle">
                                    <label class="video-option-pill">
                                        <input type="radio" value="default" v-model="editableSpecs.videoType">
                                        <span class="pill-label">Default</span>
                                    </label>
                                    <label class="video-option-pill">
                                        <input type="radio" value="custom" v-model="editableSpecs.videoType">
                                        <span class="pill-label">Personalizado</span>
                                    </label>
                                </div>
                                <button 
                                    v-if="editableSpecs.videoType === 'custom'" 
                                    class="btn btn-secondary btn-mini" 
                                    @click="selectVideo('inactivity')"
                                >Seleccionar...</button>
                            </div>
                            <div class="video-path-label">{{ formatPath(editableSpecs.customVideoPath) }}</div>
                        </div>

                        <div class="video-section">
                            <h3>Video Home (App)</h3>
                            <div class="video-control-row">
                                <div class="video-control-toggle">
                                    <label class="video-option-pill">
                                        <input type="radio" value="default" v-model="editableSpecs.landingVideoType">
                                        <span class="pill-label">Default</span>
                                    </label>
                                    <label class="video-option-pill">
                                        <input type="radio" value="custom" v-model="editableSpecs.landingVideoType">
                                        <span class="pill-label">Personalizado</span>
                                    </label>
                                </div>
                                <button 
                                    v-if="editableSpecs.landingVideoType === 'custom'" 
                                    class="btn btn-secondary btn-mini" 
                                    @click="selectVideo('landing')"
                                >Seleccionar...</button>
                            </div>
                            <div class="video-path-label">{{ formatPath(editableSpecs.customLandingVideoPath) }}</div>
                        </div>

                        <div class="system-section">
                            <h3>Opciones de Sistema</h3>
                            <div class="system-controls">
                                <button class="btn secondary btn-mini" @click="setupAutostart(true)">Activar Autostart</button>
                                <button class="btn secondary btn-mini" @click="setupAutostart(false)">Quitar Autostart</button>
                            </div>
                            <div class="system-status-msg text-center">{{ systemStatus }}</div>
                        </div>
                    </div>
                </div>
            </div>
        </div>

        <div class="modal-actions">
            <button class="btn primary" @click="save">Guardar</button>
            <button class="btn secondary" @click="emit('close')">Cerrar</button>
        </div>
    </div>
  </div>
</template>
