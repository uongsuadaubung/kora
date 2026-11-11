# Kora Engine — Hướng Dẫn Benchmark & Định Hướng Tối Ưu Hóa Hiệu Năng

Tài liệu này ghi chép chi tiết phương pháp chạy kiểm chuẩn hiệu năng (Benchmark), giải thích các cơ chế đo đạc, cách khắc phục lỗi tài nguyên đang chạy trên Windows, cùng triết lý và các định hướng tối ưu hóa chuyên sâu cho **Kora Vietnamese Input Engine**.

---

## 1. Hướng Dẫn Chạy Benchmark

### 1.1. Lệnh Benchmark Chuẩn Xác Nhất (`benchmark.bat`)

Để có kết quả kiểm chuẩn khách quan, chính xác và đồng nhất nhất, luôn sử dụng script batch chuẩn hóa của dự án:

```cmd
cmd /c benchmark.bat
```

Script `benchmark.bat` tự động thực hiện quy trình đo đạc tiêu chuẩn:
1. Đặt mã trang console sang UTF-8 (`chcp 65001`).
2. Biên dịch với cấu hình tối ưu hóa cao nhất (`cargo build --release --example benchmark` với `opt-level = 3`, `lto = true`, `codegen-units = 1`, `panic = "abort"`).
3. Thiết lập độ trễ nghỉ 1 giây để CPU ổn định luồng xử lý trước khi kích hoạt phép đo.
4. Chạy trực tiếp `target\release\examples\benchmark.exe` độc lập.

*(Trường hợp Kora đang được kích hoạt và chạy nền trên thanh Taskbar/System Tray của Windows khiến binary `target\release\Kora.exe` bị khóa bởi HĐH, có thể sử dụng cờ `--target-dir` để cách ly thư mục build: `cargo run --release --target-dir target/bench_run --example benchmark`).*

### 1.2. Phân Tích Chi Tiết Từng Tham Số

| Tham số / Lệnh | Ý nghĩa kỹ thuật |
| :--- | :--- |
| `cmd /c benchmark.bat` | Lệnh benchmark tiêu chuẩn chính thức của dự án, đảm bảo UTF-8 và làm sạch CPU trước khi đo. |
| `--release` | Kích hoạt profile biên dịch tối ưu hóa cao nhất (`opt-level = 3`, `lto = true`, `codegen-units = 1`, `panic = "abort"`). |
| `--target-dir target/bench_run` | Cách ly hoàn toàn thư mục build kiểm chuẩn, tránh xung đột quyền truy cập tệp (*Access is denied*) với tiến trình `Kora.exe` đang chạy nền. |
| `--example benchmark` | Chạy bộ đo lường đa kịch bản `examples/benchmark.rs` (Telex, Simple Telex, English Bypass, Full Screen Simulation, Macro Mute, Hook Rule Resolution). |

---

## 2. Giải Thích 2 Chỉ Số Đo Lường Cốt Lõi

Khi chạy benchmark, console sẽ in ra các chỉ số phản ánh các tầng kiến trúc khác nhau:

### 2.1. `Pure Engine on_key` (Tốc độ xử lý lõi phím thô)
- **Bản chất:** Đo đạc thời gian máy trạng thái ngữ âm (`SyllableState`) phân loại phím, chuyển dịch trạng thái, tra cứu ngữ âm học và xác định hành vi (`EngineAction`).
- **Ý nghĩa:** Phản ánh độ trễ trực tiếp trên luồng chặn bàn phím (`Low-Level Keyboard Hook`).
- **Thực tế:** Đạt **~30.91 – 32.33 triệu phím/giây** (**30.9 – 32.4 nano-giây / phím**), chính thức phá vỡ mốc **30 triệu phím/giây**! Nhanh gấp **190.000+ lần** ngưỡng mắt người có thể cảm nhận (~6 ms).

### 2.2. `Full Screen Simulation` (Mô phỏng gõ trên màn hình thực tế)
- **Bản chất:** Đo đạc toàn bộ chu trình hoàn chỉnh từ lúc nhấn phím, máy trạng thái tính toán, chạy thuật toán tiền tố chung (*Common Prefix Optimization*), phát sinh số lượng phím `Backspace` cần xóa lùi, cập nhật bộ đệm và nối chuỗi thay thế vào màn hình ảo.
- **Ý nghĩa:** Phản ánh trải nghiệm gõ thực tế của người dùng khi gõ các đoạn văn bản tiếng Việt phức tạp với đầy đủ dấu thanh, dấu mũ, dấu móc và từ tiếng Anh.
- **Thực tế:** Đạt **~24.84 – 27.83 triệu phím/giây** (**35.9 – 40.3 nano-giây / phím**, lần 3 chuẩn đạt **24.84M keys/s** và đỉnh đạt **27.83M keys/s** — trong đó phần lớn chi phí còn lại thuộc về chính vòng lặp mô phỏng màn hình `String` của test harness).

---

## 3. Lịch Sử Các Cột Mốc Đo Lường Thực Tế (Benchmark Milestones Tracker)

Bảng dưới đây ghi chép lại toàn bộ quá trình tiến hóa về mặt hiệu năng qua các phiên bản và các đợt tái cấu trúc:

