use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct TranslationResult {
    pub original: String,
    pub translated: String,
}

/// Dịch text từ tiếng Nhật sang tiếng Việt sử dụng Gemini API
pub async fn translate_japanese_to_vietnamese(
    japanese_text: &str,
    gemini_api_key: &str,
) -> Result<String, String> {
    let client = reqwest::Client::new();
    
    let prompt = format!(
        "Dịch đoạn văn tiếng Nhật sau sang tiếng Việt tự nhiên, chỉ trả về bản dịch, không giải thích:\n\n{}",
        japanese_text
    );
    
    let request_body = serde_json::json!({
        "contents": [{
            "parts": [{
                "text": prompt
            }]
        }]
    });
    
    let url = format!(
        "https://generativelanguage.googleapis.com/v1/models/gemini-1.5-flash:generateContent?key={}",
        gemini_api_key
    );
    
    let response = client
        .post(&url)
        .json(&request_body)
        .send()
        .await
        .map_err(|e| format!("Failed to call Gemini API: {}", e))?;
    
    if !response.status().is_success() {
        return Err(format!("Gemini API error: {}", response.status()));
    }
    
    let response_json: serde_json::Value = response
        .json()
        .await
        .map_err(|e| format!("Failed to parse response: {}", e))?;
    
    // Parse Gemini response
    let translated_text = response_json
        .get("candidates")
        .and_then(|c| c.get(0))
        .and_then(|c| c.get("content"))
        .and_then(|c| c.get("parts"))
        .and_then(|p| p.get(0))
        .and_then(|p| p.get("text"))
        .and_then(|t| t.as_str())
        .ok_or("Failed to extract translation from response")?;
    
    Ok(translated_text.trim().to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_translation() {
        let api_key = "YOUR_API_KEY";  // Replace with actual key for testing
        let japanese = "こんにちは、元気ですか？";
        
        match translate_japanese_to_vietnamese(japanese, api_key).await {
            Ok(vietnamese) => println!("Translation: {}", vietnamese),
            Err(e) => println!("Error: {}", e),
        }
    }
}
