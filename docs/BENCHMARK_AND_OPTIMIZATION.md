# Kora Engine — Hướng Dẫn Benchmark & Định Hướng Tối Ưu Hóa Hiệu Năng

Tài liệu này ghi chép chi tiết phương pháp chạy kiểm chuẩn hiệu năng (Benchmark), giải thích các cơ chế đo đạc, cách khắc phục lỗi tài nguyên đang chạy trên Windows, cùng triết lý và các định hướng tối ưu hóa chuyên sâu cho **Kora Vietnamese Input Engine**.

---

## 1. Hướng Dẫn Chạy Benchmark

### 1.1. Lệnh Benchmark Chuẩn Xác & Đa Kịch Bản

Khi Kora đang được kích hoạt và chạy nền trên thanh Taskbar/System Tray của Windows, tệp thực thi `target\release\Kora.exe` bị khóa bởi hệ điều hành. Để benchmark độc lập mà **không cần tắt Kora đang sử dụng**, sử dụng công cụ phân tích đa kịch bản trong `examples/benchmark.rs` cùng cờ `--target-dir` để điều hướng tiến trình build sang một thư mục đích cách ly:

```bash
cargo run --release --target-dir target/bench_run --example benchmark
```

### 1.2. Phân Tích Chi Tiết Từng Tham Số

| Tham số | Ý nghĩa kỹ thuật |
| :--- | :--- |
| `--release` | Kích hoạt profile biên dịch tối ưu hóa cao nhất (`opt-level = 3`, `lto = true`, `codegen-units = 1`, `panic = "abort"`). |
| `--target-dir target/bench_run` | Cách ly hoàn toàn thư mục build kiểm chuẩn, tránh xung đột quyền truy cập tệp (*Access is denied*) với tiến trình `Kora.exe` đang chạy nền. |
| `--example benchmark` | Chạy bộ đo lường đa kịch bản `examples/benchmark.rs` (Telex, Simple Telex, English Bypass, Full Screen Simulation, Macro Mute, Hook Rule Resolution). |

---

## 2. Giải Thích 2 Chỉ Số Đo Lường Cốt Lõi

Khi chạy benchmark, console sẽ in ra các chỉ số phản ánh các tầng kiến trúc khác nhau:

### 2.1. `Pure Engine on_key` (Tốc độ xử lý lõi phím thô)
- **Bản chất:** Đo đạc thời gian máy trạng thái ngữ âm (`SyllableState`) phân loại phím, chuyển dịch trạng thái, tra cứu ngữ âm học và xác định hành vi (`EngineAction`).
- **Ý nghĩa:** Phản ánh độ trễ trực tiếp trên luồng chặn bàn phím (`Low-Level Keyboard Hook`).
- **Thực tế:** Đạt **~25.45 triệu phím/giây** (**39.3 nano-giây / phím**). Nhanh gấp **150.000 lần** ngưỡng mắt người có thể cảm nhận (~6 ms).

### 2.2. `Full Screen Simulation` (Mô phỏng gõ trên màn hình thực tế)
- **Bản chất:** Đo đạc toàn bộ chu trình hoàn chỉnh từ lúc nhấn phím, máy trạng thái tính toán, chạy thuật toán tiền tố chung (*Common Prefix Optimization*), phát sinh số lượng phím `Backspace` cần xóa lùi, cập nhật bộ đệm và nối chuỗi thay thế vào màn hình ảo.
- **Ý nghĩa:** Phản ánh trải nghiệm gõ thực tế của người dùng khi gõ các đoạn văn bản tiếng Việt phức tạp với đầy đủ dấu thanh, dấu mũ, dấu móc và từ tiếng Anh.
- **Thực tế:** Đạt **~22.50 triệu phím/giây** (**44.5 nano-giây / phím**).

---

## 3. Lịch Sử Các Cột Mốc Đo Lường Thực Tế (Benchmark Milestones Tracker)

Bảng dưới đây ghi chép lại toàn bộ quá trình tiến hóa về mặt hiệu năng qua các phiên bản và các đợt tái cấu trúc:

