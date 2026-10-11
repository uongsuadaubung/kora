//! Interactive typing playground simulator component

pub fn render() -> &'static str {
    r####"  <!-- Interactive Playground -->
  <section class="section demo-section" id="playground">
    <div class="container">
      <div class="section-header">
        <div class="section-chip">Trải nghiệm</div>
        <h2 class="section-title">Xem cách Kora xử lý văn bản</h2>
        <p class="section-subtitle">So sánh trực tiếp kết quả gõ các từ tiếng Anh thông dụng giữa Kora và các bộ gõ truyền thống.</p>
      </div>

      <div class="demo-box">
        <div class="demo-topbar">
          <div class="demo-dots">
            <span class="dot"></span>
            <span class="dot"></span>
            <span class="dot"></span>
          </div>
          <div class="demo-title">Hộp gõ thử nghiệm</div>
          <div class="sound-toggle-wrapper">
            <label class="sound-toggle">
              <input type="checkbox" id="soundCheckbox" checked>
              <span class="toggle-slider"></span>
              <span class="toggle-text">Âm thanh phím</span>
            </label>
          </div>
        </div>

        <div class="demo-presets">
          <span class="preset-label">Từ mẫu:</span>
          <button class="preset-btn active" data-word="error">error</button>
          <button class="preset-btn" data-word="pass">pass</button>
          <button class="preset-btn" data-word="coffee">coffee</button>
        </div>

        <div class="demo-display">
          <div class="comparison-column legacy-col">
            <div class="col-header">
              <span class="col-tag bad">BỘ GÕ THÔNG THƯỜNG</span>
            </div>
            <div class="display-box legacy-display" id="legacyOutput">eror</div>
            <div class="display-caption" id="legacyCaption">Phím <code>r</code> thứ hai bị hiểu nhầm là lệnh hủy dấu, nuốt mất chữ thành <strong>eror</strong>.</div>
          </div>

          <div class="comparison-divider">
            <div class="divider-line"></div>
            <span class="vs-badge">VS</span>
            <div class="divider-line"></div>
          </div>

          <div class="comparison-column kora-col">
            <div class="col-header">
              <span class="col-tag good">KORA</span>
            </div>
            <div class="display-box kora-display" id="koraOutput">error<span class="cursor"></span></div>
            <div class="display-caption" id="koraCaption">Tự nhận diện từ tiếng Anh, giữ nguyên chữ <strong>error</strong>.</div>
          </div>
        </div>

        <div class="demo-interactive-area">
          <div class="input-wrapper">
            <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" class="input-icon"><polyline points="4 17 10 11 4 5"/><line x1="12" y1="19" x2="20" y2="19"/></svg>
            <input type="text" id="liveInput" placeholder="Tự gõ thử một từ bất kỳ tại đây (ví dụ: pass, error, coffee)..." autocomplete="off" spellcheck="false">
            <button class="clear-btn" id="clearBtn" title="Xóa">✕</button>
          </div>
        </div>
      </div>
    </div>
  </section>"####
}
