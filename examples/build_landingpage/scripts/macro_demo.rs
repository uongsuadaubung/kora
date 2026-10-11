//! Smart Macro interactive playground and table filters

pub fn render() -> &'static str {
    r####"  // ==========================================================================
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
  }"####
}
