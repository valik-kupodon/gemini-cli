mod models;

use std::env;
use std::io::{self, Write};

use models::{Content, GeminiRequest, GeminiResponse, Part};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenvy::dotenv().ok();
    // 1. Отримуємо API ключ із змінних оточення
    let api_key = env::var("GEMINI_API_KEY").expect("Встановіть змінну GEMINI_API_KEY");

    // 2. Читаємо ввід користувача
    print!("Запитай щось: ");
    io::stdout().flush()?;
    let mut user_input = String::new();
    io::stdin().read_line(&mut user_input)?;

    // 3. Формуємо запит
    let url = format!(
        "https://generativelanguage.googleapis.com/v1beta/models/gemini-2.5-flash-lite:generateContent?key={}",
        api_key
    );

    let body = GeminiRequest {
        contents: vec![Content {
            parts: vec![Part { text: user_input }],
        }],
    };

    // 4. Відправляємо запит
    // 4. Відправляємо запит
    let client = reqwest::Client::new();
    let response_raw = client.post(url).json(&body).send().await?;

    // Отримуємо текст відповіді, щоб побачити помилку, якщо вона є
    let text = response_raw.text().await?;

    // Спробуємо розпарсити
    match serde_json::from_str::<GeminiResponse>(&text) {
        Ok(res) => {
            if let Some(candidate) = res.candidates.first() {
                if let Some(part) = candidate.content.parts.first() {
                    println!("\nGemini:\n{}", part.text);
                }
            } else {
                println!("\nВідповідь порожня (можливо, спрацював фільтр безпеки).");
            }
        }
        Err(_) => {
            // Якщо не вдалося розпарсити як успіх — виводимо сирий JSON помилки
            println!("\nПомилка від API або невірний формат:");
            println!("{}", text);
        }
    }

    Ok(())
}
