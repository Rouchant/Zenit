<script setup>
import { ref, reactive, onMounted } from 'vue';
import { useSpecsStore } from '../../store/specs';

const emit = defineEmits(['close']);
const store = useSpecsStore();

const editableSpecs = reactive({ ...store.currentSpecs });

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
                    <section class="settings-section">
                        <h3 class="section-title">Configuración de Hardware</h3>
                        <div class="hardware-grid">
                            <div class="input-group">
                                <label for="brand-input">Modelo</label>
                                <div class="input-with-action">
                                    <input id="brand-input" name="brand" type="text" v-model="editableSpecs.brand">
                                    <button class="restore-btn" @click="restoreField('brand')" title="Restaurar">↺</button>
                                </div>
                            </div>
                            <div class="input-group">
                                <label for="processor-input">Procesador</label>
                                <div class="input-with-action">
                                    <input id="processor-input" name="processor" type="text" v-model="editableSpecs.processor">
                                    <button class="restore-btn" @click="restoreField('processor')" title="Restaurar">↺</button>
                                </div>
                            </div>
                            <div class="input-group">
                                <label for="ram-input">RAM (Capacidad)</label>
                                <div class="input-with-action">
                                    <input id="ram-input" name="ram" type="text" v-model="editableSpecs.ram">
                                    <button class="restore-btn" @click="restoreField('ram')" title="Restaurar">↺</button>
                                </div>
                            </div>
                            <div class="input-group">
                                <label for="ram-type-input">Tipo RAM (DDR4/5)</label>
                                <div class="input-with-action">
                                    <input id="ram-type-input" name="ramType" type="text" v-model="editableSpecs.ramType">
                                    <button class="restore-btn" @click="restoreField('ramType')" title="Restaurar">↺</button>
                                </div>
                            </div>
                            <div class="input-group">
                                <label for="storage-input">Almacenamiento</label>
                                <div class="input-with-action">
                                    <input id="storage-input" name="storage" type="text" v-model="editableSpecs.storage">
                                    <button class="restore-btn" @click="restoreField('storage')" title="Restaurar">↺</button>
                                </div>
                            </div>
                            <div class="input-group">
                                <label for="gpu-input">Gráficos</label>
                                <div class="input-with-action">
                                    <input id="gpu-input" name="gpu" type="text" v-model="editableSpecs.gpu">
                                    <button class="restore-btn" @click="restoreField('gpu')" title="Restaurar">↺</button>
                                </div>
                            </div>
                            <div class="input-group">
                                <label for="display-input">Pantalla</label>
                                <div class="input-with-action">
                                    <input id="display-input" name="display" type="text" v-model="editableSpecs.display">
                                    <button class="restore-btn" @click="restoreField('display')" title="Restaurar">↺</button>
                                </div>
                            </div>
                            <div class="input-group">
                                <label for="os-input">Sistema Operativo</label>
                                <div class="input-with-action">
                                    <input id="os-input" name="os" type="text" v-model="editableSpecs.os">
                                    <button class="restore-btn" @click="restoreField('os')" title="Restaurar">↺</button>
                                </div>
                            </div>
                            <div class="input-group">
                                <label for="store-select">Retail / Tienda</label>
                                <select id="store-select" name="store" v-model="editableSpecs.store" class="custom-select">
                                    <option value="none">Ninguna</option>
                                    <option value="falabella">Falabella</option>
                                    <option value="paris">Paris</option>
                                    <option value="ripley">Ripley</option>
                                </select>
                            </div>
                        </div>
                    </section>
                </div>

                <div class="modal-pane-divider"></div>

                <!-- Column 2: Video & Prices -->
                <div class="modal-pane-right">
                    <section class="settings-section">
                        <h3 class="section-title">Contenido Visual</h3>
                        <div class="video-settings-grid">
                            <div class="video-section">
                                <h4 class="video-section-title">Video Inactividad (Ad)</h4>
                                <div class="video-control-row">
                                    <div class="video-control-toggle">
                                        <label class="video-option-pill" for="video-type-default">
                                            <input id="video-type-default" name="videoType" type="radio" value="default" v-model="editableSpecs.videoType">
                                            <span class="pill-label">Original</span>
                                        </label>
                                        <label class="video-option-pill" for="video-type-custom">
                                            <input id="video-type-custom" name="videoType" type="radio" value="custom" v-model="editableSpecs.videoType">
                                            <span class="pill-label">Personalizado</span>
                                        </label>
                                    </div>
                                    <button 
                                        v-if="editableSpecs.videoType === 'custom'" 
                                        class="btn btn-secondary btn-mini select-file-btn" 
                                        @click="selectVideo('inactivity')"
                                    >Subir Video</button>
                                </div>
                                <div class="video-path-badge">{{ formatPath(editableSpecs.customVideoPath) }}</div>
                            </div>

                            <div class="video-section">
                                <h4 class="video-section-title">Video Home (App)</h4>
                                <div class="video-control-row">
                                    <div class="video-control-toggle">
                                        <label class="video-option-pill" for="landing-video-type-default">
                                            <input id="landing-video-type-default" name="landingVideoType" type="radio" value="default" v-model="editableSpecs.landingVideoType">
                                            <span class="pill-label">Original</span>
                                        </label>
                                        <label class="video-option-pill" for="landing-video-type-custom">
                                            <input id="landing-video-type-custom" name="landingVideoType" type="radio" value="custom" v-model="editableSpecs.landingVideoType">
                                            <span class="pill-label">Personalizado</span>
                                        </label>
                                    </div>
                                    <button 
                                        v-if="editableSpecs.landingVideoType === 'custom'" 
                                        class="btn btn-secondary btn-mini select-file-btn" 
                                        @click="selectVideo('landing')"
                                    >Subir Video</button>
                                </div>
                                <div class="video-path-badge">{{ formatPath(editableSpecs.customLandingVideoPath) }}</div>
                            </div>
                        </div>
                    </section>

                    <section class="settings-section mt-lg">
                        <h3 class="section-title">Configuración de Precios</h3>
                        <div class="price-settings-zone">
                            <div class="input-group">
                                <label for="price-primary">Precio Primario (Oferta)</label>
                                <div class="input-with-action">
                                    <input id="price-primary" name="pricePrimary" type="text" v-model="editableSpecs.pricePrimary" placeholder="Ej: $899.990">
                                </div>
                            </div>
                            <div class="input-group">
                                <label for="price-secondary">Precio Secundario (Normal)</label>
                                <div class="input-with-action">
                                    <input id="price-secondary" name="priceSecondary" type="text" v-model="editableSpecs.priceSecondary" placeholder="Ej: $1.099.990">
                                </div>
                            </div>
                            <div class="input-group checkbox-group no-label">
                                <label for="price-strike-checkbox" class="checkbox-container">
                                    <input id="price-strike-checkbox" name="priceStrike" type="checkbox" v-model="editableSpecs.priceStrike">
                                    <span class="checkmark"></span>
                                    Tachar precio secundario
                                </label>
                            </div>
                        </div>
                    </section>
                </div>
            </div>
        </div>

        <div class="modal-actions">
            <button class="btn primary save-all-btn" @click="save">Guardar Cambios</button>
            <button class="btn secondary" @click="emit('close')">Cerrar</button>
        </div>
    </div>
  </div>
</template>