| Cột mốc (Milestone) | Nội dung tối ưu hóa trọng tâm | Pure Engine `on_key` | Full Screen Simulation | Tỷ lệ cải thiện |
| :--- | :--- | :---: | :---: | :---: |
| **Giai đoạn 1** (`ba0b93a`) | Stack `InlineList<T, N>`, `SyllableState` Copy, ASCII fast-path `decompose_vowel`, `MaybeUninit` Win32 SendInput. | ~10.96M keys/s (91 ns) | ~7.80M keys/s (128 ns) | Mốc nền tảng ban đầu |
| **Giai đoạn 2** (`63eaf08`) | Đưa `raw_keys` sang `InlineList<RawKey, 32>`, `Arc<str>` cho config strings, dọn dẹp các phép clone chuỗi trên hot path. | ~13.70M keys/s (73 ns) | ~10.20M keys/s (98 ns) | **+25.0%** |
| **Giai đoạn 3** (`8974399`) | Cache bảng macro active (`ensure_active_macro_table`), mượn trực tiếp `&config` thay vì clone, fast-path 1-chu kỳ cho `is_boundary`. | ~21.01M keys/s (48 ns) | ~15.29M keys/s (65 ns) | **+53.3%** |
| **Giai đoạn 4** | Bảng tra cứu tĩnh `ASCII_VOWELS` ($O(1)$), Bitmask 128-bit cho `is_word_break`, Letter Fast-Path, `is_active_global` cờ nhị phân, Single-Pass Common Prefix Optimization. | **25.43M keys/s** (**39 ns**) | **18.39M keys/s** (**54 ns**) | **+21.0%** |
| **Giai đoạn 5** | Stack projection `render_chars_to` (0 heap allocation khi fallback Passthrough), Guard `find_tone_position()` khi không có thanh điệu, Early-exit Coda Guard trong `handle_nucleus`, Jump-Table `is_valid_onset_vowel_orthography`, Single-Pass length tracking. | ~20.66M – 25.43M keys/s (39–48 ns) | ~20.30M keys/s (49 ns) | **+10.4% Full Simulation** |
| **Giai đoạn 6** | **Bitmask Phonotactic Table cho `can_vowels_accept_coda`** (biến 16 nhánh match thành 1 lệnh bitwise AND duy nhất) + **Zero-Heap `CompactString` cho `EngineAction::Replace`** (100% stack-allocated không tốn `malloc` khi thay thế ký tự/bỏ dấu thanh). | **22.7M – 24.1M keys/s** (**41 ns**) | **20.6M – 22.7M keys/s** (**44 ns**) | **+11.6% Full Simulation** 🚀 |
| **Giai đoạn 7** (`db34a8c`) | **100% Bitmask Toàn Bộ Quy Tắc Ghép Vần Tiếng Việt**: `is_valid_onset`, `is_valid_onset_extension`, `is_valid_coda_start`, `is_valid_coda_pair`, `is_stop_coda`, `is_valid_onset_vowel_orthography`, `is_valid_coda_start_for_vowels`. Triệt tiêu hoàn toàn mọi chuỗi `match/if-else` trong toàn bộ module chính tả `spelling.rs`. | **20.9M – 24.6M keys/s** (**41–48 ns**) | **18.9M – 22.7M keys/s** (**44–50 ns**) | **100% O(1) Bitwise Zero-Branching** ⚡ |
| **Giai đoạn 8** (Hoàn tất) | **True 100% Zero-Heap trên cả nhánh Hoàn Tác & Khôi Phục Chính Tả + Thu Gọn L1 Cache**: Đổi `ModifierOutcome::Undone` và Smart English Bypass sang `CompactString` (đẩy English Bypass lên **92.8M keys/s = 10.8 ns**), kiểm tra biên UTF-8 1-chu kỳ trong `CompactString::drain`, gộp các hàm `render_*` xóa 84 dòng code thừa, thu gọn LUT phụ âm đầu từ 1.050B xuống **64B (1 L1 cache line)** không kiểm tra biên (`& 31`). | **21.6M – 24.5M keys/s** (**40.8–46.3 ns**) | **20.35M keys/s** (**49.1 ns**) | **English Bypass: 92.8M keys/s (10.8 ns)** 🔥 |
| **Giai đoạn 9** (Hoàn tất) | **Zero-Allocation Hook Rule Resolution (`3.8 ns/key`) + `VOWEL_COMPOSE_LUT` (`[u16; 360]`) + Bitmask Macro Pre-filter**: Triệt tiêu cấp phát `Vec` & clone `String` trên mọi nhát phím tại `low_level_keyboard_proc` (`is_key_disabled`, `lookup_key_remap`, `lookup_key_macro`), thay 72 nhánh `compose_vowel` bằng LUT 720B $O(1)$, thêm bitmask lọc nhanh phụ âm đầu/cuối trong `MacroTable::expand_word`. | **25.45M keys/s** (**39.3 ns**) | **22.50M keys/s** (**44.5 ns**) | **Hook Resolution: 261.6M keys/s (3.8 ns — nhanh gấp 39.2x)** ⚡🚀 |
| **Giai đoạn 10** (Hoàn tất) | **Lazy Evaluation cho Smart English Rollback**: Chuyển việc kiểm tra `is_unambiguous_completed_vietnamese` sang đánh giá lười bên trong `try_smart_english_rollback`, chỉ kích hoạt khi `next_state` thực sự là `Passthrough`. Triệt tiêu hoàn toàn chi phí kiểm tra chính tả thừa trên 99.9% phím gõ bình thường. | **25.06M – 26.24M keys/s** (**38.1–39.9 ns**) | **23.99M keys/s** (**41.7 ns**) | **+11.3% Full Simulation (24.0M keys/s, 41.7 ns)** 🚀⚡ |
| **Giai đoạn 11** | **Bitmask Telex/VNI Modifier Early Bailout & Hot-Path Memory Access Optimization**: `TELEX_MODIFIER_MASK` (0u32 bitmask) loại bỏ phụ âm không phải phím biến đổi trong 1 chu kỳ CPU; so sánh byte trực tiếp thay vì giải mã UTF-8 trong `is_rendered_identical_to_raw` và `try_smart_english_rollback`; trì hoãn nạp `bracket_w` và `auto_uppercase_first_char` trong `on_key`; trả về sớm khi buffer rỗng trong `handle_word_break`. Đo chuẩn qua `cmd /c benchmark.bat`. | **27.70M keys/s** (**36.1 ns**) | **24.07M keys/s** (**41.6 ns**) | **Hook: 300.4M keys/s (3.3 ns)** 🔥🚀 |
| **Giai đoạn 12** (Hoàn tất) | **Macro Length Bitmask Filter & Zero-Heap Expansion + Fast ASCII Common Prefix + Bracket-W Short-Circuit**: Thêm `normal_lengths_mask: u32` bỏ qua hashmap lookup cho từ thường có độ dài không trùng macro; chuyển `expand_word` và `apply_case_style` sang 100% stack `CompactString`; tối ưu hóa UTF-16 length check O(1) byte length khi ASCII trong `feed_key`; short-circuit kiểm tra phím `[` và `]` trong `State<Empty>`, `State<Onset>`, `State<Nucleus>` trong 1 chu kỳ clock. Đo chuẩn 3 lần lấy lần 3 qua `cmd /c benchmark.bat`. | **26.90M – 27.08M keys/s** (**36.9–37.2 ns**) | **25.09M keys/s** (**39.8 ns**) | **Macro: 25.5M keys/s (39.3 ns, +29.6%) \| Muted Macro: 30.3M keys/s (33.0 ns) \| Hook: 321.8M keys/s (3.1 ns)** 🚀⚡🔥 |
| **Giai đoạn 13** (Hoàn tất) | **Data Layout Diet (`SyllableState` 96B $\rightarrow$ 26B, `RawKey` 8B $\rightarrow$ 4B) + `VOWEL_DECOMPOSE_LUTS` + Raw-Byte Common Prefix + $O(1)$ Snapshot Word Restore + Pre-Lowercase Macro Filter + Zero-Pointer-Chase Mute Signature + Win32 Hook Lock/Syscall Diet**: Biến `Passthrough` thành ZST `()`, nén Onset/Coda sang `u8` ASCII, thay 66 nhánh `decompose_vowel` bằng LUT 662B $O(1)$, mã hóa UTF-8 trực tiếp từ `u16` BMP, so khớp tiền tố byte thô kèm đếm UTF-16 hậu tố không rẽ nhánh, khôi phục từ qua dấu cách $O(1)$ không replay máy trạng thái, lọc tiền-lowercase cho gõ tắt, băm chữ ký `(ptr, len)` cho muted macros, và nhả `Mutex` trước `SendInput`/`CallNextHookEx`. | **30.91M – 32.33M keys/s** (**30.9–32.4 ns**) | **24.84M – 27.83M keys/s** (**35.9–40.3 ns**) | **Telex: +20.2% (32.3M keys/s, 30.9 ns) \| English Bypass: 110.5M keys/s (9.0 ns) \| Muted Macro: 32.9M–34.1M keys/s \| Hook: 328.5M–330.9M keys/s (3.0 ns)** 🚀⚡🔥 |

