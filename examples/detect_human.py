"""
Detect người trong ảnh sử dụng Google Gemini API (FREE)
Package mới: google-genai (thay thế google-generativeai đã deprecated)

Cài đặt:
    pip install google-genai

Lấy API key miễn phí tại:
    https://aistudio.google.com/apikey

Cách dùng:
    python detect_person_gemini.py --image path/to/image.jpg
    python detect_person_gemini.py --image https://example.com/image.jpg
    python detect_person_gemini.py  (sẽ hỏi đường dẫn ảnh)
"""

import os
import sys
import base64
import argparse
from pathlib import Path


def encode_image_to_base64(image_path: str) -> str:
    with open(image_path, "rb") as f:
        return base64.b64encode(f.read()).decode("utf-8")


def get_mime_type(image_path: str) -> str:
    ext = Path(image_path).suffix.lower()
    return {
        ".jpg": "image/jpeg",
        ".jpeg": "image/jpeg",
        ".png": "image/png",
        ".gif": "image/gif",
        ".webp": "image/webp",
    }.get(ext, "image/jpeg")


def download_image_as_base64(url: str) -> tuple[str, str]:
    import urllib.request, urllib.error
    try:
        req = urllib.request.Request(url, headers={"User-Agent": "Mozilla/5.0"})
        with urllib.request.urlopen(req, timeout=15) as response:
            data = response.read()
            content_type = response.headers.get("Content-Type", "image/jpeg").split(";")[0].strip()
            return base64.b64encode(data).decode("utf-8"), content_type
    except urllib.error.URLError as e:
        raise ConnectionError(f"Không tải được ảnh từ URL: {e}")


def detect_person(image_source: str, api_key: str) -> dict:
    try:
        from google import genai
        from google.genai import types
    except ImportError:
        print("❌ Chưa cài thư viện. Chạy: pip install google-genai")
        sys.exit(1)

    client = genai.Client(api_key=api_key)

    is_url = image_source.startswith("http://") or image_source.startswith("https://")

    if is_url:
        print(f"🌐 Đang tải ảnh từ URL...")
        base64_data, mime_type = download_image_as_base64(image_source)
    else:
        if not Path(image_source).exists():
            raise FileNotFoundError(f"Không tìm thấy file: {image_source}")
        print(f"📁 Đọc ảnh local: {image_source}")
        base64_data = encode_image_to_base64(image_source)
        mime_type = get_mime_type(image_source)

    prompt = """Hãy phân tích ảnh này và trả lời theo đúng định dạng sau (không thêm gì khác):
PERSON_DETECTED: [YES hoặc NO]
COUNT: [số nguyên, 0 nếu không có người]
CONFIDENCE: [HIGH hoặc MEDIUM hoặc LOW]
DESCRIPTION: [mô tả ngắn gọn bằng tiếng Việt]

Quy tắc:
- YES nếu có ít nhất 1 người thật trong ảnh (kể cả bị che khuất một phần)
- NO nếu không có người (ảnh phong cảnh, đồ vật, động vật, tranh vẽ...)
- Không tính hoạt hình, robot, tượng, tranh vẽ là người"""

    print("🚀 Đang gửi ảnh lên Google Gemini...")

    response = client.models.generate_content(
        model="gemini-2.5-flash",
        contents=[
            types.Part.from_bytes(
                data=base64.b64decode(base64_data),
                mime_type=mime_type,
            ),
            prompt,
        ],
    )

    raw_text = response.text.strip()
    result = parse_response(raw_text)
    result["raw_response"] = raw_text
    result["model"] = "gemini-2.5-flash"
    return result


def parse_response(text: str) -> dict:
    result = {"has_person": False, "count": 0, "confidence": "LOW", "description": ""}
    for line in text.split("\n"):
        upper = line.upper()
        if "PERSON_DETECTED:" in upper:
            result["has_person"] = "YES" in upper
        elif "COUNT:" in upper:
            try:
                digits = "".join(filter(str.isdigit, line.split(":", 1)[1]))
                result["count"] = int(digits) if digits else 0
            except (ValueError, IndexError):
                pass
        elif "CONFIDENCE:" in upper:
            for level in ["HIGH", "MEDIUM", "LOW"]:
                if level in upper:
                    result["confidence"] = level
                    break
        elif "DESCRIPTION:" in upper:
            result["description"] = line.split(":", 1)[1].strip()
    return result


def print_result(result: dict, image_source: str):
    print("\n" + "=" * 55)
    print("         KẾT QUẢ PHÂN TÍCH ẢNH")
    print("=" * 55)
    print(f"📸 Ảnh   : {image_source}")
    print(f"🤖 Model : {result.get('model', 'N/A')}")
    print("-" * 55)

    if result["has_person"]:
        print(f"✅ CÓ NGƯỜI trong ảnh")
        if result["count"] > 0:
            print(f"👥 Số lượng: {result['count']} người")
    else:
        print(f"❌ KHÔNG CÓ NGƯỜI trong ảnh")

    emoji = {"HIGH": "🟢", "MEDIUM": "🟡", "LOW": "🔴"}.get(result["confidence"], "⚪")
    print(f"{emoji} Độ tin cậy: {result['confidence']}")

    if result["description"]:
        print(f"📝 Mô tả : {result['description']}")

    print("-" * 55)
    print("📄 Phản hồi gốc từ Gemini:")
    print(result.get("raw_response", ""))
    print("=" * 55)


def main():
    parser = argparse.ArgumentParser(description="Detect người trong ảnh dùng Google Gemini API")
    parser.add_argument("--image", type=str, help="Đường dẫn file ảnh hoặc URL")
    parser.add_argument("--api-key", type=str, help="Google Gemini API key")
    args = parser.parse_args()

    api_key = args.api_key or os.environ.get("GEMINI_API_KEY")
    if not api_key:
        print("⚠️  Chưa có Gemini API key!")
        print("   Lấy key miễn phí tại: https://aistudio.google.com/apikey")
        api_key = input("\nNhập API key: ").strip()
        if not api_key:
            print("❌ Không có API key, thoát.")
            sys.exit(1)

    image_source = args.image
    if not image_source:
        print("\nNhập đường dẫn ảnh (local path hoặc URL):")
        image_source = input(">>> ").strip()
        if not image_source:
            print("❌ Không có ảnh đầu vào, thoát.")
            sys.exit(1)

    try:
        result = detect_person(image_source, api_key)
        print_result(result, image_source)
    except FileNotFoundError as e:
        print(f"❌ Lỗi file: {e}")
        sys.exit(1)
    except ConnectionError as e:
        print(f"❌ Lỗi mạng: {e}")
        sys.exit(1)
    except Exception as e:
        print(f"❌ Lỗi: {e}")
        sys.exit(1)


if __name__ == "__main__":
    main()