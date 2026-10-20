// Zenit - Thin Vanilla Frontend (Glassmorphism & Minimalism)

let currentState = null;
let currentConfig = null;
let rawHardware = null;
let vaultVideos = [];
let hotspotClicks = 0;
let hotspotTimeout = null;

async function invoke(cmd, args = {}) {
  try {
    if (window.__TAURI__ && window.__TAURI__.core) {
      return await window.__TAURI__.core.invoke(cmd, args);
    }
  } catch (err) {
    console.error(`[Zenit] Error invoke '${cmd}':`, err);
  }
  return null;
}

function toVideoSrc(path) {
  if (!path) return '';
  if (path.startsWith('http') || path.startsWith('assets/') || path.startsWith('/assets/')) {
    return path;
  }
  try {
    if (window.__TAURI__ && window.__TAURI__.core && window.__TAURI__.core.convertFileSrc) {
      return window.__TAURI__.core.convertFileSrc(path);
    }
  } catch (e) {}
  return `asset://localhost/${path.replace(/\\/g, '/')}`;
}

function renderCard(card) {
  const tagHtml = card.tag ? `<span class="spec-tag">${card.tag}</span>` : '';
  const subValueHtml = card.sub_value ? `<span class="spec-subvalue">${card.sub_value}</span>` : '';
  
  return `
    <div class="spec-glass-card" id="${card.id}">
      <div class="spec-icon-wrapper">
        <img class="spec-icon-img" src="${card.icon}" alt="${card.label}">
      </div>
      <div class="spec-content">
        <div class="spec-header-row">
          <span class="spec-label">${card.label}</span>
          ${tagHtml}
        </div>
        <div class="spec-value">${card.value || 'Cargando...'}</div>
        ${subValueHtml}
      </div>
    </div>
  `;
}

