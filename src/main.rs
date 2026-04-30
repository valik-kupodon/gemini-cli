mod cli;
mod config;
mod features;
mod models;

use clap::Parser;
use config::Config;
use features::bush_runner::BashRunner;
use features::feature_trait::Feature;
use futures_util::StreamExt;
use models::{Content, GeminiRequest, GeminiResponse, Part};
use std::io::{self, Write};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = cli::Cli::parse();
    let config = Config::new();
    if let Some(new_key) = cli.set_key {
        match config.save_api_key(&new_key) {
            Ok(path) => println!("✅ Ключ успішно збережено у: {}", path.display()),
            Err(e) => eprintln!("❌ {}", e),
        }
        return Ok(()); // Виходимо після збереження
    }
    let api_key = match config.get_api_key() {
        Some(key) => key,
        None => {
            eprintln!("❌ Ключ API не знайдено!");
            eprintln!("Використай команду: gemini-cli --set-key \"ТВІЙ_КЛЮЧ\"");
            return Ok(());
        }
    };
    let user_input = match cli.prompt {
        Some(prompt) => prompt,
        None => {
            print!("Запитай щось: ");
            io::stdout().flush()?;
            let mut input = String::new();
            io::stdin().read_line(&mut input)?;
            input
        }
    };

    let url = format!(
        "https://generativelanguage.googleapis.com/v1beta/models/{}:streamGenerateContent?alt=sse&key={}",
        cli.model, api_key
    );

    let body = GeminiRequest {
        contents: vec![Content {
            parts: vec![Part { text: user_input }],
        }],
    };

    let client = reqwest::Client::new();
    let res = client.post(url).json(&body).send().await?;

    if !res.status().is_success() {
        println!("Помилка API: {}", res.status());
        return Ok(());
    }

    let mut response_stream = res.bytes_stream();
    let mut buffer = String::new();
    let mut full_response = String::new();

    println!("\nGemini:");

    while let Some(item) = response_stream.next().await {
        let bytes = item?;
        buffer.push_str(&String::from_utf8_lossy(&bytes));

        // Обробляємо буфер рядок за рядком
        while let Some(line_end) = buffer.find('\n') {
            let line = buffer[..line_end].to_string();
            buffer.drain(..line_end + 1);

            if line.starts_with("data: ") {
                let json_str = &line[6..];

                if json_str.trim() == "[" || json_str.trim() == "]" || json_str.is_empty() {
                    continue;
                }

                match serde_json::from_str::<GeminiResponse>(json_str) {
                    Ok(gemini_res) => {
                        if let Some(candidate) = gemini_res.candidates.first() {
                            if let Some(part) = candidate.content.parts.first() {
                                print!("{}", part.text);
                                io::stdout().flush()?;
                                full_response.push_str(&part.text);
                            }
                        }
                    }
                    Err(_) => {
                        eprintln!("Помилка парсингу: {}", json_str);
                    }
                }
            }
        }
    }

    println!("\n\n[Кінець відповіді]");
    let active_features: Vec<Box<dyn Feature>> = vec![Box::new(BashRunner {})];
    for feature in active_features {
        if let Err(e) = feature.execute(&full_response) {
            eprintln!("Помилка виконання фічі: {}", e);
        }
    }
    Ok(())
}
