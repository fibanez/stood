use std::io::{self, Write};
use stood::llm::models::Bedrock;
use stood::llm::traits::LlmModel;

#[derive(Debug, Clone)]
pub enum SelectedModel {
    ClaudeHaiku45(Bedrock::ClaudeHaiku45),
    ClaudeSonnet45(Bedrock::ClaudeSonnet45),
    ClaudeOpus45(Bedrock::ClaudeOpus45),
    NovaMicro(Bedrock::NovaMicro),
    NovaLite(Bedrock::NovaLite),
    NovaPro(Bedrock::NovaPro),
}

impl SelectedModel {
    pub fn display_name(&self) -> &'static str {
        match self {
            Self::ClaudeHaiku45(m) => m.display_name(),
            Self::ClaudeSonnet45(m) => m.display_name(),
            Self::ClaudeOpus45(m) => m.display_name(),
            Self::NovaMicro(m) => m.display_name(),
            Self::NovaLite(m) => m.display_name(),
            Self::NovaPro(m) => m.display_name(),
        }
    }
}

pub fn select_model_interactively() -> SelectedModel {
    println!("Select a model:");
    println!("1. Claude Haiku 4.5 (fast, cost-effective)");
    println!("2. Claude Sonnet 4.5 (balanced performance)");
    println!("3. Claude Opus 4.5 (maximum intelligence)");
    println!("4. Nova Micro (AWS optimized)");
    println!("5. Nova Lite (AWS mid-tier)");
    println!("6. Nova Pro (AWS high-performance)");

    loop {
        print!("Enter your choice (1-6) [default: 1]: ");
        io::stdout().flush().unwrap();

        let mut input = String::new();
        io::stdin().read_line(&mut input).unwrap();
        let input = input.trim();

        // Default to Haiku if empty
        if input.is_empty() {
            println!("Selected: Claude Haiku 4.5");
            return SelectedModel::ClaudeHaiku45(Bedrock::ClaudeHaiku45);
        }

        match input {
            "1" => {
                println!("Selected: Claude Haiku 4.5");
                return SelectedModel::ClaudeHaiku45(Bedrock::ClaudeHaiku45);
            },
            "2" => {
                println!("Selected: Claude Sonnet 4.5");
                return SelectedModel::ClaudeSonnet45(Bedrock::ClaudeSonnet45);
            },
            "3" => {
                println!("Selected: Claude Opus 4.5");
                return SelectedModel::ClaudeOpus45(Bedrock::ClaudeOpus45);
            },
            "4" => {
                println!("Selected: Nova Micro");
                return SelectedModel::NovaMicro(Bedrock::NovaMicro);
            },
            "5" => {
                println!("Selected: Nova Lite");
                return SelectedModel::NovaLite(Bedrock::NovaLite);
            },
            "6" => {
                println!("Selected: Nova Pro");
                return SelectedModel::NovaPro(Bedrock::NovaPro);
            },
            _ => {
                println!("Invalid choice. Please enter 1-6.");
                continue;
            }
        }
    }
}
