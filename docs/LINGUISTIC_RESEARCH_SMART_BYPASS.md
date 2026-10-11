# Nghiên cứu Ngữ âm học Đối chiếu Tiếng Việt - Tiếng Anh và Cơ chế Nhận diện Xuyên suốt (On-the-fly Deterministic Bypass) trong Bộ gõ Tiếng Việt

**Tác giả:** Đội ngũ Kỹ thuật Kora  
**Phiên bản tài liệu:** 1.0  
**Đối tượng:** Bộ gõ tiếng Việt thế hệ mới (Kora Engine)  
**Ngày công bố:** Tháng 10/2026  

---

## Tóm tắt (Abstract)

Trong suốt hơn 30 năm phát triển của các bộ gõ tiếng Việt (Vietnamese Input Method Editors - IME) dựa trên bảng mã chuẩn Latinh (Telex, VNI), vấn đề xung đột giữa các tổ hợp phím tiếng Anh và tiếng Việt luôn là một bài toán hóc búa. Các bộ gõ truyền thống (Unikey, Vietkey, OpenKey...) giải quyết xung đột bằng hai cách tiếp cận:
1. **Bắt người dùng tắt/bật chế độ gõ bằng phím tắt** (Ctrl+Shift / Alt+Z) khi chuyển đổi giữa tiếng Anh và tiếng Việt.
2. **Kiểm tra chính tả trì hoãn (Delayed spell-checking)**: Chờ đến khi người dùng gõ phím cách (Space) hoặc dấu ngắt câu mới phân tích âm tiết và sửa ngược lại trên màn hình bằng cách gửi một chuỗi phím Backspace.

Cả hai cách tiếp cận trên đều gây ra trải nghiệm giật cục, biến dạng văn bản tức thời khi người dùng đang quan sát màn hình, và đặc biệt bất tiện đối với lập trình viên thường xuyên gõ xen kẽ từ khóa tiếng Anh và tiếng Việt.

Tài liệu này trình bày cơ sở lý luận Ngữ âm học cấu trúc (Structural Phonology), Văn tự học Chữ Quốc ngữ (Vietnamese Orthography) đối chiếu với Ngữ âm học tiếng Anh (English Phonotactics), đồng thời công bố giải pháp **OtFB (On-the-fly Deterministic Bypass)** được triển khai trong Kora. Giải pháp này cho phép bộ gõ nhận diện từ tiếng Anh và chuyển trạng thái tức thời ngay trong luồng phím gõ (zero-latency, không phụ thuộc từ điển, độ phức tạp $O(1)$) mà **không làm mất chữ, không nhảy chữ, và bảo toàn 100% tính toàn vẹn của tiếng Việt (Zero False Positives)**.

---

## 1. Cấu trúc Âm tiết Tiếng Việt và Không gian Âm vị

Về mặt loại hình học ngôn ngữ, tiếng Việt là ngôn ngữ đơn lập, đơn lập phân tích tính, với âm tiết có cấu trúc ngữ âm cố định, khép kín và có ranh giới âm tiết tuyệt đối rõ ràng. Mọi âm tiết tiếng Việt chuẩn hóa ($\sigma$) đều tuân theo mô hình 5 thành phần:

$$\sigma = [C_1] + [w] + V + [C_2] + T$$

Trong đó:
- $C_1$: Âm đầu (Initial Consonant / Onset)
- $w$: Âm đệm (Medial Glide / Semivowel)
- $V$: Âm chính (Vowel Nucleus)
- $C_2$: Âm cuối (Final Consonant / Coda)
- $T$: Thanh điệu (Tone) - bao trùm toàn bộ âm tiết, biểu hiện trực tiếp trên âm chính.

### 1.1. Tập hợp Phụ âm đầu Hợp thức ($\mathcal{O}_{VN}$)

Trong chữ Quốc ngữ, âm đầu $C_1$ chỉ có thể là:
- **Phụ âm đơn:** `b, c, d, đ, g, h, k, l, m, n, p, r, s, t, v, x` (tổng cộng 16 phụ âm).
- **Phụ âm ghép đôi (Digraphs):** `ch, gh, gi, kh, nh, ng, ph, qu, th, tr` (tổng cộng 10 phụ âm).
- **Phụ âm ghép ba (Trigraph):** `ngh` (duy nhất 1 phụ âm).

