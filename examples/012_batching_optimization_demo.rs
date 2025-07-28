//! Batching optimization demonstration
//!
//! This example demonstrates how batching techniques dramatically improve performance
//! for high-frequency I/O operations by reducing expensive file system calls.

use stood::agent::{Agent, LogLevel};
use stood::agent::callbacks::{
    BatchingCallbackHandler, BatchConfig, CallbackHandler, CallbackEvent, CallbackError, 
};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use tokio::fs;

/// Performance monitoring callback handler that writes to files (expensive I/O)
#[derive(Debug)]
struct FileWritingMonitor {
    event_count: Arc<Mutex<usize>>,
    start_time: Arc<Mutex<Option<Instant>>>,
    file_path: String,
}

impl FileWritingMonitor {
    fn new(file_path: String) -> Self {
        Self {
            event_count: Arc::new(Mutex::new(0)),
            start_time: Arc::new(Mutex::new(None)),
            file_path,
        }
    }
    
    fn get_stats(&self) -> (usize, Option<Duration>) {
        let count = *self.event_count.lock().unwrap();
        let elapsed = self.start_time.lock().unwrap()
            .map(|start| start.elapsed());
        (count, elapsed)
    }
    
    #[allow(dead_code)]
    fn reset(&self) {
        *self.event_count.lock().unwrap() = 0;
        *self.start_time.lock().unwrap() = None;
    }
}

#[async_trait::async_trait]
impl CallbackHandler for FileWritingMonitor {
    async fn handle_event(&self, event: CallbackEvent) -> Result<(), CallbackError> {
        // Set start time on first event
        {
            let mut start_guard = self.start_time.lock().unwrap();
            if start_guard.is_none() {
                *start_guard = Some(Instant::now());
            }
        }
        
        // Increment counter
        let count = {
            let mut count_guard = self.event_count.lock().unwrap();
            *count_guard += 1;
            *count_guard
        };
        
        // Expensive I/O operation: Write each event to a file
        let content = match event {
            CallbackEvent::ContentDelta { delta, .. } => format!("[{}] Content: {}\n", count, delta),
            CallbackEvent::ToolStart { tool_name, .. } => format!("[{}] Tool started: {}\n", count, tool_name),
            CallbackEvent::ToolComplete { tool_name, .. } => format!("[{}] Tool completed: {}\n", count, tool_name),
            _ => format!("[{}] Other event\n", count),
        };
        
        // Individual file write for each event (this is expensive!)
        fs::write(&self.file_path, content).await.map_err(|e| {
            CallbackError::ExecutionFailed(format!("File write failed: {}", e))
        })?;
        
        Ok(())
    }
}

/// Batching file writer that accumulates events and writes in batches
#[derive(Debug)]
struct BatchingFileMonitor {
    event_count: Arc<Mutex<usize>>,
    start_time: Arc<Mutex<Option<Instant>>>,
    file_path: String,
    batch_buffer: Arc<Mutex<Vec<String>>>,
}

impl BatchingFileMonitor {
    fn new(file_path: String) -> Self {
        Self {
            event_count: Arc::new(Mutex::new(0)),
            start_time: Arc::new(Mutex::new(None)),
            file_path,
            batch_buffer: Arc::new(Mutex::new(Vec::new())),
        }
    }
    
    fn get_stats(&self) -> (usize, Option<Duration>) {
        let count = *self.event_count.lock().unwrap();
        let elapsed = self.start_time.lock().unwrap()
            .map(|start| start.elapsed());
        (count, elapsed)
    }
    
    async fn flush_batch(&self) -> Result<(), CallbackError> {
        let batch = {
            let mut buffer = self.batch_buffer.lock().unwrap();
            if buffer.is_empty() {
                return Ok(());
            }
            let batch = buffer.join("");
            buffer.clear();
            batch
        };
        
        // Single file write for entire batch (much more efficient!)
        fs::write(&self.file_path, batch).await.map_err(|e| {
            CallbackError::ExecutionFailed(format!("Batch file write failed: {}", e))
        })?;
        
        Ok(())
    }
}