| Cột mốc (Milestone) | Nội dung tối ưu hóa trọng tâm | Pure Engine `on_key` | Full Screen Simulation | Tỷ lệ cải thiện |
| :--- | :--- | :---: | :---: | :---: |
| **Giai đoạn 1** (`ba0b93a`) | Stack `InlineList<T, N>`, `SyllableState` Copy (24B), ASCII fast-path `decompose_vowel`, `MaybeUninit` Win32 SendInput. | ~10.96M keys/s (91 ns) | ~7.80M keys/s (128 ns) | Mốc nền tảng ban đầu |
| **Giai đoạn 2** (`63eaf08`) | Đưa `raw_keys` sang `InlineList<RawKey, 32>`, `Arc<str>` cho config strings, dọn dẹp các phép clone chuỗi trên hot path. | ~13.70M keys/s (73 ns) | ~10.20M keys/s (98 ns) | **+25.0%** |
| **Giai đoạn 3** (`8974399`) | Cache bảng macro active (`ensure_active_macro_table`), mượn trực tiếp `&config` thay vì clone, fast-path 1-chu kỳ cho `is_boundary`. | ~21.01M keys/s (48 ns) | ~15.29M keys/s (65 ns) | **+53.3%** |
| **Giai đoạn 4** | Bảng tra cứu tĩnh `ASCII_VOWELS` ($O(1)$), Bitmask 128-bit cho `is_word_break`, Letter Fast-Path, `is_active_global` cờ nhị phân, Single-Pass Common Prefix Optimization. | **25.43M keys/s** (**39 ns**) | **18.39M keys/s** (**54 ns**) | **+21.0%** |
| **Giai đoạn 5** | Stack projection `render_chars_to` (0 heap allocation khi fallback Passthrough), Guard `find_tone_position()` khi không có thanh điệu, Early-exit Coda Guard trong `handle_nucleus`, Jump-Table `is_valid_onset_vowel_orthography`, Single-Pass length tracking. | ~20.66M – 25.43M keys/s (39–48 ns) | ~20.30M keys/s (49 ns) | **+10.4% Full Simulation** |
| **Giai đoạn 6** | **Bitmask Phonotactic Table cho `can_vowels_accept_coda`** (biến 16 nhánh match thành 1 lệnh bitwise AND duy nhất) + **Zero-Heap `CompactString` cho `EngineAction::Replace`** (stack buffer 24 bytes, 100% stack-allocated không tốn `malloc` khi thay thế ký tự/bỏ dấu thanh). | **22.7M – 24.1M keys/s** (**41 ns**) | **20.6M – 22.7M keys/s** (**44 ns**) | **+11.6% Full Simulation** 🚀 |
| **Giai đoạn 7** (`db34a8c`) | **100% Bitmask Toàn Bộ Quy Tắc Ghép Vần Tiếng Việt**: `is_valid_onset`, `is_valid_onset_extension`, `is_valid_coda_start`, `is_valid_coda_pair`, `is_stop_coda`, `is_valid_onset_vowel_orthography`, `is_valid_coda_start_for_vowels`. Triệt tiêu hoàn toàn mọi chuỗi `match/if-else` trong toàn bộ module chính tả `spelling.rs`. | **20.9M – 24.6M keys/s** (**41–48 ns**) | **18.9M – 22.7M keys/s** (**44–50 ns**) | **100% O(1) Bitwise Zero-Branching** ⚡ |
| **Giai đoạn 8** (Hoàn tất) | **True 100% Zero-Heap trên cả nhánh Hoàn Tác & Khôi Phục Chính Tả + Thu Gọn L1 Cache**: Đổi `ModifierOutcome::Undone` và Smart English Bypass sang `CompactString` (đẩy English Bypass lên **92.8M keys/s = 10.8 ns**), kiểm tra biên UTF-8 1-chu kỳ trong `CompactString::drain`, gộp các hàm `render_*` xóa 84 dòng code thừa, thu gọn LUT phụ âm đầu từ 1.050B xuống **64B (1 L1 cache line)** không kiểm tra biên (`& 31`). | **21.6M – 24.5M keys/s** (**40.8–46.3 ns**) | **20.35M keys/s** (**49.1 ns**) | **English Bypass: 92.8M keys/s (10.8 ns)** 🔥 |
| **Giai đoạn 9** (Hoàn tất) | **Zero-Allocation Hook Rule Resolution (`3.8 ns/key`) + `VOWEL_COMPOSE_LUT` (`[u16; 360]`) + Bitmask Macro Pre-filter**: Triệt tiêu cấp phát `Vec` & clone `String` trên mọi nhát phím tại `low_level_keyboard_proc` (`is_key_disabled`, `lookup_key_remap`, `lookup_key_macro`), thay 72 nhánh `compose_vowel` bằng LUT 720B $O(1)$, thêm bitmask lọc nhanh phụ âm đầu/cuối trong `MacroTable::expand_word`. | **25.45M keys/s** (**39.3 ns**) | **22.50M keys/s** (**44.5 ns**) | **Hook Resolution: 261.6M keys/s (3.8 ns — nhanh gấp 39.2x)** ⚡🚀 |

### 3.1. Chi Tiết Đo Đạc Cột Mốc Hiện Tại (Benchmark Log Record)