### 3.1. Chi Tiết Đo Đạc Cột Mốc Hiện Tại (Benchmark Log Record - Lần 3 Chuẩn)

- **Thời gian đo:** Ngày 11 tháng 11 năm 2026.
- **Lệnh thực thi:** `cmd /c benchmark.bat` (chạy tuần tự 3 lần độc lập, lấy lần 3 làm chuẩn chính thức).
- **Môi trường:** Windows 11 x64, Rust toolchain 2024 edition.
- **Cấu hình Profile Release:** `opt-level = 3`, `lto = true`, `codegen-units = 1`, `panic = "abort"`.

```text
[1] PURE ENGINE ON_KEY LATENCY & THROUGHPUT (No screen allocation):
  Benchmark Case                             |       Throughput |    Latency per Key
  -------------------------------------------+------------------+-------------------
  Telex (on_key raw)                         |  32329031 keys/s |    30.9 ns/key (0.031 µs)
  Simple Telex (on_key raw)                  |  30911249 keys/s |    32.4 ns/key (0.032 µs)
  English Bypass (on_key raw)                | 110522854 keys/s |     9.0 ns/key (0.009 µs)

[2] FULL SCREEN SIMULATION (Handling Backspace & Replacement):
  Benchmark Case                             |       Throughput |    Latency per Key
  -------------------------------------------+------------------+-------------------
  End-to-End Vietnamese Typing               |  24841710 keys/s |    40.3 ns/key (0.040 µs)

[3] MACRO EXPANSION & ZERO-ALLOCATION ROW MUTE BENCHMARK:
  Benchmark Case                             |       Throughput |    Latency per Key
  -------------------------------------------+------------------+-------------------
  Macro (All Unmuted)                        |  26118185 keys/s |    38.3 ns/key (0.038 µs)
  Macro with Muted Rows (Cached Zero-Alloc)  |  32942281 keys/s |    30.4 ns/key (0.030 µs)

[4] KEYBOARD HOOK RULE RESOLUTION (Remap, Disable, Key Macro):
  Benchmark Case                             |       Throughput |    Latency per Key
  -------------------------------------------+------------------+-------------------
  Hook Rule Resolution (4 keys)              | 328461159 keys/s |     3.0 ns/key (0.003 µs)
```

---

## 4. Các Triết Lý & Định Hướng Tối Ưu Hóa (Optimization Philosophy)

Đạt được thông lượng hàng chục triệu phím/giây trong Rust đòi hỏi tuân thủ nghiêm ngặt 5 nguyên tắc cốt lõi:

```mermaid
flowchart LR
    A["1. Zero-Heap Allocation"] --> B["2. Fast-Path Early Bailout"]
    B --> C["3. Zero-Branching / Bitmask & LUT"]
    C --> D["4. Cache Locality & Single-Pass"]
    D --> E["5. Eliminate String Comparisons"]
```

---

### Nguyên Tắc 1: Zero-Heap Allocation & Compact Stack Layout trên Hot-Path
- **Vấn đề:** Mọi thao tác cấp phát bộ nhớ động (`malloc`, `String`, `Vec`) đều phải tương tác với bộ quản lý bộ nhớ của hệ điều hành, gây mất ổn định về độ trễ (*jitter*) và tiêu tốn hàng trăm chu kỳ CPU; đồng thời struct trạng thái quá lớn sẽ gây tràn thanh ghi và tốn băng thông `memcpy` mỗi nhát phím.
- **Giải pháp áp dụng:**
  - Chuyển toàn bộ các mảng lưu trữ trạng thái (`Onset`, `Nucleus`, `Coda`, `raw_keys`) sang mảng tĩnh kích thước cố định trên Stack: `InlineList<T, N>` (kèm `InlineList::from_slice` chỉ sao chép phần tử đang hoạt động `..len`).
  - Biến `SyllableState` và toàn bộ các thành phần con thành kiểu dữ liệu 100% `Copy` siêu gọn (**26 bytes** trên Stack — thu gọn từ 96 bytes nhờ biến `Passthrough` thành ZST `()`, nén `OnsetState` xuống 6B và `CodaState` xuống 25B với `u8` ASCII kèm `#[repr(u8)]` trên các enum ngữ âm), kết hợp đóng gói `RawKey(u32)` **4 bytes** (giảm một nửa `InlineList<RawKey, 32>` từ 260B xuống 132B).
  - Sử dụng `CompactString` (64 bytes inline stack buffer, tổng 65B) cho cả `EngineAction::Replace`, `ModifierOutcome::Undone`, khôi phục tiếng Anh lẫn hạ chữ thường (`lowercase_to_compact`) trong bảng gõ tắt, và `MaybeUninit` cho mảng `SendInput` ở tầng Win32 Hook.

---

### Nguyên Tắc 2: Fast-Path Bỏ Qua Sớm (Early Bailout)
- **Vấn đề:** Hơn 90% thao tác gõ thông thường là các chữ cái (`a-z`, `A-Z`). Nếu mỗi phím đều phải chạy qua tất cả các bộ lọc ngắt câu, khoảng trắng, gõ tắt và kiểm tra chính tả, CPU sẽ lãng phí chu kỳ vô ích.
- **Giải pháp áp dụng:**
  - **Letter Fast-Path:** Thêm điều kiện `!is_letter && (ch.is_whitespace() || is_word_break(ch, bracket_w))` trong `on_key`. Mọi chữ cái bỏ qua toàn bộ khối kiểm tra ngắt từ ngay trong **1 chu kỳ CPU**.
  - **$O(1)$ Length Check trước khi so khớp chuỗi:** Trong `handle_word_break`, nếu `raw_keys.len() != emitted_len` (áp dụng cho mọi từ có dấu như *tiếng*, *việt*, *được*, *rất*), kết luận ngay `is_same = false` mà không cần duyệt chuỗi qua UTF-8.

---

### Nguyên Tắc 3: Triệt Tiêu Nhánh Nhảy (Zero-Branching / Bitmask & LUT)
- **Vấn đề:** Các câu lệnh `match` dài và chuỗi `if-else` phức tạp khiến bộ dự đoán nhánh của CPU (*Branch Predictor*) có nguy cơ đoán sai (*branch misprediction penalty*, tốn từ 10–20 chu kỳ máy).
- **Giải pháp áp dụng:**
  - **Compile-Time Bitmask 128-bit cho Word-Break:** Chuyển 31 ký tự ngắt câu thành 2 hằng số `u128` (`WORD_BREAK_MASK`). Hàm `is_word_break` chỉ tốn đúng **1 phép dịch bit (`1u128 << cp`) và 1 phép `AND`**.
  - **Bảng tra cứu tĩnh `ASCII_VOWELS`, `INITIAL_ASCII_LUT`, `VOWEL_COMPOSE_LUT` & `VOWEL_DECOMPOSE_LUTS`:** Xác định nguyên âm cơ bản, phân loại phím đầu âm tiết, tổng hợp và phân rã 132 nguyên âm tiếng Việt (kèm hoa/thường) trong đúng 1 lệnh truy xuất chỉ mục mảng $O(1)$.

