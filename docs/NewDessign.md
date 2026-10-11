# TÀI LIỆU THIẾT KẾ KIẾN TRÚC HỆ THỐNG KORA KEYBOARD HUB
## Tính Năng: Multi-Profile Workspace, Key Remapping & Key Disabling

---

## 1. TỔNG QUAN & TRIẾT LÝ THIẾT KẾ (DESIGN PHILOSOPHY)

### 1.1. Bối cảnh
Kora là bộ gõ tiếng Việt siêu nhẹ, hiệu năng cao viết bằng Pure Rust và Native Win32 API. Nhằm nâng tầm Kora từ một "bộ gõ tiếng Việt" thông thường trở thành một **Keyboard Workspace Hub** toàn diện (sánh ngang Razer Synapse, Logitech G HUB hay PowerToys nhưng giữ vững bản sắc siêu nhẹ 3-5MB RAM, zero-dependency), hệ thống **Multi-Profile** được thiết kế như một kiến trúc trung tâm (First-class Citizen).

### 1.2. Triết lý "Profile-First Hub"
* **Mọi cấu hình thuộc về một Profile:** Toàn bộ trạng thái gõ (Tiếng Việt/Tiếng Anh), kiểu gõ (Telex/VNI), phím chuyển $E \leftrightarrow V$, âm thanh gõ phím cơ, gõ tắt macro, bảng đổi phím (Remap) và bảng khóa phím (Disable) đều nằm trọn vẹn trong một Profile.
* **Tự động nhận diện ứng dụng (Active App Tracking):** Khi người dùng chuyển cửa sổ (Alt+Tab hoặc click chuột), Kora tự động kích hoạt Profile tương ứng của tiến trình đang active với độ trễ micro-giây.

---

## 2. MÔ HÌNH KẾ THỪA VÀ PHÂN LOẠI THIẾT LẬP (INHERITANCE & CONFIGURATION MODEL)

Nhằm tối ưu trải nghiệm người dùng, loại bỏ sự mơ hồ và nâng tầm trực quan, toàn bộ thiết lập trong Kora được chia thành **3 nhóm hành vi dứt khoát**:

```
[🌐 Profile Toàn Hệ Thống (Root / Global)]
  ├── Nguồn chân lý duy nhất (Single Source of Truth)
  └── Luôn chạy nền mặc định cho mọi ứng dụng không có profile riêng
         │
         ├── (1) Nhóm Select: Tự động kế thừa thời gian thực (hoặc chọn riêng)
         ├── (2) Nhóm Checkbox 3 trạng thái: Kế thừa (None) hoặc Ghi đè riêng (Some)
         └── (3) Nhóm Bảng dữ liệu: Kế thừa động Delta (Live Delta Overlay)
         │
         ▼
[🎯 Profile Ứng Dụng Con (CS2, VS Code, Word...)]
```

---

### 2.1. Phân loại 3 Nhóm Thiết Lập

#### 📌 Nhóm 1: Thiết lập dạng Lựa chọn (Select / ComboBox / Hotkey Picker)
* **Cơ chế:** **Kế thừa động có thể tùy biến (Dynamic Inherit or Specific Override)**.
* **Bao gồm:**
  - **Chế độ gõ (`LanguageMode`):** `Inherit` (Kế thừa từ Root) | `Vietnamese` | `English`.
  - **Kiểu gõ (`MethodMode`):** `Inherit` (Kế thừa từ Root) | `Telex` | `VNI` | `SimpleTelex`.
  - **Phím chuyển đổi E ⇄ V (`switch_key`):** `None` (Kế thừa phím của Root) | `Some(Hotkey)` (Phím riêng).
* **Quy tắc vận hành:**
  - Mặc định khi tạo profile con: Các mục này ở trạng thái `Inherit` / `None`. Khi Root thay đổi (ví dụ đổi kiểu gõ từ Telex sang VNI), profile con **tự động cập nhật theo ngay lập tức**.
  - Khi người dùng chủ động chọn một giá trị cụ thể (ví dụ chọn cứng `English` cho Game): Liên kết kế thừa của mục đó ngắt ra, app con sử dụng giá trị riêng mà không bị Root tác động.

#### 📌 Nhóm 2: Thiết lập Checkbox 3 Trạng thái kèm Nhãn Ngữ cảnh (Tri-State Contextual Checkbox)
* **Cơ chế:** **Kế thừa 3 trạng thái có chú thích (Tri-State with Dynamic Contextual Label)**.
* **Đột phá trải nghiệm người dùng (UX Innovation):**
  - Khắc phục triệt để nhược điểm "ô xám khó hiểu" của checkbox 3 trạng thái truyền thống bằng cách **luôn mở ngoặc chú thích rõ ràng nguồn gốc và giá trị đang kế thừa**.