- **Thời gian đo:** Ngày 14 tháng 10 năm 2026.
- **Môi trường:** Windows 11 x64, Rust toolchain 2024 edition.
- **Cấu hình Profile Release:** `opt-level = 3`, `lto = true`, `codegen-units = 1`, `panic = "abort"`.

```text
[1] PURE ENGINE ON_KEY LATENCY & THROUGHPUT (No screen allocation):
  Benchmark Case                             |       Throughput |    Latency per Key
  -------------------------------------------+------------------+-------------------
  Telex (on_key raw)                         |  25451390 keys/s |    39.3 ns/key (0.039 µs)
  Simple Telex (on_key raw)                  |  24428842 keys/s |    40.9 ns/key (0.041 µs)
  English Bypass (on_key raw)                |  90025602 keys/s |    11.1 ns/key (0.011 µs)

[2] FULL SCREEN SIMULATION (Handling Backspace & Replacement):
  Benchmark Case                             |       Throughput |    Latency per Key
  -------------------------------------------+------------------+-------------------
  End-to-End Vietnamese Typing               |  22495089 keys/s |    44.5 ns/key (0.044 µs)

[3] MACRO EXPANSION & ZERO-ALLOCATION ROW MUTE BENCHMARK:
  Benchmark Case                             |       Throughput |    Latency per Key
  -------------------------------------------+------------------+-------------------
  Macro (All Unmuted)                        |  15988552 keys/s |    62.5 ns/key (0.063 µs)
  Macro with Muted Rows (Cached Zero-Alloc)  |  20795641 keys/s |    48.1 ns/key (0.048 µs)

[4] KEYBOARD HOOK RULE RESOLUTION (Remap, Disable, Key Macro):
  Benchmark Case                             |       Throughput |    Latency per Key
  -------------------------------------------+------------------+-------------------
  Hook Rule Resolution (4 keys)              | 247524752 keys/s |     4.0 ns/key (0.004 µs)
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

### Nguyên Tắc 1: Zero-Heap Allocation trên Hot-Path
- **Vấn đề:** Mọi thao tác cấp phát bộ nhớ động (`malloc`, `String`, `Vec`) đều phải tương tác với bộ quản lý bộ nhớ của hệ điều hành, gây mất ổn định về độ trễ (*jitter*) và tiêu tốn hàng trăm chu kỳ CPU.
- **Giải pháp áp dụng:**
  - Chuyển toàn bộ các mảng lưu trữ trạng thái (`Onset`, `Nucleus`, `Coda`, `raw_keys`) sang mảng tĩnh kích thước cố định trên Stack: `InlineList<T, N>`.
  - Biến `SyllableState` và toàn bộ các thành phần con thành kiểu dữ liệu 100% `Copy` (chỉ 24 bytes trên Stack), sao chép trực tiếp qua thanh ghi CPU.
  - Sử dụng buffer stack `with_lowercase_key` cho bảng gõ tắt và `MaybeUninit` cho mảng `SendInput` ở tầng Win32 Hook.

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
  - **Bảng tra cứu tĩnh `ASCII_VOWELS: [Option<BaseVowel>; 128]`:** Xác định nguyên âm cơ bản và phân tách nguyên âm thô trong đúng 1 lệnh truy xuất chỉ mục mảng $O(1)$.

---

### Nguyên Tắc 4: Thuật Toán Một Lượt (Single-Pass) & Bảo Tồn Vùng Đệm
- **Vấn đề:** Việc duyệt qua duyệt lại một chuỗi nhiều lần (ví dụ đếm số ký tự, tìm vị trí byte, rồi lại cắt chuỗi) làm suy giảm hiệu năng bộ nhớ đệm L1/L2.
- **Giải pháp áp dụng:**
  - **Single-Pass `is_boundary`:** Gom kiểm tra dấu gạch dưới và phát hiện chữ thường vào 1 vòng lặp duy nhất, trả về sớm ngay khi gặp `_`.
  - **Tối ưu hóa Common Prefix:** Tích lũy trực tiếp số byte trùng khớp `common_bytes += c1.len_utf8()` ngay trong vòng lặp so sánh đầu tiên. Dùng `last_rendered.truncate(common_bytes)` để tái sử dụng tiền tố cũ thay vì xóa sạch rồi copy lại từ đầu.

---

### Nguyên Tắc 5: Triệt Tiêu So Sánh Chuỗi (Heap String Comparison)
- **Vấn đề:** So sánh chuỗi `active_id == "global"` đòi hỏi dereference con trỏ heap và gọi hàm `memcmp`. Trên 25 triệu phím/giây, điều này tạo ra hơn 50 triệu lần so sánh chuỗi vô ích mỗi giây.
- **Giải pháp áp dụng:**
  - Lưu cờ nhị phân `is_active_global: bool` trên struct `VietnameseEngine` và cập nhật khi chuyển đổi profile ứng dụng. Mọi thao tác kiểm tra cấu hình biến thành 1 phép thử cờ thanh ghi máy tính.

---

## 5. Định Hướng Nâng Cấp Tiềm Năng (Optimization Roadmap)

### 5.1. [ĐÃ HOÀN THÀNH - Giai đoạn 6] Zero-Heap `CompactString` cho `EngineAction::Replace`
- **Mục tiêu:** Đạt **100% Zero-Allocation** tuyệt đối trên toàn bộ hot-path gõ phím.
- **Giải pháp đã triển khai:**
  - Thiết kế `CompactString` với layout `Inline { buf: [u8; 24], len: u8 }` (32 bytes trên Stack, 0 heap allocations cho 100% các âm tiết tiếng Việt như *"nghiêng"*, *"chuyển"*, *"phượng"*).
  - Triệt tiêu hoàn toàn `malloc` khi bỏ dấu thanh hoặc dấu phụ trên máy trạng thái.
  - Tự động fallback sang `Heap(String)` an toàn nếu mở rộng các đoạn macro gõ tắt văn bản dài.

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

### 5.8. [ĐÃ HOÀN THÀNH] Win32 App Tracker: Event-Driven Foreground Window & Atomic Caching
- **Bối cảnh:** Kora hỗ trợ profile cấu hình độc lập cho từng ứng dụng (tự động tắt tiếng Việt khi chơi Game, bật macro riêng khi mở VS Code, Photoshop).
- **Giải pháp đã triển khai trong `src/platform/win32/app_tracker.rs` & `app_detect.rs`:**
  1. **Đăng ký `SetWinEventHook(EVENT_SYSTEM_FOREGROUND)`:** Chỉ lắng nghe sự kiện khi người dùng thực sự chuyển đổi qua lại giữa các cửa sổ ứng dụng (Event-Driven 100%, 0% CPU khi đứng yên trong một ứng dụng).
  2. **Atomic Process & Focus Caching (`CACHED_PID`, `CACHED_APP_KIND`, `CACHED_FOCUS_HWND`, `CACHED_FIX_TYPE`):** Lưu đệm PID và HWND focus bằng các biến `Atomic` không khóa, giúp nhận diện trình duyệt/Omnibox trong $O(1)$ mà không cần mở lại handle tiến trình qua Kernel.

---

### 5.9. [ĐÃ HOÀN THÀNH] Tối Ưu Hóa Khởi Động (Ultra-Fast Startup & Lazy SoundBank Loading)
- **Bối cảnh:** Kora là phần mềm chạy nền cùng hệ điều hành (Run on Windows Startup), yêu cầu thời gian khởi động tức thì để không làm chậm trễ quá trình mở máy của người dùng.
- **Giải pháp đã triển khai trong `src/platform/win32/sound/mod.rs`:**
  1. **Lazy SoundBank Loading trên luồng nền (`kora-wasapi-audio`):** Quá trình nạp WAV chạy tách biệt hoàn toàn trên worker thread và chỉ đọc đĩa khi `sound_enabled == true` (mặc định khởi tạo `SoundBank::empty()` 0 byte).
  2. **Zero-Cost Sleep khi tắt âm thanh:** Khi tắt Soundpack, luồng âm thanh giải phóng toàn bộ mẫu WAV, gọi `trim_working_set()` và ngủ vô thời hạn trên `receiver.recv()` (0% CPU, 0 wakeups).

---

### 5.10. [ĐÃ HOÀN THÀNH] Kiến Trúc Giao Diện Pure Win32 GDI Native & Working Set Trimming (< 1 MB RAM)
- **Bối cảnh:** Thay vì sử dụng các framework đồ họa nặng (Egui / WebView / Electron), toàn bộ bảng điều khiển Control Panel được viết bằng **100% Pure Win32 GDI Native** (`src/ui/`).
- **Giải pháp đã triển khai trong `src/ui/mod.rs` & `src/platform/win32/mod.rs`:**
  1. **Event-Driven GDI Rendering & Double Buffering:** Chỉ vẽ lại khi có sự kiện `WM_PAINT` với `BackBuffer` chống nháy hình, tiêu thụ **0% GPU** khi không tương tác.
  2. **Tự động thu hồi RAM (`SetProcessWorkingSetSize`):** Khi ẩn Control Panel xuống System Tray (`hide_control_panel`), Kora gọi `trim_working_set()` trả toàn bộ trang bộ nhớ đệm về cho Windows, đưa mức tiêu thụ RAM chạy ngầm về **0.5 – 0.9 MB** (vượt xa mục tiêu 15 MB ban đầu).

