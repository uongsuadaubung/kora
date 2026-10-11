//! Macro demo & table component

pub fn render() -> &'static str {
    r####"  <!-- Smart Macro Playground -->
  <section class="section demo-section" id="macro">
    <div class="container">
      <div class="section-header">
        <div class="section-chip">GÕ TẮT THÔNG MINH</div>
        <h2 class="section-title">3 Kiểu gõ tắt linh hoạt</h2>
        <p class="section-subtitle">Tùy biến bảng viết tắt theo thói quen của bạn: đổi hoa/thường cả từ, viết tắt phụ âm đầu hoặc phụ âm cuối.</p>
      </div>

      <!-- Macro Type Tabs -->
      <div class="macro-tabs">
        <button class="macro-tab-btn active" data-type="normal">
          <span>Gõ tắt cả từ</span>
          <span class="badge-pill">Hoa / Thường</span>
        </button>
        <button class="macro-tab-btn" data-type="start">
          <span>Phụ âm đầu</span>
          <span class="badge-pill">f &rarr; ph</span>
        </button>
        <button class="macro-tab-btn" data-type="end">
          <span>Phụ âm cuối</span>
          <span class="badge-pill">g &rarr; ng</span>
        </button>
      </div>

      <div class="demo-box macro-box">
        <div class="demo-topbar">
          <div class="demo-dots">
            <span class="dot"></span>
            <span class="dot"></span>
            <span class="dot"></span>
          </div>
          <div class="demo-title" id="macroModeTitle">Kiểu 1: Gõ tắt cả từ (Tự động đổi hoa / thường)</div>
          <div class="macro-rule-hint" id="macroRuleHint">Chỉ cần cài 1 từ viết tắt, tự động đổi hoa thường</div>
        </div>

        <div class="demo-presets" id="macroPresetsContainer">
          <span class="preset-label">Từ mẫu:</span>
          <button class="preset-btn macro-preset-btn active" data-macro="ko">ko</button>
          <button class="preset-btn macro-preset-btn" data-macro="Ko">Ko</button>
          <button class="preset-btn macro-preset-btn" data-macro="KO">KO</button>
          <button class="preset-btn macro-preset-btn" data-macro="dc">dc</button>
          <button class="preset-btn macro-preset-btn" data-macro="vn">vn</button>
        </div>

        <div class="demo-display">
          <div class="comparison-column macro-input-col">
            <div class="col-header">
              <span class="col-tag">TỪ VIẾT TẮT</span>
            </div>
            <div class="display-box macro-input-display" id="macroInputDisplay">ko</div>
            <div class="display-caption" id="macroInputCaption">Từ bạn gõ khi soạn thảo văn bản hàng ngày.</div>
          </div>

          <div class="comparison-divider">
            <div class="divider-line"></div>
            <span class="vs-badge">&rarr;</span>
            <div class="divider-line"></div>
          </div>

          <div class="comparison-column kora-col">
            <div class="col-header">
              <span class="col-tag good">KORA TỰ ĐỘNG MỞ RỘNG</span>
            </div>
            <div class="display-box kora-display" id="macroOutputDisplay">không<span class="cursor"></span></div>
            <div class="display-caption" id="macroOutputCaption">Tự động nhận diện từ viết tắt: <code>ko</code> &rarr; <strong>không</strong>.</div>
          </div>
        </div>

        <div class="demo-interactive-area">
          <div class="input-wrapper">
            <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" class="input-icon"><polyline points="4 17 10 11 4 5"/><line x1="12" y1="19" x2="20" y2="19"/></svg>
            <input type="text" id="macroLiveInput" placeholder="Tự gõ thử từ viết tắt tại đây (ví dụ: ko, Ko, KO, fa, sag, tih)..." autocomplete="off" spellcheck="false">
            <button class="clear-btn" id="macroClearBtn" title="Xóa">✕</button>
          </div>
          <div class="demo-hint">Gõ từ viết tắt hoặc chọn các từ mẫu / bấm vào bảng quy tắc bên dưới để thử nghiệm trực tiếp</div>
        </div>
      </div>

      <!-- Macro Rules Table (Collapsible Accordion) -->
      <details class="accordion-details macro-accordion">
        <summary class="accordion-summary">
          <div class="summary-left">
            <span class="summary-icon">📋</span>
            <span class="summary-text"><strong>Bảng tra cứu 10 quy tắc gõ tắt tham khảo</strong> &amp; Lưu ý khi dùng gõ tắt phụ âm</span>
          </div>
          <span class="summary-badge">Xem chi tiết &darr;</span>
        </summary>

        <div class="macro-table-wrapper">
        <div class="macro-table-header">
          <div class="macro-table-title-area">
            <h3 class="macro-table-title">Ví dụ các quy tắc gõ tắt tham khảo</h3>
            <p class="macro-table-subtitle">Minh họa 3 cơ chế gõ tắt Kora hỗ trợ &mdash; nhấp vào bất kỳ dòng nào để xem cách hoạt động</p>
          </div>
          <div class="macro-table-filter">
            <button class="macro-filter-btn active" data-filter="all">Tất cả (10)</button>
            <button class="macro-filter-btn" data-filter="normal">Cả từ (4)</button>
            <button class="macro-filter-btn" data-filter="start">Phụ âm đầu (3)</button>
            <button class="macro-filter-btn" data-filter="end">Phụ âm cuối (3)</button>
          </div>
        </div>

        <div class="macro-table-box">
          <table class="macro-table">
            <thead>
              <tr>
                <th class="th-key">Phím tắt</th>
                <th class="th-arrow"></th>
                <th class="th-val">Thay thế bằng</th>
                <th class="th-type">Phân loại</th>
                <th class="th-desc">Cơ chế hoạt động &amp; Ví dụ thực tế</th>
                <th class="th-action">Thử nghiệm</th>
              </tr>
            </thead>
            <tbody id="macroTableRows">
              <!-- Normal entries -->
              <tr class="macro-row" data-type="normal" data-test="ko">
                <td class="td-key"><kbd class="macro-kbd">ko</kbd></td>
                <td class="td-arrow">&rarr;</td>
                <td class="td-val"><strong>không</strong></td>
                <td class="td-type"><span class="macro-tag tag-normal">Cả từ</span></td>
                <td class="td-desc">Tự động giữ dạng chữ: gõ <kbd>ko</kbd> &rarr; <em>không</em>, <kbd>Ko</kbd> &rarr; <em>Không</em>, <kbd>KO</kbd> &rarr; <em>KHÔNG</em></td>
                <td class="td-action"><button class="btn-try-macro" data-macro="ko" data-type="normal">Thử ngay</button></td>
              </tr>
              <tr class="macro-row" data-type="normal" data-test="dc">
                <td class="td-key"><kbd class="macro-kbd">dc</kbd></td>
                <td class="td-arrow">&rarr;</td>
                <td class="td-val"><strong>được</strong></td>
                <td class="td-type"><span class="macro-tag tag-normal">Cả từ</span></td>
                <td class="td-desc">Tự động giữ dấu và viết hoa: gõ <kbd>dc</kbd> &rarr; <em>được</em>, <kbd>Dc</kbd> &rarr; <em>Được</em>, <kbd>DC</kbd> &rarr; <em>ĐƯỢC</em></td>
                <td class="td-action"><button class="btn-try-macro" data-macro="dc" data-type="normal">Thử ngay</button></td>
              </tr>
              <tr class="macro-row" data-type="normal" data-test="vn">
                <td class="td-key"><kbd class="macro-kbd">vn</kbd></td>
                <td class="td-arrow">&rarr;</td>
                <td class="td-val"><strong>Việt Nam</strong></td>
                <td class="td-type"><span class="macro-tag tag-normal">Cả từ</span></td>
                <td class="td-desc">Viết tắt cụm từ nhiều chữ: gõ <kbd>vn</kbd> &rarr; <em>Việt Nam</em>, <kbd>VN</kbd> &rarr; <em>VIỆT NAM</em></td>
                <td class="td-action"><button class="btn-try-macro" data-macro="vn" data-type="normal">Thử ngay</button></td>
              </tr>
              <tr class="macro-row" data-type="normal" data-test="ng">
                <td class="td-key"><kbd class="macro-kbd">ng</kbd></td>
                <td class="td-arrow">&rarr;</td>
                <td class="td-val"><strong>người</strong></td>
                <td class="td-type"><span class="macro-tag tag-normal">Cả từ</span></td>
                <td class="td-desc">Từ thông dụng hàng ngày: gõ <kbd>ng</kbd> &rarr; <em>người</em>, <kbd>Ng</kbd> &rarr; <em>Người</em>, <kbd>NG</kbd> &rarr; <em>NGƯỜI</em></td>
                <td class="td-action"><button class="btn-try-macro" data-macro="ng" data-type="normal">Thử ngay</button></td>
              </tr>

              <!-- Start Consonant entries -->
              <tr class="macro-row" data-type="start" data-test="fong">
                <td class="td-key"><kbd class="macro-kbd">f</kbd></td>
                <td class="td-arrow">&rarr;</td>
                <td class="td-val"><strong>ph</strong></td>
                <td class="td-type"><span class="macro-tag tag-start">Phụ âm đầu</span></td>
                <td class="td-desc">Đi kèm nguyên âm: gõ <kbd>fa</kbd> &rarr; <em>pha</em>, <kbd>fong</kbd> &rarr; <em>phong</em> (đứng 1 mình như <code>f(x)</code> không bị đổi)</td>
                <td class="td-action"><button class="btn-try-macro" data-macro="fong" data-type="start">Thử ngay</button></td>
              </tr>
              <tr class="macro-row" data-type="start" data-test="ja">
                <td class="td-key"><kbd class="macro-kbd">j</kbd></td>
                <td class="td-arrow">&rarr;</td>
                <td class="td-val"><strong>gi</strong></td>
                <td class="td-type"><span class="macro-tag tag-start">Phụ âm đầu</span></td>
                <td class="td-desc">Đi kèm nguyên âm: gõ <kbd>ja</kbd> &rarr; <em>gia</em>, <kbd>jo</kbd> &rarr; <em>gio</em> (nếu gõ thêm dấu: <kbd>jos</kbd> &rarr; <em>gió</em>, <kbd>jeengs</kbd> &rarr; <em>giếng</em>)</td>
                <td class="td-action"><button class="btn-try-macro" data-macro="ja" data-type="start">Thử ngay</button></td>
              </tr>
              <tr class="macro-row" data-type="start" data-test="wa">
                <td class="td-key"><kbd class="macro-kbd">w</kbd></td>
                <td class="td-arrow">&rarr;</td>
                <td class="td-val"><strong>qu</strong></td>
                <td class="td-type"><span class="macro-tag tag-start">Phụ âm đầu</span></td>
                <td class="td-desc">Đi kèm nguyên âm: gõ <kbd>wa</kbd> &rarr; <em>qua</em>, <kbd>we</kbd> &rarr; <em>que</em>, <kbd>wi</kbd> &rarr; <em>qui</em> (nếu gõ thêm dấu: <kbd>woocs</kbd> &rarr; <em>quốc</em>)</td>
                <td class="td-action"><button class="btn-try-macro" data-macro="wa" data-type="start">Thử ngay</button></td>
              </tr>

              <!-- End Consonant entries -->
              <tr class="macro-row" data-type="end" data-test="sag">
                <td class="td-key"><kbd class="macro-kbd">g</kbd></td>
                <td class="td-arrow">&rarr;</td>
                <td class="td-val"><strong>ng</strong></td>
                <td class="td-type"><span class="macro-tag tag-end">Phụ âm cuối</span></td>
                <td class="td-desc">Đứng sau nguyên âm: gõ <kbd>sag</kbd> &rarr; <em>sang</em>, <kbd>mag</kbd> &rarr; <em>mang</em>, <kbd>lag</kbd> &rarr; <em>lang</em> (nếu gõ <kbd>ddag</kbd> &rarr; <em>đang</em>)</td>
                <td class="td-action"><button class="btn-try-macro" data-macro="sag" data-type="end">Thử ngay</button></td>
              </tr>
              <tr class="macro-row" data-type="end" data-test="tih">
                <td class="td-key"><kbd class="macro-kbd">h</kbd></td>
                <td class="td-arrow">&rarr;</td>
                <td class="td-val"><strong>nh</strong></td>
                <td class="td-type"><span class="macro-tag tag-end">Phụ âm cuối</span></td>
                <td class="td-desc">Đứng sau nguyên âm: gõ <kbd>tih</kbd> &rarr; <em>tinh</em>, <kbd>xih</kbd> &rarr; <em>xinh</em>, <kbd>mih</kbd> &rarr; <em>minh</em></td>
                <td class="td-action"><button class="btn-try-macro" data-macro="tih" data-type="end">Thử ngay</button></td>
              </tr>
              <tr class="macro-row" data-type="end" data-test="sak">
                <td class="td-key"><kbd class="macro-kbd">k</kbd></td>
                <td class="td-arrow">&rarr;</td>
                <td class="td-val"><strong>ch</strong></td>
                <td class="td-type"><span class="macro-tag tag-end">Phụ âm cuối</span></td>
                <td class="td-desc">Đứng sau nguyên âm: gõ <kbd>sak</kbd> &rarr; <em>sach</em>, <kbd>lik</kbd> &rarr; <em>lich</em> (kèm phím dấu: <kbd>saks</kbd> &rarr; <em>sách</em>, <kbd>thiks</kbd> &rarr; <em>thích</em>)</td>
                <td class="td-action"><button class="btn-try-macro" data-macro="sak" data-type="end">Thử ngay</button></td>
              </tr>
            </tbody>
          </table>
        </div>

        <div class="macro-table-footer">
          <div class="macro-footer-tip">
            <span class="tip-icon">💡</span>
            <span class="tip-text"><strong>Bạn nắm toàn quyền chủ động:</strong> Kora không cài sẵn bất kỳ từ viết tắt nào làm phiền bạn. Bạn hoàn toàn tự do thêm mới, sửa hoặc xóa quy tắc gõ tắt theo thói quen cá nhân trong Bảng điều khiển.</span>
          </div>

          <!-- Macro Caution Warning -->
          <div class="macro-warning-box">
            <div class="warning-header">
              <span class="warning-icon">⚠️</span>
              <span class="warning-title">Cân nhắc khi dùng gõ tắt phụ âm (Đầu / Cuối)</span>
            </div>
            <p class="warning-text">
              Gõ tắt phụ âm giúp bạn gõ tiếng Việt siêu tốc nhưng cũng có thể gây phiền toái khi bạn gõ từ tiếng Anh hoặc thuật ngữ chuyên ngành do bị biến đổi ký tự ngoài ý muốn:
            </p>
            <div class="warning-grid">
              <div class="warning-item">
                <span class="warning-item-rule">f &rarr; ph</span>
                <span class="warning-item-desc">Gõ <em>fong</em> &rarr; <strong>phong</strong>, nhưng gõ <em>file</em> &rarr; <strong>phile</strong>, <em>fast</em> &rarr; <strong>phast</strong>, <em>fix</em> &rarr; <strong>phix</strong></span>
              </div>
              <div class="warning-item">
                <span class="warning-item-rule">w &rarr; qu</span>
                <span class="warning-item-desc">Gõ <em>wa</em> &rarr; <strong>qua</strong>, nhưng gõ <em>web</em> &rarr; <strong>queb</strong>, <em>win</em> &rarr; <strong>quin</strong>, <em>word</em> &rarr; <strong>quord</strong></span>
              </div>
              <div class="warning-item">
                <span class="warning-item-rule">j &rarr; gi</span>
                <span class="warning-item-desc">Gõ <em>ja</em> &rarr; <strong>gia</strong>, nhưng gõ <em>job</em> &rarr; <strong>giob</strong>, <em>join</em> &rarr; <strong>gioin</strong>, <em>json</em> &rarr; <strong>gison</strong></span>
              </div>
              <div class="warning-item">
                <span class="warning-item-rule">g &rarr; ng</span>
                <span class="warning-item-desc">Gõ <em>sag</em> &rarr; <strong>sang</strong>, nhưng gõ <em>bag</em> &rarr; <strong>bang</strong>, <em>tag</em> &rarr; <strong>tang</strong>, <em>bug</em> &rarr; <strong>bung</strong></span>
              </div>
              <div class="warning-item">
                <span class="warning-item-rule">k &rarr; ch</span>
                <span class="warning-item-desc">Gõ <em>sak</em> &rarr; <strong>sach</strong>, nhưng gõ <em>link</em> &rarr; <strong>linch</strong>, <em>disk</em> &rarr; <strong>disch</strong>, <em>check</em> &rarr; <strong>chench</strong></span>
              </div>
              <div class="warning-item">
                <span class="warning-item-rule">h &rarr; nh</span>
                <span class="warning-item-desc">Gõ <em>tih</em> &rarr; <strong>tinh</strong>, nhưng gõ <em>with</em> &rarr; <strong>witnh</strong>, <em>path</em> &rarr; <strong>patnh</strong></span>
              </div>
            </div>
            <div class="warning-advice">
              👉 <strong>Lời khuyên:</strong> Nếu công việc thường xuyên phải gõ tiếng Anh hoặc code, bạn nên ưu tiên dùng <strong>Gõ tắt cả từ (Kiểu 1)</strong> và chỉ kích hoạt gõ tắt phụ âm khi thực sự cần soạn thảo văn bản tiếng Việt tốc độ cao.
            </div>
          </div>
        </div>
      </details>
    </div>
  </section>"####
}