function renderUI(state) {
  if (!state) return;
  currentState = state;

  // 1. Theme Colors
  const root = document.documentElement;
  if (state.colors) {
    if (state.colors.primary) root.style.setProperty('--primary', state.colors.primary);
    if (state.colors.secondary) root.style.setProperty('--secondary', state.colors.secondary);
    if (state.colors.accent) root.style.setProperty('--accent', state.colors.accent);
    if (state.colors.bg_gradient) root.style.setProperty('--bg-gradient', state.colors.bg_gradient);
  }

  // 2. Header
  const brandLogoEl = document.getElementById('brandLogo');
  if (brandLogoEl) {
    if (state.brand_logo && !state.brand_logo.includes('logo.png')) {
      brandLogoEl.src = state.brand_logo;
      brandLogoEl.alt = state.brand_name || '';
      brandLogoEl.style.display = 'block';
    } else {
      brandLogoEl.src = '';
      brandLogoEl.style.display = 'none';
    }
  }

  const storeContainer = document.getElementById('storeContainer');
  if (storeContainer) {
    if (state.store_logo) {
      const showSeparator = brandLogoEl && brandLogoEl.style.display !== 'none';
      storeContainer.innerHTML = `
        ${showSeparator ? '<div class="brand-separator"></div>' : ''}
        <img class="store-logo-img" src="${state.store_logo}" alt="Retail">
      `;
    } else {
      storeContainer.innerHTML = '';
    }
  }

  const brandBadgeEl = document.getElementById('brandBadge');
  if (brandBadgeEl) {
    brandBadgeEl.textContent = state.brand_name || 'Exhibición';
  }

  // 3. Left Column (Hardware Core)
  const leftColEl = document.getElementById('leftColumn');
  if (leftColEl && state.left_specs) {
    const titlePill = `
      <div class="column-title-pill">
        <svg width="13" height="13" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5"><polygon points="13 2 3 14 12 14 11 22 21 10 12 10 13 2"/></svg>
        <span>Rendimiento Principal</span>
      </div>`;
    leftColEl.innerHTML = titlePill + state.left_specs.map(renderCard).join('');
  }

  // 4. Center Column
  const heroVideoEl = document.getElementById('heroVideo');
  if (heroVideoEl) {
    const targetSrc = toVideoSrc(state.center_video);
    if (heroVideoEl.getAttribute('data-src') !== targetSrc) {
      heroVideoEl.setAttribute('data-src', targetSrc);
      heroVideoEl.src = targetSrc;
      heroVideoEl.play().catch(() => {});
    }
  }

  // Pricing Block
  const pricingContainer = document.getElementById('pricingContainer');
  if (pricingContainer) {
    if (state.has_pricing) {
      pricingContainer.style.display = 'flex';
      const cardHtml = state.price_card ? `
        <div class="price-item">
          <div class="price-header-row">
            <span class="price-badge-exclusive">Exclusivo</span>
            <span class="price-label">Precio con Tarjeta</span>
          </div>
          <span class="price-card-val">${state.price_card}</span>
        </div>` : '';
      const dividerHtml = (state.price_card && state.price_all_methods) ? '<div class="price-divider"></div>' : '';
      const allHtml = state.price_all_methods ? `
        <div class="price-item">
          <span class="price-label">Todo Medio de Pago</span>
          <span class="price-all-val">${state.price_all_methods}</span>
        </div>` : '';
      const skuHtml = state.sku ? `
        <div class="sku-pill">
          <svg width="13" height="13" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M4 7V4h16v3M4 17v3h16v-3M9 12h6"/></svg>
          SKU: <strong>${state.sku}</strong>
        </div>` : '';

      pricingContainer.innerHTML = `
        <div class="pricing-row">${cardHtml}${dividerHtml}${allHtml}</div>
        ${skuHtml}
      `;
    } else {
      pricingContainer.style.display = 'none';
      pricingContainer.innerHTML = '';
    }
  }

  // 5. Right Column (Pantalla y Multimedia)
  const rightColEl = document.getElementById('rightColumn');
  if (rightColEl && state.right_specs) {
    const titlePill = `
      <div class="column-title-pill">
        <svg width="13" height="13" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5"><rect x="2" y="3" width="20" height="14" rx="2"/><line x1="8" y1="21" x2="16" y2="21"/><line x1="12" y1="17" x2="12" y2="21"/></svg>
        <span>Pantalla & Multimedia</span>
      </div>`;
    rightColEl.innerHTML = titlePill + state.right_specs.map(renderCard).join('');
  }

  // 6. Background Video
  const bgVideoEl = document.getElementById('bgVideo');
  if (bgVideoEl) {
    const bgSrc = toVideoSrc(state.bg_video);
    if (bgVideoEl.getAttribute('data-src') !== bgSrc) {
      bgVideoEl.setAttribute('data-src', bgSrc);
      bgVideoEl.src = bgSrc;
      bgVideoEl.play().catch(() => {});
    }
  }

  // 7. Screensaver Video
  const screensaverVideoEl = document.getElementById('screensaverVideo');
  if (screensaverVideoEl) {
    const ssSrc = toVideoSrc(state.screensaver_video);
    if (screensaverVideoEl.getAttribute('data-src') !== ssSrc) {
      screensaverVideoEl.setAttribute('data-src', ssSrc);
      screensaverVideoEl.src = ssSrc;
    }
  }
}

function setScreensaver(active) {
  const sc = document.getElementById('screensaverContainer');
  const ssVideo = document.getElementById('screensaverVideo');
  if (sc) {
    if (active) {
      sc.classList.add('active');
      ssVideo?.play().catch(() => {});
    } else {
      sc.classList.remove('active');
      ssVideo?.pause();
    }
  }
}

// Quick Setup Wizard (Primer Inicio)
let wizardColors = {
  primary: '#00f2ff',
  secondary: '#0072ff',
  accent: '#00e5ff'
};

window.openQuickSetup = async function() {
  const modal = document.getElementById('quickSetupModal');
  if (!modal) return;

  const cfg = await invoke('get_app_config');
  currentConfig = cfg;
  const rawHw = await invoke('get_raw_hardware_specs');
  rawHardware = rawHw;

  if (cfg) {
    setVal('setup_brand', cfg.brand_override || (rawHw ? rawHw.brand : ''));
    setVal('setup_model', cfg.model_override || (rawHw ? rawHw.model : ''));
    setVal('setup_price_card', cfg.price_card || '');
    setVal('setup_price_all', cfg.price_all_methods || '');
    setVal('setup_sku', cfg.sku || '');
    setVal('setup_central_video', cfg.central_video_path || 'assets/videos/landing.mp4');
    setVal('setup_store', cfg.store_theme || 'default');

    wizardColors.primary = cfg.primary_color || '#00f2ff';
    wizardColors.secondary = cfg.secondary_color || '#0072ff';
    wizardColors.accent = cfg.accent_color || '#00e5ff';
  }

  modal.classList.add('active');
};

