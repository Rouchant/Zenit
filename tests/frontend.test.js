const { describe, it } = require('node:test');
const assert = require('node:assert');

// 1. Path Normalization Test
function toVideoSrc(path) {
  if (!path) return '';
  if (path.startsWith('http') || path.startsWith('assets/') || path.startsWith('/assets/')) {
    return path;
  }
  return `asset://localhost/${path.replace(/\\/g, '/')}`;
}

// 2. Scale Immunity Calculation Test
function calculateImmunityZoom(dpr, innerHeight) {
  let targetZoom = 1;
  if (Math.abs(dpr - 1) > 0.05) {
    targetZoom = 1 / dpr;
  }
  const effectiveHeight = innerHeight * (targetZoom !== 1 ? dpr : 1);
  if (effectiveHeight < 860 && effectiveHeight > 350) {
    const fitFactor = Math.min(1, effectiveHeight / 880);
    targetZoom = targetZoom * fitFactor;
  }
  return Number(targetZoom.toFixed(4));
}

// 3. Card Renderer Test
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

// 4. Password Verification Test
function verifyPassword(input) {
  const clean = String(input || '').trim().toLowerCase();
  return clean === 'demo' || clean === 'z3n1t';
}

// --- TEST SUITES ---

describe('Frontend: Scale Immunity Calculation', () => {
  it('should maintain 1.0 zoom on 100% scale (dpr = 1.0) on 1080p', () => {
    const zoom = calculateImmunityZoom(1.0, 1080);
    assert.strictEqual(zoom, 1.0);
  });

  it('should calculate 0.8 zoom on 125% Windows scale (dpr = 1.25)', () => {
    const zoom = calculateImmunityZoom(1.25, 864);
    assert.strictEqual(zoom, 0.8);
  });

  it('should calculate 0.6667 zoom on 150% Windows scale (dpr = 1.5)', () => {
    const zoom = calculateImmunityZoom(1.5, 720);
    assert.strictEqual(zoom, 0.6667);
  });

  it('should calculate 0.5 zoom on 200% Windows scale (dpr = 2.0)', () => {
    const zoom = calculateImmunityZoom(2.0, 540);
    assert.strictEqual(zoom, 0.5);
  });

  it('should auto-fit smaller heights (e.g. 768p displays)', () => {
    const zoom = calculateImmunityZoom(1.0, 768);
    assert.ok(zoom < 1.0, 'Zoom should scale down slightly to fit 768p height without cutoffs');
  });
});

describe('Frontend: Video Path Normalization', () => {
  it('should leave relative assets intact', () => {
    assert.strictEqual(toVideoSrc('assets/videos/landing.mp4'), 'assets/videos/landing.mp4');
  });

  it('should leave http/https URLs intact', () => {
    assert.strictEqual(toVideoSrc('https://example.com/video.mp4'), 'https://example.com/video.mp4');
  });

  it('should convert local Windows paths to asset:// protocol with forward slashes', () => {
    const winPath = 'C:\\Users\\User\\AppData\\Roaming\\Zenit\\vault\\hero.mp4';
    const result = toVideoSrc(winPath);
    assert.strictEqual(result, 'asset://localhost/C:/Users/User/AppData/Roaming/Zenit/vault/hero.mp4');
  });
});

describe('Frontend: Spec Card Renderer', () => {
  it('should render correct card HTML with tag and subvalue', () => {
    const card = {
      id: 'card-processor',
      label: 'Procesador',
      value: 'Intel Core i7 13700H',
      tag: '13ª Gen',
      sub_value: '14 Núcleos / 20 Hilos',
      icon: 'assets/logos/intel.svg'
    };
    const html = renderCard(card);
    assert.ok(html.includes('id="card-processor"'));
    assert.ok(html.includes('Intel Core i7 13700H'));
    assert.ok(html.includes('13ª Gen'));
    assert.ok(html.includes('14 Núcleos / 20 Hilos'));
    assert.ok(html.includes('src="assets/logos/intel.svg"'));
  });
});

describe('Frontend: Password Verification', () => {
  it('should accept demo in any case', () => {
    assert.strictEqual(verifyPassword('demo'), true);
    assert.strictEqual(verifyPassword('DEMO'), true);
    assert.strictEqual(verifyPassword('  Demo  '), true);
  });

  it('should accept master key in any case', () => {
    assert.strictEqual(verifyPassword('z3n1t'), true);
    assert.strictEqual(verifyPassword('Z3N1T'), true);
  });

  it('should reject unauthorized passwords', () => {
    assert.strictEqual(verifyPassword('admin'), false);
    assert.strictEqual(verifyPassword('1234'), false);
    assert.strictEqual(verifyPassword(''), false);
  });
});
