use clap::Parser;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Cli {
    /// Your command description here
    #[arg(short, long)]
    pub prompt: Option<String>,
    #[arg(short, long, default_value_t = String::from("gemini-2.5-flash-lite"))]
    pub model: String,
    #[arg(long)]
    pub set_key: Option<String>,
}
