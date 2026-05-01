mod cli;
mod config;
mod features;
mod gemini;
mod models;

use clap::Parser;
use config::Config;
use features::bush_runner::BashRunner;
use features::feature_trait::Feature;
use gemini::{GeminiClient, GeminiClientError, StdoutResponseHandler};
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
        return Ok(());
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

    let mut final_prompt = user_input.trim().to_string();

    if !cli.file.is_empty() {
        println!(); // Пустий рядок для красивого виводу
        for file_path in &cli.file {
            // Спроба прочитати файл як текст
            match std::fs::read_to_string(file_path) {
                Ok(content) => {
                    // Витягуємо лише ім'я файлу (без повного шляху) для красивого форматування
                    let file_name = std::path::Path::new(file_path)
                        .file_name()
                        .unwrap_or_default()
                        .to_string_lossy();

                    // Доклеюємо вміст до нашого запиту
                    final_prompt.push_str(&format!(
                        "\n\n--- Вміст файлу `{}` ---\n```text\n{}\n```",
                        file_name, content
                    ));

                    println!("📎 Долучено файл: {}", file_path);
                }
                Err(e) => {
                    // Якщо файл не знайдено або це бінарник, краще одразу зупинити виконання
                    eprintln!("❌ Помилка читання файлу '{}': {}", file_path, e);
                    std::process::exit(1);
                }
            }
        }
    }

    println!("\nGemini:");
    let client = GeminiClient::new(api_key);
    let mut response_handler = StdoutResponseHandler;
    let full_response = match client
        .stream_generate(&cli.model, &final_prompt, &mut response_handler)
        .await
    {
        Ok(response) => response,
        Err(GeminiClientError::ApiStatus(status)) => {
            println!("Помилка API: {}", status);
            return Ok(());
        }
        Err(err) => {
            let err: Box<dyn std::error::Error> = Box::new(err);
            return Err(err);
        }
    };

    println!("\n\n[Кінець відповіді]");
    let active_features: Vec<Box<dyn Feature>> = vec![Box::new(BashRunner {})];
    for feature in active_features {
        if let Err(e) = feature.execute(&full_response) {
            eprintln!("Помилка виконання фічі: {}", e);
        }
    }
    Ok(())
}