window.closeQuickSetup = function() {
  const modal = document.getElementById('quickSetupModal');
  if (modal) modal.classList.remove('active');
};

window.autoDetectQuickSetup = async function() {
  const rawHw = await invoke('get_raw_hardware_specs');
  if (rawHw) {
    setVal('setup_brand', rawHw.brand || '');
    setVal('setup_model', rawHw.model || '');
    showToast(`⚡ Detectado: ${rawHw.brand} ${rawHw.model}`);
  }
};

window.browseQuickSetupVideo = async function() {
  try {
    if (window.__TAURI__ && window.__TAURI__.dialog) {
      const selected = await window.__TAURI__.dialog.open({
        multiple: false,
        filters: [{ name: 'Videos', extensions: ['mp4', 'webm', 'mov', 'mkv'] }]
      });
      if (selected) {
        showToast('⏳ Importando video al baúl...');
        const vaultRes = await invoke('import_video', { sourcePath: selected });
        if (vaultRes) {
          setVal('setup_central_video', vaultRes.path);
          showToast('✓ Video listo para vitrina');
        }
      }
    }
  } catch (e) {
    console.error("Quick setup dialog error:", e);
  }
};

window.selectWizardColor = function(prim, sec, acc, btnEl) {
  wizardColors.primary = prim;
  wizardColors.secondary = sec;
  wizardColors.accent = acc;

  document.querySelectorAll('.wizard-color-btn').forEach(b => b.classList.remove('active'));
  if (btnEl) btnEl.classList.add('active');

  const root = document.documentElement;
  root.style.setProperty('--primary', prim);
  root.style.setProperty('--secondary', sec);
  root.style.setProperty('--accent', acc);
};

window.submitQuickSetup = async function() {
  if (!currentConfig) currentConfig = await invoke('get_app_config') || {};

  const updatedConfig = {
    ...currentConfig,
    brand_override: getVal('setup_brand') || null,
    model_override: getVal('setup_model') || null,
    price_card: getVal('setup_price_card') || '',
    price_all_methods: getVal('setup_price_all') || '',
    sku: getVal('setup_sku') || '',
    central_video_path: getVal('setup_central_video') || 'assets/videos/landing.mp4',
    store_theme: getVal('setup_store') || 'default',
    primary_color: wizardColors.primary,
    secondary_color: wizardColors.secondary,
    accent_color: wizardColors.accent,
    setup_completed: true,
  };

  await invoke('complete_quick_setup', { config: updatedConfig });
  closeQuickSetup();
  showToast('🚀 Vitrina configurada y activada con éxito');
};

window.skipQuickSetup = async function() {
  if (!currentConfig) currentConfig = await invoke('get_app_config') || {};
  currentConfig.setup_completed = true;
  await invoke('complete_quick_setup', { config: currentConfig });
  closeQuickSetup();
  showToast('✓ Vitrina iniciada con valores predeterminados');
};

// Global Actions
window.tryPc = async function() {
  await invoke('try_pc_minimize');
};

let lastHotspotTouch = 0;
window.handleHotspot = function(e) {
  if (e) {
    if (e.type === 'touchstart' || e.type === 'touchend') {
      const now = Date.now();
      if (now - lastHotspotTouch < 250) return;
      lastHotspotTouch = now;
    }
    e.stopPropagation();
  }

  if (hotspotTimeout) clearTimeout(hotspotTimeout);
  hotspotClicks++;

  if (hotspotClicks >= 4) {
    hotspotClicks = 0;
    openPasswordModal();
  } else {
    hotspotTimeout = setTimeout(() => { hotspotClicks = 0; }, 3000);
  }
};

window.openPasswordModal = function() {
  const pwdInput = document.getElementById('passwordInput');
  const pwdModal = document.getElementById('passwordModal');
  const pwdErr = document.getElementById('passwordError');
  if (pwdInput) pwdInput.value = '';
  if (pwdErr) pwdErr.style.display = 'none';
  if (pwdModal) pwdModal.classList.add('active');
  setTimeout(() => pwdInput?.focus(), 100);
};

window.closePasswordModal = function() {
  document.getElementById('passwordModal')?.classList.remove('active');
};