* **Bao gồm:**
  - `use_macro`: Bật/tắt tính năng gõ tắt.
  - `use_macro_in_english_mode`: Cho phép gõ tắt cả khi ở chế độ tiếng Anh.
  - `restore_on_wrong_spelling`: Tự động khôi phục từ sai chính tả.
  - `auto_uppercase_first_char`: Tự động viết hoa chữ cái đầu.
  - `bracket_w`: Gõ `[` thành `ư`.
  - `remember_history_across_space`: Khôi phục dấu từ cũ qua phím cách.
  - `sound_enabled`: Bật/tắt âm thanh phím cơ.
* **Bảng trạng thái trực quan:**

| Kiểu dữ liệu trong Rust | Biểu tượng Glyphs | Nhãn hiển thị trên UI | Hành vi hệ thống |
| :--- | :---: | :--- | :--- |
| **`None`** *(Kế thừa Global)* | `[-]` hoặc `[■]` | `Tên tùy chọn (Kế thừa Global: Bật / Tắt)` | Ăn theo Global theo thời gian thực. Global đổi là app đổi theo. |
| **`Some(true)`** *(Bật riêng)* | `[✓]` *(Checked)* | `Tên tùy chọn (Bật riêng)` | Ép bật cho riêng app này, bất chấp Global. |
| **`Some(false)`** *(Tắt riêng)* | `[ ]` *(Unchecked)* | `Tên tùy chọn (Tắt riêng)` | Ép tắt cho riêng app này, bất chấp Global. |

* **Luồng tương tác người dùng (Cyclic Toggle):**
  - Mặc định khi tạo profile con: Tất cả checkbox ở trạng thái `None` (`Kế thừa Global`).
  - Mỗi lần click chuột vào checkbox sẽ xoay vòng tuần tự:
    $$\text{Kế thừa } (None) \xrightarrow{\text{click}} \text{Bật riêng } (Some(true)) \xrightarrow{\text{click}} \text{Tắt riêng } (Some(false)) \xrightarrow{\text{click}} \text{Kế thừa } (None)$$
  - **Lợi ích:** Người dùng có thể tùy biến riêng cho từng app, nhưng bất cứ lúc nào cũng có thể click quay về trạng thái `(Kế thừa Global)` mà không cần tạo lại profile.
  - Ánh xạ 1:1 chuẩn xác với kiểu `Option<bool>` trong struct `AppProfile` của Rust.

#### 📌 Nhóm 3: Bảng dữ liệu quy tắc (Macro Table, Key Remap, Key Disable)
* **Cơ chế:** **Kế thừa động theo mô hình Delta (Live Delta Overlay)**.
* **Bao gồm:**
  - Bảng từ viết tắt (Macro).
  - Bảng đổi phím (Key Remap).
  - Bảng khóa phím (Key Disable).
* **Công thức hợp nhất thời gian thực:**
  $$\text{ActiveRules}_{\text{App}} = (\text{GlobalRules} \setminus \text{MutedRules}_{\text{App}}) \cup \text{OverriddenRules}_{\text{App}} \cup \text{AppSpecificRules}$$
* **Quy tắc vận hành:**
  - Profile con **không copy dữ liệu thô** của Root mà luôn đọc trực tiếp từ Root qua bộ lọc Delta.
  - Mọi từ gõ tắt / phím remap mới thêm ở Root đều **ngay lập tức xuất hiện** trong tất cả profile con.
  - Profile con có quyền:
    - `[+]` Thêm từ/phím mới: Chỉ lưu vào `app_*`, hiển thị nhãn `✨ Riêng app này`.
    - `[~]` Sửa từ/phím của Root: Lưu vào `overridden_*`, hiển thị nhãn `✏️ Ghi đè Global`.
    - `[-]` Xóa từ/phím của Root: Lưu vào `muted_*` (Tombstone), hiển thị nhãn `🚫 Đã tắt`.

---

## 3. THIẾT KẾ CẤU TRÚC DỮ LIỆU (DATA MODEL TRONG RUST)