---

### Nguyên Tắc 4: Cờ Gia Tăng $O(1)$ & So Khớp Tiền Tố Byte Thô Một Lượt
- **Vấn đề:** Việc duyệt qua duyệt lại bộ đệm `raw_keys` hay giải mã UTF-8 `chars()` nhiều lần trên mỗi nhát phím làm suy giảm hiệu năng bộ nhớ đệm L1 và tiêu tốn chu kỳ giải mã đa byte.
- **Giải pháp áp dụng:**
  - **$O(1)$ Incremental `is_boundary`:** Duy trì hai cờ gia tăng `has_underscore` và `has_lower_alpha` ngay khi đẩy phím vào `TypingBuffer`, biến `is_boundary` (phát hiện ranh giới `camelCase`) thành phép kiểm tra boolean $O(1)$ `key.is_upper() && !self.has_underscore && self.has_lower_alpha` không vòng lặp.
  - **Raw-Byte Common Prefix & Suffix-Only UTF-16 Count:** So sánh trực tiếp hai lát cắt byte UTF-8 (`prev_bytes` vs `out_bytes`), lùi tối đa 1–3 byte nếu điểm khác biệt rơi vào byte tiếp nối UTF-8 (`(b & 0xC0) == 0x80`), rồi chỉ tính độ dài UTF-16 không rẽ nhánh (`(b & 0xC0) != 0x80` + surrogate offset) trên đúng phần hậu tố thay đổi.

---

### Nguyên Tắc 5: Triệt Tiêu So Sánh Chuỗi & Đuổi Con Trỏ Heap (Pointer-Chase Elimination)
- **Vấn đề:** So sánh chuỗi `active_id == "global"` hay duyệt so sánh từng phần tử `Vec<String>` trên hot-path đòi hỏi dereference con trỏ heap và gọi `memcmp` hàng chục triệu lần mỗi giây.
- **Giải pháp áp dụng:**
  - Lưu cờ nhị phân `is_active_global: bool` trên struct `VietnameseEngine` (bỏ qua luôn việc nạp `active_profile_id()` trên fast-path toàn cục) và kiểm tra chữ ký FNV-1a 64-bit trên header `(ptr, len)` của slice `muted_macros` kết hợp `MacroTable::mutation_gen()` thay vì đuổi theo dữ liệu chuỗi trên heap.

---

## 5. Định Hướng Nâng Cấp Tiềm Năng (Optimization Roadmap)

### 5.1. [ĐÃ HOÀN THÀNH - Giai đoạn 6] Zero-Heap `CompactString` cho `EngineAction::Replace`
- **Mục tiêu:** Đạt **100% Zero-Allocation** tuyệt đối trên toàn bộ hot-path gõ phím.
- **Giải pháp đã triển khai:**
  - Thiết kế `CompactString` với vùng đệm nội tuyến trên Stack (`INLINE_CAP = 64` bytes, kích thước struct 65 bytes, 0 heap allocations cho 100% các âm tiết tiếng Việt như *"nghiêng"*, *"chuyển"*, *"phượng"* cũng như các cụm từ gõ tắt ngắn).
  - Triệt tiêu hoàn toàn `malloc` khi bỏ dấu thanh hoặc dấu phụ trên máy trạng thái.
  - Tự động fallback sang `Heap(String)` an toàn nếu mở rộng các đoạn macro gõ tắt văn bản dài (> 64 bytes).

### 5.2. [ĐÃ HOÀN THÀNH - Giai đoạn 6] Bitmask Bảng Tra Cứu Ngữ Âm Học (`can_vowels_accept_coda`)
- **Mục tiêu:** Triệt tiêu 16 nhánh so khớp pattern (`match` arms) phức tạp khi xác định khả năng nhận phụ âm cuối.
- **Giải pháp đã triển khai:**
  - Đóng gói cặp nguyên âm 5-bit (`base << 2 | diacritic`) thành chỉ số index 10-bit.
  - Bảng tra cứu compile-time `CODA_ACCEPT_2VOWELS_MASK: [u64; 16]` kiểm tra tính chấp nhận phụ âm cuối chỉ trong đúng **1 phép tính bitwise AND duy nhất $O(1)$**:
    `((CODA_ACCEPT_2VOWELS_MASK[idx >> 6] & (1u64 << (idx & 63))) != 0)`.

### 5.3. [ĐÃ HOÀN THÀNH - Giai đoạn 6] Bitmask Bảng Tra Cứu Nguyên Âm Ghép (`is_valid_vowel_combination`)
- **Mục tiêu:** Triệt tiêu toàn bộ 60 nhánh so khớp mẫu tuple (`match (v0, v1)` và `match (v0, v1, v2)`) trong cả 2 hàm `is_valid_vowel_combination` và `is_valid_intermediate_vowel_combination`.
- **Giải pháp đã triển khai:**
  - `VALID_2VOWELS_MASK: [u64; 16]` thay thế 33 match arms cho nguyên âm đôi.
  - `VALID_INTERMEDIATE_2VOWELS_MASK: [u64; 16]` kế thừa trực tiếp từ `VALID_2VOWELS_MASK` và bổ sung 7 vần trung gian gõ phím trong 1 phép AND duy nhất.
  - Đóng gói 3 nguyên âm thành số nguyên 15-bit `u16` (`pack_3vowels`) so khớp với 15 hằng số tam nguyên âm chuẩn trong 1 phép thử số nguyên.

### 5.4. [ĐÃ HOÀN THÀNH - Giai đoạn 7] Ma Trận Bitmask Cho 100% Quy Tắc Ghép Vần Tiếng Việt
- **Mục tiêu:** Đưa toàn bộ các quy tắc ghép âm tiết (phụ âm đầu, phụ âm cuối, âm tắc, tương thích chính tả nguyên âm - phụ âm) về kiểm tra bitmask số nguyên $O(1)$ không rẽ nhánh.
- **Giải pháp đã triển khai trong `src/vietnamese/spelling.rs`:**
  1. `SINGLE_ONSET_MASK: u32` & `ONSET_2LETTERS_MASK: [u64; 16]`: Tra cứu hợp lệ phụ âm đầu đơn và kép trong 1 phép dịch bit.
  2. `is_valid_onset_extension`: Kiểm tra khả năng mở rộng phụ âm đầu bằng bitmask 2-ký tự.
  3. `STOP_CODA_CHARS_MASK: u32`: Kiểm tra phụ âm tắc (`c, p, t, k`) bằng phép thử bit đơn.
  4. `CODA_START_CHARS_MASK: u32` & `CODA_2LETTERS_MASK: [u64; 16]`: Kiểm tra phụ âm cuối đơn (`c, m, n, p, t`) và phụ âm cuối đôi (`ng, nh, ch`) bằng bảng tra cứu 128 bytes.
  5. `SINGLE_ONSET_ALLOWED_VOWELS: [u8; 32]` & `ONSET_H_ALLOWED_VOWELS: [u8; 32]`: Ma trận nhị phân 6-bit ánh xạ nguyên âm cơ bản (A, E, I, O, U, Y) hợp lệ cho từng phụ âm đầu (`q` chỉ đi với `u`, `k` chỉ đi với `i, e, y`, `c` không đi với `i, e, y`, `gh/ngh` chỉ đi với `i, e`,...) nằm gọn trong đúng 1 L1 cache line (64 bytes) với chỉ mục `& 31` triệt tiêu hoàn toàn bounds check.
  6. `CODA_START_2VOWELS_MASK: [u32; 36]`: Ma trận bitmask 144 bytes ánh xạ 36 cặp nguyên âm cơ bản với phụ âm cuối cho phép (`oo` chỉ nhận `n`, `oe` nhận `t, n`, `uy` nhận `n, t, p, c`, `uê` nhận `n, c`).

