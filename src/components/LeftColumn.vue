<script setup>
import { computed } from 'vue';
import { useSpecsStore } from '../store/specs';
import SpecCard from './SpecCard.vue';

const store = useSpecsStore();
const baseUrl = import.meta.env.BASE_URL;

const specs = computed(() => store.currentSpecs);

const procIcon = computed(() => {
  const v = (specs.value.vendor || '').toLowerCase();
  const folder = (v === 'intel' || v === 'amd') ? 'logos' : 'ui';
  const icon = (v === 'intel' || v === 'amd') ? v : 'cpu';
  return `${baseUrl}assets/${folder}/${icon}.svg`;
});

const osLogo = computed(() => `${baseUrl}assets/ui/windows-11.svg`);
</script>

<template>
  <aside class="column-specs left-column">
    <div class="specs-grid">
      <!-- Procesador -->
      <SpecCard 
        id="card-processor"
        label="Procesador"
        :tag="specs.gen"
        :value="specs.processor"
        :subValue="specs.cores ? `${specs.cores} Núcleos / ${specs.threads} Hilos` : ''"
        :icon="procIcon"
      />
      
      <!-- Memoria RAM -->
      <SpecCard 
        id="card-ram"
        label="Memoria RAM"
        :tag="specs.ram_type"
        :value="specs.ram"
        :icon="`${baseUrl}assets/ui/ram.svg`"
      />

      <!-- Almacenamiento -->
      <SpecCard 
        id="card-storage"
        label="Almacenamiento"
        :value="specs.storage"
        :icon="`${baseUrl}assets/ui/storage.svg`"
      />

      <!-- Sistema Operativo -->
      <SpecCard 
        id="card-os"
        label="Sistema Operativo"
        :value="specs.os"
        :icon="osLogo"
      />
    </div>
  </aside>
</template>

<style scoped>
.left-column {
  display: flex;
  flex-direction: column;
  justify-content: center;
  width: 100%;
}
.specs-grid {
  display: flex;
  flex-direction: column;
  gap: clamp(10px, 1.4vh, 16px);
}
</style>
