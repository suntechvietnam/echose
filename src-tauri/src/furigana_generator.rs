use std::process::Command;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct FuriganaToken {
    pub surface: String,      // Kanji gốc
    pub reading: String,      // Hiragana reading
    pub is_kanji: bool,       // Có phải Kanji không
}

/// Chuyển text tiếng Nhật sang Furigana sử dụng MeCab
pub fn generate_furigana(japanese_text: &str) -> Result<Vec<FuriganaToken>, String> {
    let mecabrc_path = std::env::current_dir()
        .map_err(|e| format!("Cannot get current dir: {}", e))?
        .join("mecabrc");
    
    // MeCab format: %m = surface, %f[7] = reading (katakana), %f[8] = reading (hiragana)
    // We use custom format to get: surface\treading
    let output = Command::new("mecab")
        .env("MECABRC", &mecabrc_path)
        .arg("-F")
        .arg("%m\\t%f[7]\\n")  // Format: surface\treading (katakana)
        .arg("-E")
        .arg("EOS\\n")
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .and_then(|mut child| {
            use std::io::Write;
            if let Some(mut stdin) = child.stdin.take() {
                stdin.write_all(japanese_text.as_bytes())?;
            }
            child.wait_with_output()
        })
        .map_err(|e| format!("Failed to run MeCab: {}", e))?;
    
    if !output.status.success() {
        return Err(format!("MeCab error: {}", String::from_utf8_lossy(&output.stderr)));
    }
    
    let result = String::from_utf8_lossy(&output.stdout);
    let mut tokens = Vec::new();
    
    for line in result.lines() {
        if line.trim().is_empty() || line == "EOS" {
            continue;
        }
        
        let parts: Vec<&str> = line.split('\t').collect();
        if parts.len() < 2 {
            continue;
        }
        
        let surface = parts[0].to_string();
        let reading_katakana = parts[1].to_string();
        
        // Convert Katakana to Hiragana
        let reading = katakana_to_hiragana(&reading_katakana);
        
        // Kiểm tra xem có phải Kanji không (nếu surface != reading)
        let is_kanji = surface != reading && !reading.is_empty() && reading != "*";
        
        tokens.push(FuriganaToken {
            surface,
            reading: if is_kanji { reading } else { String::new() },
            is_kanji,
        });
    }
    
    Ok(tokens)
}

/// Convert Katakana to Hiragana
fn katakana_to_hiragana(katakana: &str) -> String {
    katakana.chars().map(|c| {
        if c >= 'ァ' && c <= 'ヶ' {
            // Katakana range: 0x30A1-0x30F6
            // Hiragana range: 0x3041-0x3096
            // Offset: 0x60
            char::from_u32(c as u32 - 0x60).unwrap_or(c)
        } else {
            c
        }
    }).collect()
}

/// Chuyển Furigana tokens thành ASS ruby text format
pub fn tokens_to_ass_ruby(tokens: &[FuriganaToken]) -> String {
    let mut result = String::new();
    
    for token in tokens {
        if token.is_kanji && !token.reading.is_empty() {
            // Format: {\\rRuby}base|ruby
            result.push_str(&format!("{{\\\\rRuby}}{}|{}", token.surface, token.reading));
        } else {
            result.push_str(&token.surface);
        }
    }
    
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_furigana_generation() {
        let text = "日本語";
        let tokens = generate_furigana(text).unwrap();
        println!("Tokens: {:?}", tokens);
        
        let ruby_text = tokens_to_ass_ruby(&tokens);
        println!("Ruby text: {}", ruby_text);
    }
}