### 5.5. [ĐÃ HOÀN THÀNH - Giai đoạn 8] True 100% Zero-Heap Hoàn Tác/Khôi Phục, UTF-8 Soundness & L1 Cache Diet
- **Mục tiêu:** Xóa bỏ nốt các điểm cấp phát `String` cuối cùng trên đường hoàn tác dấu và khôi phục từ tiếng Anh, đảm bảo an toàn bộ nhớ UTF-8 tuyệt đối và tối giản hóa kích thước bảng tra cứu trong L1 Data Cache.
- **Giải pháp đã triển khai:**
  1. **Zero-Heap `ModifierOutcome::Undone(CompactString)` & Smart English Bypass:** Thay thế toàn bộ `.collect::<String>()` khi hoàn tác dấu (`aa -> a`, `ss -> s`, `ww -> w`) và khi tự động khôi phục từ tiếng Anh sai chính tả bằng `CompactString` (`FromIterator<char>`). Đẩy tốc độ `English Bypass` lên **94.22 triệu phím/giây (10.6 ns/phím)**.
  2. **Kiểm tra biên UTF-8 1-chu kỳ trong `CompactString::drain`:** Kiểm tra bit tiếp nối UTF-8 bằng phép so sánh số nguyên có dấu 8-bit `(buf[idx] as i8) >= -0x40` trước khi `copy_within`, đảm bảo an toàn `from_utf8_unchecked` mà không sinh panic landing pad.
  3. **Thu gọn LUT phụ âm đầu từ 1.050B xuống 64B (1 L1 Cache Line):** Thay bảng `DOUBLE_ONSET_ALLOWED_VOWELS: [u8; 1024]` bằng `ONSET_H_ALLOWED_VOWELS: [u8; 32]` kết hợp `SINGLE_ONSET_ALLOWED_VOWELS: [u8; 32]` đánh chỉ mục `& 31` không bounds-check.
  4. **Hợp nhất hệ thống hàm `render_*`:** Tách `render_compact_to` và `tone_position` (`#[inline(always)]`), loại bỏ 84 dòng code trùng lặp giữa `render_to`, `render_chars_to` và `render_compact`.

### 5.6. [ĐÃ HOÀN THÀNH - Giai đoạn 9] Zero-Allocation Hook Rule Resolution (39.2x Speedup), `VOWEL_COMPOSE_LUT` & Bitmask Macro Pre-filter
- **Mục tiêu:** Triệt tiêu hoàn toàn việc cấp phát `Vec` và clone `String` trên mỗi sự kiện phím ở tầng Win32 Keyboard Hook (`low_level_keyboard_proc`), đồng thời loại bỏ 72 nhánh `match` trong `compose_vowel`.
- **Giải pháp đã triển khai:**
  1. **Zero-Allocation Single-Key Hook Lookups (`is_key_disabled`, `lookup_key_remap`, `lookup_key_macro`):** Truy vấn trực tiếp trên slice của `ProfileManager` kết hợp cờ `is_active_global` mà không cấp phát `Vec<u32>` hay clone `KeyRemapRule` (`from_name`/`to_name`). Tăng tốc `Hook Rule Resolution` từ **6.66M keys/s (150.1 ns)** lên **261.59M keys/s (3.8 ns/phím — nhanh gấp 39.2 lần)**.
  2. **Bảng hằng số Compile-Time `VOWEL_COMPOSE_LUT: [u16; 360]` (720 bytes):** Ánh xạ trực tiếp bộ 4 tham số `(BaseVowel, Diacritic, Tone, uppercase)` sang mã UTF-16 chỉ trong đúng **1 lệnh đọc mảng $O(1)$** không nhánh nhảy, thay thế toàn bộ 72 nhánh `match` lúc chạy của `compose_vowel`.
  3. **Bitmask Pre-filter cho Gõ Tắt Phụ Âm Đầu/Cuối (`start_first_mask` & `end_last_mask`):** Lưu mặt nạ bit 32-bit cho ký tự đầu/cuối của các quy tắc `StartConsonant`/`EndConsonant` và kiểm tra độ dài 1 ký tự trong $O(1)$ (`word.chars().nth(1).is_none()`), giúp >95% từ thông thường bỏ qua vòng lặp quét `Vec` trong 1 chu kỳ CPU.

### 5.7. [ĐÃ HOÀN THÀNH - Giai đoạn 10] WASAPI Audio Worker: Zero-Heap Allocation & SIMD Polyphonic Mixer
- **Bối cảnh:** Khi bật tính năng âm thanh bàn phím cơ (Mechanical Soundpack), mỗi nhát phím kích hoạt luồng phát âm thanh nền Windows Audio Session API (WASAPI) với tần suất cập nhật buffer 5ms (200 lần/giây).
- **Giải pháp đã triển khai trong `src/platform/win32/sound/wasapi.rs`:**
  1. **Zero-Heap Scratch Buffer:** Cấp phát trước `mix_buf: Vec<f32>` một lần duy nhất theo `buffer_frame_count` khi mở `WasapiSession` và tái sử dụng qua `mix_slice.fill(0.0)`, triệt tiêu 100% lệnh `malloc`/`free` trên luồng âm thanh thời gian thực (giải phóng hoàn toàn trong `WasapiSession::close()` khi tắt âm thanh).
  2. **Auto-Vectorized FMA & Stereo Fast-Path:** Sử dụng `f32::mul_add` cho vòng lặp trộn đa âm và tách nhánh `chunks_exact_mut(2)` cố định cho đầu ra Stereo (2 kênh), giúp trình biên dịch LLVM tự động vector hóa toàn bộ khâu trộn và clamp biên độ bằng tập lệnh SIMD.

---

