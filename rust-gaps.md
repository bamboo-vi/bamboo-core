# Các giới hạn hành vi đã biết (bamboo-rust)

> **Cập nhật 18/11/2026 — 0.6.0.** Tài liệu này chỉ ghi các điểm còn cần chốt
> hợp đồng hoặc cải thiện. Các số liệu lịch sử nằm ở
> `../bench-fcitx5/comparison-2026-10-21.md`; chúng không phải kết luận cho mã hiện tại.

## Hành vi còn cần chốt

### Ký tự xuống dòng trong preedit

- Tái hiện: đưa `\n` vào cuối câu trong preedit.
- Rust giữ `\n` trong preedit; bản Go hiện trả chuỗi rỗng.
- Chưa kết luận bên nào đúng: frontend như fcitx5 có thể xem Enter là thao tác commit thay vì
  một ký tự đầu vào. Cần chốt hợp đồng frontend/core rồi khóa hành vi bằng một kiểm thử.

## Các ca đã được giải quyết

- **Dài hơn cửa sổ active:** cửa sổ có 32 transformation và từ 0.4.1 khi đầy sẽ commit phần
  đầu, nên định danh như `getUserNameByIdentifier` không còn bị cắt preedit. Từ 0.6.0, ranh giới
  chỉ xảy ra khi transformation của phím không vừa slot còn lại và được báo cho host qua
  `EditDelta::committed` / `Engine::just_committed()` thay vì reset ngầm (issue #41).
- **Horn `uow`:** từ 0.4.1, `duow` tạo `dươ` trong đúng lớp onset; `khuow` vẫn tạo `khuơ`. Sau 0.6.0,
  `cuow`, `kuow` và `uow` (VNI `uo7`) cũng tạo `ươ` ngay, khác `cvMatrix` của Go (#47); `uơ` chỉ
  còn sau `h kh qu th` (`thuở`, `huơ`, `khuơ`).
- **camelCase không có khoảng trắng:** với `auto_correct=true`, core chủ động trả về keystroke
  thô khi phần nối không còn là tiền tố tiếng Việt hợp lệ. Đây là chính sách kiểm tra chính tả,
  không phải lỗi mất preedit. Với `auto_correct=false`, `VieetjNam` cho `ViệtNam`; addon Ngó
  Sen dùng chế độ này.
- **Phím dấu bị từ chối sau âm tiết hợp lệ:** từ 0.5.5, giữ âm tiết đã hoàn tất và thêm phím bị từ
  chối theo dạng literal thay vì khôi phục các phím dấu đã được tiêu thụ trước đó.
- **Onset phiên âm mở rộng:** `f`, `j`, `w`, `z` được chấp nhận như phụ âm đầu theo cùng nhóm vần;
  đây là hỗ trợ kiểu gõ/phiên âm, không phải chính tả tiếng Việt chuẩn. Từ 0.6.0 có thể tắt theo
  từng app bằng `Config::extended_onsets` (mặc định bật).
- **Final mở rộng:** từ 0.6.0, `Config::extended_finals` (mặc định tắt) cho phép `g`, `k`, `nk`,
  `h` đứng ở vị trí âm cuối và được kiểm như `ng`, `c`, `nh`, `nh`.
- **Đổi config giữa từ:** từ 0.6.0, thao tác kế tiếp gõ lại phím của từ đang gõ với config mới
  như một engine mới. Thành phần không thể replay trung thực (văn bản từ `rebuild_from_text`,
  phím `Mode::English`, replay phát sinh commit, hoặc phím kế tiếp kết thúc từ) được giữ nguyên;
  xem doc `Engine::set_config`.

## Hành vi đã được xác nhận, không nên sửa nhầm

- Undo từng phím: `tiếng` + Backspace → `tiêng`.
- Chữ lạ như `qwer` và `getUs` giữ nguyên thay vì bị ép dấu sớm.
- `khuow` giữ `khuơ`; VNI, VIQR và các câu chuẩn giữ parity với Go trong bộ kiểm thử hiện có.
