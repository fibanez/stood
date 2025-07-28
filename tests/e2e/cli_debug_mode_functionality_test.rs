//! Debug test to troubleshoot CLI spawning issues

use expectrl;
use std::time::Duration;

#[tokio::test]
async fn test_cli_spawn_debug() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    println!("🔍 Testing CLI spawn process...");

    // Try to spawn the CLI directly with cargo run
    let cmd = format!("cargo run --bin stood-agentic-cli -- --model claude-haiku-3 chat --agentic");
    println!("🚀 Spawning: {}", cmd);

    match expectrl::spawn(&cmd) {
        Ok(mut session) => {
            println!("✅ CLI process spawned successfully");

            // Try to expect the banner
            match tokio::time::timeout(Duration::from_secs(10), async {
                session.expect("🤖 Stood Agentic CLI")
            })
            .await
            {
                Ok(Ok(_)) => {
                    println!("✅ Found CLI banner");

                    // Try to send help command
                    if let Err(e) = session.send_line("help") {
                        println!("❌ Failed to send 'help': {}", e);
                    } else {
                        println!("✅ Sent 'help' command");

                        // Try to expect help response
                        match tokio::time::timeout(Duration::from_secs(5), async {
                            session.expect("Commands:")
                        })
                        .await
                        {
                            Ok(Ok(_)) => println!("✅ Got help response"),
                            Ok(Err(e)) => println!("❌ Error expecting help: {}", e),
                            Err(_) => println!("⏱️ Timeout waiting for help response"),
                        }
                    }

                    // Try to exit gracefully
                    if let Err(e) = session.send_line("exit") {
                        println!("❌ Failed to send 'exit': {}", e);
                    } else {
                        println!("✅ Sent 'exit' command");
                    }
                }
                Ok(Err(e)) => println!("❌ Error expecting banner: {}", e),
                Err(_) => println!("⏱️ Timeout waiting for CLI banner"),
            }
        }
        Err(e) => {
            println!("❌ Failed to spawn CLI: {}", e);
            return Err(e.into());
        }
    }

    println!("🏁 Debug test completed");
    Ok(())
}
