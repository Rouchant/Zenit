<template>
  <div id="app" class="app-container" :style="{ backgroundColor: '#000' }">
    <!-- Loading Screen -->
    <Transition name="fade">
      <div v-if="store.isLoading" class="loading-screen">
        <div class="loader-container">
          <div class="loader"></div>
          <div class="loader-glow"></div>
        </div>
        <p class="loading-text">Cargando especificaciones...</p>
      </div>
    </Transition>

    <!-- Background Video -->
    <video 
      v-show="!store.isLoading"
      id="bg-video" 
      autoplay 
      loop 
      muted 
      playsinline 
      poster="/assets/images/background.png"
      ref="bgVideo"
    >
      <source src="/assets/videos/bg.mp4" type="video/mp4">
    </video>
    
    <!-- Background Overlay -->
    <div class="bg-blur"></div>
    
    <!-- Info View -->
    <div id="info-view" v-show="!store.isVideoMode && !store.isLoading" class="view active">
      <Header />

      <main class="main-content">
        <SpecsGrid @open-specs="showSpecsModal = true" />
        
        <div class="landing-video-container">
          <video 
            id="landing-video" 
            autoplay 
            loop 
            muted 
            playsinline 
            :src="store.currentSpecs.landingVideoType === 'custom' ? store.getVideoUrl(store.currentSpecs.customLandingVideoPath) : '/assets/videos/landing.mp4'"
            ref="landingVideo"
          >
          </video>
          <div id="display-price" class="price-tag">
             <div v-if="store.currentSpecs.priceSecondary" class="price-secondary" :class="{ strike: store.currentSpecs.priceStrike }">
               {{ store.currentSpecs.priceSecondary }}
             </div>
             <div v-if="store.currentSpecs.pricePrimary" class="price-primary">
               {{ store.currentSpecs.pricePrimary }}
             </div>
          </div>
        </div>
      </main>

      <footer class="footer"></footer>
    </div>

    <!-- Admin Hotspots (Now require 4 clicks) -->
    <div id="settings-hotspot" class="admin-hotspot top-right" @click="handleHotspotClick('settings')"></div>
    <div id="exit-hotspot" class="admin-hotspot bottom-right" @click="handleHotspotClick('exit')"></div>

    <!-- Video View (Inactivity) -->
    <div id="video-view" v-show="store.isVideoMode && !store.isLoading" class="view active">
       <VideoPlayer v-if="store.isVideoMode" />
    </div>

    <!-- Modals -->
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

    <SpecsModal 
      v-if="showSpecsModal"
      @close="showSpecsModal = false"
    />
  </div>
</template>

<script setup>
import { onMounted, onUnmounted, ref, watch, reactive } from 'vue';
import { useSpecsStore } from './store/specs';

// Components
import Header from './components/Header.vue';
import SpecsGrid from './components/SpecsGrid.vue';
import VideoPlayer from './components/VideoPlayer.vue';
import AdminModal from './components/Modals/AdminModal.vue';
import PasswordModal from './components/Modals/PasswordModal.vue';
import SpecsModal from './components/Modals/SpecsModal.vue';

const store = useSpecsStore();
const inactivityTimer = ref(null);
const showPasswordModal = ref(false);
const showAdminModal = ref(false);
const showSpecsModal = ref(false);
const passwordMode = ref('settings');

const bgVideo = ref(null);
const landingVideo = ref(null);

// Admin Hotspot Secrets
const hotspotCounts = reactive({
  settings: 0,
  exit: 0
});
let hotspotTimeout = null;

const handleHotspotClick = (mode) => {
  // Clear previous reset timer
  if (hotspotTimeout) clearTimeout(hotspotTimeout);

  // Increment specific counter
  hotspotCounts[mode]++;

  // Check if target reached
  if (hotspotCounts[mode] >= 4) {
    hotspotCounts[mode] = 0;
    openPassword(mode);
  } else {
    // Set reset timer (2 seconds)
    hotspotTimeout = setTimeout(() => {
      hotspotCounts.settings = 0;
      hotspotCounts.exit = 0;
    }, 2000);
  }
};

watch(() => store.isModalOpen, (isOpen) => {
  if (isOpen) {
    bgVideo.value?.pause();
    landingVideo.value?.pause();
  } else {
    bgVideo.value?.play().catch(() => {});
    landingVideo.value?.play().catch(() => {});
  }
});

watch([showPasswordModal, showAdminModal, showSpecsModal], () => {
  store.isModalOpen = showPasswordModal.value || showAdminModal.value || showSpecsModal.value;
});

let lastReset = 0;
const resetTimer = () => {
  const now = Date.now();
  // Optimization: Throttle events every 300ms to reduce CPU usage
  if (now - lastReset < 300 && !store.isVideoMode) return;
  lastReset = now;

  clearTimeout(inactivityTimer.value);
  if (store.isVideoMode) store.isVideoMode = false;
  inactivityTimer.value = setTimeout(() => {
    store.isVideoMode = true;
  }, store.CONFIG.INACTIVITY_LIMIT);
};

const openPassword = (mode) => {
  passwordMode.value = mode;
  showPasswordModal.value = true;
};

const onPasswordVerified = () => {
  showPasswordModal.value = false;
  if (passwordMode.value === 'exit') {
    window.electronAPI.quitApp();
  } else {
    showAdminModal.value = true;
  }
};

onMounted(async () => {
  await store.loadSpecs();
  resetTimer();

  window.addEventListener('mousemove', resetTimer);
  window.addEventListener('keydown', resetTimer);
  window.addEventListener('mousedown', resetTimer);
});

onUnmounted(() => {
  window.removeEventListener('mousemove', resetTimer);
  window.removeEventListener('keydown', resetTimer);
  window.removeEventListener('mousedown', resetTimer);
  clearTimeout(inactivityTimer.value);
});
</script>

<style>
/* Global styles are imported in main.js */
.loading-screen {
  position: fixed;
  top: 0; left: 0; right: 0; bottom: 0;
  background: radial-gradient(circle at center, #111 0%, #000 100%);
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
  margin-bottom: 30px;
}

.loader {
  position: absolute;
  top: 0; left: 0;
  border: 2px solid rgba(0, 242, 255, 0.1);
  border-top: 2px solid var(--primary, #00f2ff);
  border-radius: 50%;
  width: 100%;
  height: 100%;
  animation: spin 1s cubic-bezier(0.5, 0, 0.5, 1) infinite;
  z-index: 2;
}

.loader-glow {
  position: absolute;
  top: 0; left: 0;
  width: 100%;
  height: 100%;
  border-radius: 50%;
  background: var(--primary, #00f2ff);
  filter: blur(15px);
  opacity: 0.2;
  animation: pulse 2s ease-in-out infinite;
}

.loading-text {
  font-size: 1.1rem;
  letter-spacing: 2px;
  text-transform: uppercase;
  font-weight: 300;
  color: rgba(255, 255, 255, 0.8);
  animation: fadePulse 2s ease-in-out infinite;
}

@keyframes spin {
  0% { transform: rotate(0deg); }
  100% { transform: rotate(360deg); }
}

@keyframes pulse {
  0%, 100% { opacity: 0.1; transform: scale(0.8); }
  50% { opacity: 0.3; transform: scale(1.2); }
}

@keyframes fadePulse {
  0%, 100% { opacity: 0.5; }
  50% { opacity: 1; }
}

/* Transitions */
.fade-enter-active,
.fade-leave-active {
  transition: opacity 0.8s ease;
}

.fade-enter-from,
.fade-leave-to {
  opacity: 0;
}
</style>