### 5.8. [ĐÃ HOÀN THÀNH - Giai đoạn 11] Bitmask Telex/VNI Modifier Early Bailout, Zero-Decode ASCII Byte Comparison & Hot-Path Memory Deferral
- **Bối cảnh & Mục tiêu:** Tối ưu hóa sâu trên hot-path xử lý phím của Engine (`src/vietnamese/modifier.rs`, `src/engine/buffer.rs`, `src/engine/mod.rs`), đưa thông lượng phím thuần lên trên 27 triệu phím/s và mô phỏng gõ màn hình lên trên 24 triệu phím/s.
- **Giải pháp đã triển khai:**
  1. **Bitmask 1-chu kỳ cho Bộ lọc Phím Dấu Telex/VNI (`TELEX_MODIFIER_MASK: u32`):**
     - Trong Telex, chỉ có 12 phím là phím dấu/mũ/móc (`s, f, r, x, j, z, w, a, e, o, d`). Hơn 14 phụ âm còn lại (`b, c, g, h, i, k, l, m, n, p, q, t, u, v, y`) **không bao giờ** là phím biến đổi.
     - Khi gõ từ thông thường (ví dụ: `tiếng`, `chúng`), mỗi khi gõ `n, g, c...` trong Nucleus hoặc Coda, engine cũ phải duyệt qua 11 nhánh `match` và closure `has_vowel` trước khi trả về `None`.
     - Giải pháp mới: Tạo bitmask hằng số compile-time `TELEX_MODIFIER_MASK = 0b0011_0100_0100_0100_0111_0011_0001` (chỉ chứa các bit tương ứng 12 phím). Chỉ với 1 phép tính nhị phân:
       ```rust
       let cp = ch as u32 | 0x20;
       let delta = cp.wrapping_sub('a' as u32);
       if delta >= 26 || (TELEX_MODIFIER_MASK & (1u32 << delta)) == 0 {
           return None;
       }
       ```
       Và với VNI: kiểm tra phạm vi số `ch as u32 - '0' < 10` trong 1 chu kỳ máy.
       Toàn bộ phụ âm không phải phím dấu bị loại bỏ ngay lập tức trong đúng **1 chu kỳ CPU**, triệt tiêu chi phí rẽ nhánh trên phần lớn các phím gõ.
  2. **So sánh Trực tiếp Mảng Byte Thô (Zero-Decode ASCII Byte Equality):**
     - Trong `is_rendered_identical_to_raw` và `try_smart_english_rollback`, code cũ duyệt `chars().eq(raw_keys.iter().map(|k| k.ch))`, phải giải mã UTF-8 từng ký tự và phân bổ iterator.
     - Về mặt ngữ âm học và cấu trúc Unicode, khi `raw_keys.len() == emitted_len`, chuỗi hiển thị đảm bảo 100% là ký tự ASCII thuần (vì bất kỳ ký tự tiếng Việt có dấu nào trong BMP đều có UTF-8 length $\ge 2$ trong khi UTF-16 length $= 1$).
     - Giải pháp mới: Thay thế toàn bộ bằng so sánh trực tiếp mảng byte thô:
       ```rust
       let rendered_bytes = self.last_rendered.as_bytes();
       if rendered_bytes.len() != self.raw_keys.len() { return false; }
       rendered_bytes.iter().zip(self.raw_keys.iter()).all(|(&b, k)| b as char == k.ch)
       ```
       Không tốn một chu kỳ giải mã UTF-8 nào, thực thi trực tiếp trên thanh ghi CPU scalar unrolled.
  3. **Trì Hoãn Nạp Cấu Hình trên Hot-Path (`on_key` Memory Deferral):**
     - Cờ cấu hình `bracket_w` và `auto_uppercase_first_char` trước đây bị nạp từ struct `EngineConfig` ở đầu mỗi nhát phím, kể cả khi 95%+ phím gõ là chữ cái thông thường (`'a'..'z'`).
     - Giải pháp mới: Chuyển việc đọc `bracket_w` vào bên trong nhánh `if !is_letter`. Đồng thời kiểm tra cờ nhị phân `self.auto_uppercase_next` trước (vốn bằng `false` trên 99.9% phím) rồi mới đọc `auto_uppercase_first_char`. Tiết kiệm hàng triệu lần nạp bộ nhớ đệm L1 vô ích mỗi giây.
  4. **Thoát Sớm Khi Bộ Đệm Rỗng Trong `handle_word_break` (Early Empty Buffer Bailout):**
     - Khi người dùng gõ nhiều dấu cách liên tiếp hoặc dấu câu khi chưa có từ nào trong bộ đệm, hàm cũ vẫn nạp cấu hình `use_macro` và `restore_on_wrong_spelling`.
     - Giải pháp mới: Đưa kiểm tra `if self.buffer.is_empty()` lên ngay đầu hàm, xử lý cờ ngắt câu và trả về `EngineAction::Passthrough` trong đúng 1 lệnh so sánh độ dài số nguyên.
- **Kết quả đo đạc chính thức qua `cmd /c benchmark.bat`:**
  - `Telex on_key`: **27.70M keys/s (36.1 ns/key)** — nhanh hơn **+17.0%**.
  - `Simple Telex`: **27.54M keys/s (36.3 ns/key)** — nhanh hơn **+20.0%**.
  - `End-to-End Simulation`: **24.07M keys/s (41.6 ns/key)** — nhanh hơn **+16.4%**.
  - `Hook Rule Resolution`: Vượt **300.41M keys/s (3.3 ns/key)**.

---

### 5.9. [ĐÃ HOÀN THÀNH] Macro Length Bitmask Filter & Zero-Heap Expansion, Fast ASCII Length & Bracket Short-Circuit (Milestone 12)
- **Bối cảnh & Vấn đề:**
  1. **Hash Lookup Lãng Phí Trên Mọi Từ Thông Thường Trong `expand_word`:** Khi người dùng gõ từ và ngắt từ bằng dấu cách, `handle_word_break` luôn gọi `macro_table.expand_word(rendered, ...)`. Dù người dùng gõ từ tiếng Việt bình thường (như *"người"*, *"trường"*, *"không"*), hàm cũ luôn băm SipHash trên từng byte để tra cứu `entries.get(lower)`. Đồng thời `expand_word` và `apply_case_style` trả về heap `String`, gây cấp phát động liên tục.
  2. **Kiểm Tra Điều Kiện Thừa Phím Ngoặc Trong Typestate Handlers:** Trong `State<Empty>`, `State<Onset>`, `State<Nucleus>`, logic kiểm tra `config.bracket_w && config.method.has_bracket_shortcuts()` chạy trước khi kiểm tra ký tự phím, dù 99.9% phím gõ là chữ cái thông thường.
  3. **Fast ASCII UTF-16 Length Trong `feed_key`:** Đối với các âm tiết ASCII thuần (khi gõ phụ âm đầu, chữ tiếng Anh, hoàn tác), độ dài UTF-16 luôn bằng đúng byte length $O(1)$, không cần gọi `.chars()`.
