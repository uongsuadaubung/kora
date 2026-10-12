//! Performance metrics and Task Manager benchmark showcase component

pub fn render() -> &'static str {
    r####"  <!-- Performance & Engineering Metrics -->
  <section class="section" id="performance">
    <div class="container">
      <div class="section-header">
        <div class="section-chip">HIỆU NĂNG</div>
        <h2 class="section-title">Gọn nhẹ, mượt mà và bền bỉ</h2>
        <p class="section-subtitle">Khởi động ngay tức khắc, chạy ngầm êm ái với mức tiêu thụ tài nguyên gần như bằng không.</p>
      </div>

      <div class="telemetry-grid">
        <div class="telemetry-card highlight">
          <div class="telemetry-value">&lt; 1<span class="telemetry-unit">MB</span></div>
          <div class="telemetry-label">Bộ nhớ RAM sử dụng</div>
          <div class="telemetry-sub">Chỉ tốn chưa đầy 1 MB RAM, hoàn toàn không làm nặng máy tính của bạn</div>
          <div class="telemetry-bar"><div class="bar-fill cyan" style="width: 100%"></div></div>
        </div>

        <div class="telemetry-card">
          <div class="telemetry-value">0<span class="telemetry-unit">độ trễ</span></div>
          <div class="telemetry-label">Phản hồi tức thì</div>
          <div class="telemetry-sub">Chữ xuất hiện ngay lập tức dưới đầu ngón tay, cảm giác gõ dính phím mượt mà</div>
          <div class="telemetry-bar"><div class="bar-fill green" style="width: 100%"></div></div>
        </div>

        <div class="telemetry-card">
          <div class="telemetry-value">~1<span class="telemetry-unit">MB</span></div>
          <div class="telemetry-label">Dung lượng siêu nhỏ gọn</div>
          <div class="telemetry-sub">Tải về trong 1 giây, mở lên là dùng được ngay, không cần cài đặt rườm rà</div>
          <div class="telemetry-bar"><div class="bar-fill crimson" style="width: 100%"></div></div>
        </div>

        <div class="telemetry-card">
          <div class="telemetry-value">100<span class="telemetry-unit">%</span></div>
          <div class="telemetry-label">An toàn & Riêng tư</div>
          <div class="telemetry-sub">Hoạt động hoàn toàn ngoại tuyến, không gửi bất kỳ dữ liệu phím bấm nào ra ngoài</div>
          <div class="telemetry-bar"><div class="bar-fill purple" style="width: 100%"></div></div>
        </div>
      </div>
    </div>
  </section>"####
}
