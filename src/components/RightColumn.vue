<script setup>
import { useSpecsStore } from '../store/specs';
import SpecCard from './SpecCard.vue';

const store = useSpecsStore();
const baseUrl = import.meta.env.BASE_URL;

const formatIcon = (iconPath) => {
  if (!iconPath) return `${baseUrl}assets/ui/screen.svg`;
  if (iconPath.startsWith('http') || iconPath.startsWith('/')) return iconPath;
  return `${baseUrl}${iconPath}`;
};
</script>

<template>
  <aside class="column-specs right-column">
    <div class="specs-grid">
      <SpecCard 
        v-for="card in store.uiState.right_specs"
        :key="card.id"
        :id="card.id"
        :label="card.label"
        :value="card.value"
        :subValue="card.sub_value"
        :tag="card.tag"
        :icon="formatIcon(card.icon)"
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
