<script setup>
import { computed } from 'vue';
import { useSpecsStore } from '../store/specs';

const store = useSpecsStore();
const baseUrl = import.meta.env.BASE_URL;

const brandLogoSrc = computed(() => {
  if (store.uiState.brand_logo) {
    return `${baseUrl}${store.uiState.brand_logo}`;
  }
  return `${baseUrl}assets/logo.png`;
});

const storeLogoSrc = computed(() => {
  if (store.uiState.store_logo) {
    return `${baseUrl}${store.uiState.store_logo}`;
  }
  return null;
});
</script>

<template>
  <header class="header">
    <div class="header-branding">
      <div class="logo-placeholder" id="header-logo-container">
        <img :src="brandLogoSrc" :alt="store.uiState.brand_name" class="zenit-logo">
      </div>
      <div class="logo-separator" v-if="storeLogoSrc"></div>
      <div class="store-logo-placeholder" id="store-logo-container" v-if="storeLogoSrc">
        <img :src="storeLogoSrc" alt="Store Logo">
      </div>
    </div>
    <div class="brand-badge" id="display-brand">
      {{ store.uiState.brand_name || 'Cargando...' }}
    </div>
  </header>
</template>

<style scoped>
.zenit-logo {
  height: 40px;
  object-fit: contain;
}
</style>
