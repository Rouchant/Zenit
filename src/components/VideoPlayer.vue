<script setup>
import { computed, onMounted, ref } from 'vue';
import { useSpecsStore } from '../store/specs';

const store = useSpecsStore();
const videoRef = ref(null);

const videoUrl = computed(() => {
  if (store.currentSpecs.videoType === 'custom' && store.currentSpecs.customVideoPath) {
    return store.getVideoUrl(store.currentSpecs.customVideoPath);
  }
  return '/assets/videos/promo.mp4';
});

onMounted(() => {
  if (videoRef.value) {
    videoRef.value.play().catch(e => console.warn('Inactivity video failed to play:', e));
  }
});
</script>

<template>
  <div class="video-container">
    <video 
      ref="videoRef"
      id="promo-video" 
      loop 
      muted 
      :src="videoUrl"
    ></video>
    <div class="video-overlay">
      <div class="video-caption"></div>
    </div>
  </div>
</template>

<style scoped>
.video-container {
  width: 100%;
  height: 100%;
  background: black;
}
video {
  width: 100%;
  height: 100%;
  object-fit: cover;
}
</style>
