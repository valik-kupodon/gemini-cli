use super::feature_trait::Feature;
use regex::Regex;
use std::io::{self, Write};
use std::process::Command as SysCommand;

pub struct BashRunner;

enum CommandSelection {
    Selected(Vec<usize>),
    All,
    Cancel,
}

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

    fn parse_selection(choice: &str, command_count: usize) -> Result<CommandSelection, String> {
        let trimmed = choice.trim();
        if trimmed.is_empty() {
            return Ok(CommandSelection::Cancel);
        }

        if trimmed.eq_ignore_ascii_case("all") {
            return Ok(CommandSelection::All);
        }

        let mut selected = Vec::new();

        for part in trimmed.split(',') {
            let raw_index = part.trim();
            if raw_index.is_empty() {
                return Err("Порожній елемент у списку номерів.".to_string());
            }

            let index = raw_index
                .parse::<usize>()
                .map_err(|_| format!("Невірний номер команди: {raw_index}"))?;

            if index == 0 || index > command_count {
                return Err(format!("Невірний номер команди: {index}"));
            }

            if !selected.contains(&index) {
                selected.push(index);
            }
        }

        Ok(CommandSelection::Selected(selected))
    }

    fn remove_selected_commands(commands: &mut Vec<String>, selected: &[usize]) {
        let mut indexes: Vec<usize> = selected.iter().map(|index| index - 1).collect();
        indexes.sort_unstable();
        indexes.dedup();

        for index in indexes.into_iter().rev() {
            commands.remove(index);
        }
    }

    fn print_commands(commands: &[String]) {
        println!("\n💻 Знайдено bash-команди:");
        for (i, cmd) in commands.iter().enumerate() {
            println!("{}", Self::format_command_preview(i + 1, cmd));
        }
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
        let mut commands = Self::extract_commands(full_response);

        if commands.is_empty() {
            return Ok(()); // Якщо команд немає, фіча просто мовчки завершує роботу
        }

        while !commands.is_empty() {
            Self::print_commands(&commands);

            print!(
                "\nБажаєте виконати команду? (введіть номери через кому, 'all' для всіх, або Enter для відміни): "
            );
            io::stdout().flush()?;

            let mut choice = String::new();
            io::stdin().read_line(&mut choice)?;

            match Self::parse_selection(&choice, commands.len()) {
                Ok(CommandSelection::Cancel) => {
                    println!("Відмінено.");
                    break;
                }
                Ok(CommandSelection::All) => {
                    for cmd in &commands {
                        self.run_command(cmd);
                    }
                    commands.clear();
                }
                Ok(CommandSelection::Selected(selected)) => {
                    let commands_to_run: Vec<String> = selected
                        .iter()
                        .map(|index| commands[index - 1].clone())
                        .collect();

                    for cmd in &commands_to_run {
                        self.run_command(cmd);
                    }

                    Self::remove_selected_commands(&mut commands, &selected);
                }
                Err(message) => {
                    println!("❌ {}", message);
                }
            }
        }

        if commands.is_empty() {
            println!("✅ Усі вибрані команди оброблено.");
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::{BashRunner, CommandSelection};

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
        let preview =
            BashRunner::format_command_preview(7, "sudo dnf check-update\nsudo dnf upgrade");

        assert_eq!(preview, " [7] sudo dnf check-update\n     sudo dnf upgrade");
    }

    #[test]
    fn parses_multiple_indexes_in_input_order() {
        let selection = BashRunner::parse_selection("3, 6, 3", 6).unwrap();

        match selection {
            CommandSelection::Selected(selected) => assert_eq!(selected, vec![3, 6]),
            _ => panic!("expected selected command indexes"),
        }
    }

    #[test]
    fn parses_all_and_cancel_options() {
        assert!(matches!(
            BashRunner::parse_selection("all", 4).unwrap(),
            CommandSelection::All
        ));
        assert!(matches!(
            BashRunner::parse_selection("", 4).unwrap(),
            CommandSelection::Cancel
        ));
    }

    #[test]
    fn rejects_invalid_command_numbers() {
        assert!(BashRunner::parse_selection("0", 4).is_err());
        assert!(BashRunner::parse_selection("5", 4).is_err());
        assert!(BashRunner::parse_selection("2, nope", 4).is_err());
    }

    #[test]
    fn removes_selected_commands_and_keeps_remaining_order() {
        let mut commands = vec![
            "cmd1".to_string(),
            "cmd2".to_string(),
            "cmd3".to_string(),
            "cmd4".to_string(),
        ];

        BashRunner::remove_selected_commands(&mut commands, &[3, 1]);

        assert_eq!(commands, vec!["cmd2", "cmd4"]);
    }
}
