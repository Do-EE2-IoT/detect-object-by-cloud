use std::{env, fs};

use base64::{engine::general_purpose, Engine as _};
use reqwest::blocking::Client;
use serde_json::json;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let api_key =
        env::var("GEMINI_API_KEY").expect("Missing GEMINI_API_KEY environment variable");

    let image_path = "resource/image.jpg";

    println!("📁 Đọc ảnh: {}", image_path);

    let image_data = fs::read(image_path)?;
    let image_base64 = general_purpose::STANDARD.encode(image_data);

    let prompt = r#"Hãy phân tích ảnh này và trả lời đúng format:

PERSON_DETECTED: YES hoặc NO
COUNT: số người
CONFIDENCE: HIGH/MEDIUM/LOW
DESCRIPTION: mô tả ngắn tiếng Việt"#;

    let body = json!({
        "contents": [{
            "parts": [
                {
                    "inline_data": {
                        "mime_type": "image/jpeg",
                        "data": image_base64
                    }
                },
                {
                    "text": prompt
                }
            ]
        }]
    });

    println!("🚀 Đang gửi ảnh lên Gemini...");

    let url = format!(
        "https://generativelanguage.googleapis.com/v1beta/models/gemini-2.5-flash:generateContent?key={}",
        api_key
    );

    let response = Client::new()
        .post(url)
        .json(&body)
        .send()?
        .text()?;

    let json: serde_json::Value = serde_json::from_str(&response)?;

    let text = json["candidates"][0]["content"]["parts"][0]["text"]
        .as_str()
        .unwrap_or("No response");

    println!("\n{}", text);

    Ok(())
}