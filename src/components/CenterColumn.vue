<script setup>
import { computed } from 'vue';
import { useSpecsStore, toVideoSrc } from '../store/specs';

const store = useSpecsStore();
const baseUrl = import.meta.env.BASE_URL;

const videoSrc = computed(() => {
  const p = store.appConfig.central_video_path;
  if (!p) return `${baseUrl}assets/videos/landing.mp4`;
  return toVideoSrc(p);
});

const hasPricingOrSku = computed(() => {
  const cfg = store.appConfig;
  return Boolean(
    (cfg.price_card && cfg.price_card.trim()) ||
    (cfg.price_all_methods && cfg.price_all_methods.trim()) ||
    (cfg.sku && cfg.sku.trim())
  );
});

const handleTryPc = () => {
  store.tryPc();
};
</script>

<template>
  <main class="center-column">
    <!-- Contenedor del Video Central -->
    <div class="video-card-container">
      <div class="video-glow-effect"></div>
      <video 
        id="landing-video"
        class="landing-video"
        autoplay 
        loop 
        muted 
        playsinline
        :key="videoSrc"
        :src="videoSrc"
        :poster="`${baseUrl}assets/images/background.png`"
      >
      </video>
    </div>

    <!-- Bloque Optativo de Precios y SKU -->
    <div v-if="hasPricingOrSku" class="pricing-sku-container">
      <div class="pricing-row">
        <!-- Precio con Tarjeta -->
        <div v-if="store.appConfig.price_card" class="price-box price-card-box">
          <span class="price-title">Precio con Tarjeta</span>
          <span class="price-amount">{{ store.appConfig.price_card }}</span>
        </div>

        <!-- Precio Todo Medio de Pago -->
        <div v-if="store.appConfig.price_all_methods" class="price-box price-all-box">
          <span class="price-title">Todo Medio de Pago</span>
          <span class="price-amount">{{ store.appConfig.price_all_methods }}</span>
        </div>
      </div>

      <!-- SKU Identificador -->
      <div v-if="store.appConfig.sku" class="sku-tag">
        <span class="sku-label">SKU:</span>
        <span class="sku-code">{{ store.appConfig.sku }}</span>
      </div>
    </div>

    <!-- Botón Central Prueba esta PC -->
    <div class="try-pc-container">
      <button id="view-pc" class="try-pc-btn" @click="handleTryPc">
        <div class="btn-glow"></div>
        <svg xmlns="http://www.w3.org/2000/svg" width="22" height="22" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.2" stroke-linecap="round" stroke-linejoin="round">
          <rect x="2" y="3" width="20" height="14" rx="2" ry="2"/>
          <line x1="8" y1="21" x2="16" y2="21"/>
          <line x1="12" y1="17" x2="12" y2="21"/>
        </svg>
        <span>Prueba esta PC</span>
      </button>
    </div>
  </main>
</template>

<style scoped>
.center-column {
  display: flex;
  flex-direction: column;
  justify-content: center;
  align-items: center;
  width: 100%;
  gap: clamp(14px, 2.2vh, 24px);
  position: relative;
  z-index: 5;
}

.video-card-container {
  position: relative;
  width: 100%;
  max-width: 680px;
  aspect-ratio: 16 / 9;
  border-radius: 20px;
  overflow: hidden;
  box-shadow: 0 15px 45px rgba(0, 0, 0, 0.7);
  border: 1px solid rgba(255, 255, 255, 0.12);
  background: #000;
}

.video-glow-effect {
  position: absolute;
  top: -20px;
  left: -20px;
  right: -20px;
  bottom: -20px;
  background: radial-gradient(circle at center, var(--primary, #00f2ff) 0%, transparent 70%);
  opacity: 0.15;
  filter: blur(25px);
  pointer-events: none;
  z-index: 0;
}

.landing-video {
  position: relative;
  z-index: 1;
  width: 100%;
  height: 100%;
  object-fit: cover;
  display: block;
}

/* Pricing and SKU */
.pricing-sku-container {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 8px;
  background: rgba(12, 16, 26, 0.75);
  border: 1px solid rgba(255, 255, 255, 0.1);
  padding: 12px 24px;
  border-radius: 16px;
  backdrop-filter: blur(16px);
  -webkit-backdrop-filter: blur(16px);
  box-shadow: 0 8px 30px rgba(0, 0, 0, 0.4);
}

.pricing-row {
  display: flex;
  align-items: center;
  gap: 20px;
}

.price-box {
  display: flex;
  flex-direction: column;
  align-items: center;
}

.price-title {
  font-size: 0.75rem;
  text-transform: uppercase;
  letter-spacing: 1px;
  color: rgba(255, 255, 255, 0.6);
  margin-bottom: 2px;
}

.price-card-box .price-amount {
  font-size: 1.45rem;
  font-weight: 800;
  color: var(--primary, #00f2ff);
  text-shadow: 0 0 12px rgba(0, 242, 255, 0.35);
}

.price-all-box .price-amount {
  font-size: 1.3rem;
  font-weight: 700;
  color: #ffffff;
}

.sku-tag {
  display: flex;
  align-items: center;
  gap: 6px;
  font-size: 0.8rem;
  color: rgba(255, 255, 255, 0.5);
  background: rgba(255, 255, 255, 0.05);
  padding: 2px 10px;
  border-radius: 20px;
}

.sku-label {
  font-weight: 600;
  color: var(--primary, #00f2ff);
}

.sku-code {
  letter-spacing: 0.5px;
}

/* Button Prueba esta PC */
.try-pc-container {
  display: flex;
  justify-content: center;
  width: 100%;
}

.try-pc-btn {
  position: relative;
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 12px;
  background: linear-gradient(135deg, var(--secondary, #0072ff) 0%, var(--primary, #00f2ff) 100%);
  color: #000000;
  border: none;
  padding: 14px 44px;
  border-radius: 50px;
  font-size: 1.15rem;
  font-weight: 800;
  letter-spacing: 0.5px;
  cursor: pointer;
  box-shadow: 0 8px 28px rgba(0, 242, 255, 0.35), 0 4px 12px rgba(0, 0, 0, 0.4);
  transition: all 0.25s cubic-bezier(0.16, 1, 0.3, 1);
}

.try-pc-btn:hover {
  transform: translateY(-2px) scale(1.03);
  box-shadow: 0 12px 35px rgba(0, 242, 255, 0.5), 0 6px 18px rgba(0, 0, 0, 0.5);
  filter: brightness(1.08);
}

.try-pc-btn:active {
  transform: translateY(1px) scale(0.98);
}
</style>
