<script setup>
import { computed, onMounted, ref } from 'vue';
import { useSpecsStore, toVideoSrc } from '../store/specs';

const store = useSpecsStore();
const videoRef = ref(null);
const baseUrl = import.meta.env.BASE_URL;

const videoUrl = computed(() => {
  const p = store.appConfig.inactivity_video_path;
  if (!p) return `${baseUrl}assets/videos/promo.mp4`;
  return toVideoSrc(p);
});

onMounted(() => {
  if (videoRef.value) {
    videoRef.value.play().catch(e => console.warn('Inactivity video failed to play:', e));
  }
});
</script>

<template>
  <div class="fullscreen-video-container">
    <video 
      ref="videoRef"
      id="promo-video" 
      autoplay
      loop 
      muted 
      playsinline
      :key="videoUrl"
      :src="videoUrl"
    ></video>
    <div class="screensaver-hint">
      <span>Toca la pantalla o mueve el mouse para explorar</span>
    </div>
  </div>
</template>

<style scoped>
.fullscreen-video-container {
  position: fixed;
  top: 0;
  left: 0;
  width: 100vw;
  height: 100vh;
  background: #000;
  z-index: 9999;
  display: flex;
  justify-content: center;
  align-items: center;
}

video {
  width: 100%;
  height: 100%;
  object-fit: cover;
}

.screensaver-hint {
  position: absolute;
  bottom: 30px;
  background: rgba(0, 0, 0, 0.6);
  padding: 8px 24px;
  border-radius: 30px;
  backdrop-filter: blur(10px);
  border: 1px solid rgba(255, 255, 255, 0.15);
  color: rgba(255, 255, 255, 0.8);
  font-size: 0.9rem;
  letter-spacing: 0.5px;
  animation: pulse-slow 3s infinite ease-in-out;
}

@keyframes pulse-slow {
  0%, 100% { opacity: 0.4; }
  50% { opacity: 0.9; }
}
</style>
