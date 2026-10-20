<template>
  <div id="app" class="app-container" :style="{ background: 'var(--bg-gradient, #06070a)' }">
    <!-- Loading Screen -->
    <Transition name="fade">
      <div v-if="store.isLoading" class="loading-screen">
        <div class="loader-container">
          <div class="loader"></div>
          <div class="loader-glow"></div>
        </div>
        <p class="loading-text">Iniciando Zenit Native...</p>
      </div>
    </Transition>

    <!-- Background Video -->
    <video 
      v-show="!store.isLoading && !store.isVideoMode"
      id="bg-video" 
      autoplay 
      loop 
      muted 
      playsinline 
      :poster="`${baseUrl}assets/images/background.png`"
      :key="bgVideoSrc"
      :src="bgVideoSrc"
      ref="bgVideo"
    >
    </video>
    
    <!-- Background Gradient & Glassmorphism Blur Overlay -->
    <div class="bg-blur"></div>
    
    <!-- Info View (Main 3-Column Showcase) -->
    <div id="info-view" v-show="!store.isVideoMode && !store.isLoading" class="view active">
      <Header />

      <main class="main-content">
        <!-- Columna Izquierda: Especificaciones Principales -->
        <LeftColumn />
        
        <!-- Columna Central (La más ancha): Video, Precios Opcionales y Botón Prueba esta PC -->
        <CenterColumn />

        <!-- Columna Derecha: Pantalla Hz, GPU, Batería, Modelo y Otros -->
        <RightColumn />
      </main>

      <footer class="footer"></footer>
    </div>

    <!-- Admin Hotspot (4 clics en la esquina superior derecha) -->
    <div 
      id="settings-hotspot" 
      class="admin-hotspot top-right" 
      @click="handleHotspotClick('settings')"
      title="Admin Hotspot (4 clics)"
    ></div>

    <!-- Video View (Salvapantallas por Inactividad) -->
    <div id="video-view" v-if="store.isVideoMode && !store.isLoading" class="view active">
       <VideoPlayer />
    </div>

    <!-- Modales -->
    <PasswordModal 
      v-if="showPasswordModal" 
      :mode="passwordMode"
      @close="showPasswordModal = false"
      @verified="onPasswordVerified"
    />

    <AdminModal 
      v-if="showAdminModal"
      @close="showAdminModal = false"
    />
  </div>
</template>

<script setup>
import { onMounted, onUnmounted, ref, watch, reactive, computed } from 'vue';
import { useSpecsStore, toVideoSrc } from './store/specs';

// Components
import Header from './components/Header.vue';
import LeftColumn from './components/LeftColumn.vue';
import CenterColumn from './components/CenterColumn.vue';
import RightColumn from './components/RightColumn.vue';
import VideoPlayer from './components/VideoPlayer.vue';
import AdminModal from './components/Modals/AdminModal.vue';
import PasswordModal from './components/Modals/PasswordModal.vue';

const store = useSpecsStore();
const baseUrl = import.meta.env.BASE_URL;

const inactivityTimer = ref(null);
const showPasswordModal = ref(false);
const showAdminModal = ref(false);
const passwordMode = ref('settings');

const bgVideo = ref(null);

const bgVideoSrc = computed(() => {
  const p = store.appConfig.bg_video_path;
  if (!p) return `${baseUrl}assets/videos/bg.mp4`;
  return toVideoSrc(p);
});

// Admin Hotspot: 4 Clics en la esquina superior derecha
const hotspotCount = ref(0);
let hotspotTimeout = null;

const handleHotspotClick = (mode) => {
  if (hotspotTimeout) clearTimeout(hotspotTimeout);

  hotspotCount.value++;

  if (hotspotCount.value >= 4) {
    hotspotCount.value = 0;
    passwordMode.value = mode;
    showPasswordModal.value = true;
  } else {
    // Reset counter if remaining clicks aren't finished in 2.5s
    hotspotTimeout = setTimeout(() => {
      hotspotCount.value = 0;
    }, 2500);
  }
};

watch(() => store.isModalOpen, (isOpen) => {
  if (isOpen) {
    bgVideo.value?.pause();
  } else {
    bgVideo.value?.play().catch(() => {});
  }
});

watch([showPasswordModal, showAdminModal], () => {
  store.isModalOpen = showPasswordModal.value || showAdminModal.value;
});

// Inactivity Screensaver Logic
let lastInteraction = 0;
const resetInactivityTimer = () => {
  const now = Date.now();
  if (now - lastInteraction < 200 && !store.isVideoMode) return;
  lastInteraction = now;

  clearTimeout(inactivityTimer.value);

  // If currently in screensaver video mode, wake up back to showcase
  if (store.isVideoMode) {
    store.isVideoMode = false;
  }

  // Calculate timeout in milliseconds from config
  const timeoutMs = (store.appConfig.inactivity_timeout_sec || 120) * 1000;

  inactivityTimer.value = setTimeout(() => {
    if (!store.isModalOpen) {
      store.isVideoMode = true;
    }
  }, timeoutMs);
};

const onPasswordVerified = () => {
  showPasswordModal.value = false;
  showAdminModal.value = true;
};

onMounted(async () => {
  await store.loadSpecs();
  resetInactivityTimer();

  window.addEventListener('mousemove', resetInactivityTimer, { passive: true });
  window.addEventListener('keydown', resetInactivityTimer, { passive: true });
  window.addEventListener('mousedown', resetInactivityTimer, { passive: true });
  window.addEventListener('touchstart', resetInactivityTimer, { passive: true });
});

onUnmounted(() => {
  window.removeEventListener('mousemove', resetInactivityTimer);
  window.removeEventListener('keydown', resetInactivityTimer);
  window.removeEventListener('mousedown', resetInactivityTimer);
  window.removeEventListener('touchstart', resetInactivityTimer);
  clearTimeout(inactivityTimer.value);
});
</script>

<style>
.loading-screen {
  position: fixed;
  top: 0; left: 0; right: 0; bottom: 0;
  background: radial-gradient(circle at center, #111422 0%, #06070a 100%);
  display: flex;
  flex-direction: column;
  justify-content: center;
  align-items: center;
  z-index: 9999;
  color: white;
}

.loader-container {
  position: relative;
  width: 60px;
  height: 60px;
  margin-bottom: 24px;
}

.loader {
  position: absolute;
  top: 0; left: 0;
  border: 2px solid rgba(0, 242, 255, 0.15);
  border-top: 2px solid var(--primary, #00f2ff);
  border-radius: 50%;
  width: 100%;
  height: 100%;
  animation: spin 1s linear infinite;
}

.loader-glow {
  position: absolute;
  top: -5px; left: -5px;
  width: calc(100% + 10px);
  height: calc(100% + 10px);
  border-radius: 50%;
  box-shadow: 0 0 25px var(--primary, #00f2ff);
  opacity: 0.25;
  animation: pulse-loader 1.5s ease-in-out infinite alternate;
}

.loading-text {
  font-size: 1.05rem;
  font-weight: 600;
  letter-spacing: 1.5px;
  color: rgba(255, 255, 255, 0.85);
  text-transform: uppercase;
}

@keyframes spin {
  0% { transform: rotate(0deg); }
  100% { transform: rotate(360deg); }
}

@keyframes pulse-loader {
  0% { transform: scale(0.95); opacity: 0.15; }
  100% { transform: scale(1.05); opacity: 0.35; }
}

.admin-hotspot.top-right {
  position: fixed;
  top: 0;
  right: 0;
  width: 90px;
  height: 90px;
  z-index: 10000;
  cursor: default;
  background: transparent;
}
</style>
