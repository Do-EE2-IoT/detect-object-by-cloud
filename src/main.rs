
use std::{
    env,
    fs,
    io::{self, Write},
    path::Path,
    process,
};

use base64::{engine::general_purpose, Engine as _};
use clap::Parser;
use reqwest::blocking::Client;
use serde_json::{json, Value};

#[derive(Parser, Debug)]
#[command(author, version, about)]
struct Args {
    /// Đường dẫn ảnh hoặc URL
    #[arg(long)]
    image: Option<String>,

    /// Gemini API key
    #[arg(long)]
    api_key: Option<String>,
}

#[derive(Debug)]
struct DetectResult {
    has_person: bool,
    count: i32,
    confidence: String,
    description: String,
    raw_response: String,
    model: String,
}

fn get_mime_type(path: &str) -> &'static str {
    let ext = Path::new(path)
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_lowercase();

    match ext.as_str() {
        "jpg" | "jpeg" => "image/jpeg",
        "png" => "image/png",
        "gif" => "image/gif",
        "webp" => "image/webp",
        _ => "image/jpeg",
    }
}

fn encode_image_to_base64(path: &str) -> Result<String, Box<dyn std::error::Error>> {
    let data = fs::read(path)?;
    Ok(general_purpose::STANDARD.encode(data))
}

fn download_image_as_base64(
    url: &str,
) -> Result<(String, String), Box<dyn std::error::Error>> {
    let client = Client::builder()
        .timeout(std::time::Duration::from_secs(15))
        .build()?;

    let response = client
        .get(url)
        .header("User-Agent", "Mozilla/5.0")
        .send()?;

    if !response.status().is_success() {
        return Err(format!("HTTP Error: {}", response.status()).into());
    }

    let content_type = response
        .headers()
        .get("content-type")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("image/jpeg")
        .split(';')
        .next()
        .unwrap_or("image/jpeg")
        .trim()
        .to_string();

    let bytes = response.bytes()?;

    Ok((
        general_purpose::STANDARD.encode(bytes),
        content_type,
    ))
}

fn parse_response(text: &str) -> DetectResult {
    let mut result = DetectResult {
        has_person: false,
        count: 0,
        confidence: "LOW".to_string(),
        description: "".to_string(),
        raw_response: text.to_string(),
        model: "gemini-2.5-flash".to_string(),
    };

    for line in text.lines() {
        let upper = line.to_uppercase();

        if upper.contains("PERSON_DETECTED:") {
            result.has_person = upper.contains("YES");
        } else if upper.contains("COUNT:") {
            if let Some(part) = line.split(':').nth(1) {
                let digits: String = part.chars().filter(|c| c.is_ascii_digit()).collect();

                if !digits.is_empty() {
                    result.count = digits.parse().unwrap_or(0);
                }
            }
        } else if upper.contains("CONFIDENCE:") {
            if upper.contains("HIGH") {
                result.confidence = "HIGH".to_string();
            } else if upper.contains("MEDIUM") {
                result.confidence = "MEDIUM".to_string();
            } else {
                result.confidence = "LOW".to_string();
            }
        } else if upper.contains("DESCRIPTION:") {
            if let Some(part) = line.split(':').nth(1) {
                result.description = part.trim().to_string();
            }
        }
    }

    result
}

