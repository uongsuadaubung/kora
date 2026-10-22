# Cẩm nang Sử dụng UI Framework (`src/ui_framework/`)

Tài liệu này hướng dẫn chi tiết cách khởi tạo, dàn trang tự động bằng `CardLayout`, đọc/ghi dữ liệu và bắt sự kiện ngữ nghĩa cho toàn bộ các UI Component của **Kora** khi xây dựng các Tab, Panel hoặc Dialog tại tầng tiêu thụ (`src/ui/views/`).

> Về cơ chế kiến trúc bên dưới tầng `src/ui_framework/` (`OCM_*` Message Reflection, `GWLP_USERDATA` bit-packing, Subclassing), xem tài liệu riêng: **[`docs/UI_COMPONENT_ARCHITECTURE.md`](file:///c:/Users/kien.hm/Desktop/OpenKey/docs/UI_COMPONENT_ARCHITECTURE.md)**.

---

## 1. Khai báo & Quy ước Chung

Kiến trúc giao diện được tách bạch thành 2 tầng rõ ràng:
- **`src/ui_framework/` (Tầng Nền tảng UI — Zero Engine Dependency):** Chứa trọn bộ 9 UI Component tự chủ, `CardLayout`, `ThemePalette`, `tokens`, `theme` và `dialog` scaffold.
- **`src/ui/` (Tầng Tiêu thụ Ứng dụng):** Chỉ tập trung lắp ghép các View, Tab, Dialog và kết nối với `engine` thông qua `controller`.

Tại đầu mỗi file View hoặc Dialog trong `src/ui/views/`, bạn chỉ cần **1 dòng import duy nhất** từ `prelude`:

```rust
use crate::ui_framework::prelude::*;
```
*(Ghi chú: Các đường dẫn cũ như `crate::ui::components::*`, `crate::ui::colors::*`, `crate::ui::win32::*` vẫn được re-export tại `src/ui/mod.rs` để tương thích ngược 100%).*

### 5 Quy ước cốt lõi khi viết View:
1. **Không cần khai báo hằng số `IDC_*` (`Auto-ID`)**: Truyền `0` vào tham số `id` của bất kỳ hàm `create*` nào (`PushButton`, `CheckBox`, `ComboBox`, `TextBox`, `Slider`, `TabBar`, `ListBox`, `ActionTable`). Hệ thống sẽ tự động cấp phát một ID duy nhất từ bộ đếm nguyên tử (`resolve_id(0)`).
2. **Dàn trang tự động bằng `CardLayout::with_parent`**:
   - Khởi tạo `CardLayout::with_parent(parent, hfont_normal, hfont_bold)` để tự động giữ ngữ cảnh cửa sổ cha và font chữ.
   - Dùng các hàm **1-Line Row Builders** (`ly.page_header`, `ly.section_title`, `ly.label_row`, `ly.checkbox_row`) để vừa tính toạ độ `Y`, vừa tạo component, vừa tự động đăng ký ẩn/hiện (`track`) và đa ngôn ngữ (`track_i18n`) chỉ trong 1 dòng lệnh.
3. **Tự động Quản lý Ẩn/Hiện (`set_visible`) & Đa ngôn ngữ (`update_language`)**:
   - Không cần liệt kê mảng `set_hwnds_visible(&[...])` hay viết hàng chục dòng `set_text(strings....)` thủ công trong từng Tab.
   - Mọi component tạo qua row builder hoặc bọc trong `ly.track(...)` / `ly.track_i18n(..., |s| s.field)` đều được `self.layout.set_visible(visible)` và `self.layout.update_language(strings)` xử lý tự động chỉ với 1 dòng lệnh.
4. **Bắt sự kiện ngữ nghĩa (Semantic Event Matchers) hoặc Closure**:
   - Không cần tra cứu mã thông báo Win32 thô (`CBN_SELCHANGE`, `BN_CLICKED`, `EN_CHANGE`, `TB_THUMBTRACK`).
   - Sử dụng trực tiếp các hàm kiểm tra trên từng component:
     - `btn.is_clicked(ctrl_id, notify_code)` hoặc gắn trực tiếp `.on_click(|| { ... })`
     - `lbl.is_clicked(ctrl_id, notify_code)` (cho `Label` có `.as_link()`)
     - `chk.is_toggled(ctrl_id, notify_code)`
     - `cmb.is_changed(ctrl_id, notify_code)`
     - `txt.is_changed(ctrl_id, notify_code)`
     - `tabs.is_changed(ctrl_id, notify_code)`
     - `list.is_sel_changed(ctrl_id, notify_code)`
     - `slider.parse_scroll(wparam, lparam)` (trả về `Some((pos, is_dragging))` trong `on_hscroll`)
5. **An toàn (`Safe API`) & Quản lý Font**:
   - Mọi hàm `create*` đều là `pub fn` an toàn và trả về `Option<Self>` (dùng toán tử `?` khi khởi tạo).
   - Luôn dùng `get_app_font_normal()`, `get_app_font_bold()` hoặc `cached_font(size, weight)` (không bao giờ cần gọi `DeleteObject` cho font).

---

## 2. Dàn Trang, Ẩn/Hiện & Đa Ngôn Ngữ Tự Động với `CardLayout<S>` (`src/ui_framework/components/card_layout.rs`)

`CardLayout<S>` vừa là bộ dàn trang dọc tự động (tính toạ độ `Y` và đóng khung Card), vừa là **Container quản lý vòng đời hiển thị (`HWND`) và đa ngôn ngữ (`i18n`)** cho toàn bộ các component trong một Tab hoặc Sub-tab.

Đặc biệt, `CardLayout<S>` sử dụng **Typestate Pattern** ở thời điểm biên dịch (`OutsideCard` $\leftrightarrow$ `InCard`):
- `begin_card(self) -> CardLayout<InCard>`: Tiêu thụ trạng thái ngoài Card, trả về trạng thái trong Card (không thể gọi `begin_card()` hai lần liên tiếp hay gọi `paint_cards()` khi quên đóng Card).
- `end_card(self) -> CardLayout<OutsideCard>` / `end_card_at(self, bottom_y) -> CardLayout<OutsideCard>`: Đóng Card và trả lại trạng thái `OutsideCard`.

```rust
let mut ly = CardLayout::with_parent(parent, hfont_normal, hfont_bold);

// 1. Tạo tiêu đề trang (chỉ gọi được khi ở trạng thái OutsideCard):
ly.page_header(|s| s.tab_Macro, |s| s.sub_Macro)?;

// --- Card 1 (chuyển sang trạng thái CardLayout<InCard>) ---
let mut ly = ly.begin_card();

// 2. Tạo tiêu đề nhóm (chữ in hoa Secondary đậm) chỉ với 1 dòng:
ly.section_title(|s| s.section_general, 8)?;

// 3. Tạo dòng CheckBox tự động gắn i18n và tự động track HWND:
let chk_enable = ly.checkbox_row(|s| s.chk_spell_check, 22, 6, false, true)?;

// 4. Chia hàng tự động bằng Layout Engine (HStack, GridLayout, FormLayout, SplitLayout):
card.horizontal(28, 10, |row| {
    row.align(CrossAlign::Center);
    row.add_stretch(1, Some(cmb_left.hwnd()));
    row.add_fixed(20, None);
    row.add_stretch(1, Some(cmb_right.hwnd()));
});

// 5. Căn gióng tự động nhãn và nút bấm:
card.horizontal(28, 0, |row| {
    row.align(CrossAlign::Center);
    row.add_stretch(1, Some(lbl_desc.hwnd()));
    row.add_fixed(12, None);
    row.add_fixed(110, Some(btn_act.hwnd()));
});
let ly = ly.end_card();
```

#### Bảng phương thức của `CardLayout<S>`:
| Phương thức | Trạng thái `S` | Mô tả |
| :--- | :--- | :--- |
| `CardLayout::with_parent(parent, hfont_normal, hfont_bold)` | `OutsideCard` | Khởi tạo tại `layout::CARD_START_Y` (`112`) kèm ngữ cảnh cửa sổ cha & font chữ. |
| `ly.at(start_y)` | `OutsideCard` | Đặt lại toạ độ `Y` bắt đầu (ví dụ `.at(86)` cho các Tab có `page_header` hoặc không có thanh Sub-tab). |
| `ly.with_bounds(content_x, content_w)` | `OutsideCard` | Tuỳ chỉnh toạ độ `X` và độ rộng nội dung (hữu ích khi dùng trong Dialog hoặc Sub-card). |
| `ly.page_header(title_fn, sub_fn)` | `OutsideCard` | Tạo cặp `Label` tiêu đề trang (20px Bold + 18px Secondary) và tự động đăng ký `track_i18n`. |
| `ly.begin_card(self)` / `ly.begin_card_with_pad(pad)` | `OutsideCard` $\rightarrow$ `InCard` | Mở một khung thẻ Card mới (mặc định padding `14px`). |
| `ly.end_card(self)` / `ly.end_card_with_pad(pad)` | `InCard` $\rightarrow$ `OutsideCard` | Đóng khung thẻ Card hiện tại và tiến con trỏ xuống khoảng cách giữa 2 Card (`12px`). |
| `ly.end_card_at(self, bottom_y) -> CardLayout<OutsideCard>` | `InCard` $\rightarrow$ `OutsideCard` | Đóng Card tại một toạ độ đáy cố định (ví dụ `layout::CARD_BOTTOM_Y` cho các Card chứa bảng co giãn). |
| `ly.section_title(text_fn, gap)` | `InCard` | Tạo `Label<StaticText>` tiêu đề nhóm (Bold, cao 20px) và tự động đăng ký `track_i18n`. |
| `ly.label_row(text_fn, height, gap, bold)` | `InCard` | Tạo 1 dòng `Label<StaticText>` trải rộng hết chiều ngang Card và tự động đăng ký `track_i18n`. |
| `ly.checkbox_row(text_fn, height, gap, bold, checked)` | `InCard` | Tạo 1 dòng `CheckBox<TwoState>` trải rộng hết chiều ngang Card và tự động đăng ký `track_i18n`. |
| `ly.inheritable_checkbox_row(text_fn, height, gap, bold)` | `InCard` | Tạo 1 dòng `CheckBox<Inheritable>` (hỗ trợ kế thừa Hồ sơ) trải rộng hết chiều ngang Card và tự động đăng ký `track_i18n`. |
| `ly.two_col_combos(...)` | `InCard` | Tạo 1 hàng 2 cột (mỗi cột gồm 1 `Label` tiêu đề tự động `track_i18n` + 1 `ComboBox` bên dưới) chỉ với 1 lệnh. |
| `card.label(text_fn, bold)` | Mọi `S` | Tạo `Label` (zero-coordinate) tự động đăng ký `track_i18n`. Dùng kết hợp với các bộ bố cục layout. |
| `card.button(text_fn, bold, variant)` | Mọi `S` | Tạo `PushButton` (zero-coordinate) với `ButtonVariant` và tự động đăng ký `track_i18n`. |
| `card.checkbox(text_fn, bold, checked)` | Mọi `S` | Tạo `CheckBox<TwoState>` (zero-coordinate) tự động đăng ký `track_i18n`. |
| `card.inheritable_checkbox(text_fn, bold)` | Mọi `S` | Tạo `CheckBox<Inheritable>` (zero-coordinate) tự động đăng ký `track_i18n`. |
| `card.combo(items_fn)` | Mọi `S` | Tạo `ComboBox` (zero-coordinate) tự động đăng ký `track_combo_i18n`. |
| `ly.track(comp) -> C` | Mọi `S` | Đăng ký `HWND` của component vào danh sách quản lý ẩn/hiện tự động và trả về chính `comp`. |
| `ly.track_i18n(comp, text_fn) -> C` | Mọi `S` | Đăng ký `HWND` vào danh sách ẩn/hiện kèm hàm chọn chuỗi `TextFn` (`fn(&LanguageStrings) -> &'static str`). |
| `ly.track_combo_i18n(cmb, items_fn) -> ComboBox` | Mọi `S` | Vừa đăng ký ẩn/hiện cho `ComboBox`, vừa tự động nạp lại danh sách mục chọn và giữ nguyên mục đang chọn khi đổi ngôn ngữ. |
| `ly.track_table_i18n(tbl, cols_fn) -> ActionTable` | Mọi `S` | Vừa đăng ký ẩn/hiện cho `ActionTable`, vừa tự động dịch lại tiêu đề các cột khi đổi ngôn ngữ. |
| `ly.row(height, gap_after) -> i32` | Mọi `S` | Trả về toạ độ `Y` của dòng hiện tại, sau đó tiến con trỏ xuống `height + gap_after`. |
| `card.horizontal(h, gap, \|row\| ...)` | `InCard` | Bố cục hàng ngang tự động co giãn (`HStack`), hỗ trợ `add_stretch`, `add_fixed`, `add_spacer`. |
| `card.vertical(h, gap, \|col\| ...)` | `InCard` | Bố cục cột dọc tự động co giãn (`VStack`). |
| `card.grid(h, gap, cols, rows, \|grid\| ...)` | `InCard` | Bố cục ma trận lưới 2 chiều $M \times N$ (`GridLayout`). |
| `card.form(h, gap, \|form\| ...)` | `InCard` | Bố cục biểu mẫu chuẩn baseline `[Label] : [Control] : [Value]` (`FormLayout`). |
| `card.split(h, gap, dir, rule, \|split\| ...)` | `InCard` | Bố cục chia 2 vùng (`SplitLayout`) với kích thước cố định hoặc tỷ lệ. |
| `card.hotkey_row(...)` | `InCard` | Tạo hàng phím tắt chuẩn `[Label] --- [PillBadge] --- [Button]` chỉ với 1 dòng. |
| `ly.set_visible(visible)` | `OutsideCard` | Ẩn hoặc hiện toàn bộ các component đã được `track` / `track_*_i18n` trong layout này. |
| `ly.update_language(strings)` | `OutsideCard` | Tự động cập nhật văn bản đa ngôn ngữ cho mọi `Label`, `PushButton`, `CheckBox`, `ComboBox` và `ActionTable` đã đăng ký. |
| `ly.paint_cards(hdc, left, right)` | `OutsideCard` | Tự động vẽ toàn bộ các khung thẻ Card đã khai báo trong layout lên `hdc`. |

---

## 3. Chi Tiết Cách Dùng Từng Component

### 3.1. `Label<K>` — Nhãn văn bản, Liên kết & Huy hiệu Phím (`src/ui_framework/components/label.rs`)

Sử dụng Typestate `Label<K>` với 3 trạng thái:
- `Label<StaticText>` *(mặc định)*: Nhãn văn bản tĩnh (chỉ giữ 1 `HWND`, không tốn trường `id: u32`).
- `Label<Link>`: Liên kết bấm được (chuyển đổi qua `.as_link() -> Label<Link>`, tự cấp phát control ID và mở khóa hàm `.is_clicked(ctrl_id, notify_code)`).
- `Label<PillBadge>`: Huy hiệu phím bo tròn tự co giãn độ rộng theo nội dung chữ (chuyển đổi qua `.as_pill_badge(min_w, max_w) -> Label<PillBadge>`).

#### Các Enum cấu hình:
| Enum | Giá trị | Ý nghĩa hiển thị |
| :--- | :--- | :--- |
| **`LabelTone`** | `Primary` *(mặc định)* | Chữ chính (`palette.text_primary`) |
| | `Secondary` | Chữ phụ mờ, tiêu đề nhóm (`palette.text_secondary`) |
| | `Accent` | Chữ màu nhấn thương hiệu (`palette.accent`) |
| | `Link` | Chữ màu liên kết (`palette.text_link`) |
| **`SurfaceRole`** | `Card` *(mặc định)* | Nằm bên trong khung thẻ Card (`palette.bg_card`) |
| | `Window` | Nằm trực tiếp trên nền cửa sổ / Dialog (`palette.bg_window`) |
| | `Sidebar` | Nằm trên thanh điều hướng bên trái (`palette.bg_sidebar`) |
| | `Input` | Nằm trên nền ô nhập liệu (`palette.bg_input`) |

#### API Khởi tạo & Builder:
```rust
// 1. Nhãn thường nằm trên Card (Label<StaticText>, mặc định: Primary + Card)
let lbl_name = Label::create(parent, "Kiểu gõ:", x, y, 120, 20, hfont_normal)?;

// 2. Nhãn tiêu đề nhóm hoặc mô tả phụ (Secondary)
let lbl_section = Label::create(parent, "CẤU HÌNH BỘ GÕ", x, y, 250, 20, hfont_bold)?
    .with_tone(LabelTone::Secondary);

// 3. Nhãn biến thành Liên kết bấm được (Label<Link>, tự cấp phát ID và hỗ trợ .is_clicked)
let lbl_link: Label<Link> = Label::create(parent, "Trang chủ GitHub", x, y, 160, 20, hfont_normal)?
    .as_link();

// 4. Nhãn biến thành Huy hiệu Phím bo góc (Label<PillBadge>) tự động co giãn chiều rộng trong [min_w, max_w]
let lbl_hotkey: Label<PillBadge> = Label::create_centered(parent, "Ctrl + Shift", x, y, 124, 24, hfont_bold)?
    .with_tone(LabelTone::Accent)
    .as_pill_badge(80, 250);
```

#### Phương thức điều khiển & Sự kiện:
- `lbl.set_text("Nội dung mới")`: Đổi chữ và tự vẽ lại (trên `Label<PillBadge>`, chiều rộng huy hiệu sẽ tự động co giãn theo độ dài chuỗi mới!).
- `lbl.set_tone(LabelTone::Accent)`: Đổi màu chữ động lúc runtime.
- `lbl_link.is_clicked(ctrl_id, notify_code) -> bool`: Chỉ khả dụng trên `Label<Link>`, kiểm tra người dùng vừa click vào liên kết.

---

### 3.2. `PushButton` — Nút bấm (`src/ui_framework/components/button.rs`)

Nút bấm bo góc chống nháy (double-buffered), tự xử lý hiệu ứng hover, trạng thái bấm (`ODS_SELECTED`), trạng thái khoá (`WS_DISABLED`) và con trỏ bàn tay.

#### Các biến thể (`ButtonVariant`):
- `ButtonVariant::Primary`: Nút hành động chính (nền xanh `accent`, chữ trắng).
- `ButtonVariant::Secondary`: Nút phụ (nền `btn_secondary_bg`, viền `border`, chữ `text_primary`).
- `ButtonVariant::Danger`: Nút nguy hiểm dạng viền (nền đỏ nhạt, viền đỏ, chữ đỏ).
- `ButtonVariant::DangerSolid`: Nút hành động nguy hiểm chính dạng đặc (nền đỏ đặc, chữ trắng).
- `ButtonVariant::Ghost`: Nút ẩn nền, chỉ hiện nền khi rê chuột vào.

#### API Khởi tạo & Bắt sự kiện:
```rust
// Cách 1: Khởi tạo trong CardLayout không cần toạ độ (tự gắn HWND + i18n) và bố cục bằng HStack / GridLayout:
let btn_save = card.button(|s| s.btn_save, true, ButtonVariant::Primary)?;
card.horizontal(30, 0, |row| {
    row.add_fixed(120, Some(btn_save.hwnd()));
});

// Cách 2: Gắn trực tiếp hàm closure .on_click(...) ngay lúc tạo (tự chạy qua OCM_COMMAND):
let btn_open = PushButton::new(parent, 0, "Mở thư mục", hfont_normal)?;
btn_open.on_click(|| {
    // Tự động thực thi khi người dùng bấm nút!
});
```

---

### 3.3. `CheckBox<M>` — Hộp kiểm 2 Trạng thái & 3 Trạng thái Kế thừa (`src/ui_framework/components/checkbox.rs`)

Hộp kiểm tự vẽ hoàn toàn với Typestate `CheckBox<TwoState>` (cấu hình đơn `bool`) và `CheckBox<Inheritable>` (cấu hình hỗ trợ kế thừa theo Hồ sơ `Option<bool>`):

```rust
// 1. CheckBox<TwoState> — Chỉ dùng cho cờ toàn cục không theo Hồ sơ (ví dụ: Khởi động cùng Windows):
let chk_global: CheckBox<TwoState> = ly.checkbox_row(|s| s.check_autostart, 24, 6, false, false)?;
chk_global.set_checked(config.show_dialog_on_startup);
if self.chk_global.is_toggled(ctrl_id, notify_code) {
    let enabled: bool = self.chk_global.is_checked();
}

// 2. CheckBox<Inheritable> — Dùng cho các cờ hỗ trợ kế thừa giữa Root và Child Profile:
let chk_restore: CheckBox<Inheritable> = ly.inheritable_checkbox_row(|s| s.check_restore_wrong, 22, 4, false)?;
// Khi nạp dữ liệu (Root: child.is_none(), Child: child.is_some()):
chk_restore.set_inherited(
    child.is_none(),
    child.and_then(|c| c.restore_on_wrong_spelling),
    global_config.restore_on_wrong_spelling,
);
// Khi bắt sự kiện, đồng bộ 1 dòng cho cả Global & Child qua EditTarget::sync_checkbox:
if self.chk_restore.is_toggled(ctrl_id, notify_code) {
    crate::ui::controller::edit_selected(|mut target| {
        target.sync_checkbox(
            &self.chk_restore,
            |c| &mut c.restore_on_wrong_spelling,
            |p| &mut p.restore_on_wrong_spelling,
        );
    });
}
```

---

### 3.4. `ComboBox` — Menu thả xuống (`src/ui_framework/components/combobox.rs`)

Menu chọn 1 phần tử từ danh sách thả xuống (`CBS_DROPDOWNLIST`), tự động vẽ giao diện Sáng/Tối.

```rust
let cmb = ComboBox::create_with_items(
    parent, 0,
    x, y, 248, 150,
    hfont_normal,
    &["Telex", "VNI", "Simple Telex"],
    0,
)?;

// Bắt sự kiện khi người dùng chọn mục khác:
if self.cmb.is_changed(ctrl_id, notify_code) {
    if let Some(index) = self.cmb.get_selected() {
        // Xử lý index
    }
}
```

---

### 3.5. `TextBox` — Ô nhập văn bản (`src/ui_framework/components/textbox.rs`)

Ô nhập liệu một dòng có khung viền bo góc (tự đổi sang viền xanh `accent` khi được focus) và chữ gợi ý (`placeholder`) không bị biến mất khi vừa click chuột vào.

```rust
let txt = TextBox::create_with_placeholder(
    parent, 0,
    "Nhập từ viết tắt...",
    x, y, 200, 30, hfont_normal,
)?;

let value: String = txt.get_text();
txt.set_text("Xin chào");
txt.set_placeholder("Gợi ý mới...");
txt.focus();

// Bắt sự kiện khi nội dung văn bản thay đổi:
if self.txt.is_changed(ctrl_id, notify_code) {
    let query = self.txt.get_text();
}
```

---

### 3.6. `Slider` — Thanh trượt giá trị (`src/ui_framework/components/slider.rs`)

Thanh kéo chọn giá trị số nguyên trong khoảng `[min, max]`, tự động đồng bộ theme Sáng/Tối khi `set_visible(true)`.

```rust
let slider = Slider::create_with_value(
    parent, 0,
    x, y, 180, 26,
    0, 100, // min = 0, max = 100
    75,     // giá trị ban đầu = 75
)?;

// Bắt sự kiện trong hàm on_hscroll(wparam, lparam):
// parse_scroll tự động kiểm tra đúng HWND và tách trạng thái đang kéo chuột (is_dragging)
if let Some((pos, is_dragging)) = self.slider.parse_scroll(wparam, lparam) {
    self.lbl_val.set_text(&format!("{}%", pos));
    if !is_dragging {
        // Chỉ phát âm thanh thử hoặc lưu đĩa khi người dùng đã nhả chuột!
    }
}
```

---

### 3.7. `TabBar` — Thanh chuyển Sub-Tab dạng viên thuốc (`src/ui_framework/components/tab_bar.rs`)

```rust
let sub_tabs = TabBar::create(
    parent, 0,
    x, y, width, 30,
    hfont_normal, hfont_bold,
    &["Tổng quan", "Đổi phím", "Khóa phím", "Phím macro"],
)?;

if self.sub_tabs.is_changed(ctrl_id, notify_code) {
    let active_idx = self.sub_tabs.get_cur_sel();
}
```

---

### 3.8. `ActionTable` — Bảng dữ liệu tương tác (`src/ui_framework/components/action_table/`)

Bảng dữ liệu custom hỗ trợ cuộn mượt, vẽ phím tắt dạng huy hiệu (`KeyBadge`), nhãn trạng thái màu sắc (`BadgeConfig`) và các nút bấm hành động gắn trực tiếp hàm callback (`closure`) trên từng dòng.

```rust
let table = ActionTable::create_styled(
    parent, 0,
    x, y, width, height, hfont_normal,
    &[
        ("Từ viết tắt", 140, ColumnStyle::KeyBadgeWithArrow),
        ("Nội dung thay thế", 300, ColumnStyle::Default),
        ("Trạng thái", 100, ColumnStyle::Secondary),
        ("", 120, ColumnStyle::Action),
    ],
)?;
```

---

### 3.9. `ListBox` & `PopupMenu`

- **`ListBox`** (`src/ui_framework/components/listbox.rs`): Khởi tạo với `ListBox::create_owner_draw(parent, 0, x, y, w, h, hfont_normal)?` và kiểm tra sự kiện chọn dòng bằng `list.is_sel_changed(ctrl_id, notify_code)`.
- **`PopupMenu`** (`src/ui_framework/components/menu.rs`): Wrapper RAII cho `HMENU`, tự động gọi `DestroyMenu` khi ra khỏi scope.

---

## 4. Mẫu Chuẩn Tạo Một Tab Mới Trong Control Panel

Khi tạo một Tab mới trong `src/ui/views/control_panel/` (ví dụ `tab_tools.rs`), chỉ cần lưu `layout: CardLayout` và các control tương tác cần đọc/ghi sự kiện (các nhãn tĩnh không cần giữ trong struct vì `layout` đã tự động quản lý cả `HWND` lẫn đa ngôn ngữ `i18n`):

```rust
use crate::ui_framework::prelude::*;

pub struct TabTools {
    layout: CardLayout,
    pub chk_option: CheckBox,
    pub btn_action: PushButton,
}

impl TabTools {
    pub fn create(parent: isize, hfont_normal: isize, hfont_bold: isize) -> Option<Self> {
        let ly = CardLayout::with_parent(parent, hfont_normal, hfont_bold);

        let mut ly = ly.begin_card();
        // Nhãn tĩnh chỉ cần tạo qua section_title / label_row, không cần lưu vào struct!
        ly.section_title(|s| s.section_general, 8)?;
        let chk_option = ly.checkbox_row(|s| s.chk_spell_check, 22, 8, false, false)?;

        let btn_action = ly.button(|s| s.btn_save, true, ButtonVariant::Primary)?;
        ly.horizontal(30, 0, |row| {
            row.add_fixed(120, Some(btn_action.hwnd()));
        });
        let ly = ly.end_card();

        Some(Self {
            layout: ly,
            chk_option,
            btn_action,
        })
    }

    pub fn paint(&self, hdc: isize, right_bound: i32) {
        self.layout.paint_cards(hdc, layout::CARD_X, right_bound);
    }

    pub fn on_command(&self, _hwnd: isize, ctrl_id: u32, notify_code: u32) -> bool {
        if self.chk_option.is_toggled(ctrl_id, notify_code) {
            // Xử lý khi bật/tắt checkbox
            return true;
        }
        if self.btn_action.is_clicked(ctrl_id, notify_code) {
            // Xử lý khi bấm nút
            return true;
        }
        false
    }

    pub fn update_visibility(&self, visible: bool) {
        self.layout.set_visible(visible);
    }

    pub fn update_language(&self, strings: &crate::language::LanguageStrings) {
        self.layout.update_language(strings);
    }
}
```

---

## 5. Mẫu Chuẩn Khi Tạo Modal Dialog (`src/ui_framework/dialog.rs` & `src/ui/views/dialogs/base.rs`)

Phân hệ [`src/ui_framework/dialog.rs`](file:///c:/Users/kien.hm/Desktop/OpenKey/src/ui_framework/dialog.rs) cung cấp cơ chế gắn trực tiếp struct trạng thái `T` vào `GWLP_USERDATA` của từng cửa sổ Dialog (`open_modal::<T>` / `open_client_modal::<T>`), loại bỏ hoàn toàn nhu cầu dùng `static Mutex` hay khai báo hằng số `IDC_*`:

| Hàm tiện ích trong `dialogs::base` (`super::*`) | Công dụng |
| :--- | :--- |
| `open_client_modal::<T>(parent, title, cw, ch, class_name, wnd_proc, once, Box::new(state))` | Đăng ký class 1 lần (`Once`), tự tính kích thước khung viền để vùng Client Area rộng đúng `(cw, ch)`, truyền `Box<T>` vào `WM_CREATE` và khoá cửa sổ cha (`EnableWindow(parent, 0)`). |
| `attach_state::<T>(hwnd, lparam)` | Đặt ở đầu nhánh `WM_CREATE` để gắn `Box<T>` từ `CREATESTRUCTW::lp_create_params` vào `GWLP_USERDATA` của `hwnd`. |
| `state::<T>(hwnd) -> Option<&'static mut T>` | Lấy tham chiếu `&mut T` trực tiếp từ `GWLP_USERDATA` của Dialog trong `wnd_proc` ($O(1)$, không cần lock). |
| `create_dialog_footer_layout(hwnd, cancel_txt, cw, submit_txt, sw, variant, y, h, right_margin, bg, on_submit)` | Tạo cặp nút Hủy (`ButtonVariant::Secondary`, tự đóng hộp thoại khi bấm) và Lưu/Áp dụng ở đáy Dialog tự động căn phải bằng auto-layout `HStack`. |
| `show_delete_confirm(parent, msg, on_confirm)` / `show_danger_confirm(parent, title, msg, on_confirm)` | Hiển thị hộp thoại xác nhận nguy hiểm (Xóa / Khôi phục mặc định) trong 1 dòng lệnh, tự động điền nút Có/Không theo ngôn ngữ hiện tại. |
| `close_modal(hwnd, parent)` | Bật lại cửa sổ cha (`EnableWindow(parent, 1)`), trả focus về `parent` và đóng Dialog (`DestroyWindow(hwnd)`). |
| `detach_state::<T>(hwnd) -> Option<Box<T>>` | Đặt trong nhánh `WM_NCDESTROY` để giải phóng `Box<T>` an toàn khi cửa sổ đóng. |
| `reflect_dialog_messages(hwnd, msg, wparam, lparam)` | Đặt ở đầu `wnd_proc` để tự động kích hoạt `OCM_DRAWITEM`, `OCM_COMMAND`, `OCM_NOTIFY`, màu nền `Label`/`TextBox` và con trỏ bàn tay `IDC_HAND`. |
| `fill_dialog_bg` / `draw_dialog_callout` / `draw_dialog_footer` | Các hàm vẽ nền, hộp hướng dẫn bo góc và chân trang Dialog chuẩn theo `ThemePalette`. |

```rust
static REGISTERED: std::sync::Once = std::sync::Once::new();
const CLASS_NAME: &str = "KoraMyDialogClass";

struct MyDialogState {
    parent: isize,
    txt_name: Option<TextBox>,
}

pub fn show_my_dialog(parent: isize) {
    super::open_client_modal(
        parent,
        "Tiêu đề Hộp thoại",
        400,
        240,
        CLASS_NAME,
        my_dialog_wnd_proc,
        &REGISTERED,
        Box::new(MyDialogState {
            parent,
            txt_name: None,
        }),
    );
}

pub unsafe extern "system" fn my_dialog_wnd_proc(
    hwnd: isize,
    msg: u32,
    wparam: usize,
    lparam: isize,
) -> isize {
    unsafe {
        if let Some(res) = super::reflect_dialog_messages(hwnd, msg, wparam, lparam) {
            return res;
        }
        match msg {
            WM_CREATE => {
                super::attach_state::<MyDialogState>(hwnd, lparam);
                let Some(st) = super::state::<MyDialogState>(hwnd) else {
                    return -1;
                };
                let palette = ThemePalette::get(is_current_dark());
                let strings = crate::language::current();

                st.txt_name = TextBox::create_with_placeholder(
                    hwnd, 0, "Nhập tên...", 20, 90, 360, 30, get_app_font_normal(),
                );
                super::create_dialog_footer_layout(
                    hwnd,
                    strings.btn_cancel,
                    84,
                    strings.btn_save,
                    92,
                    ButtonVariant::Primary,
                    192,
                    32,
                    20,
                    Some(palette.bg_sidebar),
                    move || submit_my_dialog(hwnd),
                );
                0
            }
            WM_PAINT => {
                let mut ps: PAINTSTRUCT = std::mem::zeroed();
                let hdc = BeginPaint(hwnd, &mut ps);
                if hdc != 0 {
                    let palette = ThemePalette::get(is_current_dark());
                    super::fill_dialog_bg(hwnd, hdc, &palette);
                    super::draw_dialog_footer(hdc, &palette, 400, 176, 240, None);
                    EndPaint(hwnd, &ps);
                }
                0
            }
            WM_CLOSE => {
                let parent = super::state::<MyDialogState>(hwnd).map_or(0, |s| s.parent);
                super::close_modal(hwnd, parent);
                0
            }
            WM_NCDESTROY => {
                if let Some(st) = super::detach_state::<MyDialogState>(hwnd) {
                    EnableWindow(st.parent, 1);
                }
                DefWindowProcW(hwnd, msg, wparam, lparam)
            }
            _ => DefWindowProcW(hwnd, msg, wparam, lparam),
        }
    }
}
```


