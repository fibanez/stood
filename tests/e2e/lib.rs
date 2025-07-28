//! End-to-end testing utilities for the Stood agentic CLI
//!
//! This module provides common utilities for spawning and interacting with
//! the CLI application using expectrl for expect-like testing functionality.

use expectrl::Session;
use std::env;
use std::path::Path;
use std::time::Duration;
use tempfile::TempDir;
use tokio::time::timeout;

pub type Result<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync>>;

/// Configuration for CLI test sessions
#[derive(Debug, Clone)]
pub struct TestConfig {
    /// Timeout for expect operations
    pub expect_timeout: Duration,
    /// Timeout for CLI startup
    pub startup_timeout: Duration,
    /// Whether to enable debug output from CLI
    pub debug_mode: bool,
    /// Whether to enable agentic mode by default
    pub agentic_mode: bool,
    /// Model to use for testing
    pub model: String,
    /// Additional CLI arguments
    pub extra_args: Vec<String>,
    /// Whether to show live CLI interactions (for debugging)
    pub visible_mode: bool,
    /// Whether to disable streaming (streaming is enabled by default)
    pub disable_streaming: bool,
}

impl Default for TestConfig {
    fn default() -> Self {
        Self {
            expect_timeout: Duration::from_secs(30),
            startup_timeout: Duration::from_secs(10),
            debug_mode: false,
            agentic_mode: true,
            model: "claude-haiku-3".to_string(),
            extra_args: Vec::new(),
            visible_mode: false,
            disable_streaming: false,
        }
    }
}

/// Wrapper around expectrl Session with additional utilities
pub struct CliSession {
    session: Session,
    config: TestConfig,
    _temp_dir: Option<TempDir>,
}

impl CliSession {
    /// Send a line of input to the CLI
    pub async fn send_line(&mut self, line: &str) -> Result<()> {
        if self.config.visible_mode {
            println!("📤 Sending: {}", line);
        }
        self.session.send_line(line)?;
        Ok(())
    }

    /// Expect a specific string in the CLI output
    pub async fn expect(&mut self, pattern: &str) -> Result<String> {
        if self.config.visible_mode {
            println!("🔍 Expecting: {}", pattern);
        }
        let result = timeout(self.config.expect_timeout, async {
            let _captures = self.session.expect(pattern)?;
            // For now, just return the pattern that was found
            // In a real implementation, we'd extract the actual matched text
            Ok::<String, expectrl::Error>(pattern.to_string())
        })
        .await??;
        if self.config.visible_mode {
            println!("✅ Found: {}", pattern);
        }
        Ok(result)
    }

    /// Wait for the CLI process to exit
    pub async fn wait_for_exit(&mut self) -> Result<()> {
        // expectrl doesn't directly expose wait, so we try to check if process ended
        // This is a simplified implementation
        while self.session.is_alive()? {
            tokio::time::sleep(Duration::from_millis(100)).await;
        }
        Ok(())
    }

    /// Send Ctrl+C to the CLI
    pub async fn send_control_c(&mut self) -> Result<()> {
        // Send Control+C (interrupt signal)
        // Try different control codes that might be available
        self.session.send("\x03")?; // ASCII code for Ctrl+C
        Ok(())
    }
}

/// Helper to check if AWS credentials are available for testing
pub fn check_aws_credentials() -> bool {
    env::var("AWS_ACCESS_KEY_ID").is_ok()
        || env::var("AWS_PROFILE").is_ok()
        || env::var("AWS_ROLE_ARN").is_ok()
}

/// Create a temporary directory for test files
pub fn create_temp_dir() -> Result<TempDir> {
    Ok(tempfile::tempdir()?)
}

/// Get the path to the CLI binary
pub fn get_cli_binary_path() -> Result<String> {
    // Try to find the binary in target directory
    let cargo_manifest_dir = env::var("CARGO_MANIFEST_DIR").unwrap_or_else(|_| ".".to_string());

    let debug_path = format!("{}/target/debug/stood-agentic-cli", cargo_manifest_dir);
    let release_path = format!("{}/target/release/stood-agentic-cli", cargo_manifest_dir);

    if Path::new(&debug_path).exists() {
        Ok(debug_path)
    } else if Path::new(&release_path).exists() {
        Ok(release_path)
    } else {
        // Fallback to cargo run
        Ok("cargo".to_string())
    }
}

/// Spawn the CLI with default configuration
pub async fn spawn_cli() -> Result<CliSession> {
    spawn_cli_with_config(TestConfig::default()).await
}

