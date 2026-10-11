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
          <div class="telemetry-value">~549<span class="telemetry-unit">KB</span></div>
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

      <!-- Real-world Task Manager Benchmark Showcase -->
      <div class="benchmark-showcase">
        <div class="benchmark-showcase-header">
          <div class="benchmark-badge">ĐỐI CHIẾU THỰC TẾ TRÊN WINDOWS TASK MANAGER</div>
          <h3 class="benchmark-title">Tiêu thụ RAM thực tế: Ẩn cửa sổ vs. Mở cửa sổ</h3>
          <p class="benchmark-subtitle">
            Minh chứng trực tiếp từ Task Manager khi chạy song song <strong>Kora</strong> cùng <strong>UniKey</strong> và <strong>OpenKey</strong>:
          </p>
        </div>

        <div class="benchmark-cards-grid">
          <!-- Card 1: Ẩn cửa sổ -->
          <div class="benchmark-card">
            <div class="benchmark-card-header">
              <div class="benchmark-status-badge">
                <span class="status-dot"></span>
                <span>ẨN CỬA SỔ (CHẠY NGẦM KHAY HỆ THỐNG)</span>
              </div>
              <div class="benchmark-ram-val">0.7 MB RAM</div>
            </div>
            <div class="benchmark-img-container">
              <img src="images/benchmark-tray.png" alt="Task Manager khi Kora ẩn cửa sổ - 0.7 MB RAM" loading="lazy">
            </div>
            <div class="benchmark-card-desc">
              <div class="benchmark-metric-row win">
                <span class="proc-name">Kora (Rust):</span>
                <span class="proc-num">0.7 MB</span>
                <span class="proc-tag">Tự động giải phóng bộ nhớ</span>
              </div>
              <div class="benchmark-metric-row">
                <span class="proc-name">UniKey / OpenKey:</span>
                <span class="proc-num">2.4 MB</span>
                <span class="proc-tag">Nặng gấp ~3.5 lần</span>
              </div>
              <p class="benchmark-analysis">
                🔍 <strong>Nhận xét:</strong> Khi đóng cửa sổ về khay hệ thống (không có mũi tên <code>&gt;</code>), Kora tự động cắt tỉa working set xuống chỉ còn <strong>0.7 MB RAM</strong>, tiết kiệm hơn <strong>70%</strong> so với các bộ gõ truyền thống.
              </p>
            </div>
          </div>

          <!-- Card 2: Mở cửa sổ -->
          <div class="benchmark-card">
            <div class="benchmark-card-header">
              <div class="benchmark-status-badge active-state">
                <span class="status-dot active"></span>
                <span>MỞ CỬA SỔ (BẢNG ĐIỀU KHIỂN ĐANG BẬT)</span>
              </div>
              <div class="benchmark-ram-val">1.5 MB RAM</div>
            </div>
            <div class="benchmark-img-container">
              <img src="images/benchmark-window.png" alt="Task Manager khi Kora mở cửa sổ - 1.5 MB RAM" loading="lazy">
            </div>
            <div class="benchmark-card-desc">
              <div class="benchmark-metric-row win">
                <span class="proc-name">Kora (Rust):</span>
                <span class="proc-num">1.5 MB</span>
                <span class="proc-tag">Đang hiển thị toàn bộ giao diện</span>
              </div>
              <div class="benchmark-metric-row">
                <span class="proc-name">OpenKey / UniKey:</span>
                <span class="proc-num">2.4 MB</span>
                <span class="proc-tag">Vẫn giữ 2.4 MB</span>
              </div>
              <p class="benchmark-analysis">
                🔍 <strong>Nhận xét:</strong> Ngay cả khi đang mở bảng điều khiển (có mũi tên <code>&gt;</code> trong Task Manager), Kora chỉ tiêu tốn <strong>1.5 MB RAM</strong> — con số này vẫn nhẹ hơn gần <strong>1 MB</strong> so với UniKey & OpenKey.
              </p>
            </div>
          </div>
        </div>
      </div>
    </div>
  </section>"####
}
