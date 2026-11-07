;; ============================================================================
;; Kora Input Engine — AutoCAD Smart Charset Detection Sidecar
;; Phiên bản: 1.1.0
;; Tác giả: Kora Team (Google DeepMind pair programming)
;;
;; Chức năng:
;; Tự động phát hiện Font chữ hiện tại của AutoCAD (TextStyle / Text Object)
;; và đồng bộ tức thì sang bộ gõ Kora (Port 13497) qua Local IPC.
;; - Font TCVN3 (.VnTime, .VnArial, vn_*.shx, tc.shx) -> Kora tự chuyển sang TCVN3
;; - Font Unicode (Arial, Times New Roman, Tahoma...) -> Kora tự chuyển sang Unicode
;;
;; Lưu ý cài đặt:
;; 1. Gõ lệnh APPLOAD trong AutoCAD -> Thêm file này vào "Startup Suite" (Contents...).
;; 2. Để AutoCAD không hỏi cảnh báo bảo mật mỗi lần khởi động, thêm thư mục chứa
;;    file này vào biến hệ thống TRUSTEDPATHS (Options -> Files -> Trusted Locations).
;; ============================================================================

(vl-load-com)

;; Biến toàn cục lưu trạng thái
(setq kora:*last-font* nil)
(setq kora:*enabled* t)
(setq kora:*ipc-url* "http://127.0.0.1:13497/config")
(setq kora:*cmd-reactor* nil)
(setq kora:*sys-reactor* nil)

;; ----------------------------------------------------------------------------
;; 1. Hàm thay thế toàn bộ chuỗi (Thay thế tất cả khoảng trắng thành %20)
;; ----------------------------------------------------------------------------
(defun kora:replace-all (str old new / pos)
  (while (setq pos (vl-string-search old str))
    (setq str (strcat (substr str 1 pos) new (substr str (+ pos 1 (strlen old))))))
  str
)

