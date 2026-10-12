//! Features & soundpack guide component

pub fn render() -> &'static str {
    r####"  <!-- Key Features -->
  <section class="section" id="features">
    <div class="container">
      <div class="section-header">
        <div class="section-chip">TÍNH NĂNG NỔI BẬT</div>
        <h2 class="section-title">Gõ văn bản tự nhiên, không gián đoạn</h2>
        <p class="section-subtitle">Kora hiểu cách bạn gõ hàng ngày và tự động xử lý chuẩn xác, để bạn không còn phải bực mình vì lỗi nhảy chữ hay nuốt phím.</p>
      </div>

      <div class="specs-grid">
        <div class="spec-card">
          <span class="spec-tag">CHÍNH TẢ HIỆN ĐẠI</span>
          <div class="spec-title">Bỏ dấu chuẩn xác cho nguyên âm mở</div>
          <div class="spec-desc">Áp dụng chuẩn ngữ pháp mới cho các vần đôi mở như <code>hòa</code>, <code>thúy</code>, <code>khỏe</code> (thay vì lối đặt dấu cũ hoà, thuý). Dấu thanh luôn đặt đúng âm chính, không nhảy lộn xộn khi gõ nhanh.</div>
          <div class="feature-demo-screen" data-feature="tone">
            <div class="feature-screen-top">
              <span class="screen-dot"></span>
              <span class="screen-title">Quy tắc bỏ dấu mới</span>
            </div>
            <div class="feature-screen-content">
              <span class="typing-text"></span><span class="typing-cursor"></span>
            </div>
            <div class="feature-screen-status">Chuẩn âm chính hiện đại: hòa, thúy, khỏe</div>
          </div>
        </div>

        <div class="spec-card">
          <span class="spec-tag">TRÌNH DUYỆT</span>
          <div class="spec-title">Không lặp chữ trên thanh tìm kiếm</div>
          <div class="spec-desc">Khắc phục triệt để lỗi bị nhân đôi chữ cái đầu tiên (như <code>ggoogle</code>, <code>ttoán</code>) khi bạn tìm kiếm trên Chrome hay Edge. Mọi thao tác tìm kiếm hay nhập địa chỉ web đều mượt mà ngay từ chữ cái đầu.</div>
          <div class="feature-demo-screen" data-feature="browser">
            <div class="feature-screen-top">
              <span class="screen-dot"></span>
              <span class="screen-title">🔍 Thanh tìm kiếm trình duyệt</span>
            </div>
            <div class="feature-screen-content">
              <span class="typing-text"></span><span class="typing-cursor"></span>
            </div>
            <div class="feature-screen-status">Ký tự đầu tiên không bị nhân đôi (sạch hoàn toàn)</div>
          </div>
        </div>

        <div class="spec-card">
          <span class="spec-tag">TỰ ĐỘNG</span>
          <div class="spec-title">Tự động khôi phục phím khi gõ sai từ</div>
          <div class="spec-desc">Khi bạn gõ nhầm phím hoặc tổ hợp ký tự không có thật trong tiếng Việt, Kora tự động hoàn lại các ký tự thô ban đầu, ngăn chặn việc nuốt phím hoặc tạo ra ký tự rác khó chịu.</div>
          <div class="feature-demo-screen" data-feature="restore">
            <div class="feature-screen-top">
              <span class="screen-dot"></span>
              <span class="screen-title">Khôi phục phím thông minh</span>
            </div>
            <div class="feature-screen-content">
              <span class="typing-text"></span><span class="typing-cursor"></span>
            </div>
            <div class="feature-screen-status">Tự hoàn lại phím gốc khi phát hiện gõ sai vần</div>
          </div>
        </div>

        <div class="spec-card">
          <span class="spec-tag">TIỆN LỢI</span>
          <div class="spec-title">Sửa dấu sau khi đã bấm phím cách</div>
          <div class="spec-desc">Nếu lỡ gõ sang từ tiếp theo mới nhận ra từ trước bị thiếu dấu, bạn chỉ cần bấm Backspace lùi về từ cũ rồi gõ phím dấu. Kora sẽ tự động thêm dấu vào từ trước giúp bạn gõ tiếp ngay mà không phải xóa đi gõ lại từ đầu.</div>
          <div class="feature-demo-screen" data-feature="backspace">
            <div class="feature-screen-top">
              <span class="screen-dot"></span>
              <span class="screen-title">Sửa dấu hồi quy</span>
            </div>
            <div class="feature-screen-content">
              <span class="typing-text"></span><span class="typing-cursor"></span>
            </div>
            <div class="feature-screen-status">Lùi về từ cũ + gõ phím dấu ➔ tự động sửa dấu và gõ tiếp</div>
          </div>
        </div>

        <div class="spec-card">
          <span class="spec-tag">PHÍM TẮT TỰ DO</span>
          <div class="spec-title">Tùy biến phím chuyển Anh / Việt theo ý thích</div>
          <div class="spec-desc">Mặc định dùng <code>Ctrl + Shift</code> hoặc tự do đổi sang bất kỳ phím nào (như <code>Ctrl + Space</code>, <code>Alt + Z</code>...). Có cửa sổ nhận diện phím bấm trực quan, nút đặt lại và công tắc bật/tắt chống bấm nhầm khi chơi game.</div>
          <div class="feature-demo-screen" data-feature="toggle">
            <div class="feature-screen-top">
              <span class="screen-dot"></span>
              <span class="screen-title">Tùy biến phím chuyển chế độ</span>
            </div>
            <div class="feature-screen-content">
              <span class="typing-text"></span><span class="typing-cursor"></span>
            </div>
            <div class="feature-screen-status">Tự do gán mọi tổ hợp phím, chuyển đổi êm mượt</div>
          </div>
        </div>

        <div class="spec-card">
          <span class="spec-tag">ÂM THANH PHÍM</span>
          <div class="spec-title">Giả lập âm thanh phím cơ sống động</div>
          <div class="spec-desc">Trải nghiệm tiếng gõ phím cơ chân thực với 13 bộ switch tuyển chọn (Alps SKCM Blue, Holy Panda, Cherry MX...). Tích hợp sẵn tính năng tải và kích hoạt 1-click trực tiếp trong Tab Âm thanh.</div>
          <div class="feature-demo-screen" data-feature="sound">
            <div class="feature-screen-top">
              <span class="screen-dot"></span>
              <span class="screen-title">Âm thanh phím cơ</span>
            </div>
            <div class="feature-screen-content">
              <span class="typing-text"></span><span class="typing-cursor"></span>
            </div>
            <div class="feature-screen-status audio-status">
              <span>Holy Panda</span>
              <button class="btn-play-sound-mini" id="miniSoundBtn">▶ Nghe thử</button>
            </div>
          </div>
          <div class="sound-quick-tip">
            <span class="tip-icon">⚡</span>
            <span>Tích hợp sẵn nút tải 13 bộ switch tự động trong Bảng điều khiển. <a href="#sound-guide">Xem hướng dẫn chi tiết &darr;</a></span>
          </div>
        </div>

        <div class="spec-card">
          <span class="spec-tag">BỀN BỈ &amp; ỔN ĐỊNH</span>
          <div class="spec-title">Tự phục hồi sau khi máy ngủ &amp; khóa màn hình</div>
          <div class="spec-desc">Chấm dứt hoàn toàn lỗi mất tiếng Việt hoặc tự viết hoa kỳ dị (<code>gÕ</code>, <code>tiẾng ViỆt</code>) mỗi khi laptop gập máy, khóa màn hình (<code>Win + L</code>) hay máy tự ngủ. Kora tự động nhận diện và làm mới trạng thái sẵn sàng để bạn gõ chuẩn xác ngay tức thì mà không cần khởi động lại ứng dụng.</div>
          <div class="feature-demo-screen" data-feature="resume">
            <div class="feature-screen-top">
              <span class="screen-dot"></span>
              <span class="screen-title">Tự phục hồi khi thức giấc</span>
            </div>
            <div class="feature-screen-content">
              <span class="typing-text"></span><span class="typing-cursor"></span>
            </div>
            <div class="feature-screen-status">Tự động đồng bộ và sẵn sàng gõ tiếng Việt ngay khi mở máy</div>
          </div>
        </div>

        <div class="spec-card">
          <span class="spec-tag">ĐA HỒ SƠ ỨNG DỤNG</span>
          <div class="spec-title">Tự đổi cấu hình theo Game &amp; Phần mềm</div>
          <div class="spec-desc">Tự động phát hiện ứng dụng bạn đang trỏ tới để kích hoạt cấu hình tương ứng. Thoải mái tạo profile riêng cho từng tựa game, IDE lập trình hay phần mềm đồ họa với cơ chế kế thừa phân tầng thông minh.</div>
          <div class="feature-demo-screen" data-feature="profile">
            <div class="feature-screen-top">
              <span class="screen-dot"></span>
              <span class="screen-title">Nhận diện ứng dụng tự động</span>
            </div>
            <div class="feature-screen-content">
              <span class="typing-text"></span><span class="typing-cursor"></span>
            </div>
            <div class="feature-screen-status">Tự động chuyển đổi profile theo ngữ cảnh ứng dụng</div>
          </div>
        </div>

        <div class="spec-card">
          <span class="spec-tag">BẢO VỆ PHÍM BẤM</span>
          <div class="spec-title">Đổi phím vật lý &amp; Khóa phím chơi game</div>
          <div class="spec-desc">Đổi phím vật lý theo thói quen (như <code>Caps Lock ➔ Ctrl</code>, <code>Insert ➔ Delete</code>) và khóa an toàn các phím bấm nhầm khi chơi game (khóa phím <code>Windows</code>, <code>Insert</code>...) mà không cần cài thêm phần mềm ngoài.</div>
          <div class="feature-demo-screen" data-feature="remap">
            <div class="feature-screen-top">
              <span class="screen-dot"></span>
              <span class="screen-title">Đổi phím &amp; Khóa phím</span>
            </div>
            <div class="feature-screen-content">
              <span class="typing-text"></span><span class="typing-cursor"></span>
            </div>
            <div class="feature-screen-status">Lọc sạch phím thừa, bảo vệ thao tác khi chơi game</div>
          </div>
        </div>
      </div>

      <!-- Soundpack Installation Guide -->
      <div class="soundpack-guide-card" id="sound-guide">
        <div class="guide-header">
          <div class="guide-badge">TẢI TỰ ĐỘNG BUILT-IN 1-CLICK</div>
          <h3 class="guide-title">Tải và kích hoạt âm thanh phím cơ ngay trong Kora</h3>
          <p class="guide-subtitle">File chạy Kora được tối ưu siêu nhẹ (~1 MB) nên không kèm sẵn file âm thanh lớn. Bạn chỉ cần 1 click trực tiếp trong Bảng điều khiển để kích hoạt trọn bộ 13 switch tuyển chọn:</p>
        </div>

        <div class="guide-steps-grid">
          <div class="guide-step-item">
            <div class="step-num">1</div>
            <div class="step-content">
              <strong>Mở Bảng điều khiển &amp; Tab Âm thanh</strong>
              <p>Nhấp đúp chuột vào biểu tượng Kora ở khay hệ thống (hoặc chuột phải chọn <em>Bảng điều khiển...</em>), sau đó chọn tab <strong>Âm thanh gõ</strong>.</p>
              <div class="guide-status-note">
                <span class="note-dot"></span>
                <span>Giao diện trực quan, đồng bộ với giao diện hệ thống.</span>
              </div>
            </div>
          </div>

          <div class="guide-step-item">
            <div class="step-num">2</div>
            <div class="step-content">
              <strong>Bấm nút tải tự động 1-click</strong>
              <p>Trong thẻ <em>Tải gói âm thanh switch</em>, nhấn nút <strong>📥 Tải gói âm thanh switch (13 bộ)</strong>. Kora sẽ tự động tải từ đám mây và giải nén ngay vào máy bạn.</p>
              <div class="guide-status-note">
                <span class="note-dot"></span>
                <span>Tải &amp; giải nén hoàn toàn tự động chỉ trong vài giây.</span>
              </div>
            </div>
          </div>

          <div class="guide-step-item">
            <div class="step-num">3</div>
            <div class="step-content">
              <strong>Chọn switch yêu thích &amp; Thưởng thức</strong>
              <p>Tích chọn <strong>Bật âm thanh gõ phím</strong>, chọn switch bạn muốn (Holy Panda, Cherry MX Blue, Alps SKCM Blue...) và bấm <strong>Nghe thử</strong> để cảm nhận độ đầm tay!</p>
              <div class="guide-status-note">
                <span class="note-dot"></span>
                <span>Nhận diện ngay lập tức mà không cần khởi động lại ứng dụng.</span>
              </div>
            </div>
          </div>
        </div>

        <!-- Custom Soundpack & File Naming Info (Collapsible Accordion) -->
        <details class="accordion-details sound-accordion">
          <summary class="accordion-summary">
            <div class="summary-left">
              <span class="summary-icon">🎛️</span>
              <span class="summary-text"><strong>Tự tạo gói âm thanh theo sở thích</strong> &amp; Danh sách tên file chuẩn (.wav)</span>
            </div>
            <span class="summary-badge">Xem chi tiết &darr;</span>
          </summary>

          <div class="custom-soundpack-box">
          <div class="custom-sound-header">
            <div class="custom-sound-badge">TÙY BIẾN KHÔNG GIỚI HẠN</div>
            <h4 class="custom-sound-title">Tự tạo gói âm thanh theo sở thích của riêng bạn</h4>
            <p class="custom-sound-desc">
              Bạn không hề bị giới hạn trong 13 bộ switch có sẵn! Kora cho phép bạn đưa <strong>bất kỳ file âm thanh nào bạn thích</strong> vào sử dụng — từ tiếng gõ phím cơ custom, máy đánh chữ cổ điển, meme hài hước, cho đến âm thanh anime, lồng tiếng troll :))) Chỉ cần tạo một thư mục mới trong <code>%USERPROFILE%\.config\kora\switches\&lt;Tên_Gói&gt;\</code> và thả các file <code>.wav</code> vào, Kora sẽ tự động nhận diện ngay trong Bảng điều khiển.
            </p>
          </div>

          <div class="naming-rules-wrapper">
            <div class="rules-table-title">Danh sách tên file chuẩn cố định cho từng phím (định dạng <code>.wav</code>):</div>

            <div class="naming-categories">
              <!-- Group 1 -->
              <div class="naming-group-card">
                <div class="group-title">1. Phím soạn thảo chính</div>
                <div class="group-rows">
                  <div class="naming-row">
                    <span class="row-key">Phím Cách (Spacebar)</span>
                    <span class="row-file"><code>space.wav</code></span>
                  </div>
                  <div class="naming-row">
                    <span class="row-key">Phím Xóa (Backspace)</span>
                    <span class="row-file"><code>backspace.wav</code></span>
                  </div>
                  <div class="naming-row">
                    <span class="row-key">Phím Xuống dòng (Enter)</span>
                    <span class="row-file"><code>enter.wav</code></span>
                  </div>
                  <div class="naming-row">
                    <span class="row-key">Phím Tab</span>
                    <span class="row-file"><code>tab.wav</code></span>
                  </div>
                  <div class="naming-row">
                    <span class="row-key">Phím Xóa tới (Delete)</span>
                    <span class="row-file"><code>delete.wav</code></span>
                  </div>
                </div>
              </div>

              <!-- Group 2 -->
              <div class="naming-group-card">
                <div class="group-title">2. Phím bổ trợ & Hệ thống</div>
                <div class="group-rows">
                  <div class="naming-row">
                    <span class="row-key">Phím Shift</span>
                    <span class="row-file"><code>shift.wav</code></span>
                  </div>
                  <div class="naming-row">
                    <span class="row-key">Phím Control (Ctrl)</span>
                    <span class="row-file"><code>ctrl.wav</code></span>
                  </div>
                  <div class="naming-row">
                    <span class="row-key">Phím Alt</span>
                    <span class="row-file"><code>alt.wav</code></span>
                  </div>
                  <div class="naming-row">
                    <span class="row-key">Phím Windows (Win)</span>
                    <span class="row-file"><code>win.wav</code></span>
                  </div>
                  <div class="naming-row">
                    <span class="row-key">Phím Caps Lock</span>
                    <span class="row-file"><code>capslock.wav</code></span>
                  </div>
                  <div class="naming-row">
                    <span class="row-key">Phím Thoát (Esc)</span>
                    <span class="row-file"><code>esc.wav</code></span>
                  </div>
                  <div class="naming-row">
                    <span class="row-key">Phím F1 đến F12 (Fn)</span>
                    <span class="row-file"><code>fn.wav</code></span>
                  </div>
                </div>
              </div>

              <!-- Group 3 -->
              <div class="naming-group-card">
                <div class="group-title">3. Phím điều hướng & Con trỏ</div>
                <div class="group-rows">
                  <div class="naming-row">
                    <span class="row-key">4 Phím Mũi tên</span>
                    <span class="row-file"><code>arrow.wav</code></span>
                  </div>
                  <div class="naming-row">
                    <span class="row-key">Phím Home</span>
                    <span class="row-file"><code>home.wav</code></span>
                  </div>
                  <div class="naming-row">
                    <span class="row-key">Phím End</span>
                    <span class="row-file"><code>end.wav</code></span>
                  </div>
                  <div class="naming-row">
                    <span class="row-key">Phím Page Up</span>
                    <span class="row-file"><code>pageup.wav</code></span>
                  </div>
                  <div class="naming-row">
                    <span class="row-key">Phím Page Down</span>
                    <span class="row-file"><code>pagedown.wav</code></span>
                  </div>
                  <div class="naming-row">
                    <span class="row-key">Phím Insert</span>
                    <span class="row-file"><code>insert.wav</code></span>
                  </div>
                </div>
              </div>

              <!-- Group 4 -->
              <div class="naming-group-card highlight-card">
                <div class="group-title">4. Phím gõ chữ thường (Tất cả ký tự còn lại)</div>
                <div class="group-rows">
                  <div class="naming-row-full">
                    <p><strong>Áp dụng cho:</strong> Toàn bộ chữ cái (A-Z), chữ số (0-9) và các dấu câu.</p>
                    <p><strong>Tên file chuẩn:</strong> Đánh số thứ tự <code>1.wav</code>, <code>2.wav</code>, <code>3.wav</code>, <code>4.wav</code>, <code>5.wav</code>... (hoặc bất kỳ tên file nào không trùng với 18 phím cố định ở trên).</p>
                    <p class="row-subtext">💡 <strong>Tự động luân phiên:</strong> Khi bạn để nhiều file số (1.wav, 2.wav...), Kora sẽ tự động đổi tiếng luân phiên sau mỗi lần gõ phím. Nếu bất kỳ phím chức năng nào không có file riêng, Kora sẽ tự động phát âm thanh gõ thường này làm mặc định.</p>
                  </div>
                </div>
              </div>
            </div>
          </div>

          <div class="custom-sound-footer">
            <span class="format-tip">✨ <strong>Chuẩn âm thanh hỗ trợ:</strong> Định dạng file <code>.wav</code> (chuẩn PCM 8/16/24-bit hoặc 32-bit float, Mono hoặc Stereo). Engine WASAPI thuần Rust sẽ tự động nạp vào RAM và resample về 48kHz, gõ siêu nhạy với độ trễ cực thấp (&lt; 1ms).</span>
          </div>
        </details>
      </div>
    </div>
  </section>"####
}
