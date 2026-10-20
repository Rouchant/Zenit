<script setup>
import { computed } from 'vue';
import { useSpecsStore } from '../store/specs';
import SpecCard from './SpecCard.vue';

const store = useSpecsStore();
const baseUrl = import.meta.env.BASE_URL;

const specs = computed(() => store.currentSpecs);

const screenIcon = computed(() => `${baseUrl}assets/ui/screen.svg`);
const gpuIcon = computed(() => `${baseUrl}assets/ui/gpu.svg`);
const batteryIcon = computed(() => `${baseUrl}assets/ui/battery.svg`);
const modelIcon = computed(() => `${baseUrl}assets/ui/laptop.svg`);
const othersIcon = computed(() => `${baseUrl}assets/ui/features.svg`);

const fullModelName = computed(() => {
  if (specs.value.brand && specs.value.model && specs.value.model !== 'PC Desktop') {
    if (specs.value.model.toLowerCase().includes(specs.value.brand.toLowerCase())) {
      return specs.value.model;
    }
    return `${specs.value.brand} ${specs.value.model}`;
  }
  return specs.value.brand || 'Laptop';
});
</script>

<template>
  <aside class="column-specs right-column">
    <div class="specs-grid">
      <!-- Pantalla (Resolución + Hz) -->
      <SpecCard 
        id="card-display"
        label="Pantalla"
        :tag="specs.refresh_rate ? `${specs.refresh_rate}Hz` : ''"
        :value="specs.display"
        :icon="screenIcon"
      />

      <!-- Gráficos / GPU -->
      <SpecCard 
        id="card-gpu"
        label="Gráficos"
        :value="specs.gpu"
        :icon="gpuIcon"
      />

      <!-- Batería -->
      <SpecCard 
        id="card-battery"
        label="Batería"
        :tag="specs.is_charging ? 'Cargando' : ''"
        :value="specs.battery"
        :icon="batteryIcon"
      />

      <!-- Marca y Modelo -->
      <SpecCard 
        id="card-model"
        label="Modelo"
        :value="fullModelName"
        :icon="modelIcon"
      />

      <!-- Otros (Personalizable) -->
      <SpecCard 
        v-if="specs.other_specs"
        id="card-others"
        label="Otros"
        :value="specs.other_specs"
        :icon="othersIcon"
      />
    </div>
  </aside>
</template>

<style scoped>
.right-column {
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
