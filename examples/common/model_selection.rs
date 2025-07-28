use std::io::{self, Write};
use stood::llm::models::Bedrock;
use stood::llm::traits::LlmModel;

#[derive(Debug, Clone)]
pub enum SelectedModel {
    Claude35Haiku(Bedrock::Claude35Haiku),
    Claude35Sonnet(Bedrock::Claude35Sonnet),
    NovaMicro(Bedrock::NovaMicro),
    NovaLite(Bedrock::NovaLite),
    NovaPro(Bedrock::NovaPro),
}

impl SelectedModel {
    pub fn display_name(&self) -> &'static str {
        match self {
            Self::Claude35Haiku(m) => m.display_name(),
            Self::Claude35Sonnet(m) => m.display_name(),
            Self::NovaMicro(m) => m.display_name(),
            Self::NovaLite(m) => m.display_name(),
            Self::NovaPro(m) => m.display_name(),
        }
    }
}

pub fn select_model_interactively() -> SelectedModel {
    println!("🤖 Select a model:");
    println!("1. Claude 3.5 Haiku (fast, cost-effective)");
    println!("2. Claude 3.5 Sonnet (balanced performance)");
    println!("3. Nova Micro (AWS optimized)");
    println!("4. Nova Lite (AWS mid-tier)");
    println!("5. Nova Pro (AWS high-performance)");
    
    loop {
        print!("Enter your choice (1-5) [default: 1]: ");
        io::stdout().flush().unwrap();
        
        let mut input = String::new();
        io::stdin().read_line(&mut input).unwrap();
        let input = input.trim();
        
        // Default to Haiku if empty
        if input.is_empty() {
            println!("Selected: Claude 3.5 Haiku");
            return SelectedModel::Claude35Haiku(Bedrock::Claude35Haiku);
        }
        
        match input {
            "1" => {
                println!("Selected: Claude 3.5 Haiku");
                return SelectedModel::Claude35Haiku(Bedrock::Claude35Haiku);
            },
            "2" => {
                println!("Selected: Claude 3.5 Sonnet");
                return SelectedModel::Claude35Sonnet(Bedrock::Claude35Sonnet);
            },
            "3" => {
                println!("Selected: Nova Micro");
                return SelectedModel::NovaMicro(Bedrock::NovaMicro);
            },
            "4" => {
                println!("Selected: Nova Lite");
                return SelectedModel::NovaLite(Bedrock::NovaLite);
            },
            "5" => {
                println!("Selected: Nova Pro");
                return SelectedModel::NovaPro(Bedrock::NovaPro);
            },
            _ => {
                println!("Invalid choice. Please enter 1-5.");
                continue;
            }
        }
    }
}