/// Spawn the CLI with visible interactions (for debugging)
pub async fn spawn_cli_visible() -> Result<CliSession> {
    let mut config = TestConfig::default();
    config.visible_mode = true;
    spawn_cli_with_config(config).await
}


/// Spawn the CLI with custom configuration
pub async fn spawn_cli_with_config(config: TestConfig) -> Result<CliSession> {
    let binary_path = get_cli_binary_path()?;

    let mut args = Vec::new();

    // Handle cargo run vs direct binary execution
    if binary_path == "cargo" {
        args.push("run".to_string());
        args.push("--bin".to_string());
        args.push("stood-agentic-cli".to_string());
        args.push("--".to_string());
    }

    // Add global CLI arguments first
    args.push("--model".to_string());
    args.push(config.model.clone());

    if config.debug_mode {
        args.push("--debug".to_string());
    }

    // Add global streaming option before subcommand
    if config.disable_streaming {
        args.push("--no-streaming".to_string());
    }

    // Add the subcommand
    args.push("chat".to_string());

    // Add subcommand-specific arguments
    if config.agentic_mode {
        args.push("--agentic".to_string());
    }

    // Add any extra arguments
    args.extend(config.extra_args.clone());

    // Create temporary directory for test files
    let temp_dir = create_temp_dir().ok();

    // Show command being executed if in visible mode
    let full_command = if !args.is_empty() {
        format!("{} {}", binary_path, args.join(" "))
    } else {
        binary_path.clone()
    };

    if config.visible_mode {
        println!("🚀 Spawning CLI: {}", full_command);
    }

    // Spawn the CLI process
    let cmd = expectrl::spawn(&full_command)?;

    // Create session
    let mut session = CliSession {
        session: cmd,
        config: config.clone(),
        _temp_dir: temp_dir,
    };

    // Wait for CLI to start up and show initial prompt
    timeout(config.startup_timeout, async {
        // Try to detect the CLI has started by looking for the banner
        session.expect("🤖 Stood Agentic CLI").await
    })
    .await??;

    Ok(session)
}

/// Test helper to create sample files in temp directory
pub async fn create_sample_files(temp_dir: &Path) -> Result<()> {
    use tokio::fs;

    // Create a simple text file
    fs::write(
        temp_dir.join("sample.txt"),
        "This is a sample text file for testing.\nIt has multiple lines.\n",
    )
    .await?;

    // Create a JSON file
    fs::write(
        temp_dir.join("data.json"),
        r#"{
    "name": "test",
    "value": 42,
    "items": ["a", "b", "c"]
}"#,
    )
    .await?;

    // Create a subdirectory with files
    fs::create_dir(temp_dir.join("subdir")).await?;
    fs::write(
        temp_dir.join("subdir/nested.txt"),
        "This is a nested file.\n",
    )
    .await?;

    Ok(())
}

/// Macro to skip test if AWS credentials are not available
#[macro_export]
macro_rules! require_aws_credentials {
    () => {
        if !check_aws_credentials() {
            println!("⚠️  Skipping test - AWS credentials not available");
            return;
        }
    };
}

/// Macro to create a test that requires AWS credentials
#[macro_export]
macro_rules! aws_test {
    (async fn $name:ident() $body:block) => {
        #[tokio::test]
        async fn $name() {
            if !check_aws_credentials() {
                println!("⚠️  Skipping test - AWS credentials not available");
                return;
            }
            $body
        }
    };
    (async fn $name:ident() -> $ret:ty $body:block) => {
        #[tokio::test]
        async fn $name() -> $ret {
            if !check_aws_credentials() {
                println!("⚠️  Skipping test - AWS credentials not available");
                return Ok(());
            }
            $body
        }
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_aws_credentials_check() {
        // This should not panic
        let _has_creds = check_aws_credentials();
    }

    #[test]
    fn test_default_config() {
        let config = TestConfig::default();
        assert_eq!(config.model, "claude-haiku-3");
        assert!(config.agentic_mode);
        assert!(!config.debug_mode);
    }

    #[tokio::test]
    async fn test_temp_dir_creation() -> Result<()> {
        let temp_dir = create_temp_dir()?;
        assert!(temp_dir.path().exists());

        create_sample_files(temp_dir.path()).await?;
        assert!(temp_dir.path().join("sample.txt").exists());
        assert!(temp_dir.path().join("data.json").exists());
        assert!(temp_dir.path().join("subdir/nested.txt").exists());

        Ok(())
    }

    #[test]
    fn test_binary_path_detection() {
        // This should not panic, even if binary doesn't exist
        let _path = get_cli_binary_path();
    }
}