#[async_trait::async_trait]
impl CallbackHandler for BatchingFileMonitor {
    async fn handle_event(&self, event: CallbackEvent) -> Result<(), CallbackError> {
        // Set start time on first event
        {
            let mut start_guard = self.start_time.lock().unwrap();
            if start_guard.is_none() {
                *start_guard = Some(Instant::now());
            }
        }
        
        // Increment counter
        let count = {
            let mut count_guard = self.event_count.lock().unwrap();
            *count_guard += 1;
            *count_guard
        };
        
        // Add to batch buffer instead of writing immediately
        let content = match event {
            CallbackEvent::ContentDelta { delta, .. } => format!("[{}] Content: {}\n", count, delta),
            CallbackEvent::ToolStart { tool_name, .. } => format!("[{}] Tool started: {}\n", count, tool_name),
            CallbackEvent::ToolComplete { tool_name, .. } => format!("[{}] Tool completed: {}\n", count, tool_name),
            _ => format!("[{}] Other event\n", count),
        };
        
        let should_flush = {
            let mut buffer = self.batch_buffer.lock().unwrap();
            buffer.push(content);
            buffer.len() >= 50
        };
        
        if should_flush {
            self.flush_batch().await?;
        }
        
        Ok(())
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 Batching Optimization Demo");
    println!("=============================");
    println!();
    println!("📋 What this example demonstrates:");
    println!("   This example shows how batching dramatically improves I/O performance by reducing");
    println!("   expensive file system calls. Instead of writing each event individually to disk,");
    println!("   batching accumulates events in memory and writes them in larger, more efficient chunks.");
    println!();

    // Turn off all logging for clean demo output
    tracing_subscriber::fmt()
        .with_env_filter("off")
        .init();

    // Demo 1: Compare file I/O performance with and without batching
    println!("📊 Demo 1: File I/O Batching Performance Comparison");
    println!("───────────────────────────────────────────\n");

    // Create temporary file paths for testing
    let temp_dir = std::env::temp_dir();
    let regular_file = temp_dir.join("stood_regular_test.log");
    let batched_file = temp_dir.join("stood_batched_test.log");

    // Create performance monitors with file operations
    let regular_monitor = Arc::new(FileWritingMonitor::new(regular_file.to_string_lossy().to_string()));
    let batched_monitor = Arc::new(BatchingFileMonitor::new(batched_file.to_string_lossy().to_string()));

    // Test regular file handler (individual writes)
    println!("🔄 Testing individual file writes (expensive I/O)...");
    let regular_handler = Arc::clone(&regular_monitor);
    let regular_start = Instant::now();
    
    // Simulate high-frequency content delta events with file I/O
    for i in 0..100 {  // Reduced count for realistic file I/O testing
        let event = CallbackEvent::ContentDelta {
            delta: format!("content chunk {}", i),
            complete: false,
            reasoning: false,
        };
        regular_handler.handle_event(event).await?;
    }
    let regular_duration = regular_start.elapsed();
    
    // Test batched file handler (batched writes)
    println!("🔄 Testing batched file writes (efficient I/O)...");
    let batch_config = BatchConfig {
        max_batch_size: 50,  // Larger batches for file I/O
        max_batch_delay: Duration::from_millis(100),
        batch_content_deltas: true,
        batch_tool_events: false,
    };
    
    let batched_handler = BatchingCallbackHandler::new(
        batched_monitor.clone() as Arc<dyn CallbackHandler>,
        batch_config,
    );
    
    let batched_start = Instant::now();
    for i in 0..100 {  // Same count for fair comparison
        let event = CallbackEvent::ContentDelta {
            delta: format!("content chunk {}", i),
            complete: false,
            reasoning: false,
        };
        batched_handler.handle_event(event).await?;
    }
    
    // Flush remaining batched events and final batch
    batched_handler.flush().await?;
    batched_monitor.flush_batch().await?;
    let batched_duration = batched_start.elapsed();
    
    // Wait for async I/O completion
    tokio::time::sleep(Duration::from_millis(200)).await;

    // Display results
    let (regular_count, _) = regular_monitor.get_stats();
    let (batched_count, _) = batched_monitor.get_stats();
    
    println!("✅ File I/O Performance Comparison Results:");
    println!("   Individual file writes:");
    println!("     - Events processed: {}", regular_count);
    println!("     - Total time: {:?}", regular_duration);
    println!("     - Operations/sec: {:.1}", 100.0 / regular_duration.as_secs_f64());
    
    println!("   Batched file writes:");
    println!("     - Events processed: {}", batched_count);
    println!("     - Total time: {:?}", batched_duration);
    println!("     - Operations/sec: {:.1}", 100.0 / batched_duration.as_secs_f64());
    
    let improvement = (regular_duration.as_secs_f64() / batched_duration.as_secs_f64() - 1.0) * 100.0;
    if improvement > 0.0 {
        println!("   📈 Batching improvement: {:.1}% faster", improvement);
    } else {
        println!("   📉 Batching overhead: {:.1}% slower", -improvement);
    }

    // Clean up test files
    let _ = fs::remove_file(&regular_file).await;
    let _ = fs::remove_file(&batched_file).await;

    // Demo 2: Agent configuration with optimized callbacks
    println!("\n📋 Demo 2: Agent with Performance-Optimized Callbacks");
    println!("─────────────────────────────────────────────────────\n");

    println!("Creating agents with different callback configurations...");

    // Check AWS credentials
    let has_aws = std::env::var("AWS_ACCESS_KEY_ID").is_ok() || 
                  std::env::var("AWS_PROFILE").is_ok() || 
                  std::env::var("AWS_ROLE_ARN").is_ok();

    if has_aws {
        println!("🔗 AWS credentials detected - testing with real agent...\n");

        // Agent with regular printing callbacks
        let mut regular_agent = Agent::builder()
            .with_printing_callbacks()
            .with_log_level(LogLevel::Off)
            .build().await?;

        // Agent with batched printing callbacks for better performance
        let mut batched_agent = Agent::builder()
            .with_batched_printing_callbacks()
            .with_log_level(LogLevel::Off)
            .build().await?;

        println!("🧮 Testing regular callbacks execution:");
        let regular_start = Instant::now();
        match regular_agent.execute("Count from 1 to 5 quickly").await {
            Ok(result) => {
                let regular_time = regular_start.elapsed();
                println!("   ✅ Regular execution completed in {:?}", regular_time);
                println!("   📄 Response: {}", result.response);
            }
            Err(e) => {
                println!("   ⚠️ Regular execution failed: {}", e);
            }
        }

        println!("\n🚀 Testing batched callbacks execution:");
        let batched_start = Instant::now();
        match batched_agent.execute("Count from 1 to 5 quickly").await {
            Ok(result) => {
                let batched_time = batched_start.elapsed();
                println!("   ✅ Batched execution completed in {:?}", batched_time);
                println!("   📄 Response: {}", result.response);
            }
            Err(e) => {
                println!("   ⚠️ Batched execution failed: {}", e);
            }
        }

    } else {
        println!("⚠️ No AWS credentials found - demonstrating configuration only");
        println!("Set up AWS credentials to see live performance comparison:");
        println!("   • Set AWS_ACCESS_KEY_ID and AWS_SECRET_ACCESS_KEY");
        println!("   • Or configure AWS_PROFILE");
        println!("   • Or set up IAM role");
    }

    // Demo 3: Custom performance configurations
    println!("\n⚙️ Demo 3: Custom Performance Configurations");
    println!("──────────────────────────────────────────────\n");

    println!("Available performance optimization options:");
    println!("   • .with_batched_printing_callbacks() - Default batching for printing");
    println!("   • .with_batched_callbacks(inner, config) - Custom batching configuration");
    println!("   • BatchConfig::max_batch_size - Control batch size");
    println!("   • BatchConfig::max_batch_delay - Control batching delay");
    println!("   • BatchConfig::batch_content_deltas - Enable/disable content batching");

    // Example of custom batch configuration
    let custom_batch_config = BatchConfig {
        max_batch_size: 20,  // Larger batches
        max_batch_delay: Duration::from_millis(25),  // Lower latency
        batch_content_deltas: true,
        batch_tool_events: true,  // Also batch tool events
    };

    println!("\n📝 Example custom configuration:");
    println!("   BatchConfig {{");
    println!("       max_batch_size: {},", custom_batch_config.max_batch_size);
    println!("       max_batch_delay: {:?},", custom_batch_config.max_batch_delay);
    println!("       batch_content_deltas: {},", custom_batch_config.batch_content_deltas);
    println!("       batch_tool_events: {},", custom_batch_config.batch_tool_events);
    println!("   }}");

    println!("\n✅ Batching Optimization Demo completed!");
    println!("💡 Key takeaways:");
    println!("   • Batching reduces expensive I/O operations by grouping them");
    println!("   • File system calls have significant overhead - fewer calls = better performance");
    println!("   • Memory operations are much faster than disk operations");
    println!("   • Configurable batch size and timing for different optimization needs");

    Ok(())
}