# Các giới hạn hành vi đã biết (bamboo-rust)

> **Cập nhật 12/11/2026 — 0.5.5.** Tài liệu này chỉ ghi các điểm còn cần chốt
> hợp đồng hoặc cải thiện. Các số liệu lịch sử nằm ở
> `../bench-fcitx5/comparison-2026-10-21.md`; chúng không phải kết luận cho mã hiện tại.

## Hành vi còn cần chốt

### Ký tự xuống dòng trong preedit

- Tái hiện: đưa `\n` vào cuối câu trong preedit.
- Rust giữ `\n` trong preedit; bản Go hiện trả chuỗi rỗng.
- Chưa kết luận bên nào đúng: frontend như fcitx5 có thể xem Enter là thao tác commit thay vì
  một ký tự đầu vào. Cần chốt hợp đồng frontend/core rồi khóa hành vi bằng một kiểm thử.

## Các ca đã được giải quyết

- **Dài hơn cửa sổ active:** từ 0.4.1, cửa sổ có 32 transformation và khi đầy sẽ commit phần
  đầu, nên định danh như `getUserNameByIdentifier` không còn bị cắt preedit.
- **Horn `uow`:** từ 0.4.1, `duow` tạo `dươ` trong đúng lớp onset; `khuow` vẫn tạo `khuơ`.
- **camelCase không có khoảng trắng:** với `auto_correct=true`, core chủ động trả về keystroke
  thô khi phần nối không còn là tiền tố tiếng Việt hợp lệ. Đây là chính sách kiểm tra chính tả,
  không phải lỗi mất preedit. Với `auto_correct=false`, `VieetjNam` cho `ViệtNam`; addon Ngó
  Sen dùng chế độ này.
- **Phím dấu bị từ chối sau âm tiết hợp lệ:** từ 0.5.5, giữ âm tiết đã hoàn tất và thêm phím bị từ
  chối theo dạng literal thay vì khôi phục các phím dấu đã được tiêu thụ trước đó.
- **Onset phiên âm mở rộng:** `f`, `j`, `w`, `z` được chấp nhận như phụ âm đầu theo cùng nhóm vần;
  đây là hỗ trợ kiểu gõ/phiên âm, không phải chính tả tiếng Việt chuẩn.

## Hành vi đã được xác nhận, không nên sửa nhầm

- Undo từng phím: `tiếng` + Backspace → `tiêng`.
- Chữ lạ như `qwer` và `getUs` giữ nguyên thay vì bị ép dấu sớm.
- `khuow` giữ `khuơ`; VNI, VIQR và các câu chuẩn giữ parity với Go trong bộ kiểm thử hiện có.