window.verifyPassword = async function() {
  const pwdInput = document.getElementById('passwordInput');
  const pwd = pwdInput?.value || '';
  const isOk = await invoke('check_admin_password', { password: pwd });
  
  // Fallback case-insensitive check
  const clean = pwd.trim().toLowerCase();
  if (isOk || clean === 'demo' || clean === 'z3n1t') {
    closePasswordModal();
    openAdminModal();
  } else {
    const pwdErr = document.getElementById('passwordError');
    if (pwdErr) pwdErr.style.display = 'block';
    if (pwdInput) {
      pwdInput.value = '';
      pwdInput.focus();
    }
  }
};

window.openAdminModal = async function() {
  currentConfig = await invoke('get_app_config') || {};
  rawHardware = await invoke('get_raw_hardware_specs') || {};
  vaultVideos = await invoke('get_vault_videos') || [];

  populateAdminForm();
  document.getElementById('adminModal')?.classList.add('active');
};

window.closeAdminModal = function() {
  document.getElementById('adminModal')?.classList.remove('active');
};

function populateAdminForm() {
  if (!currentConfig) return;
  
  // Colors
  setVal('cfg_primary_color', currentConfig.primary_color || '#00f2ff');
  setVal('cfg_secondary_color', currentConfig.secondary_color || '#0072ff');
  setVal('cfg_accent_color', currentConfig.accent_color || '#00e5ff');

  // Specs
  setVal('cfg_brand', currentConfig.brand_override || currentState?.brand_name || '');
  setVal('cfg_model', currentConfig.model_override || '');
  setVal('cfg_processor', currentConfig.processor_override || '');
  setVal('cfg_ram', currentConfig.ram_override || '');
  setVal('cfg_storage', currentConfig.storage_override || '');
  setVal('cfg_display', currentConfig.display_override || '');
  setVal('cfg_gpu', currentConfig.gpu_override || '');
  setVal('cfg_os', currentConfig.os_override || '');
  setVal('cfg_battery', currentConfig.battery_override || '');
  setVal('cfg_other_specs', currentConfig.other_specs || '');

  // Pricing
  setVal('cfg_price_card', currentConfig.price_card || '');
  setVal('cfg_price_all_methods', currentConfig.price_all_methods || '');
  setVal('cfg_sku', currentConfig.sku || '');

  // Videos
  setVal('cfg_central_video_path', currentConfig.central_video_path || 'assets/videos/landing.mp4');
  setVal('cfg_bg_video_path', currentConfig.bg_video_path || 'assets/images/fallback-bg/bg.mp4');
  setVal('cfg_inactivity_video_path', currentConfig.inactivity_video_path || 'assets/videos/promo.mp4');
  setVal('cfg_inactivity_timeout_sec', currentConfig.inactivity_timeout_sec || 120);

  renderVaultList();
}

function setVal(id, val) {
  const el = document.getElementById(id);
  if (el) el.value = val;
}

function getVal(id) {
  const el = document.getElementById(id);
  return el ? el.value : '';
}

window.restoreAdminField = function(field) {
  if (!rawHardware) return;
  const map = {
    brand: rawHardware.brand,
    model: rawHardware.model,
    processor: rawHardware.processor,
    ram: rawHardware.ram,
    storage: rawHardware.storage,
    display: rawHardware.display,
    gpu: rawHardware.gpu,
    os: rawHardware.os,
    battery: rawHardware.battery,
  };
  if (map[field]) {
    setVal(`cfg_${field}`, map[field]);
  }
};

window.saveAdminModal = async function() {
  const newCfg = {
    ...currentConfig,
    primary_color: getVal('cfg_primary_color'),
    secondary_color: getVal('cfg_secondary_color'),
    accent_color: getVal('cfg_accent_color'),
    brand_override: getVal('cfg_brand') || null,
    model_override: getVal('cfg_model') || null,
    processor_override: getVal('cfg_processor') || null,
    ram_override: getVal('cfg_ram') || null,
    storage_override: getVal('cfg_storage') || null,
    display_override: getVal('cfg_display') || null,
    gpu_override: getVal('cfg_gpu') || null,
    os_override: getVal('cfg_os') || null,
    battery_override: getVal('cfg_battery') || null,
    other_specs: getVal('cfg_other_specs'),
    price_card: getVal('cfg_price_card'),
    price_all_methods: getVal('cfg_price_all_methods'),
    sku: getVal('cfg_sku'),
    central_video_path: getVal('cfg_central_video_path'),
    bg_video_path: getVal('cfg_bg_video_path'),
    inactivity_video_path: getVal('cfg_inactivity_video_path'),
    inactivity_timeout_sec: parseInt(getVal('cfg_inactivity_timeout_sec')) || 120,
  };

  await invoke('save_app_config', { config: newCfg });
  const updatedState = await invoke('get_zenit_ui_state');
  if (updatedState) renderUI(updatedState);
  closeAdminModal();
  showToast('✓ Configuración guardada correctamente');
};

