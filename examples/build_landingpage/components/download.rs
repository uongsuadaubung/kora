//! Download CTA section component

pub fn render() -> &'static str {
    r####"  <!-- Download Section -->
  <section class="section" id="download">
    <div class="container download-container">
      <div class="download-card">
        <div class="download-badge">PHIÊN BẢN 0.3.0 • MIỄN PHÍ & MÃ NGUỒN MỞ</div>
        <h2 class="download-title">Bắt đầu sử dụng Kora</h2>
        <p class="download-desc">
          Tải bản chạy ngay (portable) mở lên là sử dụng ngay, không cần cài đặt rườm rà. Tương thích hoàn hảo Windows 10 &amp; 11 (64-bit), không cần quyền Quản trị viên (Admin).
        </p>

        <div class="download-buttons">
          <a href="https://github.com/uongsuadaubung/kora/releases/latest" class="btn btn-download-main">
            <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5"><path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4M7 10l5 5 5-5M12 15V3"/></svg>
            <div class="btn-text-block">
              <span class="btn-main-text">Tải Kora (.exe)</span>
              <span class="btn-sub-text">Bản Portable cho Windows 10 / 11 (64-bit)</span>
            </div>
          </a>

          <a href="#sound-guide" class="btn btn-download-sub">
            <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M9 18V5l12-2v13"/><circle cx="6" cy="18" r="3"/><circle cx="18" cy="16" r="3"/></svg>
            <div class="btn-text-block">
              <span class="btn-main-text">Gói âm thanh Switch</span>
              <span class="btn-sub-text">Xem script tải tự động từ GitHub</span>
            </div>
          </a>
        </div>

        <div class="download-guarantee">
          Mã nguồn mở theo giấy phép GPL-3.0 • Hoạt động offline 100% • Không theo dõi, không thu thập dữ liệu phím
        </div>
      </div>
    </div>
  </section>"####
}
