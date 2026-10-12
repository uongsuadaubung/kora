//! FAQ accordion component

pub fn render() -> &'static str {
    r####"  <!-- FAQ Section -->
  <section class="section" id="faq">
    <div class="container">
      <div class="section-header">
        <div class="section-chip">HỎI ĐÁP</div>
        <h2 class="section-title">Câu hỏi thường gặp</h2>
        <p class="section-subtitle">Giải đáp nhanh các thắc mắc phổ biến nhất khi bắt đầu sử dụng bộ gõ Kora.</p>
      </div>

      <div class="faq-grid">
        <div class="faq-card">
          <div class="faq-q">
            <span class="faq-icon">🛡️</span>
            <h3>Windows SmartScreen có báo file lạ không?</h3>
          </div>
          <p class="faq-a">
            Kora là phần mềm mã nguồn mở hoàn toàn miễn phí (chưa mua chứng chỉ số đắt tiền hàng năm từ Microsoft), nên khi tải về lần đầu, Windows SmartScreen có thể hiển thị cảnh báo bảo vệ. Bạn chỉ cần nhấn vào <strong>"More info" (Thông tin khác)</strong> &rarr; chọn <strong>"Run anyway" (Vẫn chạy)</strong>. Toàn bộ mã nguồn Kora đều công khai 100% trên GitHub để bạn tự kiểm chứng.
          </p>
        </div>

        <div class="faq-card">
          <div class="faq-q">
            <span class="faq-icon">💻</span>
            <h3>Kora hỗ trợ những phiên bản Windows nào?</h3>
          </div>
          <p class="faq-a">
            Kora hoạt động tối ưu trên <strong>Windows 10</strong> và <strong>Windows 11 (64-bit)</strong>. File chạy là dạng <strong>Portable độc lập</strong>, không cần cài đặt, không ghi rác vào Windows Registry và không yêu cầu quyền Quản trị viên (Administrator).
          </p>
        </div>

        <div class="faq-card">
          <div class="faq-q">
            <span class="faq-icon">⚡</span>
            <h3>Tôi có cần gỡ các bộ gõ khác trên máy không?</h3>
          </div>
          <p class="faq-a">
            Bạn không nhất thiết phải gỡ cài đặt, nhưng <strong>cần tắt hoặc thoát hoàn toàn</strong> các bộ gõ khác đang chạy ngầm trước khi bật Kora. Nếu để 2 bộ gõ cùng chạy song song, cả hai sẽ cùng can thiệp phím và gây ra hiện tượng lặp hoặc nhảy ký tự.
          </p>
        </div>

        <div class="faq-card">
          <div class="faq-q">
            <span class="faq-icon">⌨️</span>
            <h3>Làm sao để chuyển đổi nhanh giữa tiếng Việt và tiếng Anh?</h3>
          </div>
          <p class="faq-a">
            Mặc định bạn có thể dùng tổ hợp phím <code>Ctrl + Shift</code> hoặc click chuột trái trực tiếp vào biểu tượng icon <strong>V</strong> / <strong>E</strong> ở khay hệ thống. Ngoài ra, trong Bảng điều khiển bạn hoàn toàn có thể <strong>tự do đổi sang bất kỳ phím tắt nào</strong> (như <code>Ctrl + Space</code>, <code>Alt + Z</code>...) bằng cửa sổ nhận diện phím bấm trực quan, hoặc tắt hẳn phím chuyển nếu không muốn bị bấm nhầm.
          </p>
        </div>

        <div class="faq-card">
          <div class="faq-q">
            <span class="faq-icon">🌙</span>
            <h3>Máy tính Sleep hoặc khóa màn hình xong có bị mất tiếng Việt không?</h3>
          </div>
          <p class="faq-a">
            <strong>Hoàn toàn không!</strong> Kora được trang bị cơ chế tự phục hồi thông minh: dù bạn gập laptop, bấm khóa màn hình hay để máy tự ngủ sau thời gian dài, ngay khi bạn mở máy và đăng nhập lại, Kora sẽ tự động làm mới trạng thái để bạn tiếp tục gõ tiếng Việt mượt mà ngay lập tức — không lo bị mất tiếng Việt, kẹt phím hay nhảy chữ hoa kỳ dị (như <code>gÕ</code>, <code>tiẾng ViỆt</code>).
          </p>
        </div>
      </div>
    </div>
  </section>"####
}