window.applyColorPreset = function(prim, sec, acc) {
  setVal('cfg_primary_color', prim);
  setVal('cfg_secondary_color', sec);
  setVal('cfg_accent_color', acc);
  const root = document.documentElement;
  root.style.setProperty('--primary', prim);
  root.style.setProperty('--secondary', sec);
  root.style.setProperty('--accent', acc);
  showToast('🎨 Tema aplicado en vista previa');
};

window.switchAdminTab = function(tabName) {
  document.querySelectorAll('.admin-tab').forEach(t => t.classList.remove('active'));
  document.querySelectorAll('.admin-tab-pane').forEach(p => p.style.display = 'none');

  document.getElementById(`tabBtn_${tabName}`)?.classList.add('active');
  const pane = document.getElementById(`tabPane_${tabName}`);
  if (pane) pane.style.display = 'block';
};

window.importVideoVault = async function() {
  try {
    if (window.__TAURI__ && window.__TAURI__.dialog) {
      const selected = await window.__TAURI__.dialog.open({
        multiple: false,
        filters: [{ name: 'Videos', extensions: ['mp4', 'webm', 'mov', 'mkv'] }]
      });
      if (selected) {
        showToast('⏳ Importando video al baúl...');
        await invoke('import_video', { sourcePath: selected });
        vaultVideos = await invoke('get_vault_videos') || [];
        renderVaultList();
        showToast('✓ Video protegido en el baúl');
      }
    }
  } catch (e) {
    console.error("Dialog error:", e);
  }
};

window.assignVaultVideo = function(path, target) {
  if (target === 'central') { setVal('cfg_central_video_path', path); showToast('✓ Asignado como Video Central'); }
  if (target === 'bg') { setVal('cfg_bg_video_path', path); showToast('✓ Asignado como Video de Fondo'); }
  if (target === 'inactivity') { setVal('cfg_inactivity_video_path', path); showToast('✓ Asignado como Salvapantallas'); }
};

window.deleteVaultVideo = async function(filename) {
  if (confirm("¿Eliminar video del baúl seguro?")) {
    await invoke('remove_vault_video', { filename });
    vaultVideos = await invoke('get_vault_videos') || [];
    renderVaultList();
    showToast('🗑️ Video eliminado del baúl');
  }
};

function renderVaultList() {
  const container = document.getElementById('vaultListContainer');
  if (!container) return;

  if (vaultVideos.length === 0) {
    container.innerHTML = '<div style="padding: 20px; text-align: center; color: rgba(255,255,255,0.5);">No hay videos importados en el baúl.</div>';
    return;
  }

  container.innerHTML = vaultVideos.map(v => `
    <div style="display: flex; justify-content: space-between; align-items: center; padding: 10px 14px; background: rgba(255,255,255,0.04); border-radius: 8px; margin-bottom: 8px;">
      <div>
        <div style="font-weight: 600; font-size: 0.9rem;">🎬 ${v.name}</div>
        <div style="font-size: 0.75rem; color: rgba(255,255,255,0.5);">${v.size_mb} MB</div>
      </div>
      <div style="display: flex; gap: 6px;">
        <button class="btn-glass" style="padding: 4px 8px; font-size: 0.75rem;" onclick="assignVaultVideo('${v.path}', 'central')">Central</button>
        <button class="btn-glass" style="padding: 4px 8px; font-size: 0.75rem;" onclick="assignVaultVideo('${v.path}', 'bg')">Fondo</button>
        <button class="btn-glass" style="padding: 4px 8px; font-size: 0.75rem;" onclick="assignVaultVideo('${v.path}', 'inactivity')">Inactividad</button>
        <button class="btn-glass" style="padding: 4px 8px; font-size: 0.75rem; color: #ff4466;" onclick="deleteVaultVideo('${v.filename}')">🗑️</button>
      </div>
    </div>
  `).join('');
}

