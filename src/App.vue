<template>
  <div id="app" class="app-container" :style="{ backgroundColor: '#000' }">
    <!-- Loading Screen -->
    <div v-if="store.isLoading" class="loading-screen">
      <div class="loader"></div>
      <p>Detectando spec sistema...</p>
    </div>

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
            :src="store.currentSpecs.landingVideoType === 'custom' ? `file:///${store.currentSpecs.customLandingVideoPath}` : '/assets/videos/landing.mp4'"
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

    <!-- Admin Hotspots -->
    <div id="settings-hotspot" class="admin-hotspot top-right" @click="openPassword('settings')"></div>
    <div id="exit-hotspot" class="admin-hotspot bottom-right" @click="openPassword('exit')"></div>

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
import { onMounted, onUnmounted, ref, watch } from 'vue';
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

const resetTimer = () => {
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
  background: #000;
  display: flex;
  flex-direction: column;
  justify-content: center;
  align-items: center;
  z-index: 9999;
  color: white;
}
.loader {
  border: 4px solid #333;
  border-top: 4px solid var(--primary, #00f2ff);
  border-radius: 50%;
  width: 40px;
  height: 40px;
  animation: spin 1s linear infinite;
  margin-bottom: 20px;
}
@keyframes spin {
  0% { transform: rotate(0deg); }
  100% { transform: rotate(360deg); }
}
</style>