;; ----------------------------------------------------------------------------
;; 2. Hàm gửi tên Font tới Kora IPC Server (127.0.0.1:13497)
;; ----------------------------------------------------------------------------
(defun kora:send-font (font-name / font-trimmed http-obj full-url res)
  (if (and kora:*enabled* font-name)
    (progn
      (setq font-trimmed (vl-string-trim " " font-name))
      ;; Chỉ gửi nếu font thực sự thay đổi so với lần gửi thành công gần nhất
      (if (not (equal (strcase font-trimmed) (strcase (vl-princ-to-string kora:*last-font*))))
        (progn
          (setq http-obj (or (vlax-create-object "MSXML2.ServerXMLHTTP.6.0")
                             (vlax-create-object "WinHttp.WinHttpRequest.5.1")))
          (if http-obj
            (progn
              (setq full-url (strcat kora:*ipc-url* "?name=acad.exe&font="
                                     (kora:replace-all font-trimmed " " "%20")))
              ;; Gọi đồng bộ với timeout 200ms (an toàn, không crash khi Kora tắt)
              (setq res (vl-catch-all-apply
                          '(lambda ()
                             (vlax-invoke-method http-obj 'open "GET" full-url :vlax-false)
                             (vlax-invoke-method http-obj 'setTimeouts 100 100 100 200)
                             (vlax-invoke-method http-obj 'send)
                           )
                          nil
                        )
              )
              ;; Cập nhật cache font nếu gửi thành công hoặc Kora phản hồi
              (if (not (vl-catch-all-error-p res))
                (setq kora:*last-font* font-trimmed)
                ;; Nếu Kora chưa bật, cập nhật tạm để không bị đơ 200ms liên tục ở mọi lệnh sau đó
                (setq kora:*last-font* font-trimmed)
              )
              ;; Luôn giải phóng COM pointer an toàn, chống memory leak
              (vlax-release-object http-obj)
            )
          )
        )
      )
    )
  )
  (princ)
)

;; ----------------------------------------------------------------------------
;; 3. Hàm đọc tên Font của TextStyle (Hỗ trợ cả SHX và TrueType qua XData)
;; ----------------------------------------------------------------------------
(defun kora:get-current-style-font (/ cur-style ent font-name xd)
  (setq cur-style (getvar "TEXTSTYLE"))
  (if (and cur-style (setq ent (tblobjname "STYLE" cur-style)))
    (progn
      ;; Đọc kèm XData của ứng dụng ACAD để lấy font TrueType (.VnTime, Arial...)
      (setq ent (entget ent '("ACAD")))
      (setq font-name (cdr (assoc 3 ent)))              ; File font SHX hoặc TTF
      (if (or (null font-name) (equal font-name ""))
        (progn
          (setq xd (cdr (assoc "ACAD" (cdr (assoc -3 ent)))))
          (setq font-name (cdr (assoc 1000 xd)))        ; Tên Typeface TrueType từ XData
        )
      )
      (if (or (null font-name) (equal font-name ""))
        "Arial"
        font-name
      )
    )
    "Arial"
  )
)

;; ----------------------------------------------------------------------------
;; 4. Bắt sự kiện khi bắt đầu lệnh gõ Text (Command Reactor)
;; ----------------------------------------------------------------------------
(defun kora:on-command-start (reactor params / cmd font-name)
  (setq cmd (strcase (car params)))
  (if (member cmd '(
        "TEXT" "MTEXT" "-TEXT" "DTEXT"
        "DDEDIT" "TEXTEDIT" "MTEDIT"
        "QLEADER" "LEADER"
        "DIMLINEAR" "DIMALIGNED" "DIMANGULAR" "DIMRADIUS" "DIMDIAMETER"
        "ATTDEF" "ATTEDIT" "EATTEDIT" "BEDIT"
      ))
    (progn
      (setq font-name (kora:get-current-style-font))
      (kora:send-font font-name)
    )
  )
)

;; ----------------------------------------------------------------------------
;; 5. Bắt sự kiện khi đổi biến hệ thống TEXTSTYLE (SysVar Reactor)
;; ----------------------------------------------------------------------------
(defun kora:on-sysvar-changed (reactor params / var-name font-name)
  (setq var-name (strcase (car params)))
  (if (equal var-name "TEXTSTYLE")
    (progn
      (setq font-name (kora:get-current-style-font))
      (kora:send-font font-name)
    )
  )
)

;; ----------------------------------------------------------------------------
;; 6. Khởi tạo và đăng ký các Reactor lắng nghe sự kiện
;; ----------------------------------------------------------------------------
(defun kora:init-reactors ()
  (if kora:*cmd-reactor*
    (vlr-remove kora:*cmd-reactor*)
  )
  (if kora:*sys-reactor*
    (vlr-remove kora:*sys-reactor*)
  )

  (setq kora:*cmd-reactor*
    (vlr-command-reactor nil '((:vlr-commandWillStart . kora:on-command-start)))
  )
  (setq kora:*sys-reactor*
    (vlr-sysvar-reactor nil '((:vlr-sysVarChanged . kora:on-sysvar-changed)))
  )

  ;; Đồng bộ ngay lập tức font hiện tại khi file vừa load
  (kora:send-font (kora:get-current-style-font))
  (princ "\n[Kora] Auto-Charset Reactor da khoi dong thanh cong! (Port 13497)\n")
  (princ)
)

;; ----------------------------------------------------------------------------
;; 7. Lệnh tiện ích người dùng trong AutoCAD
;; ----------------------------------------------------------------------------

;; Lệnh: KORA_SYNC -> Đồng bộ ngay style chữ hiện hành sang Kora
(defun c:KORA_SYNC (/ font-name)
  (setq kora:*last-font* nil) ; Reset cache để ép gửi lại
  (setq font-name (kora:get-current-style-font))
  (kora:send-font font-name)
  (princ (strcat "\n[Kora] Da dong bo font '" font-name "' sang bo go Kora.\n"))
  (princ)
)

;; Lệnh: KORA_STATUS -> Kiểm tra kết nối tới Kora
(defun c:KORA_STATUS (/ http-obj res)
  (setq http-obj (or (vlax-create-object "MSXML2.ServerXMLHTTP.6.0")
                     (vlax-create-object "WinHttp.WinHttpRequest.5.1")))
  (if http-obj
    (progn
      (setq res (vl-catch-all-apply
                  '(lambda ()
                     (vlax-invoke-method http-obj 'open "GET" "http://127.0.0.1:13497/status" :vlax-false)
                     (vlax-invoke-method http-obj 'setTimeouts 200 200 200 400)
                     (vlax-invoke-method http-obj 'send)
                     (vlax-get-property http-obj 'responseText)
                   )
                  nil
                )
      )
      (if (vl-catch-all-error-p res)
        (princ "\n[Kora] Khong the ket noi toi Kora IPC Server tai 127.0.0.1:13497! (Hay kiem tra Kora da bat chua)\n")
        (princ (strcat "\n[Kora] Ket noi thanh cong: " res "\n"))
      )
      (vlax-release-object http-obj)
    )
    (princ "\n[Kora] Khong the tao HTTP Object tren Windows!\n")
  )
  (princ)
)

;; Lệnh: KORA_ON / KORA_OFF -> Bật/Tắt tính năng tự động nhận diện font
(defun c:KORA_ON ()
  (setq kora:*enabled* t)
  (princ "\n[Kora] Tinh nang tu dong nhan dien font: [BAT]\n")
  (c:KORA_SYNC)
)

(defun c:KORA_OFF ()
  (setq kora:*enabled* nil)
  (princ "\n[Kora] Tinh nang tu dong nhan dien font: [TAT]\n")
  (princ)
)

;; Tự động khởi tạo ngay khi tải file LISP
(kora:init-reactors)
(princ)