fn detect_person(
    image_source: &str,
    api_key: &str,
) -> Result<DetectResult, Box<dyn std::error::Error>> {
    let is_url =
        image_source.starts_with("http://") || image_source.starts_with("https://");

    let (base64_data, mime_type) = if is_url {
        println!("🌐 Đang tải ảnh từ URL...");
        download_image_as_base64(image_source)?
    } else {
        if !Path::new(image_source).exists() {
            return Err(format!("Không tìm thấy file: {}", image_source).into());
        }

        println!("📁 Đọc ảnh local: {}", image_source);

        (
            encode_image_to_base64(image_source)?,
            get_mime_type(image_source).to_string(),
        )
    };

    let prompt = r#"Hãy phân tích ảnh này và trả lời theo đúng định dạng sau (không thêm gì khác):
PERSON_DETECTED: [YES hoặc NO]
COUNT: [số nguyên, 0 nếu không có người]
CONFIDENCE: [HIGH hoặc MEDIUM hoặc LOW]
DESCRIPTION: [mô tả ngắn gọn bằng tiếng Việt]

Quy tắc:
- YES nếu có ít nhất 1 người thật trong ảnh (kể cả bị che khuất một phần)
- NO nếu không có người (ảnh phong cảnh, đồ vật, động vật, tranh vẽ...)
- Không tính hoạt hình, robot, tượng, tranh vẽ là người"#;

    println!("🚀 Đang gửi ảnh lên Google Gemini...");

    let url = format!(
        "https://generativelanguage.googleapis.com/v1beta/models/gemini-2.5-flash:generateContent?key={}",
        api_key
    );

    let body = json!({
        "contents": [
            {
                "parts": [
                    {
                        "inline_data": {
                            "mime_type": mime_type,
                            "data": base64_data
                        }
                    },
                    {
                        "text": prompt
                    }
                ]
            }
        ]
    });

    let client = Client::new();

    let response = client
        .post(url)
        .json(&body)
        .send()?;

    let status = response.status();
    let text = response.text()?;

    if !status.is_success() {
        return Err(format!("Gemini API Error: {}", text).into());
    }

    let json: Value = serde_json::from_str(&text)?;

    let raw_text = json["candidates"][0]["content"]["parts"][0]["text"]
        .as_str()
        .unwrap_or("Không có phản hồi")
        .trim()
        .to_string();

    let mut result = parse_response(&raw_text);
    result.raw_response = raw_text;

    Ok(result)
}

fn print_result(result: &DetectResult, image_source: &str) {
    println!("\n{}", "=".repeat(55));
    println!("         KẾT QUẢ PHÂN TÍCH ẢNH");
    println!("{}", "=".repeat(55));

    println!("📸 Ảnh   : {}", image_source);
    println!("🤖 Model : {}", result.model);

    println!("{}", "-".repeat(55));

    if result.has_person {
        println!("✅ CÓ NGƯỜI trong ảnh");

        if result.count > 0 {
            println!("👥 Số lượng: {} người", result.count);
        }
    } else {
        println!("❌ KHÔNG CÓ NGƯỜI trong ảnh");
    }

    let emoji = match result.confidence.as_str() {
        "HIGH" => "🟢",
        "MEDIUM" => "🟡",
        "LOW" => "🔴",
        _ => "⚪",
    };

    println!("{} Độ tin cậy: {}", emoji, result.confidence);

    if !result.description.is_empty() {
        println!("📝 Mô tả : {}", result.description);
    }

    println!("{}", "-".repeat(55));

    println!("📄 Phản hồi gốc từ Gemini:");
    println!("{}", result.raw_response);

    println!("{}", "=".repeat(55));
}

fn main() {
    let args = Args::parse();

    let api_key = args
        .api_key
        .or_else(|| env::var("GEMINI_API_KEY").ok())
        .unwrap_or_else(|| {
            println!("⚠️  Chưa có Gemini API key!");
            println!("   Lấy key miễn phí tại: https://aistudio.google.com/apikey");

            print!("\nNhập API key: ");
            io::stdout().flush().unwrap();

            let mut input = String::new();
            io::stdin().read_line(&mut input).unwrap();

            let key = input.trim().to_string();

            if key.is_empty() {
                println!("❌ Không có API key, thoát.");
                process::exit(1);
            }

            key
        });

    let image_source = args.image.unwrap_or_else(|| {
        println!("\nNhập đường dẫn ảnh (local path hoặc URL):");

        print!(">>> ");
        io::stdout().flush().unwrap();

        let mut input = String::new();
        io::stdin().read_line(&mut input).unwrap();

        let path = input.trim().to_string();

        if path.is_empty() {
            println!("❌ Không có ảnh đầu vào, thoát.");
            process::exit(1);
        }

        path
    });

    match detect_person(&image_source, &api_key) {
        Ok(result) => {
            print_result(&result, &image_source);
        }
        Err(e) => {
            eprintln!("❌ Lỗi: {}", e);
            process::exit(1);
        }
    }
}