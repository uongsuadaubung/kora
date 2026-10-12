//! Interface showcase & design philosophy component

pub fn render() -> &'static str {
    r####"  <!-- App Interface Showcase -->
  <section class="section" id="interface">
    <div class="container">
      <div class="section-header">
        <div class="section-chip">GIAO DIỆN</div>
        <h2 class="section-title">Bảng điều khiển trực quan, chuẩn Windows 11</h2>
        <p class="section-subtitle">Giao diện Dark Mode nguyên bản hiện đại, các tab cấu hình rõ ràng, dễ sử dụng ngay từ lần đầu mở.</p>
      </div>

      <div class="ui-gallery-grid">
        <!-- Card 1: Tab Bộ gõ -->
        <div class="ui-card">
          <div class="ui-card-header">
            <span class="ui-dot"></span>
            <span class="ui-window-title">Kora • Tab Bộ gõ</span>
          </div>
          <div class="ui-img-frame">
            <img src="images/ui-tab-bogo.png" alt="Giao diện Kora - Tab Bộ gõ" loading="lazy">
          </div>
          <div class="ui-card-body">
            <span class="ui-tab-tag">TAB 1 • BỘ GÕ &amp; ÂM THANH</span>
            <h3 class="ui-card-title">Cấu hình kiểu gõ, phím chuyển &amp; âm thanh</h3>
            <ul class="ui-card-list">
              <li><strong>Kiểu gõ linh hoạt:</strong> Hỗ trợ Simple Telex, Telex chuẩn, VNI.</li>
              <li><strong>Tùy biến phím chuyển chế độ:</strong> Mặc định <code>Ctrl + Shift</code> hoặc tự do đổi sang bất kỳ phím nào (<code>Ctrl + Space</code>, <code>Alt + Z</code>...), có nút bật/tắt riêng.</li>
              <li><strong>Âm thanh phím cơ tích hợp:</strong> Tải tự động 13 bộ switch tuyển chọn ngay trong Bảng điều khiển (chọn switch, nghe thử và thanh trượt âm lượng trực quan).</li>
            </ul>
          </div>
        </div>

        <!-- Card 2: Tab Gõ tắt -->
        <div class="ui-card">
          <div class="ui-card-header">
            <span class="ui-dot"></span>
            <span class="ui-window-title">Kora • Tab Gõ tắt</span>
          </div>
          <div class="ui-img-frame">
            <img src="images/ui-tab-gotat.png" alt="Giao diện Kora - Tab Gõ tắt" loading="lazy">
          </div>
          <div class="ui-card-body">
            <span class="ui-tab-tag">TAB 2 • GÕ TẮT</span>
            <h3 class="ui-card-title">Quản lý viết tắt Macro</h3>
            <ul class="ui-card-list">
              <li><strong>Bảng từ viết tắt:</strong> Thêm, sửa, xóa quy tắc gõ tắt trực quan và tức thì.</li>
              <li><strong>Gõ tắt mọi chế độ:</strong> Cho phép gõ tắt cả khi đang ở chế độ tiếng Anh.</li>
              <li><strong>Thông minh:</strong> Tự nhận diện chữ hoa / chữ thường theo ngữ cảnh gõ.</li>
            </ul>
          </div>
        </div>

        <!-- Card 3: Tab Hệ thống -->
        <div class="ui-card">
          <div class="ui-card-header">
            <span class="ui-dot"></span>
            <span class="ui-window-title">Kora • Tab Hệ thống</span>
          </div>
          <div class="ui-img-frame">
            <img src="images/ui-tab-hethong.png" alt="Giao diện Kora - Tab Hệ thống" loading="lazy">
          </div>
          <div class="ui-card-body">
            <span class="ui-tab-tag">TAB 3 • HỆ THỐNG</span>
            <h3 class="ui-card-title">Khởi động &amp; Chẩn đoán</h3>
            <ul class="ui-card-list">
              <li><strong>Khởi động cùng Windows:</strong> Tự động chạy ngầm siêu nhẹ chỉ 0.7 MB RAM.</li>
              <li><strong>Chủ đề giao diện:</strong> Đồng bộ theo hệ thống (Auto), Dark Mode hoặc Light Mode.</li>
              <li><strong>Nhật ký chẩn đoán:</strong> Mở và xem file log chỉ với 1 click.</li>
            </ul>
          </div>
        </div>
      </div>

      <!-- Design Philosophy Callout -->
      <div class="philosophy-box">
        <div class="philosophy-header">
          <div class="philosophy-badge">TRIẾT LÝ THIẾT KẾ HIỆN ĐẠI</div>
          <h3 class="philosophy-title">Tối giản để tập trung 100% vào trải nghiệm gõ</h3>
          <p class="philosophy-desc">
            Không còn rừng checkbox kỹ thuật hay những tính năng từ thập niên 90 mà chẳng còn ai dùng. Kora sinh ra để làm đúng một việc duy nhất tốt nhất: <strong>gõ phím mượt mà, chuẩn xác và không gián đoạn mạch suy nghĩ</strong>.
          </p>
        </div>

        <div class="philosophy-grid">
          <div class="philosophy-item">
            <div class="philosophy-icon">
              <svg width="22" height="22" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><circle cx="12" cy="12" r="10"/><line x1="4.93" y1="4.93" x2="19.07" y2="19.07"/></svg>
            </div>
            <div class="philosophy-content">
              <h4>Nói "Không" với các bảng mã cổ (TCVN3, VNI-Windows...)</h4>
              <p>Thập kỷ này là của <strong>chuẩn quốc tế Unicode (UTF-8)</strong>. Ngày nay, 100% website, trình duyệt, Word, Excel và ứng dụng chat đều dùng Unicode. Việc nhồi nhét hàng chục bảng mã cổ xưa (TCVN3, VNI-Windows, BK HCM, VIQR...) chỉ làm rối mắt người dùng, nặng dung lượng và dễ gây lỗi nhảy font chữ ngoài ý muốn.</p>
            </div>
          </div>

          <div class="philosophy-item">
            <div class="philosophy-icon">
              <svg width="22" height="22" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M12 2v4M12 18v4M4.93 4.93l2.83 2.83M16.24 16.24l2.83 2.83M2 12h4M18 12h4M4.93 19.07l2.83-2.83M16.24 7.76l2.83-2.83"/></svg>
            </div>
            <div class="philosophy-content">
              <h4>Không bắt bạn phải làm "chuyên gia cấu hình"</h4>
              <p>Các bộ gõ truyền thống thường phơi bày hàng tá checkbox rối rắm: <em>"Sử dụng clipboard cho unicode", "Cho phép gõ tự do"...</em> cùng hàng loạt thiết lập mở rộng phức tạp khiến người dùng phổ thông không biết phải chọn thế nào. Kora tự động hóa toàn bộ logic thông minh bên dưới — mở lên là dùng hoàn hảo ngay lập tức.</p>
            </div>
          </div>

          <div class="philosophy-item">
            <div class="philosophy-icon">
              <svg width="22" height="22" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><polygon points="13 2 3 14 12 14 11 22 21 10 12 10 13 2"/></svg>
            </div>
            <div class="philosophy-content">
              <h4>Dồn toàn bộ tài nguyên cho trải nghiệm cốt lõi</h4>
              <p>Khi giải phóng phần mềm khỏi hàng nghìn dòng mã cũ kỹ không ai dùng đến, Kora có thể tập trung tối đa vào những thứ thực sự tạo nên sự khác biệt hàng ngày: <strong>gõ tiếng Anh không bị nuốt chữ</strong>, <strong>sửa dấu hồi quy thông minh</strong>, <strong>giả lập âm thanh phím cơ WASAPI sống động</strong> và mức tiêu thụ RAM siêu nhẹ <strong>dưới 1 MB</strong>.</p>
            </div>
          </div>
        </div>
      </div>
    </div>
  </section>"####
}
