<script setup>
import { computed } from 'vue';
import { useSpecsStore } from '../store/specs';
import SpecCard from './SpecCard.vue';

const store = useSpecsStore();
const emit = defineEmits(['open-specs']);

const specs = computed(() => store.currentSpecs);

const procIcon = computed(() => {
  const v = (specs.value.vendor || '').toLowerCase();
  const folder = (v === 'intel' || v === 'amd') ? 'logos' : 'ui';
  const icon = (v === 'intel' || v === 'amd') ? v : 'cpu';
  return `/assets/${folder}/${icon}.svg`;
});

const osLogo = computed(() => '/assets/ui/windows-11.svg');

const tryPc = () => {
    window.electronAPI.minimizeApp(store.currentSpecs.store);
};
</script>

<template>
  <div class="specs-column">
    <div class="specs-grid">
      <SpecCard 
        id="card-processor"
        label="Procesador"
        :tag="specs.gen"
        :value="specs.processor"
        :subValue="specs.cores ? `${specs.cores} Núcleos / ${specs.threads} Hilos` : ''"
        :icon="procIcon"
      />
      
      <SpecCard 
        id="card-ram"
        label="Memoria RAM"
        :tag="specs.ramType"
        :value="specs.ram"
        icon="/assets/ui/ram.svg"
      />

      <SpecCard 
        id="card-storage"
        label="Almacenamiento"
        :value="specs.storage"
        icon="/assets/ui/storage.svg"
      />

      <SpecCard 
        id="card-display"
        label="Pantalla"
        :value="specs.display"
        icon="/assets/ui/screen.svg"
      />

      <SpecCard 
        id="card-gpu"
        label="Gráficos"
        :value="specs.gpu"
        icon="/assets/ui/gpu.svg"
      />

      <SpecCard 
        id="card-os"
        label="Sistema Operativo"
        :value="specs.os"
        :icon="osLogo"
      />
    </div>
    
    <button id="view-pc" class="view-pc-btn" @click="tryPc">Prueba esta PC</button>
  </div>
</template>
