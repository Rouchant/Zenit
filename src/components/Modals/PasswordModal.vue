<script setup>
import { ref, onMounted } from 'vue';
import { useSpecsStore } from '../../store/specs';

const props = defineProps(['mode']);
const emit = defineEmits(['close', 'verified']);

const store = useSpecsStore();
const password = ref('');
const error = ref(false);
const inputRef = ref(null);

const verify = async () => {
  const isOk = await store.verifyPassword(password.value);
  if (isOk) {
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
      <h2>Panel de Administración</h2>
      <p>Ingresa la contraseña para acceder a la configuración (Por defecto: <strong>demo</strong> o clave maestra).</p>
      <div class="input-group">
        <label for="admin-password" class="sr-only">Contraseña</label>
        <input 
          id="admin-password"
          name="adminPassword"
          ref="inputRef"
          type="password" 
          v-model="password"
          placeholder="Ingresa clave..." 
          @keydown.enter="verify"
        >
      </div>
      <div v-if="error" class="error-msg">Contraseña incorrecta. Inténtalo de nuevo.</div>
      <div class="modal-actions">
        <button class="btn primary" @click="verify">Ingresar</button>
        <button class="btn secondary" @click="emit('close')">Cancelar</button>
      </div>
    </div>
  </div>
</template>