```rust
use std::collections::HashMap;

/// Chế độ ngôn ngữ
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LanguageMode {
    Inherit,
    Vietnamese,
    English,
}

/// Kiểu gõ dấu tiếng Việt
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MethodMode {
    Inherit,
    Specific(crate::engine::config::InputMethod),
}

/// Quy tắc đổi phím (Virtual Key Code hoặc tên phím)
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KeyRemapRule {
    pub from_vk: u32,
    pub to_vk: u32,
    pub from_name: String,
    pub to_name: String,
}

/// Cấu hình cho một Profile ứng dụng
#[derive(Debug, Clone)]
pub struct AppProfile {
    pub id: String,                         // Định danh duy nhất: "cs2", "vscode"
    pub name: String,                       // Tên hiển thị: "Counter-Strike 2"
    pub process_names: Vec<String>,          // Tên file thực thi: ["cs2.exe"]
    pub icon: String,                       // Biểu tượng: "🎮", "💻"
    pub is_root: bool,                      // true nếu là Global Root profile

    // 1. Bộ gõ & Ngôn ngữ (Kế thừa nếu là Inherit)
    pub mode: LanguageMode,
    pub method: MethodMode,
    pub hotkey: Option<String>,             // None = Kế thừa hotkey của Global
    pub smart_spell: Option<bool>,
    pub free_typing: Option<bool>,

    // 2. Bảng đổi phím (Key Remapping - Cách 2)
    pub muted_remaps: Vec<u32>,             // Danh sách from_vk của Global bị tắt riêng trong app
    pub overridden_remaps: HashMap<u32, u32>, // from_vk -> to_vk ghi đè riêng
    pub app_remaps: Vec<KeyRemapRule>,      // Các cặp đổi phím do app tự thêm mới

    // 3. Bảng khóa phím (Key Disabling - Cách 2)
    pub unmuted_disabled: Vec<u32>,         // Phím Global chặn nhưng app này mở khóa cho phép bấm
    pub app_disabled: Vec<u32>,             // Phím bị chặn riêng trong app này

    // 4. Âm thanh gõ phím cơ (Typing Sound)
    pub sound_enabled: Option<bool>,        // None = Kế thừa
    pub sound_profile: Option<String>,      // None = Kế thừa
    pub sound_volume: Option<u8>,           // None = Kế thừa

    // 5. Gõ tắt (Macro - Cách 2)
    pub use_macro: Option<bool>,
    pub macro_in_english: Option<bool>,
    pub muted_macros: Vec<String>,          // Từ viết tắt của Global bị tắt trong app
    pub overridden_macros: HashMap<String, (String, crate::engine::macro_table::MacroType)>,
    pub app_macros: Vec<crate::engine::macro_table::MacroEntry>,
}
```

---

## 4. CƠ CHẾ HOẠT ĐỘNG PHÍA HỆ THỐNG (SYSTEM ENGINE & HOOKS)

### 4.1. Nhận diện ứng dụng Foreground (Active Window Tracker)
* Sử dụng sự kiện Windows Event Hook cực nhẹ để không tốn CPU:
  ```rust
  SetWinEventHook(
      EVENT_SYSTEM_FOREGROUND, EVENT_SYSTEM_FOREGROUND,
      0, Some(win_event_proc), 0, 0, WINEVENT_OUTOFCONTEXT
  );
  ```
* Trong callback:
  1. Lấy `hwnd = GetForegroundWindow()`.
  2. Lấy `pid` qua `GetWindowThreadProcessId(hwnd, &mut pid)`.
  3. Lấy tên tiến trình bằng `QueryFullProcessImageNameW` (ví dụ `cs2.exe`).
  4. Tra cứu Profile trong `ProfileManager`: Nếu tìm thấy tiến trình khớp $\rightarrow$ kích hoạt Profile đó. Nếu không $\rightarrow$ rơi về `Global Profile`.

### 4.2. Xử lý Low-Level Keyboard Hook (`WH_KEYBOARD_LL`)
Mỗi khi có phím bấm đi qua Hook:
1. **Kiểm tra Disable:**
   * Nếu mã phím `vkCode` nằm trong danh sách chặn có hiệu lực của Profile hiện tại $\rightarrow$ Trả về `1` (nuốt phím, Windows và app không nhận được tín hiệu).
2. **Kiểm tra Remap:**
   * Nếu mã phím `vkCode` nằm trong bảng Remap $\rightarrow$ Nuốt phím gốc (`return 1`) và phát phím thay thế qua Windows `SendInput`:
     ```rust
     send_simulated_key(target_vk, is_key_up);
     ```
3. **Bộ gõ & Macro tiếng Việt:**
   * Áp dụng trạng thái Engine tương ứng của Profile active (tiếng Việt/Anh, Telex/VNI, gõ tắt macro).

---

## 5. ĐỊNH DẠNG LƯU TRỮ CẤU HÌNH (ZERO-DEPENDENCY INI STORE)

Mở rộng trực tiếp file `config.ini` hiện tại của Kora mà không cần thư viện ngoài:

