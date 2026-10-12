//! Feature live typing animations, copy buttons, sound button, and initialization

pub fn render() -> &'static str {
    r####"  // ==========================================================================
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
    ],
    resume: [
      { text: '💤 Mở máy từ Sleep / Win+L...' },
      { pause: 900 },
      { text: '⚡ Tự động làm mới trạng thái bàn phím', status: 'Tự phục hồi thông minh sau khi mở máy' },
      { pause: 800 },
      { text: 'x' },
      { text: 'xi' },
      { text: 'xin' },
      { text: 'xin c' },
      { text: 'xin ch' },
      { text: 'xin cha' },
      { text: 'xin chào', status: 'Gõ tiếng Việt mượt mà (không lỗi hoa, không mất dấu)' },
      { pause: 600 },
      { text: 'xin chào V' },
      { text: 'xin chào Vi' },
      { text: 'xin chào Việ' },
      { text: 'xin chào Việt' },
      { text: 'xin chào Việt N' },
      { text: 'xin chào Việt Na' },
      { text: 'xin chào Việt Nam' },
      { text: 'xin chào Việt Nam!', status: '100% tự phục hồi, không cần khởi động lại app' },
      { pause: 2200 }
    ],
    profile: [
      { text: 'Chuyển sang 🎮 CS2 / Valorant...' },
      { pause: 800 },
      { text: 'Kora ➔ Kích hoạt profile "Game"', status: 'Khóa phím Win, tắt dấu tiếng Việt để di chuyển WASD' },
      { pause: 1200 },
      { text: 'Chuyển sang 💻 VS Code / Word...' },
      { pause: 800 },
      { text: 'Kora ➔ Kích hoạt profile "Code/Doc"', status: 'Bật tiếng Việt Telex, mở khóa phím tắt lập trình' },
      { pause: 2000 }
    ],
    remap: [
      { text: 'Bấm Caps Lock ➔ Chuyển thành phím Ctrl', status: 'Đổi phím vật lý theo thói quen lập trình' },
      { pause: 1200 },
      { text: 'Bấm phím Insert ➔ Chuyển thành Delete', status: 'Chống đè nhầm chế độ ghi đè văn bản' },
      { pause: 1200 },
      { text: 'Bấm phím Win khi chơi game ➔ Đã chặn', status: 'Khóa phím an toàn, không văng màn hình desktop' },
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
});"####
}