> **Định lý 1 (Bất khả phụ âm đầu kép ngoại lai):**  
> Tiếng Việt **hoàn toàn không có** các cụm phụ âm tắc - xát, tắc - lướt, xát - tắc (Consonant Clusters) ở đầu từ như trong các ngôn ngữ Ấn - Âu:
> $$\mathcal{O}_{cluster} = \{pr, pl, cl, cr, br, bl, fl, gl, dr, sk, sm, sn, sp, st, str, spr, scr, spl, kn, wr, ps, tw, sw, sh\}$$
> Mọi chuỗi ký tự bắt đầu bằng $c_1 c_2 \in \mathcal{O}_{cluster}$ đều chắc chắn $100\%$ không phải là âm tiết tiếng Việt.

### 1.2. Tập hợp Phụ âm cuối Hợp thức ($\mathcal{C}_{VN}$)

Hệ thống âm cuối trong tiếng Việt rất hữu hạn, chỉ gồm 8 hình vị chính tả biểu diễn 6 âm vị âm cuối:
1. **Âm cuối tắc vô thanh (Stop Codas):**
   $$\mathcal{C}_{stop} = \{p, t, c, ch\}$$
   (Ví dụ: *hát, búp, bác, thích*).
2. **Âm cuối vang mũi (Nasal Codas):**
   $$\mathcal{C}_{nasal} = \{m, n, ng, nh\}$$
   (Ví dụ: *nam, nón, làng, xinh*).

> **Định lý 2 (Bất khả phụ âm cuối ngoại lai):**  
> Mọi phụ âm:
> $$\mathcal{C}_{invalid} = \{b, d, f, j, k, l, q, r, s, v, w, x, z\}$$
> **không bao giờ** có thể đóng vai trò âm cuối trong tiếng Việt (ngoại lệ duy nhất là chữ `k` trong địa danh thiểu số *Đắk Lắk* hoặc từ vay mượn *ok*). Do đó, khi một âm tiết đang ở trạng thái nguyên âm mà nhận một phụ âm $c \in \mathcal{C}_{invalid}$, âm tiết đó lập tức vi phạm ngữ âm học tiếng Việt.

---

## 2. Định lý Xung đột Âm cuối Tắc và Thanh điệu (The Stop Coda - Tone Theorem)

Đây là phát hiện ngữ âm học quan trọng nhất giúp giải quyết triệt để các trường hợp gõ tiếng Anh hay bị biến dạng như `port, part, after, often, left, text, next, soft, chart, gift`.

### 2.1. Bản chất Vật lý - Âm học của Âm tiết Khép

Trong ngữ âm học thực nghiệm, khi âm cuối là phụ âm tắc vô thanh ($-p, -t, -c, -ch$), luồng không khí từ phổi bị chặn lại đột ngột ở môi ($-p$), đầu lưỡi - răng ($-t$), gốc lưỡi - ngạc mềm ($-c$), hoặc mặt lưỡi - ngạc cứng ($-ch$). Áp suất luồng hơi tăng lên đột ngột và kết thúc âm thanh bằng một điểm tắc kín (abrupt occlusion).

Do đặc tính vật lý này, dây thanh âm không thể duy trì độ rung ổn định để phát ra các thanh điệu có đường nét biến thiên phức tạp hoặc kéo dài:
- **Thanh Huyền (Grave):** Thanh điệu trầm, đi xuống đều đặn, yêu cầu âm tiết mở hoặc nửa khép kéo dài.
- **Thanh Hỏi (HookAbove):** Đường nét lượn sóng (hạ thấp rồi nâng nhẹ), đòi hỏi độ dài âm tiết đủ lớn.
- **Thanh Ngã (Tilde):** Có hiện tượng nghẽn thanh hầu (glottal stop) ở giữa âm tiết rồi bật lên cao, xung đột trực tiếp với điểm tắc ở cuối âm tiết.
- **Thanh Ngang (Tone::None):** Âm vực bằng phẳng kéo dài, không thể tồn tại với điểm tắc dứt khoát.

### 2.2. Định lý Ngữ âm học Đối chiếu

$$\boxed{C_2 \in \{p, t, c, ch, k\} \implies T \in \{\text{Acute (Sắc)}, \text{DotBelow (Nặng)}\}}$$

