//! Comparison table component

pub fn render() -> &'static str {
    r####"  <!-- Comparison Table -->
  <section class="section" id="comparison">
    <div class="container">
      <div class="section-header">
        <div class="section-chip">SO SÁNH</div>
        <h2 class="section-title">Khác biệt trong từng trải nghiệm hàng ngày</h2>
        <p class="section-subtitle">So sánh cảm giác sử dụng thực tế giữa Kora và các bộ gõ tiếng Việt trước đây.</p>
      </div>

      <div class="comparison-table-wrapper">
        <table class="comparison-table">
          <thead>
            <tr>
              <th class="feature-col">Tình huống hàng ngày</th>
              <th class="legacy-th">Bộ gõ truyền thống</th>
              <th class="kora-th">Kora</th>
            </tr>
          </thead>
          <tbody>
            <tr>
              <td class="feature-name">
                <strong>Gõ từ tiếng Anh quen thuộc</strong>
                <span>error, pass, coffee, class</span>
              </td>
              <td class="legacy-td">
                <span class="badge-fail">Bị nuốt chữ</span>
                <p>Chữ bị biến thành <code>eror</code>, <code>pas</code>, <code>cofee</code>. Phải gõ lặp 3 lần (<code>passs</code>) hoặc tắt tiếng Việt.</p>
              </td>
              <td class="kora-td">
                <span class="badge-win">Giữ nguyên vẹn</span>
                <p>Tự động nhận diện từ tiếng Anh và giữ đúng từ, không nuốt chữ, không cần gõ 3 lần.</p>
              </td>
            </tr>

            <tr>
              <td class="feature-name">
                <strong>Tìm kiếm trên Chrome & Edge</strong>
                <span>Gõ từ khóa trên thanh địa chỉ</span>
              </td>
              <td class="legacy-td">
                <span class="badge-fail">Dễ lặp chữ đầu</span>
                <p>Thường bị nhảy chữ thành <code>ggoogle</code>, <code>ttoán</code> gây khó chịu khi tìm kiếm.</p>
              </td>
              <td class="kora-td">
                <span class="badge-win">Gõ chuẩn xác</span>
                <p>Chữ hiển thị chuẩn từng ký tự ngay từ đầu, tìm kiếm hay nhập link web đều mượt mà.</p>
              </td>
            </tr>

            <tr>
              <td class="feature-name">
                <strong>Tính năng gõ tắt (viết tắt)</strong>
                <span>Tự động viết hoa / viết thường</span>
              </td>
              <td class="legacy-td">
                <span class="badge-neutral">Phải tạo nhiều lần</span>
                <p>Muốn gõ tắt cả chữ thường và hoa phải tạo thủ công từng từ như <code>ko</code>, <code>Ko</code>, <code>KO</code>.</p>
              </td>
              <td class="kora-td">
                <span class="badge-win">Tự động đổi kiểu chữ</span>
                <p>Chỉ cần tạo 1 từ <code>ko</code> &rarr; <code>không</code>, Kora tự nhận diện để xuất <code>Không</code> hoặc <code>KHÔNG</code> theo cách bạn gõ.</p>
              </td>
            </tr>

            <tr>
              <td class="feature-name">
                <strong>Quên bỏ dấu trước khi bấm Space</strong>
                <span>Nhỡ bấm phím cách khi chưa xong dấu</span>
              </td>
              <td class="legacy-td">
                <span class="badge-fail">Phải gõ lại cả từ</span>
                <p>Xóa cách rồi gõ dấu sẽ thành <code>chao s</code>. Buộc bạn phải xóa hết cả từ để gõ lại từ đầu.</p>
              </td>
              <td class="kora-td">
                <span class="badge-win">Tự động thêm dấu</span>
                <p>Chỉ cần xóa cách rồi gõ dấu, chữ tự đổi thành <code>cháo</code> mà không mất công gõ lại.</p>
              </td>
            </tr>

            <tr>
              <td class="feature-name">
                <strong>Phím chuyển chế độ gõ</strong>
                <span>Chuyển đổi Anh / Việt (E / V)</span>
              </td>
              <td class="legacy-td">
                <span class="badge-neutral">Bị bó buộc</span>
                <p>Chỉ cho chọn cố định giữa <code>Ctrl + Shift</code> hoặc <code>Alt + Z</code>, không thể tùy biến theo ý thích.</p>
              </td>
              <td class="kora-td">
                <span class="badge-win">Tự do đổi phím 100%</span>
                <p>Gán bất kỳ phím nào (như <code>Ctrl + Space</code>, <code>Alt + Z</code>...) qua giao diện nhận diện thông minh, có nút bật/tắt riêng.</p>
              </td>
            </tr>

            <tr>
              <td class="feature-name">
                <strong>Âm thanh gõ phím cơ</strong>
                <span>Cảm giác âm thanh khi nhấn phím</span>
              </td>
              <td class="legacy-td">
                <span class="badge-fail">Không có sẵn</span>
                <p>Phải cài thêm ứng dụng ngoài cồng kềnh, ngốn hàng trăm MB bộ nhớ RAM của máy.</p>
              </td>
              <td class="kora-td">
                <span class="badge-win">Hỗ trợ 13 bộ switch</span>
                <p>Hỗ trợ phát âm thanh phím cơ sống động qua WASAPI, chỉ tốn ~1.5 MB RAM khi bật và không cần cài phần mềm ngoài nặng nề.</p>
              </td>
            </tr>

            <tr>
              <td class="feature-name">
                <strong>Thức dậy từ Sleep / Màn hình khóa</strong>
                <span>Gập máy, Win + L, Windows tự ngủ</span>
              </td>
              <td class="legacy-td">
                <span class="badge-fail">Kẹt phím &amp; Lỗi gõ hoa</span>
                <p>Thường bị mất tiếng Việt, kẹt phím hoặc lỗi tự động viết hoa kỳ dị (<code>gÕ</code>, <code>tiẾng ViỆt</code>). Phải khởi động lại ứng dụng hoặc bấm phím lung tung để gõ lại.</p>
              </td>
              <td class="kora-td">
                <span class="badge-win">Tự phục hồi 100%</span>
                <p>Tự động nhận diện khi máy thức giấc và mở khóa, làm mới trạng thái bàn phím để bạn tiếp tục gõ tiếng Việt mượt mà ngay lập tức.</p>
              </td>
            </tr>

            <tr>
              <td class="feature-name">
                <strong>Mức độ ngốn máy & RAM</strong>
                <span>Tài nguyên khi mở chạy ngầm cả ngày</span>
              </td>
              <td class="legacy-td">
                <span class="badge-neutral">2.4 MB – 5 MB+</span>
                <p>Nhiều bộ gõ khác thường chiếm từ 2.4 MB đến hàng chục MB RAM dù ẩn hay mở cửa sổ.</p>
              </td>
              <td class="kora-td">
                <span class="badge-win">Chỉ ~1 MB RAM</span>
                <p>Khi ẩn cửa sổ chỉ tốn <strong>~1 MB RAM</strong>. Ngay cả khi mở toàn bộ giao diện bảng điều khiển cũng chỉ <strong>~1.5 MB</strong>.</p>
              </td>
            </tr>

            <tr>
              <td class="feature-name">
                <strong>Bảng mã &amp; Tùy chọn cài đặt</strong>
                <span>Mức độ phức tạp khi thiết lập</span>
              </td>
              <td class="legacy-td">
                <span class="badge-neutral">Rườm rà, nhiều tùy chọn cổ</span>
                <p>Nhiều bảng mã cũ từ thập niên 90 (TCVN3, VNI...) và hàng chục checkbox kỹ thuật rườm rà dễ gây rối mắt và nhầm lẫn.</p>
              </td>
              <td class="kora-td">
                <span class="badge-win">Tối giản chuẩn Unicode</span>
                <p>Chuẩn Unicode hiện đại 100%. Tự động hóa ngầm toàn bộ cơ chế, không bắt bạn phải làm chuyên gia cấu hình để gõ văn bản mượt mà.</p>
              </td>
            </tr>

            <tr>
              <td class="feature-name">
                <strong>Độ mượt mà khi gõ & chơi game</strong>
                <span>Phản hồi khi gõ nhanh hoặc chơi game</span>
              </td>
              <td class="legacy-td">
                <span class="badge-neutral">Đôi khi khựng nhịp phím</span>
                <p>Dễ xung đột các phím di chuyển khi chơi game nếu quên tắt chế độ tiếng Việt.</p>
              </td>
              <td class="kora-td">
                <span class="badge-win">Hoàn toàn mượt mà</span>
                <p>Nhạy bén tức thì, phản hồi êm ái kể cả khi bạn gõ văn bản cực nhanh hay chơi game.</p>
              </td>
            </tr>
          </tbody>
        </table>
      </div>
    </div>
  </section>"####
}
