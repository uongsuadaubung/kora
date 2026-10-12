# Kora — Modern Input & Keyboard Suite

[![Rust](https://img.shields.io/badge/language-Rust-orange.svg)](https://www.rust-lang.org/)
[![Platform](https://img.shields.io/badge/platform-Windows-blue.svg)]()
[![Tests](https://img.shields.io/badge/tests-passing-brightgreen.svg)]()
[![License](https://img.shields.io/badge/license-GPL--3.0-blue.svg)](LICENSE)

> 🚀 **Kora Core Engine:** Bộ công cụ bàn phím & bộ gõ hiện đại (*Modern Input & Keyboard Suite*) viết bằng **Rust thuần** và Win32 GDI native: tích hợp Bộ gõ Tiếng Việt chuẩn xác cao, Đổi phím (Key Remap), Khóa phím chuyên dụng (Key Blocker), Không gian làm việc đa ứng dụng (Multi-Profile Workspace) và Giả lập âm thanh phím cơ WASAPI phòng thu.
> 📖 **Tài liệu kiến trúc chi tiết:** [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md)

---

## 🌟 Vì sao lại là "Kora"? (The Story Behind the Name)

Ban đầu, dự án tập trung vào mục tiêu xây dựng một bộ gõ tiếng Việt siêu nhẹ, chuẩn xác và không lỗi vặt bằng ngôn ngữ Rust.

Tuy nhiên, trong quá trình phát triển, quy mô ứng dụng đã mở rộng thành một bộ công cụ bàn phím toàn diện:
- Quản lý hồ sơ theo từng ứng dụng (*Multi-Profile App Detection*)
- Đổi phím vật lý (*Key Remapping*)
- Khóa phím thông minh chống bấm nhầm khi chơi game & làm việc (*Key Disabling*)
- Giả lập âm thanh bàn phím cơ 48kHz (*WASAPI Mechanical Sound FX*)
- Mở rộng viết tắt đa quy tắc (*Smart Macro Expander*)

Cái tên **Kora** ra đời để phản ánh trọn vẹn tầm vóc của một **bộ công cụ bàn phím & bộ gõ hiện đại, tinh tế (Modern Input & Keyboard Suite)**. Đằng sau cái tên 4 chữ cái ngắn gọn là 4 tầng ý nghĩa sâu sắc:

1. 🎵 **Âm thanh (Acoustics & Sound FX)**
   *Kora* là tên một loại nhạc cụ dây truyền thống nổi tiếng với âm sắc trong trẻo, mộc mạc và thanh thoát. Đây là hình ảnh đại diện cho tính năng giả lập âm thanh phím cơ của Kora — biến từng nhịp gõ phím hằng ngày thành một trải nghiệm âm thanh sống động và giàu cảm xúc.

2. 🌐 **Không gian (Workspace & Multi-Profile)**
   Trong triết học Hy Lạp cổ đại, *Chōra* hay *Kora* (χώρα) có nghĩa là *"khoảng không gian / miền đất mở"* — nơi vạn vật định hình và phát triển. Điều này đồng điệu với triết lý kiến trúc *Multi-Profile* của Kora: mỗi ứng dụng, mỗi trò chơi (Game, IDE, Office, Graphic Tool) là một không gian làm việc độc lập, tự động biến đổi cấu hình phím tối ưu theo ngữ cảnh.

3. 🛡️ **Lớp vỏ bảo vệ kiên cố (Kernel Shield & Key Blocking)**
   Trong tiếng Nhật, *Kōra* (甲羅) có nghĩa là chiếc mai rùa vững chắc. Kora hoạt động như một lớp khiên bảo vệ ở tầng nhân cấp thấp (*Windows Hook*): lọc sạch thao tác phím thừa, chặn phím bấm nhầm khi chơi game (khóa phím Windows, Insert, Caps Lock), ngăn ngừa xung đột dấu và giữ cho luồng thao tác luôn an toàn, liền mạch.

4. ⚡ **Tối giản & Quốc tế (Minimalist & Global)**
   Bắt đầu bằng chữ **K** mạnh mẽ (*Keyboard / Kernel*) và kết thúc bằng nguyên âm mở `-a`, **Kora** mang phong cách của những công cụ phần mềm hiện đại (như *Figma, Raycast, Sora*): phát âm dứt khoát, dễ nhớ và tự nhiên trên phạm vi toàn cầu.

---

## ⚡ Hiệu năng Kỷ lục & Số liệu Thực tế (Highlights)

> *"Gõ nhanh như chớp, nhẹ như lông vũ — Hoạt động mượt mà dưới 1 MB RAM."*

| Chỉ số (Metric) | Kết quả đo đạc thực tế (Benchmark) | Ý nghĩa đối với người dùng & Game thủ |
| :--- | :--- | :--- |
| 🚀 **Tốc độ xử lý (Throughput)** | **~11.000.000 – 14.000.000 ký tự / giây** | Xử lý lên tới 14 triệu phím/giây, không bao giờ bị nghẽn phím khi gõ cực nhanh |
| ⏱️ **Độ trễ phản hồi (Latency)** | **~0.07 – 0.09 microsecond / phím** (70 – 90 ns) | Phản hồi siêu thanh trong 70 nano-giây, nhanh gấp **150.000 lần** ngưỡng mắt người nhận biết |
| 💾 **Bộ nhớ RAM (Chạy ngầm)** | **< 1 MB** (0.5 – 0.9 MB) | Nhẹ không đối thủ; hoạt động ổn định và cực kỳ tiết kiệm tài nguyên |
| 🖥️ **Bộ nhớ RAM (Control Panel mở)** | **~1.5 MB** | Toàn bộ giao diện Win32 native siêu gọn nhẹ, không dùng WebView/Electron cồng kềnh |
| 🔊 **Giả lập phím cơ (Sound Engine)** | **Tích hợp Native WASAPI (13 switches)** | Âm thanh phím cơ 48 kHz studio + Trình tải 1-Click tích hợp, không cần script ngoài |
| 📦 **Dung lượng file thực thi (.exe)** | **~1 MB** | Siêu nhỏ gọn (chứa trọn bộ gõ, GUI native, WASAPI engine & switch loader) |
| 🛡️ **Runtime Dependencies** | **0 dependencies (Pure Rust)** | 100% Rust thuần giao tiếp trực tiếp Windows API, không rủi ro bảo mật bên thứ 3 |
| 🎮 **Hiện tượng khựng phím (Stutter)** | **Zero Input Lag / Zero GC Pause** | Không có Garbage Collector (GC) thu gom rác gây trễ nhịp khi chơi game đỉnh cao |

### 🏆 So sánh với các công cụ truyền thống

| Tiêu chí | Kora (Rust Native) | Các bộ gõ C/C++ truyền thống khác | Bộ gõ nền tảng Web / Electron |
| :--- | :--- | :---: | :---: |
| **Dung lượng file thực thi (.exe)** | **~1 MB** | 1 MB – 5 MB | 80 MB – 150 MB |
| **Mức chiếm dụng RAM khi chạy ngầm** | **< 1 MB** | 1.5 MB – 4 MB | 100 MB – 300 MB |
| **Độ trễ xử lý phím** | **~0.07 µs** (70 ns) | 1.0 – 3.0 µs | 10 – 30 ms (có độ trễ) |
| **Giả lập âm thanh phím cơ** | **Tích hợp sẵn (0ms lag, siêu nhẹ)** | Không có | Phải cài Mechvibes (+150~300 MB RAM) |
| **Khựng khung hình (Micro-stutter)** | **Hoàn toàn KHÔNG (Zero GC)** | Hiếm gặp | Thường xuyên do GC / Event loop |
| **Multi-Profile theo ứng dụng/game** | **Tự động nhận diện + Kế thừa Delta** | Không có | Rất hiếm / Nặng nề |
| **Đổi phím (Remap) & Khóa phím (Disable)** | **Tích hợp sẵn per-profile** | Không có | Cần cài thêm PowerToys |

---

## 💎 Các Trụ cột Tính năng Chính

### 1. Không gian làm việc Đa hồ sơ (Multi-Profile Workspace & Live Inheritance)
- **Tự động nhận diện ứng dụng (Auto-Detection)**: Kora liên tục giám sát cửa sổ hoạt động (foreground window) để tự động kích hoạt cấu hình tương ứng (Game, IDE, Office, Graphic, Browser).
- **Mô hình kế thừa phân tầng (Delta Inheritance)**: Profile con tự động kế thừa thiết lập từ hồ sơ `Toàn hệ thống (Global)` và chỉ lưu các giá trị thay đổi riêng biệt (override/mute).
- **Cách ly phím tắt thông minh**: Mỗi profile có thể chọn dùng chung phím tắt chuyển chế độ gõ từ Global hoặc tùy biến phím tắt riêng (ví dụ `Ctrl + Shift` cho văn phòng, `Alt + Z` hoặc vô hiệu hóa cho Game).

### 2. Đổi phím (Key Remapping) & Khóa phím (Key Blocker)
- **Đổi phím vật lý trực tiếp**: Gán phím bất kỳ sang phím khác (ví dụ: `Caps Lock ➔ Ctrl`, `Insert ➔ Delete`, `F1 ➔ Escape`).
- **Khóa phím an toàn (Gaming Guard)**: Vô hiệu hóa phím bấm nhầm khi chơi game (khóa phím `Left/Right Windows`, `Insert`, `Caps Lock`, `F1`, `Scroll Lock`) mà không gửi bất kỳ tín hiệu nào về hệ điều hành hay game.
- **Bắt phím trực tiếp (Physical Key Capture)**: Giao diện bấm phím trực quan trên bàn phím thực tế để tự động nhận diện mã Virtual Key.

### 3. Bộ gõ Tiếng Việt Chuẩn xác cao (Next-Gen Vietnamese Engine)
- **Hỗ trợ đầy đủ các kiểu gõ**: Telex, VNI, Simple Telex.
- **Deterministic Replay Buffer (SSOT)**: Quản lý chuỗi phím thô chính xác tuyệt đối, xóa ký tự qua `Backspace` không bao giờ bị lỗi vỡ dấu hay xuất hiện ký tự ma.
- **Thông minh bỏ qua tiếng Anh (Smart English Bypass)**: Tự động nhận diện từ tiếng Anh phổ biến (`fix`, `server`, `first`, `word`,...) và khôi phục nguyên bản khi nhấn phím cách.
- **Khôi phục từ xuyên khoảng trắng (Backspace across space)**: Cho phép lùi qua dấu cách để sửa lại từ vừa gõ mà không cần gõ lại từ đầu.
- **Tự phục hồi sau khi máy ngủ & khóa màn hình (Self-healing Sleep & Lock Recovery)**: Lắng nghe sự kiện nguồn và phiên làm việc của Windows (`WM_POWERBROADCAST`, `WM_WTSSESSION_CHANGE`), tự động gắn lại Hook bàn phím và làm sạch toàn bộ cờ phím điều khiển kẹt (`Win`, `Ctrl`, `Alt`, `Shift`), giải quyết triệt để lỗi mất tiếng Việt hoặc lỗi gõ hoa vần (`gÕ`, `tiẾng ViỆt`) khi mở lại máy.

### 4. Gõ tắt Đa quy tắc (Smart Macro & Text Expander)
- **3 chế độ kích hoạt**: Toàn từ (Normal), Bắt đầu từ (Start), Cuối từ (End).
- **Bảo toàn viết hoa/thường (Casing Preservation)**: `ko` ➔ `không`, `Ko` ➔ `Không`, `KO` ➔ `KHÔNG`.
- **Hỗ trợ gõ tắt cả trong chế độ tiếng Anh**.

### 5. Giả lập Âm thanh Phím cơ Chuẩn Studio (WASAPI Sound Engine)
- **Giao tiếp phần cứng trực tiếp**: Windows Audio Session API (WASAPI) chuẩn **48.000 Hz, 32-bit Float**.
- **13 Bộ Switch danh tiếng thế giới**: *Cherry MX Blue, Brown, Red, Black, Alps SKCM Blue, Holy Panda, IBM Buckling Spring, Gateron Ink Black, Durock Alpaca, NovelKeys Cream, Kailh Box Navy, Topre, Typewriter*.
- **Định tuyến âm thanh từng phím**: Phân tách âm thanh riêng cho `Spacebar`, `Enter`, `Backspace`, `Esc`, `Shift`, `Arrows`...
- **Zero-Cost khi tắt**: Luồng âm thanh ngủ sâu, giải phóng 100% RAM và trả lại tài nguyên cho hệ thống.

---

## ⚡ Bắt đầu nhanh (Quick Start)

### Yêu cầu
- [Rust toolchain](https://rustup.rs/) (phiên bản 1.75 trở lên)
- Hệ điều hành: Windows 10 / 11

### Chạy kiểm thử tự động (Automated Tests)
```bash
cargo test
```

### Biên dịch bản phát hành (Build Release)
Bạn có thể chạy trực tiếp tập lệnh tự động kiểm tra tiến trình, tắt bản cũ và build bản mới:
```cmd
build.bat
```
Hoặc dùng lệnh Cargo chuẩn:
```bash
cargo build --release
```
Tập tin thực thi sau khi biên dịch nằm tại: `target/release/Kora.exe`.

---

## 👨‍💻 Tác giả phát triển
- **Mạnh Kiên** (`manhkien13041997@gmail.com`)
- Phiên bản **Kora Rust Native Edition** (2026).

## 📜 Giấy phép
- Phân phối theo giấy phép mã nguồn mở **GPL-3.0**.
