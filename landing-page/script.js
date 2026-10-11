// ==========================================================================
// Kora Landing Page — Interactive Demo & Web Audio Sound Synthesizer
// ==========================================================================

document.addEventListener('DOMContentLoaded', () => {
  // Preset Data Matrix
  const PRESETS = {
    'error': {
      legacy: 'eror',
      legacyCaption: 'Phím <code>r</code> thứ hai bị hiểu nhầm là lệnh hủy dấu, nuốt mất chữ thành <strong>eror</strong>.',
      kora: 'error',
      koraCaption: 'Tự nhận diện từ tiếng Anh, giữ nguyên chữ <strong>error</strong>.'
    },
    'pass': {
      legacy: 'pas',
      legacyCaption: 'Phím <code>s</code> thứ hai bị hiểu nhầm là lệnh hủy dấu, nuốt thành <strong>pas</strong> (phải gõ <code>passs</code>).',
      kora: 'pass',
      koraCaption: 'Tự nhận diện từ tiếng Anh, giữ nguyên vẹn <strong>pass</strong>.'
    },
    'coffee': {
      legacy: 'cofee',
      legacyCaption: 'Phím <code>f</code> thứ hai bị hiểu nhầm là lệnh hủy dấu, nuốt thành <strong>cofee</strong>.',
      kora: 'coffee',
      koraCaption: 'Tự nhận diện từ tiếng Anh, giữ nguyên vẹn <strong>coffee</strong>.'
    }
  };

  const legacyDisplay = document.getElementById('legacyOutput');
  const legacyCaption = document.getElementById('legacyCaption');
  const koraDisplay = document.getElementById('koraOutput');
  const koraCaption = document.getElementById('koraCaption');
  const liveInput = document.getElementById('liveInput');
  const clearBtn = document.getElementById('clearBtn');
  const presetButtons = document.querySelectorAll('.preset-btn');
  const soundCheckbox = document.getElementById('soundCheckbox');

  // ==========================================================================
  // Web Audio API Synthesizer: Mechanical Switch Simulator
  // ==========================================================================
  let audioCtx = null;

  function initAudio() {
    if (!audioCtx) {
      const AudioContextClass = window.AudioContext || window.webkitAudioContext;
      if (AudioContextClass) {
        audioCtx = new AudioContextClass();
      }
    }
    if (audioCtx && audioCtx.state === 'suspended') {
      audioCtx.resume();
    }
  }

  function playSwitchSound(isSpace = false) {
    if (!soundCheckbox.checked) return;
    initAudio();
    if (!audioCtx) return;

    try {
      const now = audioCtx.currentTime;

      // Click transient (High frequency burst)
      const osc = audioCtx.createOscillator();
      const gain = audioCtx.createGain();
      const filter = audioCtx.createBiquadFilter();

      osc.type = isSpace ? 'triangle' : 'sine';
      osc.frequency.setValueAtTime(isSpace ? 280 : 1200 + Math.random() * 400, now);
      osc.frequency.exponentialRampToValueAtTime(isSpace ? 80 : 350, now + 0.035);

      filter.type = 'bandpass';
      filter.frequency.setValueAtTime(isSpace ? 400 : 1800, now);
      filter.Q.setValueAtTime(3.5, now);

      gain.gain.setValueAtTime(isSpace ? 0.35 : 0.25, now);
      gain.gain.exponentialRampToValueAtTime(0.001, now + (isSpace ? 0.06 : 0.04));

      osc.connect(filter);
      filter.connect(gain);
      gain.connect(audioCtx.destination);

      osc.start(now);
      osc.stop(now + 0.06);

      // Thock noise resonance
      const bufferSize = audioCtx.sampleRate * 0.025;
      const noiseBuffer = audioCtx.createBuffer(1, bufferSize, audioCtx.sampleRate);
      const output = noiseBuffer.getChannelData(0);
      for (let i = 0; i < bufferSize; i++) {
        output[i] = Math.random() * 2 - 1;
      }

      const whiteNoise = audioCtx.createBufferSource();
      whiteNoise.buffer = noiseBuffer;

      const noiseFilter = audioCtx.createBiquadFilter();
      noiseFilter.type = 'lowpass';
      noiseFilter.frequency.setValueAtTime(isSpace ? 600 : 1100, now);

      const noiseGain = audioCtx.createGain();
      noiseGain.gain.setValueAtTime(0.18, now);
      noiseGain.gain.exponentialRampToValueAtTime(0.001, now + 0.025);

      whiteNoise.connect(noiseFilter);
      noiseFilter.connect(noiseGain);
      noiseGain.connect(audioCtx.destination);

      whiteNoise.start(now);
    } catch (_) {}
  }

  // ==========================================================================
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
  }

  // ==========================================================================
  // Smart Macro Interactive Playground (3 Macro Types)
  // ==========================================================================
  const MACRO_DATA = {
    normal: {
      title: 'Kiểu 1: Gõ tắt cả từ (Tự động đổi hoa / thường)',
      hint: 'Chỉ cần cài 1 từ viết tắt, tự động đổi hoa thường',
      placeholder: 'Gõ thử từ viết tắt cả từ (ví dụ: ko, Ko, KO, dc, vn)...',
      presets: [
        { key: 'ko', out: 'không', caption: 'Tự động nhận diện chữ thường: <code>ko</code> &rarr; <strong>không</strong>.' },
        { key: 'Ko', out: 'Không', caption: 'Tự động viết hoa chữ đầu: <code>Ko</code> &rarr; <strong>Không</strong>.' },
        { key: 'KO', out: 'KHÔNG', caption: 'Tự động viết hoa toàn bộ: <code>KO</code> &rarr; <strong>KHÔNG</strong>.' },
        { key: 'dc', out: 'được', caption: 'Viết tắt từ có dấu: <code>dc</code> &rarr; <strong>được</strong>.' },
        { key: 'vn', out: 'Việt Nam', caption: 'Viết tắt danh từ riêng: <code>vn</code> &rarr; <strong>Việt Nam</strong>.' },
        { key: 'ng', out: 'người', caption: 'Từ thông dụng hàng ngày: <code>ng</code> &rarr; <strong>người</strong>.' }
      ]
    },
    start: {
      title: 'Kiểu 2: Gõ tắt phụ âm đầu (f → ph, j → gi, w → qu)',
      hint: 'Gõ 1 phím thay cho 2 phím phụ âm đầu',
      placeholder: 'Gõ thử phụ âm đầu (ví dụ: fa, fong, ja, wa, we)...',
      presets: [
        { key: 'fong', out: 'phong', caption: 'Phụ âm đầu <code>f</code> trước nguyên âm: <code>fong</code> &rarr; <strong>phong</strong>.' },
        { key: 'fa', out: 'pha', caption: 'Phụ âm đầu <code>f</code> trước nguyên âm: <code>fa</code> &rarr; <strong>pha</strong>.' },
        { key: 'ja', out: 'gia', caption: 'Phụ âm đầu <code>j</code> trước nguyên âm: <code>ja</code> &rarr; <strong>gia</strong>.' },
        { key: 'wa', out: 'qua', caption: 'Phụ âm đầu <code>w</code> trước nguyên âm: <code>wa</code> &rarr; <strong>qua</strong>.' },
        { key: 'we', out: 'que', caption: 'Phụ âm đầu <code>w</code> trước nguyên âm: <code>we</code> &rarr; <strong>que</strong>.' }
      ]
    },
    end: {
      title: 'Kiểu 3: Gõ tắt phụ âm cuối (g → ng, h → nh, k → ch)',
      hint: 'Gõ 1 phím thay cho 2 phím phụ âm cuối',
      placeholder: 'Gõ thử phụ âm cuối (ví dụ: sag, mag, tih, xih, sak)...',
      presets: [
        { key: 'sag', out: 'sang', caption: 'Phụ âm cuối <code>g</code> sau nguyên âm: <code>sag</code> &rarr; <strong>sang</strong>.' },
        { key: 'mag', out: 'mang', caption: 'Phụ âm cuối <code>g</code> sau nguyên âm: <code>mag</code> &rarr; <strong>mang</strong>.' },
        { key: 'tih', out: 'tinh', caption: 'Phụ âm cuối <code>h</code> sau nguyên âm: <code>tih</code> &rarr; <strong>tinh</strong>.' },
        { key: 'xih', out: 'xinh', caption: 'Phụ âm cuối <code>h</code> sau nguyên âm: <code>xih</code> &rarr; <strong>xinh</strong>.' },
        { key: 'sak', out: 'sach', caption: 'Phụ âm cuối <code>k</code> sau nguyên âm: <code>sak</code> &rarr; <strong>sach</strong> (kèm dấu <code>saks</code> &rarr; <strong>sách</strong>).' }
      ]
    }
  };

  const macroTabButtons = document.querySelectorAll('.macro-tab-btn');
  const macroModeTitle = document.getElementById('macroModeTitle');
  const macroRuleHint = document.getElementById('macroRuleHint');
  const macroPresetsContainer = document.getElementById('macroPresetsContainer');
  const macroInputDisplay = document.getElementById('macroInputDisplay');
  const macroInputCaption = document.getElementById('macroInputCaption');
  const macroOutputDisplay = document.getElementById('macroOutputDisplay');
  const macroOutputCaption = document.getElementById('macroOutputCaption');
  const macroLiveInput = document.getElementById('macroLiveInput');
  const macroClearBtn = document.getElementById('macroClearBtn');

  let currentMacroType = 'normal';

  function renderMacroPresets(type) {
    if (!macroPresetsContainer) return;
    const data = MACRO_DATA[type];
    if (!data) return;

    macroPresetsContainer.innerHTML = '<span class="preset-label">Từ mẫu:</span>';
    data.presets.forEach((p, idx) => {
      const btn = document.createElement('button');
      btn.className = 'preset-btn macro-preset-btn' + (idx === 0 ? ' active' : '');
      btn.dataset.macro = p.key;
      btn.textContent = p.key;
      btn.addEventListener('click', () => {
        selectMacroPreset(type, p.key);
      });
      macroPresetsContainer.appendChild(btn);
    });

    if (macroModeTitle) macroModeTitle.textContent = data.title;
    if (macroRuleHint) macroRuleHint.textContent = data.hint;
    if (macroLiveInput) macroLiveInput.placeholder = data.placeholder;

    selectMacroPreset(type, data.presets[0].key);
  }

  function selectMacroPreset(type, key) {
    const data = MACRO_DATA[type];
    if (!data) return;
    const item = data.presets.find(p => p.key === key);
    if (!item) return;

    playSwitchSound();

    if (macroInputDisplay) macroInputDisplay.textContent = item.key;
    if (macroInputCaption) macroInputCaption.innerHTML = `Từ viết tắt: <code>${escapeHtml(item.key)}</code>`;
    if (macroOutputDisplay) macroOutputDisplay.innerHTML = escapeHtml(item.out) + '<span class="cursor"></span>';
    if (macroOutputCaption) macroOutputCaption.innerHTML = item.caption;

    if (macroPresetsContainer) {
      const btns = macroPresetsContainer.querySelectorAll('.macro-preset-btn');
      btns.forEach(b => b.classList.toggle('active', b.dataset.macro === key));
    }

    if (macroLiveInput) macroLiveInput.value = '';
  }

  if (macroTabButtons.length) {
    macroTabButtons.forEach(btn => {
      btn.addEventListener('click', () => {
        const type = btn.dataset.type;
        if (type === currentMacroType) return;
        currentMacroType = type;

        macroTabButtons.forEach(b => b.classList.toggle('active', b.dataset.type === type));
        renderMacroPresets(type);
        playSwitchSound();
      });
    });
  }

  function expandMacroWord(word) {
    if (!word) return { out: '', desc: '' };

    const lower = word.toLowerCase();
    const normalDict = {
      'ko': 'không',
      'k': 'không',
      'dc': 'được',
      'ddc': 'được',
      'vn': 'Việt Nam',
      'ng': 'người',
      'n': 'nhiều',
      'nt': 'như thế'
    };

    if (normalDict[lower]) {
      let exp = normalDict[lower];
      if (word === word.toUpperCase() && word.length > 1) {
        exp = exp.toUpperCase();
      } else if (word[0] === word[0].toUpperCase()) {
        exp = exp.charAt(0).toUpperCase() + exp.slice(1);
      }
      return { out: exp, desc: `Gõ tắt cả từ: <code>${escapeHtml(word)}</code> &rarr; <strong>${escapeHtml(exp)}</strong>` };
    }

    // Convert leading 'dd' -> 'đ' if user typed Telex ddag -> đang
    let w = word;
    if (w.toLowerCase().startsWith('dd')) {
      const isUpper = w[0] === w[0].toUpperCase();
      w = (isUpper ? 'Đ' : 'đ') + w.slice(2);
    }

    // Accent test words
    if (w.toLowerCase() === 'saks') {
      return { out: 'sách', desc: `Phụ âm cuối <code>k</code> &rarr; <code>ch</code> kết hợp dấu sắc: <strong>sách</strong>` };
    }
    if (w.toLowerCase() === 'thiks') {
      return { out: 'thích', desc: `Phụ âm cuối <code>k</code> &rarr; <code>ch</code> kết hợp dấu sắc: <strong>thích</strong>` };
    }
    if (w.toLowerCase() === 'jos') {
      return { out: 'gió', desc: `Phụ âm đầu <code>j</code> &rarr; <code>gi</code> kết hợp dấu sắc: <strong>gió</strong>` };
    }
    if (w.toLowerCase() === 'jeengs') {
      return { out: 'giếng', desc: `Phụ âm đầu <code>j</code> &rarr; <code>gi</code> kết hợp dấu: <strong>giếng</strong>` };
    }
    if (w.toLowerCase() === 'woocs' || w.toLowerCase() === 'wocs') {
      return { out: 'quốc', desc: `Phụ âm đầu <code>w</code> &rarr; <code>qu</code> kết hợp dấu: <strong>quốc</strong>` };
    }

    const vowels = 'aeiouyáàảãạăắằẳẵặâấầẩẫậéèẻẽẹêếềểễệíìỉĩịóòỏõọôốồổỗộơớờởỡợúùủũụưứừửữựýỳỷỹỵ';
    if (w.length >= 2) {
      const first = w[0].toLowerCase();
      const second = w[1].toLowerCase();
      if (vowels.includes(second)) {
        if (first === 'f') {
          const rep = (w[0] === 'F' ? 'Ph' : 'ph') + w.slice(1);
          return { out: rep, desc: `Đổi phụ âm đầu <code>f</code> &rarr; <code>ph</code>: <strong>${escapeHtml(rep)}</strong>` };
        } else if (first === 'j') {
          const rep = (w[0] === 'J' ? 'Gi' : 'gi') + w.slice(1);
          return { out: rep, desc: `Đổi phụ âm đầu <code>j</code> &rarr; <code>gi</code>: <strong>${escapeHtml(rep)}</strong>` };
        } else if (first === 'w') {
          const rep = (w[0] === 'W' ? 'Qu' : 'qu') + w.slice(1);
          return { out: rep, desc: `Đổi phụ âm đầu <code>w</code> &rarr; <code>qu</code>: <strong>${escapeHtml(rep)}</strong>` };
        }
      }
    }

    if (w.length >= 2) {
      const last = w[w.length - 1].toLowerCase();
      const prev = w[w.length - 2].toLowerCase();
      if (vowels.includes(prev)) {
        if (last === 'g') {
          const rep = w.slice(0, -1) + 'ng';
          return { out: rep, desc: `Đổi phụ âm cuối <code>g</code> &rarr; <code>ng</code>: <strong>${escapeHtml(rep)}</strong>` };
        } else if (last === 'h') {
          const rep = w.slice(0, -1) + 'nh';
          return { out: rep, desc: `Đổi phụ âm cuối <code>h</code> &rarr; <code>nh</code>: <strong>${escapeHtml(rep)}</strong>` };
        } else if (last === 'k') {
          const rep = w.slice(0, -1) + 'ch';
          return { out: rep, desc: `Đổi phụ âm cuối <code>k</code> &rarr; <code>ch</code>: <strong>${escapeHtml(rep)}</strong>` };
        }
      }
    }

    return { out: w, desc: `Xuất phím gõ: <strong>${escapeHtml(w)}</strong>` };
  }

  if (macroLiveInput) {
    macroLiveInput.addEventListener('keydown', (e) => {
      playSwitchSound(e.key === ' ');
    });

    macroLiveInput.addEventListener('input', () => {
      const val = macroLiveInput.value.trim();
      if (!val) {
        selectMacroPreset(currentMacroType, MACRO_DATA[currentMacroType].presets[0].key);
        return;
      }

      if (macroPresetsContainer) {
        macroPresetsContainer.querySelectorAll('.macro-preset-btn').forEach(b => b.classList.remove('active'));
      }

      const res = expandMacroWord(val);
      if (macroInputDisplay) macroInputDisplay.textContent = val;
      if (macroInputCaption) macroInputCaption.innerHTML = `Từ bạn gõ: <code>${escapeHtml(val)}</code>`;
      if (macroOutputDisplay) macroOutputDisplay.innerHTML = escapeHtml(res.out) + '<span class="cursor"></span>';
      if (macroOutputCaption) macroOutputCaption.innerHTML = res.desc;
    });
  }

  if (macroClearBtn) {
    macroClearBtn.addEventListener('click', () => {
      if (macroLiveInput) {
        macroLiveInput.value = '';
        macroLiveInput.focus();
      }
      selectMacroPreset(currentMacroType, MACRO_DATA[currentMacroType].presets[0].key);
    });
  }

  // ==========================================================================
  // Macro Table Filters & Row Interactions
  // ==========================================================================
  const macroFilterBtns = document.querySelectorAll('.macro-filter-btn');
  const macroRows = document.querySelectorAll('.macro-row');

  if (macroFilterBtns.length) {
    macroFilterBtns.forEach(btn => {
      btn.addEventListener('click', () => {
        const filter = btn.dataset.filter;
        macroFilterBtns.forEach(b => b.classList.toggle('active', b === btn));
        macroRows.forEach(row => {
          if (filter === 'all' || row.dataset.type === filter) {
            row.style.display = '';
          } else {
            row.style.display = 'none';
          }
        });
        playSwitchSound();
      });
    });
  }

  function activateMacroRowTest(type, testKey) {
    if (type !== currentMacroType) {
      currentMacroType = type;
      macroTabButtons.forEach(b => b.classList.toggle('active', b.dataset.type === type));
      renderMacroPresets(type);
    }

    const data = MACRO_DATA[type];
    const foundPreset = data && data.presets.find(p => p.key === testKey);
    if (foundPreset) {
      selectMacroPreset(type, testKey);
    } else {
      if (macroPresetsContainer) {
        macroPresetsContainer.querySelectorAll('.macro-preset-btn').forEach(b => b.classList.remove('active'));
      }
      const res = expandMacroWord(testKey);
      if (macroInputDisplay) macroInputDisplay.textContent = testKey;
      if (macroInputCaption) macroInputCaption.innerHTML = `Từ viết tắt: <code>${escapeHtml(testKey)}</code>`;
      if (macroOutputDisplay) macroOutputDisplay.innerHTML = escapeHtml(res.out) + '<span class="cursor"></span>';
      if (macroOutputCaption) macroOutputCaption.innerHTML = res.desc;
      if (macroLiveInput) macroLiveInput.value = testKey;
    }

    playSwitchSound();

    const demoBox = document.querySelector('.macro-box');
    if (demoBox) {
      demoBox.scrollIntoView({ behavior: 'smooth', block: 'center' });
    }
  }

  if (macroRows.length) {
    macroRows.forEach(row => {
      row.addEventListener('click', () => {
        const type = row.dataset.type;
        const testKey = row.dataset.test;
        activateMacroRowTest(type, testKey);
      });
    });
  }

  // ==========================================================================
  // Live Typing Demos in "Tính năng nổi bật" (Features Section)
  // ==========================================================================
  const FEATURE_SCRIPTS = {
    tone: [
      { text: 'h' },
      { text: 'ho' },
      { text: 'hoa' },
      { text: 'hòa', status: 'Đặt dấu âm chính: hòa (thay vì hoà cũ)' },
      { pause: 800 },
      { text: 'hòa t' },
      { text: 'hòa th' },
      { text: 'hòa thu' },
      { text: 'hòa thuy' },
      { text: 'hòa thúy', status: 'Nguyên âm mở: thúy (thay vì thuý)' },
      { pause: 800 },
      { text: 'hòa thúy k' },
      { text: 'hòa thúy kh' },
      { text: 'hòa thúy kho' },
      { text: 'hòa thúy khoe' },
      { text: 'hòa thúy khỏe', status: 'Chuẩn chính tả mới, không lệch vị trí dấu' },
      { pause: 2000 }
    ],
    english: [
      { text: 'p' },
      { text: 'pa' },
      { text: 'pas' },
      { text: 'pass' },
      { text: 'pass ', status: 'Giữ nguyên từ: pass' },
      { pause: 800 },
      { text: 'pass e' },
      { text: 'pass er' },
      { text: 'pass err' },
      { text: 'pass erro' },
      { text: 'pass error' },
      { text: 'pass error ', status: 'Giữ nguyên: error (không bị nuốt)' },
      { pause: 800 },
      { text: 'pass error c' },
      { text: 'pass error co' },
      { text: 'pass error cof' },
      { text: 'pass error coff' },
      { text: 'pass error coffe' },
      { text: 'pass error coffee', status: 'Giữ nguyên: coffee (không biến dạng)' },
      { pause: 2000 }
    ],
    browser: [
      { text: 'g' },
      { text: 'go' },
      { text: 'goo' },
      { text: 'goog' },
      { text: 'googl' },
      { text: 'google' },
      { text: 'google.', status: 'Ký tự đầu g không bị lặp thành ggoogle' },
      { text: 'google.c' },
      { text: 'google.co' },
      { text: 'google.com' },
      { pause: 1200 },
      { text: '' },
      { text: 't' },
      { text: 'to' },
      { text: 'toa' },
      { text: 'toan' },
      { text: 'toán', status: 'Chữ t đầu sạch sẽ, không bị ttoán' },
      { text: 'toán học ' },
      { pause: 1800 }
    ],
    restore: [
      { text: 't' },
      { text: 'th' },
      { text: 'thu' },
      { text: 'thư' },
      { text: 'thử', status: 'Đang gõ từ "thử"...' },
      { pause: 600 },
      { text: 'thửr' },
      { text: 'thưr', status: 'Gõ lặp "r" sai vần ➔ Tự hủy dấu, trả phím gốc' },
      { pause: 1000 },
      { text: 'b' },
      { text: 'ba' },
      { text: 'ban' },
      { text: 'banc' },
      { text: 'banc', status: 'Ký tự không hợp lệ tiếng Việt ➔ Giữ nguyên phím thô' },
      { pause: 1800 }
    ],
    macro: [
      { text: 'k' },
      { text: 'ko' },
      { text: 'không ', status: 'ko ➔ không' },
      { pause: 800 },
      { text: 'không K' },
      { text: 'không Ko' },
      { text: 'không Không ', status: 'Ko ➔ Không (tự viết hoa đầu)' },
      { pause: 800 },
      { text: 'không Không K' },
      { text: 'không Không KO' },
      { text: 'không Không KHÔNG ', status: 'KO ➔ KHÔNG (tự viết hoa toàn bộ)' },
      { pause: 2000 }
    ],
    backspace: [
      { text: 't' },
      { text: 'to' },
      { text: 'toi' },
      { text: 'toi ', status: 'Gõ lỡ quên bỏ dấu...' },
      { pause: 250 },
      { text: 'toi n' },
      { text: 'toi na' },
      { text: 'toi nay' },
      { text: 'toi nay ', status: 'Đã gõ sang từ tiếp theo...' },
      { pause: 900 },
      { text: 'toi nay', status: 'Phát hiện thiếu dấu ➔ Backspace lùi về' },
      { pause: 140 },
      { text: 'toi na' },
      { pause: 110 },
      { text: 'toi n' },
      { pause: 110 },
      { text: 'toi ' },
      { pause: 140 },
      { text: 'toi', status: 'Quay lại đúng từ "toi" thiếu dấu' },
      { pause: 550 },
      { text: 'tối', status: 'Bấm phím "s" ➔ Tự động bù dấu sắc: "tối"!' },
      { pause: 700 },
      { text: 'tối ' },
      { pause: 250 },
      { text: 'tối n' },
      { text: 'tối na' },
      { text: 'tối nay' },
      { text: 'tối nay ', status: 'Gõ tiếp bình thường, không cần xóa từ đầu!' },
      { pause: 2200 }
    ],
    toggle: [
      { text: '[VIE] ' },
      { text: '[VIE] x' },
      { text: '[VIE] xi' },
      { text: '[VIE] xin' },
      { text: '[VIE] xin c' },
      { text: '[VIE] xin ch' },
      { text: '[VIE] xin cha' },
      { text: '[VIE] xin chao' },
      { text: '[VIE] xin chào ', status: 'Đang gõ ở chế độ tiếng Việt' },
      { pause: 800 },
      { text: '[Ctrl+Space ➔ ENG] xin chào ', status: 'Nhấn phím tắt tùy chọn (Ctrl+Shift, Ctrl+Space, Alt+Z...) sang tiếng Anh' },
      { pause: 600 },
      { text: '[ENG] xin chào h' },
      { text: '[ENG] xin chào he' },
      { text: '[ENG] xin chào hel' },
      { text: '[ENG] xin chào hell' },
      { text: '[ENG] xin chào hello', status: 'Tự do gán mọi tổ hợp phím, chuyển đổi êm mượt' },
      { pause: 2000 }
    ],
    sound: [
      { text: 'Holy Panda: ' },
      { text: 'Holy Panda: c' },
      { text: 'Holy Panda: cạ' },
      { text: 'Holy Panda: cạch' },
      { text: 'Holy Panda: cạch ' },
      { pause: 300 },
      { text: 'Holy Panda: cạch t' },
      { text: 'Holy Panda: cạch tạ' },
      { text: 'Holy Panda: cạch tạch' },
      { text: 'Holy Panda: cạch tạch...' },
      { pause: 500 },
      { text: 'Holy Panda: cạch tạch êm ái ♪', status: '13 bộ âm thanh phím cơ sống động' },
      { pause: 2000 }
    ]
  };

  const featureScreens = document.querySelectorAll('.feature-demo-screen');
  let isFeaturesSectionVisible = false;
  const activeFeatureTimers = {};

  function startFeatureTyping(featureName, container) {
    const script = FEATURE_SCRIPTS[featureName];
    if (!script || !container) return;

    const textEl = container.querySelector('.typing-text');
    const statusEl = container.querySelector('.feature-screen-status:not(.audio-status)');
    let index = 0;

    function step() {
      if (!isFeaturesSectionVisible) return;
      const item = script[index];
      if (!item) {
        index = 0;
        activeFeatureTimers[featureName] = setTimeout(step, 500);
        return;
      }

      if (item.text !== undefined && textEl) {
        textEl.textContent = item.text;
      }
      if (item.status && statusEl) {
        statusEl.textContent = item.status;
      }

      const delay = item.pause || Math.floor(Math.random() * 45 + 75);
      index = (index + 1) % script.length;
      activeFeatureTimers[featureName] = setTimeout(step, delay);
    }

    step();
  }

  function startAllFeatureDemos() {
    featureScreens.forEach(screen => {
      const feat = screen.dataset.feature;
      if (activeFeatureTimers[feat]) {
        clearTimeout(activeFeatureTimers[feat]);
      }
      startFeatureTyping(feat, screen);
    });
  }

  function stopAllFeatureDemos() {
    Object.keys(activeFeatureTimers).forEach(key => {
      clearTimeout(activeFeatureTimers[key]);
      delete activeFeatureTimers[key];
    });
  }

  const featuresSection = document.getElementById('features');
  if (featuresSection && 'IntersectionObserver' in window) {
    const observer = new IntersectionObserver((entries) => {
      entries.forEach(entry => {
        if (entry.isIntersecting) {
          isFeaturesSectionVisible = true;
          startAllFeatureDemos();
        } else {
          isFeaturesSectionVisible = false;
          stopAllFeatureDemos();
        }
      });
    }, { threshold: 0.1 });
    observer.observe(featuresSection);
  } else {
    isFeaturesSectionVisible = true;
    startAllFeatureDemos();
  }

  // Mini Sound Button click handler on Card 6
  const miniSoundBtn = document.getElementById('miniSoundBtn');
  if (miniSoundBtn) {
    miniSoundBtn.addEventListener('click', (e) => {
      e.stopPropagation();
      initAudio();
      playSwitchSound();
      setTimeout(() => playSwitchSound(), 90);
      setTimeout(() => playSwitchSound(), 180);
      setTimeout(() => playSwitchSound(true), 290);
    });
  }

  // Copy path buttons (e.g. soundpack config path)
  document.querySelectorAll('.btn-copy-mini').forEach(btn => {
    btn.addEventListener('click', (e) => {
      e.stopPropagation();
      const textToCopy = btn.getAttribute('data-copy');
      if (!textToCopy) return;

      const performFeedback = () => {
        const origText = btn.textContent;
        btn.textContent = 'Đã chép!';
        btn.classList.add('copied');
        setTimeout(() => {
          btn.textContent = origText;
          btn.classList.remove('copied');
        }, 1800);
      };

      if (navigator.clipboard && window.isSecureContext) {
        navigator.clipboard.writeText(textToCopy)
          .then(performFeedback)
          .catch(() => fallbackCopy(textToCopy, performFeedback));
      } else {
        fallbackCopy(textToCopy, performFeedback);
      }
    });
  });

  function fallbackCopy(text, callback) {
    const textArea = document.createElement('textarea');
    textArea.value = text;
    textArea.style.position = 'fixed';
    textArea.style.left = '-999999px';
    textArea.style.top = '-999999px';
    document.body.appendChild(textArea);
    textArea.focus();
    textArea.select();
    try {
      document.execCommand('copy');
      if (callback) callback();
    } catch (err) {
      console.error('Fallback copy failed', err);
    }
    document.body.removeChild(textArea);
  }

  // Initialize macro playground
  renderMacroPresets('normal');

  // First user interaction initializes Web Audio
  document.addEventListener('click', () => initAudio(), { once: true });
});

