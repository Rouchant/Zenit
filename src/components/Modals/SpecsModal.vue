<script setup>
import { useSpecsStore } from '../../store/specs';

const emit = defineEmits(['close']);
const store = useSpecsStore();
const specs = store.currentSpecs;

const tryPc = () => {
    emit('close');
    window.electronAPI.minimizeApp(specs.store);
};
</script>

<template>
  <div id="specs-modal" class="modal active">
    <div class="modal-content specs-modal-content glass">
        <div class="modal-header">
            <h2>Especificaciones Técnicas</h2>
            <button class="close-btn" @click="emit('close')">&times;</button>
        </div>
        <div class="specs-full-grid">
            <div class="spec-card">
                <div class="spec-icon"><img src="/assets/ui/cpu.svg" alt="CPU"></div>
                <div class="spec-info">
                    <div class="spec-label">Procesador</div>
                    <div class="spec-value">{{ specs.processor || 'Detectando...' }}</div>
                </div>
            </div>
            <div class="spec-card">
                <div class="spec-icon"><img src="/assets/ui/ram.svg" alt="RAM"></div>
                <div class="spec-info">
                    <div class="spec-label">RAM</div>
                    <div class="spec-value">{{ specs.ram || 'Detectando...' }}</div>
                </div>
            </div>
            <div class="spec-card">
                <div class="spec-icon"><img src="/assets/ui/storage.svg" alt="SSD"></div>
                <div class="spec-info">
                    <div class="spec-label">Disco SSD</div>
                    <div class="spec-value">{{ specs.storage || 'Detectando...' }}</div>
                </div>
            </div>
            <div class="spec-card">
                <div class="spec-icon"><img src="/assets/ui/gpu.svg" alt="GPU"></div>
                <div class="spec-info">
                    <div class="spec-label">Gráficos</div>
                    <div class="spec-value">{{ specs.gpu || 'Detectando...' }}</div>
                </div>
            </div>
            <div class="spec-card">
                <div class="spec-icon"><img src="/assets/ui/screen.svg" alt="Pantalla"></div>
                <div class="spec-info">
                    <div class="spec-label">Pantalla</div>
                    <div class="spec-value">{{ specs.display || 'Detectando...' }}</div>
                </div>
            </div>
            <div class="spec-card">
                <div class="spec-icon"><img src="/assets/ui/windows-11.svg" alt="OS"></div>
                <div class="spec-info">
                    <div class="spec-label">Sistema Operativo</div>
                    <div class="spec-value">{{ specs.os || 'Detectando...' }}</div>
                </div>
            </div>
        </div>
        <div class="modal-footer" style="padding-top: 30px; display: flex; justify-content: center;">
            <button class="view-pc-btn" style="max-width: 400px;" @click="tryPc">Prueba esta PC</button>
        </div>
    </div>
  </div>
</template>
