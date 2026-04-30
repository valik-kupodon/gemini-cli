use super::feature_trait::Feature;
use regex::Regex;
use std::io::{self, Write};
use std::process::Command as SysCommand;

pub struct BashRunner;

impl BashRunner {
    fn run_command(&self, cmd: &str) {
        println!("\n▶ Виконую: \n{}", cmd);
        let status = SysCommand::new("sh").arg("-c").arg(cmd).status();

        match status {
            Ok(s) if s.success() => println!("✅ Виконано успішно."),
            Ok(s) => eprintln!("❌ Процес завершився з помилкою: {}", s),
            Err(e) => eprintln!("❌ Не вдалося запустити команду: {}", e),
        }
    }
}

impl Feature for BashRunner {
    fn execute(&self, full_response: &str) -> Result<(), Box<dyn std::error::Error>> {
        let re = Regex::new(r"```(?:bash|sh)\n([\s\S]*?)```").unwrap();
        let mut commands = Vec::new();

        for cap in re.captures_iter(full_response) {
            if let Some(cmd) = cap.get(1) {
                commands.push(cmd.as_str().trim().to_string());
            }
        }

        if commands.is_empty() {
            return Ok(()); // Якщо команд немає, фіча просто мовчки завершує роботу
        }

        println!("\n💻 Знайдено bash-команди:");
        for (i, cmd) in commands.iter().enumerate() {
            println!(" [{}] {}", i + 1, cmd.lines().next().unwrap_or(""));
            if cmd.lines().count() > 1 {
                println!("     (і ще {} рядків...)", cmd.lines().count() - 1);
            }
        }

        print!(
            "\nБажаєте виконати команду? (введіть номер, 'all' для всіх, або Enter для відміни): "
        );
        io::stdout().flush()?;

        let mut choice = String::new();
        io::stdin().read_line(&mut choice)?;
        let choice = choice.trim();

        if !choice.is_empty() {
            if choice == "all" {
                for cmd in &commands {
                    self.run_command(cmd);
                }
            } else if let Ok(idx) = choice.parse::<usize>() {
                if idx > 0 && idx <= commands.len() {
                    self.run_command(&commands[idx - 1]);
                } else {
                    println!("❌ Невірний номер команди.");
                }
            } else {
                println!("❌ Відмінено або невідомий ввід.");
            }
        } else {
            println!("Відмінено.");
        }

        Ok(())
    }
}