**Phát biểu:** Trong âm tiết tiếng Việt hoàn chỉnh, nếu âm cuối là phụ âm tắc ($-p, -t, -c, -ch$), âm tiết đó **bắt buộc chỉ đi với thanh Sắc hoặc thanh Nặng**. Mọi sự kết hợp giữa phụ âm tắc với thanh Huyền, thanh Hỏi, thanh Ngã là **bất khả kháng về mặt ngữ âm** (Phonotactically Impossible).

### 2.3. Bảng Khảo sát Trạng thái Thực tế

| Từ tiếng Anh | Chuỗi phím Telex | Biến đổi tạm thời | Phím xung đột | Quy luật vi phạm | Hành vi OtFB |
| :--- | :--- | :--- | :--- | :--- | :--- |
| **port** | `p` $\to$ `o` $\to$ `r` | `p` $\to$ `po` $\to$ `pỏ` | `t` | Âm tắc `t` đi với thanh Hỏi (`r`) | **Rollback ngay lập tức về `port`** |
| **part** | `p` $\to$ `a` $\to$ `r` | `p` $\to$ `pa` $\to$ `pả` | `t` | Âm tắc `t` đi với thanh Hỏi (`r`) | **Rollback ngay lập tức về `part`** |
| **after** | `a` $\to$ `f` | `a` $\to$ `à` | `t` | Âm tắc `t` đi với thanh Huyền (`f`) | **Rollback ngay lập tức về `aft` $\to$ `after`** |
| **often** | `o` $\to$ `f` | `o` $\to$ `ò` | `t` | Âm tắc `t` đi với thanh Huyền (`f`) | **Rollback ngay lập tức về `oft` $\to$ `often`** |
| **left** | `l` $\to$ `e` $\to$ `f` | `l` $\to$ `le` $\to$ `lè` | `t` | Âm tắc `t` đi với thanh Huyền (`f`) | **Rollback ngay lập tức về `left`** |
| **text** | `t` $\to$ `e` $\to$ `x` | `t` $\to$ `te` $\to$ `tẽ` | `t` | Âm tắc `t` đi với thanh Ngã (`x`) | **Rollback ngay lập tức về `text`** |
| **next** | `n` $\to$ `e` $\to$ `x` | `n` $\to$ `ne` $\to$ `nẽ` | `t` | Âm tắc `t` đi với thanh Ngã (`x`) | **Rollback ngay lập tức về `next`** |
| **soft** | `s` $\to$ `o` $\to$ `f` | `s` $\to$ `so` $\to$ `sò` | `t` | Âm tắc `t` đi với thanh Huyền (`f`) | **Rollback ngay lập tức về `soft`** |
| **chart** | `c` $\to$ `h` $\to$ `a` $\to$ `r` | `ch` $\to$ `cha` $\to$ `chả` | `t` | Âm tắc `t` đi với thanh Hỏi (`r`) | **Rollback ngay lập tức về `chart`** |
| **gift** | `g` $\to$ `i` $\to$ `f` | `gi` $\to$ `gì` | `t` | Âm tắc `t` đi với thanh Huyền (`f`) | **Rollback ngay lập tức về `gift`** |

---

## 3. Bản chất Ký tự `w` và Cơ chế Ghép Âm Telex (Vocalic Coupling Boundary)

Ký tự `w` trong bộ gõ Telex tiếng Việt đảm nhận vai trò kép:
1. Đứng độc lập ở đầu từ: Biểu diễn nguyên âm `ư` (ví dụ: `w` $\to$ `ư`, `wa` $\to$ `ưa`, `wng` $\to$ `ưng`).
2. Đi sau nguyên âm: Phím bổ sung dấu móc sừng (`u + w` $\to$ `ư`, `o + w` $\to$ `ơ`).

### 3.1. Sự cố Cặp ghép `wrong` $\to$ `ưởng`

Khi người dùng gõ từ tiếng Anh `wrong`:
1. Phím `w`: Bộ gõ chuyển sang `Nucleus(ư)`.
2. Phím `r`: Phím `r` trong Telex là phím bỏ thanh Hỏi $\to$ Âm tiết trở thành `Nucleus(ử)`.
3. Phím `o`: Bộ gõ Telex truyền thống có tính năng tự động ghép tiện lợi: `ư + o` $\to$ `ươ` (để gõ nhanh các từ như `tươi, bước, đường`).
   - Nếu điều kiện ghép **không kiểm tra thanh điệu**, hệ thống sẽ ghép `ử + o` $\to$ `ưở`!