- **Giải pháp kỹ thuật đã triển khai:**
  1. **Bộ Lọc Bitmask Độ Dài Phím Tắt (`normal_lengths_mask: u32`):**
     - Lưu trữ bitmask 32-bit trong `MacroTable`: bit thứ $k$ được bật nếu tồn tại ít nhất 1 macro loại `Normal` có độ dài $k$ ký tự. Cập nhật tức thì cả khi gọi `insert_typed_no_cache` hay `rebuild_cache`.
     - Trong `expand_word`: Nếu `word.len() < 32 && (normal_lengths_mask & (1 << word.len())) == 0`, bỏ qua hoàn toàn việc tính mã băm SipHash và tra cứu `HashMap` trong đúng 1 lệnh bitwise CPU.
     - Khi người dùng chỉ bật gõ nhanh phụ âm đầu/cuối mặc định mà không có Normal macro, `normal_lengths_mask == 0`, triệt tiêu 100% việc lookup HashMap.
  2. **100% Zero-Heap Expansion Với `CompactString`:**
     - Chuyển `expand_word` và `apply_case_style` sang trả về `Option<CompactString>`, thực hiện toàn bộ việc copy/uppercase trên buffer stack 64 bytes của `CompactString`, triệt tiêu `malloc` khi gõ tắt. Thêm `impl From<CompactString> for String` phục vụ chuyển đổi không chi phí.
  3. **Fast ASCII Length Check trong Common Prefix:**
     - Nếu `output.is_ascii()` (bao gồm tất cả các âm tiết ASCII, phụ âm đầu, chữ hoàn tác), tính ngay độ dài UTF-16 qua $O(1)$ byte length `output.len()`.
  4. **Short-Circuit Phím Ngoặc `[` và `]`:**
     - Đặt điều kiện `if key.ch != '[' && key.ch != ']' { return None; }` lên đầu các hàm xử lý shortcut, loại bỏ 99.9% ký tự chữ cái trong đúng 1 chu kỳ clock.
- **Kết quả đo đạc chính thức qua `cmd /c benchmark.bat` (Lần 3 chuẩn):**
  - **Full Screen Simulation:** Phá vỡ rào cản 40 ns, đạt **25.09M keys/s (39.8 ns/key)** — tăng tốc thêm **+4.3%** so với Milestone 11 (**+21.3%** so với Milestone 10).
  - **Macro Expansion (All Unmuted):** Bứt phá từ 19.65M lên **25.47M keys/s (39.3 ns/key)** — tăng tốc **+29.6%**!
  - **Macro with Muted Rows:** Lần đầu tiên vượt mốc 30 triệu phím/giây, đạt **30.34M keys/s (33.0 ns/key)** (**+19.4%**).
  - **Keyboard Hook Rule Resolution:** Đạt **321.83M keys/s (3.1 ns/key)**.

---

### 5.10. [ĐÃ HOÀN THÀNH] Win32 App Tracker: Event-Driven Foreground/Focus Hook & Lock-Free Target Strategy
- **Bối cảnh:** Kora hỗ trợ profile cấu hình độc lập cho từng ứng dụng (tự động tắt tiếng Việt khi chơi Game, bật macro riêng khi mở VS Code, Photoshop) và tự động chọn chiến lược bơm phím phù hợp cho trình duyệt/Omnibox/IDE.
- **Giải pháp đã triển khai trong `src/platform/win32/app_tracker.rs`, `uia_detect.rs` & `app_detect.rs`:**
  1. **Đăng ký `SetWinEventHook` (`EVENT_SYSTEM_FOREGROUND` & `EVENT_OBJECT_FOCUS`):** Chỉ kích hoạt khi người dùng thực sự chuyển đổi cửa sổ hoặc vùng nhập liệu (Event-Driven 100%, 0% CPU khi đứng yên gõ phím), đẩy toàn bộ truy vấn UI Automation sang luồng worker nền riêng biệt để không bao giờ chặn luồng giao diện hay luồng hook.
  2. **Lock-Free Target Strategy (`CURRENT_TARGET_STRATEGY: AtomicU8`):** Lưu chiến lược bơm phím đã phân giải vào biến nguyên tử 1 byte (`get_current_target_strategy_raw` / `set_current_target_strategy_raw`), giúp luồng bàn phím đọc chiến lược trong $O(1)$ không khóa và không gọi lại Win32/UIA trên mỗi nhát phím.

---

### 5.11. [ĐÃ HOÀN THÀNH] Tối Ưu Hóa Khởi Động (Ultra-Fast Startup & Lazy SoundBank Loading)
- **Bối cảnh:** Kora là phần mềm chạy nền cùng hệ điều hành (Run on Windows Startup), yêu cầu thời gian khởi động tức thì để không làm chậm trễ quá trình mở máy của người dùng.
- **Giải pháp đã triển khai trong `src/platform/win32/sound/mod.rs`:**
  1. **Lazy SoundBank Loading trên luồng nền (`kora-wasapi-audio`):** Quá trình nạp WAV chạy tách biệt hoàn toàn trên worker thread và chỉ đọc đĩa khi `sound_enabled == true` (mặc định khởi tạo `SoundBank::empty()` 0 byte).
  2. **Zero-Cost Sleep khi tắt âm thanh:** Khi tắt Soundpack, luồng âm thanh giải phóng toàn bộ mẫu WAV, gọi `trim_working_set()` và ngủ vô thời hạn trên `receiver.recv()` (0% CPU, 0 wakeups).

---

### 5.12. [ĐÃ HOÀN THÀNH] Kiến Trúc Giao Diện Pure Win32 GDI Native & Working Set Trimming (< 1 MB RAM)
- **Bối cảnh:** Thay vì sử dụng các framework đồ họa nặng (Egui / WebView / Electron), toàn bộ bảng điều khiển Control Panel được viết bằng **100% Pure Win32 GDI Native** (`src/ui/`).
- **Giải pháp đã triển khai trong `src/ui/mod.rs` & `src/platform/win32/mod.rs`:**
  1. **Event-Driven GDI Rendering & Double Buffering:** Chỉ vẽ lại khi có sự kiện `WM_PAINT` với `BackBuffer` chống nháy hình, tiêu thụ **0% GPU** khi không tương tác.
  2. **Tự động thu hồi RAM (`SetProcessWorkingSetSize`):** Khi ẩn Control Panel xuống System Tray (`hide_control_panel`), Kora gọi `trim_working_set()` trả toàn bộ trang bộ nhớ đệm về cho Windows, đưa mức tiêu thụ RAM chạy ngầm về **0.5 – 0.9 MB** (vượt xa mục tiêu 15 MB ban đầu).

---

### 5.13. [ĐÃ HOÀN THÀNH - Giai đoạn 13] Data Layout Diet, `VOWEL_DECOMPOSE_LUTS`, Raw-Byte Common Prefix, $O(1)$ Word Snapshot & Win32 Hook Syscall/Lock Diet
- **Bối cảnh & Điểm nghẽn phát hiện qua rà soát chuyên sâu:**
  1. **Phình kích thước `SyllableState` (96 bytes) & `RawKey` (8 bytes):** Biến thể `Passthrough(InlineList<char, 16>)` chiếm 68 bytes dù `TypingBuffer` đã có sẵn `raw_keys`/`last_rendered`; `OnsetState` và `CodaState` lưu tuple `(char, bool)` 8 bytes cho các phụ âm thuần ASCII; `RawKey` tốn 8 bytes khiến `InlineList<RawKey, 32>` nặng 260 bytes.
  2. **Giải mã/Mã hóa UTF-8 lặp lại & 66 nhánh `decompose_vowel`:** `render_compact` phải chuyển từ `u16` BMP sang `char` rồi gọi `encode_utf8`; `decompose_vowel` vẫn dùng `match` 66 nhánh và `Tcvn3Charset::encode_char` phải gọi thêm `char::is_uppercase()` trên bảng Unicode.
  3. **Chi phí ẩn trên `TypingBuffer` & `WordHistory`:** `is_boundary` quét lại `raw_keys` trên mỗi chữ hoa; Common Prefix giải mã `chars()` hai luồng và đếm lại UTF-16 cho cả phần tiền tố không đổi; `on_backspace` qua dấu cách phải chạy lại máy trạng thái cho từng phím trong từ cũ.
  4. **Chi phí kiểm tra Macro & Mute Cache:** Từ tiếng Việt có dấu bị gọi `to_lowercase()` cấp phát `String` trên heap trước khi kiểm tra bitmask gõ tắt; `ensure_active_macro_table` duyệt `Vec<String>` đuổi theo từng con trỏ heap trên mọi lần ngắt từ khi có dòng macro bị mute.
  5. **Độ trễ hệ thống trên luồng Win32 Low-Level Hook:** Giữ `Mutex` trên `ENGINE_INSTANCE` xuyên suốt lời gọi đồng bộ `SendInput` và `CallNextHookEx`; kiểm tra IME `SendMessageTimeoutW` liên tiến trình trên cả `KEYUP` và layout bàn phím Latin/US; lặp lại 11 lần syscall `GetAsyncKeyState` trên mỗi phím nhấn và 5 lần trên mỗi cú click chuột thường; quét hai lượt UTF-16 trong `send_replace`.
