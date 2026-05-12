# Detect Human By Cloud (Google Gemini API)

Detect người trong ảnh sử dụng Google Gemini API bằng Rust.

Hỗ trợ:
- Ảnh local
- URL ảnh online
- Phân tích số lượng người
- Độ tin cậy
- Mô tả ngắn bằng tiếng Việt

---

# Demo

```bash
Running `target/release/detect_human_by_cloud --image resource/image.jpg --api-key ********`

📁 Đọc ảnh local: resource/image.jpg
🚀 Đang gửi ảnh lên Google Gemini...

=======================================================
         KẾT QUẢ PHÂN TÍCH ẢNH
=======================================================
📸 Ảnh   : resource/image.jpg
🤖 Model : gemini-2.5-flash
-------------------------------------------------------
✅ CÓ NGƯỜI trong ảnh
👥 Số lượng: 8 người
🟢 Độ tin cậy: HIGH
📝 Mô tả : Một nhóm tám người đa dạng, tươi cười và đứng cùng nhau trong một không gian văn phòng.
-------------------------------------------------------
📄 Phản hồi gốc từ Gemini:
PERSON_DETECTED: YES
COUNT: 8
CONFIDENCE: HIGH
DESCRIPTION: Một nhóm tám người đa dạng, tươi cười và đứng cùng nhau trong một không gian văn phòng.
=======================================================