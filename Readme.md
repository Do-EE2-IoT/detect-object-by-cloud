// detect_person_gemini.rs
//
// Detect người trong ảnh sử dụng Google Gemini API (FREE)
// Rust version
//
// =======================
// Cài dependency:
// =======================
//
// cargo add reqwest --features blocking,json,multipart,rustls-tls
// cargo add serde --features derive
// cargo add serde_json
// cargo add clap --features derive
// cargo add base64
//
// =======================
// Chạy:
// =======================
//
// export GEMINI_API_KEY="your_key"
//
// cargo run -- --image image.jpg
// cargo run -- --image https://example.com/image.jpg
//
// Hoặc:
//
// cargo run
//
<!-- 
Running `target/release/detect_human_by_cloud --image resource/image.jpg --api-key  ********`
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
DESCRIPTION: Một nhóm tám người đa dạng, tươi cười và đứng cùng nhau trong một không gian văn phòng -->