- **Giải pháp kỹ thuật đã triển khai đồng bộ (0 crate phụ thuộc ngoài):**
  1. **Thu gọn triệt để cấu trúc dữ liệu lõi (`SyllableState`: 96B $\rightarrow$ 26B, `RawKey`: 8B $\rightarrow$ 4B):**
     - Đổi `Passthrough::Data` thành ZST `()`, nén `OnsetState` xuống **6 bytes** (`InlineList<u8, 4>` + `bool`) và `CodaState` xuống **25 bytes** (`NucleusState` + `InlineList<u8, 2>` + `Option<BaseVowel>`), gắn `#[repr(u8)]` cho `Tone`, `Diacritic`, `BaseVowel`.
     - Đóng gói `#[repr(transparent)] pub struct RawKey(u32)` (bit 31 lưu `caps`, 21 bit thấp lưu `char`), giảm `InlineList<RawKey, 32>` từ 260B xuống **132B**, bổ sung `InlineList::from_slice` chỉ copy phần tử đang hoạt động `..len`.
     - Bổ sung hệ hàm chính tả thao tác trực tiếp trên lát cắt byte ASCII (`is_valid_onset_ascii`, `is_valid_onset_vowel_orthography_ascii`, `is_valid_vietnamese_components_ascii`, `is_unambiguous_completed_vietnamese_syllable_ascii`) và bảo toàn tương thích tuyệt đối với bảng mã TCVN3.
  2. **Bảng hằng Compile-Time `VOWEL_DECOMPOSE_LUTS` (662B), `INITIAL_ASCII_LUT` (128B) & Direct UTF-8 Byte Push:**
     - Thay 66 nhánh `match` của `decompose_vowel` bằng hai mảng hằng số `([u16; 241], [u16; 90])` đóng gói sẵn cả cờ viết hoa (`decompose_vowel_with_case`), kết hợp `is_d_stroke` không rẽ nhánh `((c as u32) & !1) == 0x0110`.
     - Bổ sung `INITIAL_ASCII_LUT: [u8; 128]` phân loại phím đầu tiên trong `State<Empty>::feed_key` bằng 1 lần đọc mảng, đánh giá lười closure `has_onset_d`, và thêm `CompactString::push_bmp_u16` / `push_ascii_bytes` ghi thẳng byte UTF-8 từ `compose_vowel_u16`.
  3. **Cờ gia tăng $O(1)$ `is_boundary`, Raw-Byte Common Prefix & Khôi phục từ $O(1)$ (`restore_snapshot`):**
     - Theo dõi `has_underscore` và `has_lower_alpha` gia tăng để kiểm tra biên `camelCase` trong $O(1)$.
     - So khớp tiền tố trực tiếp trên byte UTF-8, lùi về ranh giới codepoint gần nhất và chỉ tính `utf16_len_utf8` không rẽ nhánh trên phần hậu tố thay đổi.
     - Lưu ảnh chụp trạng thái `(raw_keys, state, rendered, emitted_len, is_raw_restored)` trong `CommittedWord` để `on_backspace` qua dấu cách khôi phục tức thì bằng `restore_snapshot` mà không cần phát lại từng phím.
  4. **Bộ lọc Macro Tiền-Lowercase, Stack Lowercase & Chữ ký Mute $O(1)$ không đuổi con trỏ:**
     - Kiểm tra độ dài, ký tự ASCII cuối cùng và nguyên âm liền kề **trước khi** hạ chữ thường trong `MacroTable::expand_word`, loại bỏ tức thì các từ tiếng Việt phổ biến (*"tiếng"*, *"không"*, *"những"*, *"thành"*) trong ~2 chu kỳ CPU; các từ còn lại hạ chữ thường trên stack qua `lowercase_to_compact` (0 heap alloc).
     - Thay vòng lặp so sánh chuỗi trong `ensure_active_macro_table` bằng chữ ký băm FNV-1a 64-bit trên cặp `(ptr, len)` của slice `muted_macros` kết hợp bộ đếm thế hệ `MacroTable::mutation_gen()`.
  5. **Tối ưu hóa tầng Win32 Hook & `SendInput` Injector:**
     - Nhả khóa `Mutex` của `ENGINE_INSTANCE` **trước khi** gọi `send_replace` (`SendInput`), `CallNextHookEx` và `notify_mode_change`.
     - Bỏ kiểm tra IME trên `WM_KEYUP`/`WM_SYSKEYUP` và chỉ gọi `ImmGetDefaultIMEWnd` + `SendMessageTimeoutW` khi layout bàn phím hiện hành (`GetKeyboardLayout`) thực sự thuộc nhóm CJK (`0x04`, `0x11`, `0x12`).
     - Gộp các lần đọc trạng thái modifier trùng lặp, giảm `GetAsyncKeyState` từ 11 xuống 5 syscall trên mỗi phím nhấn và xuống **0 syscall** trên các cú click chuột thông thường; gộp mã hóa UTF-16 một lượt duy nhất vào mảng `MaybeUninit<INPUT>` trong `send_replace`.
- **Kết quả đo đạc chính thức qua `cmd /c benchmark.bat`:**
  - **`Telex (on_key raw)`:** Bứt phá từ 26.90M lên **32.33M keys/s (30.9 ns/key)** — tăng tốc **+20.2%** (phá vỡ mốc 30 triệu phím/giây!).
  - **`Simple Telex (on_key raw)`:** Bứt phá từ 27.08M lên **30.91M keys/s (32.4 ns/key)** — tăng tốc **+14.1%**.
  - **`English Bypass (on_key raw)`:** Vượt mốc 110 triệu phím/giây, đạt **110.52M keys/s (9.0 ns/key)**.
  - **`Macro with Muted Rows`:** Tăng tốc từ 30.34M lên **32.94M – 34.11M keys/s (29.3 – 30.4 ns/key)**.
  - **`Hook Rule Resolution (4 keys)`:** Chạm mốc **328.46M – 330.88M keys/s (3.0 ns/key)**.