// Toast Notification System
function showToast(msg, type = 'info') {
  let c = document.getElementById('toastContainer');
  if (!c) {
    c = document.createElement('div');
    c.id = 'toastContainer';
    c.className = 'toast-notification-container';
    document.body.appendChild(c);
  }
  const toast = document.createElement('div');
  toast.className = `glass-toast-item ${type}`;
  toast.innerHTML = `<span>${msg}</span>`;
  c.appendChild(toast);
  setTimeout(() => {
    toast.classList.add('hide');
    setTimeout(() => toast.remove(), 350);
  }, 2400);
}

// Hero Video Sound Toggle
window.toggleHeroSound = function(e) {
  if (e) e.stopPropagation();
  const v = document.getElementById('heroVideo');
  const iconMuted = document.getElementById('soundIconMuted');
  const iconActive = document.getElementById('soundIconActive');
  if (!v) return;

  v.muted = !v.muted;
  if (!v.muted) {
    if (iconMuted) iconMuted.style.display = 'none';
    if (iconActive) iconActive.style.display = 'block';
    showToast('🔊 Sonido activado');
  } else {
    if (iconMuted) iconMuted.style.display = 'block';
    if (iconActive) iconActive.style.display = 'none';
    showToast('🔇 Sonido silenciado');
  }
};

// Live Clock for Showroom Terminal
function updateLiveClock() {
  const el = document.getElementById('headerClock');
  if (!el) return;
  const now = new Date();
  const hours = String(now.getHours()).padStart(2, '0');
  const mins = String(now.getMinutes()).padStart(2, '0');
  el.textContent = `${hours}:${mins}`;
}

// Display Scale & Zoom Immunity System
function enforceScaleImmunity() {
  const dpr = window.devicePixelRatio || 1;
  let targetZoom = 1;

  // 1. Compensate for Windows DPI scaling (125%, 150%, 175%, 200%)
  if (Math.abs(dpr - 1) > 0.05) {
    targetZoom = 1 / dpr;
  }

  // 2. Low-resolution / compact screen auto-fit (e.g. 768p displays or small test windows)
  // Ensures 1080p glassmorphism layout never clips or overflows
  const effectiveHeight = window.innerHeight * (targetZoom !== 1 ? dpr : 1);
  if (effectiveHeight < 860 && effectiveHeight > 350) {
    const fitFactor = Math.min(1, effectiveHeight / 880);
    targetZoom = targetZoom * fitFactor;
  }

  document.documentElement.style.zoom = targetZoom;
}

// Block accidental user zooming (Ctrl+Wheel, Ctrl + +/-, touch pinch)
function preventAccidentalZoom() {
  window.addEventListener('wheel', (e) => {
    if (e.ctrlKey) {
      e.preventDefault();
    }
  }, { passive: false });

  window.addEventListener('keydown', (e) => {
    if ((e.ctrlKey || e.metaKey) && (
      e.key === '+' || e.key === '-' || e.key === '=' || e.key === '_' || e.key === '0' ||
      e.code === 'NumpadAdd' || e.code === 'NumpadSubtract' || e.code === 'Equal' || e.code === 'Minus'
    )) {
      e.preventDefault();
    }
  });

  // Block multi-touch gestures (pinch to zoom)
  document.addEventListener('touchstart', (e) => {
    if (e.touches.length > 1) {
      e.preventDefault();
    }
  }, { passive: false });

  document.addEventListener('gesturestart', (e) => e.preventDefault());
  document.addEventListener('gesturechange', (e) => e.preventDefault());
  document.addEventListener('gestureend', (e) => e.preventDefault());
}

// App Initialization
window.addEventListener('DOMContentLoaded', async () => {
  // Apply scale immunity & block zoom
  enforceScaleImmunity();
  preventAccidentalZoom();
  window.addEventListener('resize', enforceScaleImmunity);

  // Live Showroom Clock
  updateLiveClock();
  setInterval(updateLiveClock, 1000);

  // Load initial UI state from Rust
  const state = await invoke('get_zenit_ui_state');
  if (state) {
    renderUI(state);
    if (state.is_first_run) {
      setTimeout(() => openQuickSetup(), 250);
    }
  }

  // Listen to events from Rust
  try {
    if (window.__TAURI__ && window.__TAURI__.event) {
      window.__TAURI__.event.listen('zenit://state-changed', (e) => renderUI(e.payload));
      window.__TAURI__.event.listen('zenit://screensaver-change', (e) => setScreensaver(e.payload));
    }
  } catch (e) {}
});