```ini
[general]
method = telex
enabled = true
switch_key = Ctrl + Shift

[sound]
enabled = true
switch = Mechanical Cherry Blue
volume = 80

[macro]
vn = Việt Nam
dc = được
email = manhkien13041997@gmail.com

[remap]
Caps Lock = Ctrl
Scroll Lock = Mute

[disable]
keys = Insert

# --- HỒ SƠ CON ---
[profile:cs2]
name = Counter-Strike 2
process = cs2.exe
icon = 🎮
mode = english
sound_enabled = false
disable_keys = Left Windows, Right Windows, Caps Lock
muted_macros = vn, dc
app_macros = gg = Good Game! Well played.

[profile:vscode]
name = Visual Studio Code
process = Code.exe
icon = 💻
override_remaps = Caps Lock = Escape
app_remaps = F1 = F5
allow_disabled = Insert
override_macros = email = kien.manh@rust-coder.io
app_macros = fn = pub fn new() -> Self {}
```

---

## 6. THIẾT KẾ GIAO DIỆN NATIVE WIN32 (PURE WIN32 UI)

### 6.1. Bố cục cửa sổ chính (Main Window: 820 × 590 px)
```
┌───────────────────────────┬─────────────────────────────────────────────────────────┐
│ CỘT TRÁI: SIDEBAR (230px) │ CỘT PHẢI: KHÔNG GIAN CẤU HÌNH PROFILE (590px)           │
├───────────────────────────┼─────────────────────────────────────────────────────────┤
│ [HỒ SƠ (PROFILES)]  [➕]  │ Banner: [Icon + Tên Profile]              [Xóa Profile] │
│ 🌐 Toàn hệ thống   [Root] │ ─────────────────────────────────────────────────────── │
│ 🎮 Counter-Strike 2       │ TabBar con: [⌨️ Bộ gõ] [🔄 Đổi/Khóa phím] [🔊 Âm thanh] │
│ 💻 Visual Studio Code     │                                              [⚡ Gõ tắt] │
│ 📝 Microsoft Word         │ ─────────────────────────────────────────────────────── │
│                           │ Thân Tab:                                               │
│                           │   - Các thẻ Card vẽ bo góc (RoundRect)                  │
│                           │   - Bảng ListView Report với NM_CUSTOMDRAW              │
├───────────────────────────┼─────────────────────────────────────────────────────────┤
│ ⚙️ Cài đặt chung hệ thống │ Footer: Trạng thái kích hoạt tự động             [Áp dụng]│
│ ℹ️ Thông tin & Tác giả    │                                                         │
└───────────────────────────┴─────────────────────────────────────────────────────────┘
```

### 6.2. Hệ thống 4 Modal Dialogs (Win32 Dialogs)
Tất cả modal đều hoạt động theo cơ chế `EnableWindow(parent_hwnd, FALSE)` để đảm bảo tính an toàn:
1. **Modal `➕ Thêm Profile`:** Nhập tên, chọn icon, và ListView danh sách ứng dụng đang chạy (quét qua `EnumWindows` + `GetWindowTextW`).
2. **Modal `🔄 + Đổi phím`:** Bắt phím vật lý trực tiếp qua `WM_KEYDOWN` để điền ô `Phím gốc ➜ Phím đích`.
3. **Modal `🚫 + Chặn phím`:** Bắt phím cần khóa hoặc chọn nhanh phím tắt nguy hiểm.
4. **Modal `⌨️ Đổi phím E ⇄ V`:** Bắt tổ hợp phím bổ trợ (`Ctrl`, `Alt`, `Shift`, `Win`) + ký tự.

---

## 7. KẾ HOẠCH TRIỂN KHAI THEO GIAI ĐOẠN (IMPLEMENTATION ROADMAP)

* **Phase 1: Backend Core & Data Architecture (`engine/` & `platform/`)**
  * Tạo module `engine/profile/` (`model.rs`, `manager.rs`).
  * Nâng cấp `config_store/codec.rs` để đọc/ghi các section `[profile:...]`, `[remap]`, `[disable]`.
  * Viết module `platform/win32/app_tracker.rs` lắng nghe sự kiện đổi ứng dụng.
  * Cập nhật `platform/win32/hook.rs` để xử lý intercept Remap & Disable phím.
* **Phase 2: Components & Window Extension (`ui/components/`)**
  * Nâng cấp cửa sổ chính Control Panel lên kích thước $820 \times 590$ px.
  * Xây dựng `SidebarProfileTree` bên trái và liên kết sự kiện click đổi profile.
* **Phase 3: Tái cấu trúc các Tab con bên phải (`ui/views/control_panel/`)**
  * Cập nhật Tab 1: Bộ gõ & phím tắt.
  * Cập nhật Tab 2: Bảng đôi Remap & Disable (dùng ListView Win32).
  * Cập nhật Tab 3: Âm thanh gõ phím cơ.
  * Cập nhật Tab 4: Bảng Macro theo Cách 2.
* **Phase 4: Các Modal Dialogs & Tinh chỉnh Hoàn thiện**
  * Modal Thêm Profile, Modal Đổi phím, Modal Chặn phím, Modal Hotkey E/V.
  * Test tương thích ngược và kiểm tra hiệu năng.