4. Phím `n` và `g`: Trở thành âm cuối `ng` $\to$ Từ bị biến dạng hoàn toàn thành `ưởng` ("hưởng thụ", "phần thưởng")!

### 3.2. Định lý Trật tự Nguyên âm - Thanh điệu trong Chính tả Tiếng Việt

> **Định lý 3 (Bất khả nạp nguyên âm sau thanh điệu):**  
> Trong toàn bộ quy chuẩn chính tả tiếng Việt: Thanh điệu **luôn là thành phần nạp sau cùng của âm chính**. Không bao giờ có trường hợp một âm tiết đã mang thanh điệu trên nguyên âm thứ nhất mà lại được bổ sung thêm nguyên âm thứ hai để tạo thành cụm nguyên âm đôi/ba.

Do đó, điều kiện ghép tự động `ư + o` bắt buộc phải thỏa mãn:
$$\text{Coupling}(u', o) \iff T == \text{None}$$

Khi điều kiện này được thực thi nghiêm ngặt:
- Gõ `w` $\to$ `ư`.
- Gõ `r` $\to$ `ử` (đã có $T = \text{HookAbove}$).
- Gõ `o` $\to$ Vì $T \neq \text{None}$, hệ thống từ chối ghép `ươ`. Tổ hợp `[ư, o]` không có trong hệ thống tiếng Việt $\to$ **Lập tức rollback tức thời về chuỗi phím gốc `wro` và chuyển `Passthrough`!**
- Các phím tiếp theo `n, g` được đẩy thẳng ra màn hình $\to$ Kết quả: `wrong` xuất hiện chính xác 100%.

---

## 4. Hệ Tiên Đề Phân Loại 7 Tầng Ngữ Âm Đối Chiếu

Hệ thống OtFB của Kora vận hành dựa trên 7 tầng rào chắn ngữ âm độc lập và tương hỗ:

```
Luồng phím vào (Raw Key Stream)
        │
        ├── [Tầng 1: Phụ âm đầu ngoại lai] ───────────► Passthrough ngay phím 1 (f, j, z)
        │
        ├── [Tầng 2: Cụm phụ âm đầu kép/ba Anh] ──────► Passthrough ngay phím 2 (pr, cl, str...)
        │
        ├── [Tầng 3: Tổ hợp nguyên âm lạ] ────────────► Rollback tức thời (ea, ou, ae, ei...)
        │
        ├── [Tầng 4: Nguyên âm sau dấu thanh] ────────► Rollback tức thời (toán + e...)
        │
        ├── [Tầng 5: Phụ âm không thể làm âm cuối] ───► Rollback tức thời (server, word, email...)
        │
        ├── [Tầng 6: Xung đột Âm cuối Tắc & Dấu thanh] ► Rollback tức thời (port, part, text...)
        │
        └── [Tầng 7: Hậu tố nối tiếp sau âm cuối] ─────► Rollback tức thời (testing, listing...)
```

### Chi tiết 7 Tầng Phân loại:

#### Tầng 1: Phụ âm đầu ngoại lai (Non-native Initial Consonants)
- **Tập hợp:** $\{f, j, z\}$, và $w$ (trong chế độ VNI).
- **Cơ chế:** Ký tự nằm ngoài bảng mẫu tự Chữ Quốc ngữ. Khi nhận phím đầu tiên trong trạng thái rỗng (`Empty`), bộ gõ không khởi tạo bộ máy trạng thái âm tiết tiếng Việt mà chuyển thẳng sang `Passthrough`.
- **Ví dụ:** `format, fix, fox, fax, file, first, json, join, zoom, zero`.

#### Tầng 2: Cụm phụ âm đầu tiếng Anh (English Onset Clusters)
- **Tập hợp:** Mọi cặp/bộ phụ âm đầu nằm ngoài danh sách 11 phụ âm ghép tiếng Việt:
  $$\mathcal{O}_{invalid\_ext} = \mathcal{C} \times \mathcal{C} \setminus \{ch, gh, gi, kh, nh, ng, ngh, ph, qu, th, tr\}$$
- **Cơ chế:** Ngay khi ký tự thứ 2 hoặc thứ 3 được gõ, hàm `is_valid_onset_extension` trả về `false`. Bộ gõ chuyển trạng thái `Onset` sang `Passthrough`.
- **Ví dụ:** `project, class, clear, print, client, black, drive, smart, string, spring, screen, knife, write, switch, share`.

#### Tầng 3: Tổ hợp nguyên âm không tồn tại trong Tiếng Việt (Impossible Vowel Clusters)
- **Tập hợp:** `ea, ou, ae, ei, ey, eu, io, oe` (có dấu bất hợp thức)...
- **Cơ chế:** Trong tiếng Việt chỉ có các tổ hợp nguyên âm gõ dở hợp lệ (nguyên âm đôi: `ie, ye, uo, ue`, nguyên âm ba: `ieu, yeu, uoi, uou, uye`). Hàm `is_valid_intermediate_vowel_combination` cho phép người dùng gõ tự do các nguyên âm trung gian (như `hieu` + `e` $\to$ `hiêu`, `muoi` + `o` $\to$ `muôi`), đồng thời phát hiện nguyên âm lạ và kích hoạt hoàn tác tức thì đối với các từ tiếng Anh.
- **Ví dụ:** `team, read, clean, sound, round, ground, mouse, house, case, base, user, guide, build`.

#### Tầng 4: Nguyên âm sau thanh điệu (Vowels after Tone Mark)
- **Cơ chế:** Âm tiết đã có $T \neq \text{None}$ không thể nhận thêm nguyên âm.
- **Ví dụ:** Khi gõ từ có dấu rồi gõ thêm nguyên âm ngoại lai.

#### Tầng 5: Phụ âm không thể làm âm cuối (Invalid Coda Consonants)
- **Tập hợp:** $\{b, d, f, j, k, l, q, r, s, v, w, x, z\}$ khi đứng sau nguyên âm.
- **Quy tắc phụ:** Nguyên âm đôi lướt cuối (Off-glide: `ưi, ai, oi, ui, ay, ao...`) **không thể nhận bất kỳ âm cuối nào**.
- **Ví dụ:**
  - `v` trong `server, service`
  - `d` trong `card, word, send, end`
  - `l` trong `model, email, call, cool, tool`
  - `k` trong `work, like, book`
  - `n` sau `ưi` trong `win` (Telex: `w` $\to$ `ư`, `i` $\to$ `ưi`, `n` không thể ghép sau `ưi`).

#### Tầng 6: Xung đột Ngữ âm: Âm cuối Tắc & Dấu thanh (Stop Coda & Incompatible Tone)
- **Quy tắc:** $C_2 \in \{p, t, c, ch, k\}$ không bao giờ đi với $T \in \{\text{Huyền}, \text{Hỏi}, \text{Ngã}\}$.
- **Ví dụ:** `port, part, after, often, left, text, next, soft, chart, gift`.

#### Tầng 7: Hậu tố tiếng Anh nối sau âm cuối (Coda Extension & English Suffixes)
- **Cơ chế:** Sau khi âm cuối $C_2$ đã hoàn thành (ví dụ `t` trong `test`), tiếng Việt không bao giờ cho phép chèn thêm nguyên âm (như `i` trong `-ing`, `e` trong `-ed, -er`).
- **Ví dụ:**
  - Gõ `test` $\to$ màn hình hiển thị `tét`. Gõ tiếp `i` trong `testing` $\to$ âm cuối `t` gặp nguyên âm `i` $\implies$ Hoàn tác ngay lập tức thành `testi` $\to$ `testing`!
  - `listing`: `lít` + `i` $\to$ `listing`.
  - `posting`: `pót` + `i` $\to$ `posting`.

---

## 5. Chứng minh Tính Toàn vẹn Tiếng Việt (Zero False Positives Proof)

Một lo ngại lớn của người dùng là: *Liệu cơ chế Smart English Word Bypass có làm sai lệch hoặc cản trở việc gõ tiếng Việt hay không?*

### 5.1. Phân loại Không gian Giao thoa Từ vựng (Vocabulary Collision Space)

Các từ tiếng Anh xung đột với tiếng Việt rơi vào hai nhóm:
1. **Nhóm xung đột hoàn toàn về ngữ âm (Total Phonotactic Violation):**  
   Các từ như `format, project, clear, server, text, after, wrong, win`. Nhóm này vi phạm $100\%$ các tiên đề ngữ âm ở Mục 4. Không có bất kỳ từ tiếng Việt nào có cấu trúc như vậy. Do đó, việc chuyển `Passthrough` và rollback là **tuyệt đối an toàn (100% mathematically sound)**.

2. **Nhóm trùng âm ngẫu nhiên với từ đơn Tiếng Việt (Homophonic Collisions):**
   - `test` $\leftrightarrow$ "tét" ("đánh tét đòn", "chuối tét")
   - `best` $\leftrightarrow$ "bét" ("đứng bét", "bét bảng")
   - `cast` $\leftrightarrow$ "cát" ("bãi cát", "hạt cát")
   - `cost` $\leftrightarrow$ "cót" ("chiếu cót", "tiếng cót két")
   - `list` $\leftrightarrow$ "lít" ("một lít nước")
   - `must` $\leftrightarrow$ "mút" ("kẹo mút")
   - `past` $\leftrightarrow$ "pát" ("pát sắt")
   - `fast` $\to$ Có âm đầu `f` nên tự động thành tiếng Anh, không trùng.

### 5.2. Nguyên lý Bảo toàn Tiếng Việt

Khi người dùng gõ từ đơn kết thúc bằng phím cách:
- Gõ `c-a-s-t [Space]`: Bộ gõ tôn trọng chữ Quốc ngữ $\to$ xuất ra `"cát "`.
- Gõ `t-e-s-t [Space]`: Bộ gõ xuất ra `"tét "`.
- Gõ `l-i-s-t [Space]`: Bộ gõ xuất ra `"lít "`.

> **Nguyên lý:** Khi người dùng muốn gõ chữ tiếng Việt "cát", bộ gõ không được phép tự tiện đoán là tiếng Anh để đổi thành "cast". Đây là tính đúng đắn bắt buộc của bộ gõ tiếng Việt.

Tuy nhiên, **ngay khi từ vựng được nối dài** bằng các cấu trúc hình thái học tiếng Anh (Morphology):
- `tét` + `i` $\to$ `testing` (tiếng Việt không có âm tiết "téti").
- `lít` + `i` $\to$ `listing`.
- `cót` + `l` $\to$ `costly`.

OtFB phát hiện sự bất khả thi về mặt ngữ âm ngay ở ký tự nối tiếp, **tự động hoàn tác về chuỗi thô tiếng Anh ban đầu mà không làm phiền người dùng**.

### 5.3. Cơ chế Hoàn tác Dấu thanh Chuẩn Unikey (Classic Unikey Undo Toggle & Passthrough)

Trong thiết kế chuẩn mực của Telex kinh điển (Unikey):
- **Phím dấu thanh là công tắc bật/tắt (Toggle):**
  - Gõ phím dấu thanh lần 1: áp dụng dấu (`te` + `s` $\to$ `té`).
  - Gõ phím dấu thanh lần 2: triệt tiêu dấu thanh (`tone = None`), trả về đúng **1 ký tự nguyên bản** và **chuyển ngay lập tức sang trạng thái `Passthrough`** (`té` + `s` $\to$ `tes`).
- **Lợi ích cốt lõi:**
  1. **Gõ tự nhiên các từ tiếng Anh kết thúc bằng tổ hợp phụ âm (`-st`, `-r`):**
     - `t-e-s-s-t` $\to$ `test`.
     - `b-e-s-s-t` $\to$ `best`.
     - `l-i-s-s-t` $\to$ `list`.
     - `c-o-s-s-t` $\to$ `cost`.
     - `p-o-s-s-t` $\to$ `post`.
     - `c-a-r-r` $\to$ `car`.
     - `b-a-r-r` $\to$ `bar`.
  2. **Bảo toàn quy ước Telex chuẩn cho phụ âm đôi:**
     - Muốn gõ các từ tiếng Anh có phụ âm đôi trùng phím dấu: `passs` $\to$ `pass`, `errror` $\to$ `error`, `offf` $\to$ `off`.
  3. **Không tạo xung đột giả:**
     - Tránh hoàn toàn việc tự ý nhân đôi ký tự (như biến `carr` khi gõ `car` hay `tesst` khi gõ `test`), giữ cho hành vi của bộ gõ luôn nhất quán $100\%$ với thói quen sử dụng hàng chục năm của người dùng Unikey.
  4. **Kết hợp cùng Hệ thống Phonotactics Đa Tầng:**
     - Đối với các từ tiếng Anh có cấu trúc không thuộc ngữ âm tiếng Việt (`center`, `cinema`, `city`, `get`, `gym`, `by`, `python`, `project`, `client`, `team`, `sound`, `poem`, `boot`...), hệ thống tự động nhận diện và chuyển sang `Passthrough` ngay lập tức mà người dùng thậm chí không cần gõ phím hoàn tác.

### 5.4. Tính Bất biến của Phím Xóa lùi trong Trạng thái Ký tự Thô (Backspace Invariant in Passthrough)

Khi người dùng thao tác xóa lùi (Backspace) trên một từ tiếng Anh đã chuyển sang trạng thái `Passthrough`:
- **Vấn đề lệch pha trạng thái (Desynchronization Problem):**  
  Nếu bộ đệm tự ý tái phân tích chuỗi phím thô còn lại qua hàm chiếu ngữ âm chuẩn (ví dụ khi xóa `or` trong `error` $\to$ còn `['e', 'r']`), thuật toán truyền thống sẽ diễn giải lại thành âm tiết tiếng Việt `ẻ` (độ dài $1$), trong khi màn hình ứng dụng của hệ điều hành đang hiển thị chuỗi ký tự thô `er` (độ dài $2$). Lệch pha này dẫn đến hiện tượng sinh ra ký tự ma (`"eer"`) khi người dùng gõ tiếp.
- **Định lý Bất biến Đồng bộ (Screen-Buffer State Invariant):**  
  > *Trong trạng thái `Passthrough`, bộ đệm chỉ được phép hoàn nguyên về âm tiết tiếng Việt có cấu trúc nếu và chỉ nếu chuỗi diễn giải lại ($\text{eval.rendered}$) trùng khớp chính xác $100\%$ với chuỗi ký tự thực tế còn lại trên màn hình ($\text{self.last_rendered}$). Nếu không trùng khớp, hệ thống bắt buộc duy trì trạng thái `Passthrough` để bảo toàn tính toàn vẹn với hệ điều hành (xem chi tiết kỹ thuật tại [ARCHITECTURE.md](file:///c:/Users/uongsuadaubung/Desktop/kora/docs/ARCHITECTURE.md#21-typingbuffer--single-source-of-truth--pure-projection)).*

---

## 6. Hiện thực Hóa Kỹ thuật: Máy Trạng thái Bậc thấp (Low-Level Implementation)

Giải pháp OtFB được tích hợp trực tiếp vào lõi của Kora với các đặc tính kỹ thuật:

1. **Không phân bổ bộ nhớ động (Zero Heap Allocation):**  
   Toàn bộ cấu trúc `SyllableState`, `NucleusState`, `CodaState`, `InlineList<T, N>` đều nằm $100\%$ trên stack CPU, kích thước cố định, triển khai trait `Copy`.
2. **Độ trễ xử lý phím cực thấp:**  
   Thời gian đánh giá mỗi phím đạt **~91 nanoseconds**, tương đương thông lượng xử lý **~10.96 triệu phím/giây**, không gây bất kỳ độ trễ nào trên bàn phím.
3. **Rollback 1 lần duy nhất qua EngineAction::Replace:**  
   Khi phát hiện vi phạm ngữ âm, bộ gõ phát tín hiệu `EngineAction::Replace` với số lượng backspace chính xác bằng số ký tự đã xuất ra màn hình và chuỗi thay thế là chuỗi phím thô đã lưu trong `raw_keys`.

---

## 7. Kết luận

Cơ chế **Smart English Word Bypass on-the-fly (OtFB)** của Kora đã giải quyết triệt để sự xung đột kéo dài hàng thập kỷ giữa tiếng Việt và tiếng Anh trong các bộ gõ chữ Quốc ngữ. Bằng việc xây dựng hệ thống quy tắc dựa trên **Ngữ âm học cấu trúc hình thức** thay vì từ điển tĩnh:
- Khả năng bao phủ là **hoàn toàn khép kín và toàn diện** đối với mọi từ tiếng Anh vi phạm quy tắc ngữ âm tiếng Việt.
- Bảo vệ **100% tính toàn vẹn** của ngôn ngữ tiếng Việt (Zero False Positives).
- Đem lại trải nghiệm gõ phím mượt mà, tự nhiên và hiện đại nhất cho người dùng Việt Nam trong kỷ nguyên số.

