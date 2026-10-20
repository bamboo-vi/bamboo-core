# Các ca Rust chưa hoàn hảo (riêng bamboo-rust)

> **Lưu ý cập nhật 21/10/2026:** Đây là ghi chép của phép đo cũ, không phải kết luận cho mã hiện tại. `VieetjNam` chỉ trả chữ thô khi `auto_correct=true` của core; với `auto_correct=false` trả `ViệtNam`, và addon Ngó Sen đặt core ở chế độ này. Giới hạn 16 ký tự và lệch `duow` bên dưới đã được sửa. Xem `../bench-fcitx5/comparison-2026-10-21.md` để có phép đo hiện tại.

> Nguồn số liệu: bench fcitx5 hợp nhất (`bench-fcitx5/report.md` + phụ lục 95 ca lệch).
> Quy ước: không lấy bên nào làm chuẩn tuyệt đối; "chưa hoàn hảo" ở đây nghĩa là
> hành vi Rust gây giật, mất chữ, hoặc thua kỳ vọng gõ thông thường — đều có bước tái hiện.

## 1. Tự động gỡ về chữ thô khi gặp biên từ dính liền (nặng nhất)

- Tái hiện: gõ `VieetjNam` (camelCase, không dấu cách), chế độ Vietnamese.
- Từng bước: `V-i-e-e-t-j` → `Việt` đúng; thêm `N` → rớt về `VieetjN`; thêm `am` → `VieetjNam` đứng hình.
- Kỳ vọng: giữ `Việt` và bắt đầu âm tiết mới (`ViệtNam` như Go).
- Thực tế: mất luôn dấu `j` đã ăn, các phím sau đi đường thô.
- Nguyên nhân (mã): `Engine::apply_auto_correct` (`src/engine/mod.rs`) — khi cụm sau biến đổi
  không còn là tiền tố tiếng Việt hợp lệ thì bẻ gãy toàn bộ biến đổi về chữ thô
  (`break_composition_slice`) và bật cờ `english_bypass` (mặc định `auto_correct: true`).
  Cơ chế này đúng khi người dùng gõ sai chính tả thật (`qwer` giữ nguyên là nhờ nó),
  nhưng sai khi chỉ là biên từ dính liền: nó gỡ cả dấu của âm tiết trước vốn đã đúng.
- Đối chứng: `Vieetj Nam` (có dấu cách) Rust đúng `Việt` rồi `Nam` — lỗi chỉ xảy ra khi dính liền.
- Hướng xử lý: trước khi bẻ về thô, chốt (commit ngầm) phần đầu đã hoàn chỉnh (`Việt`)
  rồi mới bắt đầu âm tiết mới ở chế độ thô; hoặc chỉ gỡ biến đổi của âm tiết hiện tại,
  không gỡ ngược âm tiết trước.

## 2. Tràn bộ đệm 16 ký tự cắt preedit

- Tái hiện: gõ `getUserNameByIdentifier` (23 ký tự), chế độ English.
- Từng bước: tới ký tự 16 preedit đầy đủ; ký tự 17 (`n`) preedit còn mỗi `n`.
- Kỳ vọng: preedit không giật (giữ cả cụm hoặc commit ngầm phần đầu).
- Thực tế: `MAX_ACTIVE_TRANS = 16` (`src/engine/state.rs`), tràn là cắt.
- Hướng xử lý: commit ngầm phần đầu hoặc trượt cửa sổ thay vì cắt phăng.

## 3. Dạng dở `duơ` thay vì `dươ` (mức đánh bóng)

- Tái hiện: gõ `duow` → Rust `duơ`, Go `dươ`; thêm `c`/`j` cả hai hội tụ `dược`.
- Kỳ vọng theo thói quen gõ: `dươ` (lối tắt `uow`→`ươ`).
- Nguyên nhân: Rust chỉ lan mũ thứ hai khi âm tiết thiếu nó sẽ không hoàn chỉnh
  (`src/orthography/syllable.rs` — chú thích trong mã nêu rõ), mà `duơ` tạm coi là
  hoàn chỉnh nên dừng.
- Ranh giới không được phạm: ca `khuow` hai bên đang đồng thuận giữ `khuơ` —
  sửa kiểu lan mũ sớm bừa bãi sẽ phá ca này. Chỉ nên ưu tiên mục tiêu `u` trong đúng
  mẫu `uow`.

## 4. Ký tự xuống dòng giữ trong preedit (chờ chốt hợp đồng)

- Tái hiện: cuối mỗi câu có `\n`, Rust giữ `\n` trong preedit, Go trả rỗng.
- Chưa kết luận đúng sai: tùy frontend fcitx5 coi Enter là commit hay ký tự.
  Cần chốt hợp đồng rồi khóa bằng một ca kiểm thử.

## 5. Điểm cộng: giữ nguyên, đừng sửa nhầm

- Undo từng phím (`tiếng`+BS → `tiêng`) khớp thói quen phổ biến; Go ra `tiến` mới là lạ.
- Giữ chữ lạ (`qwer`, `getUs`) trong khi Go ép dấu sớm (`qwẻ`, `getÚ`).
- `khuow` giữ `khuơ` đúng; VNI/VIQR và câu chuẩn khớp Go 100%.
