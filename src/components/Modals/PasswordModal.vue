<script setup>
import { ref, onMounted } from 'vue';
import { useSpecsStore } from '../../store/specs';

const props = defineProps(['mode']);
const emit = defineEmits(['close', 'verified']);

const store = useSpecsStore();
const password = ref('');
const error = ref(false);
const inputRef = ref(null);

const verify = () => {
  if (password.value === store.CONFIG.PASSWORD) {
    error.value = false;
    emit('verified');
  } else {
    error.value = true;
    password.value = '';
    inputRef.value?.focus();
  }
};

onMounted(() => {
  inputRef.value?.focus();
});
</script>

<template>
  <div id="password-modal" class="modal active">
    <div class="modal-content password-content">
      <h2>Acceso Restringido</h2>
      <p>Ingresa el código para editar la configuración.</p>
      <div class="input-group">
        <label for="admin-password" class="sr-only">Código de acceso</label>
        <input 
          id="admin-password"
          name="adminPassword"
          ref="inputRef"
          type="password" 
          v-model="password"
          placeholder="Código..." 
          @keydown.enter="verify"
        >
      </div>
      <div v-if="error" class="error-msg">Código incorrecto. Inténtalo de nuevo.</div>
      <div class="modal-actions">
        <button class="btn primary" @click="verify">Entrar</button>
        <button class="btn secondary" @click="emit('close')">Cancelar</button>
      </div>
    </div>
  </div>
</template>
