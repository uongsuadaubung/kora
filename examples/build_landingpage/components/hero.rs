//! Hero banner component

pub fn render() -> &'static str {
    r####"  <!-- Hero Section -->
  <section class="hero">
    <div class="container hero-container">
      <div class="hero-tag">
        <span class="pulse-indicator"></span>
        <span>Bộ gõ tiếng Việt thế hệ mới cho Windows</span>
      </div>

      <h1 class="hero-title">
        Gõ tiếng Việt tự nhiên.<br>
        <span class="gradient-text">Không nuốt từ tiếng Anh.</span>
      </h1>

      <p class="hero-desc">
        Bộ gõ tối giản, mượt mà và chuẩn xác. Thiết kế để phím bấm luôn theo kịp mạch suy nghĩ của bạn.
      </p>

      <div class="hero-cta-group">
        <a href="#download" class="btn btn-hero-primary">
          <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5"><path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4M7 10l5 5 5-5M12 15V3"/></svg>
          <span>Tải Kora cho Windows</span>
        </a>
        <a href="#playground" class="btn btn-hero-secondary">
          <span>Gõ thử trực tiếp</span>
          <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M7 13l5 5 5-5M12 4v14"/></svg>
        </a>
      </div>
    </div>
  </section>"####
}
