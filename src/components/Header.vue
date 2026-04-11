<script setup>
import { computed } from 'vue';
import { useSpecsStore } from '../store/specs';

const store = useSpecsStore();

const brandLogo = computed(() => {
  const brand = (store.currentSpecs.brand || '').toLowerCase();
  const knownBrands = ['asus', 'hp', 'samsung', 'acer', 'lenovo'];
  const matched = knownBrands.find(b => brand.includes(b));
  
  if (matched) return `/assets/logos/${matched}.svg`;
  return null;
});

const storeLogo = computed(() => {
  const s = (store.currentSpecs.store || 'none').toLowerCase();
  if (s === 'none') return null;
  const ext = s === 'paris' ? 'png' : 'svg';
  return `/assets/logos/${s}.${ext}`;
});
</script>

<template>
  <header class="header">
    <div class="header-branding">
      <div class="logo-placeholder" id="header-logo-container">
        <img v-if="brandLogo" :src="brandLogo" :alt="store.currentSpecs?.brand" :class="{ 'zenit-logo': !brandLogo.includes('logos') }">
        <img v-else src="/assets/logo.png" alt="Zenit" class="zenit-logo">
      </div>
      <div class="logo-separator" v-if="storeLogo"></div>
      <div class="store-logo-placeholder" id="store-logo-container" v-if="storeLogo">
        <img :src="storeLogo" :alt="store.currentSpecs.store">
      </div>
    </div>
    <div class="brand-badge" id="display-brand">
      {{ store.currentSpecs?.brand || 'Cargando...' }}
    </div>
  </header>
</template>

<style scoped>
.zenit-logo {
  height: 40px;
}
</style>
