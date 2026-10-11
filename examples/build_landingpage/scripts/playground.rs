//! Preset selection logic and interactive live input simulator

pub fn render() -> &'static str {
    r####"  // ==========================================================================
  // Preset Selection Logic
  // ==========================================================================
  function selectPreset(wordKey) {
    const data = PRESETS[wordKey];
    if (!data) return;

    playSwitchSound();

    legacyDisplay.textContent = data.legacy;
    legacyCaption.innerHTML = data.legacyCaption;

    koraDisplay.innerHTML = data.kora + '<span class="cursor"></span>';
    koraCaption.innerHTML = data.koraCaption;

    presetButtons.forEach(btn => {
      btn.classList.toggle('active', btn.dataset.word === wordKey);
    });

    liveInput.value = '';
  }

  presetButtons.forEach(btn => {
    btn.addEventListener('click', () => {
      selectPreset(btn.dataset.word);
    });
  });

  // ==========================================================================
  // Interactive Live Input Simulator
  // ==========================================================================
  liveInput.addEventListener('keydown', (e) => {
    playSwitchSound(e.key === ' ');
  });

  liveInput.addEventListener('input', (e) => {
    const val = liveInput.value;
    if (!val) {
      selectPreset('error');
      return;
    }

    // Deselect preset buttons
    presetButtons.forEach(btn => btn.classList.remove('active'));

    // Check if input contains repeated consonants or known English words
    const lower = val.toLowerCase();
    let legacySimulation = val;

    // Simulate legacy IME vs Kora behavior
    let koraSimulation = val;
    let koraExplain = `Bảo toàn từ bạn gõ: <strong>${escapeHtml(val)}</strong>`;

    // Smart Macro Simulation: ko -> không, Ko -> Không, KO -> KHÔNG
    if (val === 'ko') {
      koraSimulation = 'không';
      koraExplain = `Gõ tắt thông minh: <strong>ko</strong> &rarr; <strong>không</strong>`;
      legacySimulation = 'ko';
    } else if (val === 'Ko') {
      koraSimulation = 'Không';
      koraExplain = `Tự động viết hoa: <strong>Ko</strong> &rarr; <strong>Không</strong>`;
      legacySimulation = 'Ko';
    } else if (val === 'KO') {
      koraSimulation = 'KHÔNG';
      koraExplain = `Tự động viết hoa toàn bộ: <strong>KO</strong> &rarr; <strong>KHÔNG</strong>`;
      legacySimulation = 'KO';
    } else if (lower === 'hieuer') {
      koraSimulation = 'hiểu';
      koraExplain = `Tự động ghép đúng dấu: <strong>hiểu</strong>`;
    } else if (lower.includes('hieuer')) {
      koraSimulation = val.replace(/hieuer/gi, 'hiểu');
      koraExplain = `Tự động ghép đúng dấu: <strong>${escapeHtml(koraSimulation)}</strong>`;
    }

    // Simulate legacy IME swallowing double consonants or breaking words
    legacySimulation = legacySimulation
      .replace(/rr/gi, 'r')
      .replace(/ss/gi, 's')
      .replace(/ff/gi, 'f')
      .replace(/server/gi, 'sevơ')
      .replace(/port/gi, 'pọt')
      .replace(/part/gi, 'pạt')
      .replace(/after/gi, 'àter')
      .replace(/text/gi, 'tẽt');

    legacyDisplay.textContent = legacySimulation;
    legacyCaption.innerHTML = legacySimulation !== val
      ? `Bị nuốt chữ hoặc gõ sai: <strong>${escapeHtml(legacySimulation)}</strong>`
      : `Bộ gõ truyền thống`;

    koraDisplay.innerHTML = escapeHtml(koraSimulation) + '<span class="cursor"></span>';
    koraCaption.innerHTML = koraExplain;
  });

  clearBtn.addEventListener('click', () => {
    liveInput.value = '';
    selectPreset('error');
    liveInput.focus();
  });

  function escapeHtml(text) {
    const div = document.createElement('div');
    div.textContent = text;
    return div.innerHTML;
  }"####
}
