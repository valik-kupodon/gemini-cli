use super::feature_trait::Feature;
use regex::Regex;
use std::io::{self, Write};
use std::process::Command as SysCommand;

pub struct BashRunner;

impl BashRunner {
    fn extract_commands(full_response: &str) -> Vec<String> {
        let re = Regex::new(r"```(?:bash|sh)\n([\s\S]*?)```").unwrap();
        re.captures_iter(full_response)
            .filter_map(|cap| cap.get(1).map(|cmd| cmd.as_str().trim().to_string()))
            .collect()
    }

    fn format_command_preview(index: usize, cmd: &str) -> String {
        let mut lines = cmd.lines();
        let Some(first_line) = lines.next() else {
            return format!(" [{index}]");
        };

        let mut preview = format!(" [{index}] {first_line}");
        for line in lines {
            preview.push_str(&format!("\n     {line}"));
        }

        preview
    }

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
        let commands = Self::extract_commands(full_response);

        if commands.is_empty() {
            return Ok(()); // Якщо команд немає, фіча просто мовчки завершує роботу
        }

        println!("\n💻 Знайдено bash-команди:");
        for (i, cmd) in commands.iter().enumerate() {
            println!("{}", Self::format_command_preview(i + 1, cmd));
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

#[cfg(test)]
mod tests {
    use super::BashRunner;

    #[test]
    fn extracts_bash_and_sh_blocks() {
        let response = r#"
Here you go:
```bash
echo "hello"
pwd
```

```text
ignore me
```

```sh
ls -la
```
"#;

        let commands = BashRunner::extract_commands(response);

        assert_eq!(commands, vec!["echo \"hello\"\npwd", "ls -la"]);
    }

    #[test]
    fn returns_empty_when_no_shell_blocks_exist() {
        let response = "No commands here";

        let commands = BashRunner::extract_commands(response);

        assert!(commands.is_empty());
    }

    #[test]
    fn formats_full_multiline_preview() {
        let preview = BashRunner::format_command_preview(7, "sudo dnf check-update\nsudo dnf upgrade");

        assert_eq!(preview, " [7] sudo dnf check-update\n     sudo dnf upgrade");
    }
}
