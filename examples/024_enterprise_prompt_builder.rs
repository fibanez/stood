//! Example 024: Interactive Enterprise Prompt Builder (egui GUI)
//!
//! This example demonstrates an interactive multi-agent system that helps build enterprise-level prompts.
//! The system features a persistent coordinating agent that manages specialized task agents while
//! maintaining an interactive GUI interface with the user.
//!
//! Key features:
//! - egui-based graphical interface with message window and input area
//! - Message display with tree nodes (collapsible for details)
//! - Persistent coordinating agent with dual chat/orchestration role
//! - Specialized task agents for each prompt section
//! - Context accumulation between task agents
//! - User feedback integration and task recovery
//! - Progress tracking and display tools
//! - Debugging mode with expandable debug information

use std::collections::{VecDeque, HashMap};
use std::sync::{Arc, Mutex, mpsc};
use stood::agent::{Agent, result::AgentResult};
use stood::agent::callbacks::{CallbackHandler, CallbackEvent, CallbackError};
use stood::tool;
use stood::telemetry::TelemetryConfig;
use chrono::{DateTime, Utc, Local};
use egui::{CollapsingHeader, Color32, RichText, ScrollArea, TextEdit};
use eframe::{egui, App, Frame};
use tracing::{debug, info, warn, error, trace};
use tracing_subscriber::EnvFilter;
use async_trait::async_trait;
use serde_json;
use uuid::Uuid;


// ============================================================================
// MESSAGE SYSTEM FOR egui
// ============================================================================

/// Response from agent execution in separate thread
#[derive(Debug)]
enum AgentResponse {
    Success(AgentResult),
    Error(String),
    JsonDebug(JsonDebugData),
}

/// JSON debug data captured from model interactions
#[derive(Debug, Clone)]
pub struct JsonDebugData {
    pub json_type: JsonDebugType,
    pub json_content: String,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone)]
pub enum JsonDebugType {
    Request,
    Response,
}

/// Message types for the GUI
#[derive(Debug, Clone)]
pub enum MessageRole {
    User,
    Assistant,
    System,
    Debug,
    JsonRequest,   // Model request JSON data
    JsonResponse,  // Model response JSON data
}

impl MessageRole {
    fn icon(&self) -> &'static str {
        match self {
            MessageRole::User => "👤",
            MessageRole::Assistant => "⚡", // Lightning bolt - supported by egui
            MessageRole::System => "ℹ", 
            MessageRole::Debug => "🔧",
            MessageRole::JsonRequest => "📤", // Outgoing request 
            MessageRole::JsonResponse => "📥", // Incoming response
        }
    }
    
    fn color(&self, dark_mode: bool) -> Color32 {
        match (self, dark_mode) {
            // Dark mode: bright colors for visibility on dark background
            (MessageRole::User, true) => Color32::from_rgb(100, 150, 255),      // Bright blue
            (MessageRole::Assistant, true) => Color32::from_rgb(100, 255, 150), // Bright green
            (MessageRole::System, true) => Color32::from_rgb(255, 200, 100),    // Bright orange
            (MessageRole::Debug, true) => Color32::from_rgb(180, 180, 180),     // Light gray
            (MessageRole::JsonRequest, true) => Color32::from_rgb(255, 140, 0), // Bright orange for JSON
            (MessageRole::JsonResponse, true) => Color32::from_rgb(255, 140, 0), // Bright orange for JSON
            
            // Light mode: darker colors for visibility on light background
            (MessageRole::User, false) => Color32::from_rgb(50, 75, 150),       // Dark blue
            (MessageRole::Assistant, false) => Color32::from_rgb(50, 150, 75),  // Dark green
            (MessageRole::System, false) => Color32::from_rgb(180, 120, 50),    // Dark orange
            (MessageRole::Debug, false) => Color32::from_rgb(100, 100, 100),    // Dark gray
            (MessageRole::JsonRequest, false) => Color32::from_rgb(200, 100, 0), // Dark orange for JSON
            (MessageRole::JsonResponse, false) => Color32::from_rgb(200, 100, 0), // Dark orange for JSON
        }
    }
}

/// A single message in the conversation
#[derive(Debug, Clone)]
pub struct Message {
    pub id: String, // Unique identifier for this message
    pub role: MessageRole,
    pub content: String,
    pub timestamp: DateTime<Utc>,
    pub summary: Option<String>,
    pub debug_info: Option<String>,
    pub nested_messages: Vec<Message>,
    pub agent_source: Option<String>, // Track which agent/tool generated this message
}

impl Message {
    pub fn new(role: MessageRole, content: String) -> Self {
        let summary = Self::generate_summary(&content);
        Self {
            id: Uuid::new_v4().to_string(),
            role,
            content,
            timestamp: Utc::now(),
            summary: Some(summary),
            debug_info: None,
            nested_messages: Vec::new(),
            agent_source: None,
        }
    }
    
    pub fn new_with_agent(role: MessageRole, content: String, agent_source: String) -> Self {
        let summary = Self::generate_summary(&content);
        Self {
            id: Uuid::new_v4().to_string(),
            role,
            content,
            timestamp: Utc::now(),
            summary: Some(summary),
            debug_info: None,
            nested_messages: Vec::new(),
            agent_source: Some(agent_source),
        }
    }
    
    pub fn with_debug(mut self, debug_info: String) -> Self {
        self.debug_info = Some(debug_info);
        self
    }
    
    pub fn add_nested_message(&mut self, message: Message) {
        self.nested_messages.push(message);
    }
    
    fn generate_summary(content: &str) -> String {
        let words: Vec<&str> = content.split_whitespace().take(5).collect();
        if words.len() < 5 && content.len() < 30 {
            content.to_string()
        } else {
            format!("{}...", words.join(" "))
        }
    }
}

// ============================================================================
// egui APPLICATION STATE
// ============================================================================

/// Represents the current state of the task orchestration process
#[derive(Debug, Clone)]
pub struct TaskOrchestrationState {
    pub current_section: usize,
    pub completed_sections: Vec<String>,
    pub accumulated_context: String,
    pub pending_sections: Vec<String>,
}

impl TaskOrchestrationState {
    pub fn new() -> Self {
        Self {
            current_section: 0,
            completed_sections: Vec::new(),
            accumulated_context: String::new(),
            pending_sections: vec![
                "Task Context & Goals".to_string(),
                "Data Section & XML Tags".to_string(),
                "Detailed Task Instructions".to_string(),
                "Examples".to_string(),
                "Critical Instructions Repetition".to_string(),
                "Tool Descriptions".to_string(),
                "Evaluation Prompt".to_string(),
            ],
        }
    }
    
    pub fn progress_percentage(&self) -> f32 {
        if self.pending_sections.is_empty() {
            100.0
        } else {
            (self.completed_sections.len() as f32 / 7.0) * 100.0
        }
    }
}

/// Main egui application for Enterprise Prompt Builder
pub struct EnterprisePromptBuilderApp {
    messages: VecDeque<Message>,
    input_text: String,
    agent: Arc<Mutex<Option<Agent>>>,
    response_receiver: mpsc::Receiver<AgentResponse>,
    response_sender: mpsc::Sender<AgentResponse>,
    orchestration_state: TaskOrchestrationState,
    debug_mode: bool,
    show_json_debug: bool,
    dark_mode: bool,
    
    // UI state
    auto_scroll: bool,
    scroll_to_bottom: bool,
    last_message_time: Option<std::time::Instant>, // Time when last message was added
    
    // Agent processing
    processing_message: bool,
    
    // Option change tracking
    prev_debug_mode: bool,
    prev_json_debug: bool,
    
    // UI Debug information
    show_debug_panel: bool,
    last_processing_time: Option<std::time::Duration>,
    message_count_stats: (usize, usize, usize, usize), // user, assistant, system, debug
    processing_start_time: Option<std::time::Instant>,
    
    // Expand/Collapse state management
    force_expand_all: bool,
    force_collapse_all: bool,
    message_expand_states: HashMap<String, bool>, // Track expand state by message ID
}

impl EnterprisePromptBuilderApp {
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        info!("🚀 Initializing Enterprise Prompt Builder (Agent will be created on first message)");
        
        // Create response channel for thread communication
        let (response_sender, response_receiver) = mpsc::channel();
        
        let mut app = Self {
            messages: VecDeque::new(),
            input_text: String::new(),
            agent: Arc::new(Mutex::new(None)), // Agent created lazily on first message
            response_receiver,
            response_sender,
            orchestration_state: TaskOrchestrationState::new(),
            debug_mode: false,
            show_json_debug: true, // Enable JSON debug by default
            dark_mode: true, // Default to dark mode
            auto_scroll: true,
            scroll_to_bottom: false,
            last_message_time: None,
            processing_message: false,
            prev_debug_mode: false,
            prev_json_debug: false,
            show_debug_panel: false,
            last_processing_time: None,
            message_count_stats: (0, 0, 0, 0),
            processing_start_time: None,
            force_expand_all: false,
            force_collapse_all: false,
            message_expand_states: HashMap::new(),
        };
        
        // Add welcome message
        app.add_message(Message::new_with_agent(
            MessageRole::System,
            "Welcome to the Interactive Enterprise Prompt Builder!\n\nThis tool will help you create comprehensive enterprise prompts through conversation. Type your message below to get started.".to_string(),
            "System".to_string(),
        ));
        
        app
    }
    
    fn add_message(&mut self, message: Message) {
        debug!("Adding message: role={:?}, agent_source={:?}, content_len={}", 
               message.role, message.agent_source, message.content.len());
        trace!("Message content preview: {}", 
               &message.content[..std::cmp::min(100, message.content.len())]);
        
        self.messages.push_back(message);
        // Keep only last 100 messages to prevent memory issues
        if self.messages.len() > 100 {
            let removed = self.messages.pop_front();
            debug!("Removed oldest message due to 100 message limit: {:?}", removed.map(|m| m.role));
        }
        // Set flag to scroll to bottom when auto-scroll is enabled
        if self.auto_scroll {
            self.scroll_to_bottom = true; 
            self.last_message_time = Some(std::time::Instant::now()); // Track when message was added
            trace!("Auto-scroll enabled, setting scroll_to_bottom flag");
        }
        
        info!("Message added. Total messages: {}", self.messages.len());
    }
    
    fn process_user_input(&mut self, input: String) {
        info!("📝 Processing user input: '{}'", input);
        debug!("Input length: {} chars, processing_message: {}", input.len(), self.processing_message);
        
        if input.trim().is_empty() {
            warn!("Empty input received, ignoring");
            return;
        }
        
        // Add user message to display
        self.add_message(Message::new_with_agent(MessageRole::User, input.clone(), "User".to_string()));
        
        // Handle commands
        if input.starts_with('/') {
            info!("📝 Command detected: {}", input);
            self.handle_command(&input);
            return;
        }
        
        debug!("Input is not a command, proceeding with agent processing");
        
        // Process with real ConversationAgent
        info!("🤖 Starting agent processing for input");
        self.processing_message = true;
        self.processing_start_time = Some(std::time::Instant::now());
        debug!("Set processing_message = true, spawning async task");
        
        // Direct blocking call like example 003 - this will freeze UI during processing but that's exactly how 003 works
        info!("🗨 Sending message to agent directly (blocking UI like example 003)");
        
        // Spawn thread to execute with persistent agent (avoids runtime conflicts)
        info!("🔄 Spawning thread for agent execution with persistent agent");
        let agent = self.agent.clone();
        let sender = self.response_sender.clone();
        let json_debug_enabled = self.show_json_debug;
        
        std::thread::spawn(move || {
            let result = tokio::runtime::Runtime::new().unwrap().block_on(async {
                let mut agent_guard = agent.lock().unwrap();
                
                // Create agent on first use if not already created
                if agent_guard.is_none() {
                    info!("🚀 Creating Agent on first message with full telemetry configuration");
                    
                    // Configure telemetry for the agent (same config as before)
                    let mut telemetry_config = TelemetryConfig::default()
                        .with_service_name("promptapp-conversation-agent")
                        .with_service_version("1.0.0")
                        .with_otlp_endpoint("http://localhost:4319")  // Existing OTEL collector
                        .with_batch_processing();
                    
                    // Enable debug tracing and add service attributes manually
                    telemetry_config.enable_debug_tracing = true;
                    telemetry_config.service_attributes.insert("application".to_string(), "enterprise-prompt-builder".to_string());
                    telemetry_config.service_attributes.insert("agent.type".to_string(), "conversation".to_string());
                    
                    let mut agent_builder = Agent::builder()
                        .system_prompt("You are an expert enterprise prompt engineer. Help users create comprehensive, well-structured prompts for business applications. Provide clear guidance, examples, and best practices.")
                        .with_telemetry(telemetry_config);  // Enable telemetry via agent builder
                    
                    // Always add JSON capture callback to ensure we never lose JSON data
                    info!("📊 Adding JSON capture callback handler (always active)");
                    agent_builder = agent_builder.with_callback_handler(JsonCaptureHandler::new(sender.clone()));
                    
                    match agent_builder.build().await {
                        Ok(new_agent) => {
                            info!("✅ Agent created successfully with telemetry{}", 
                                  if json_debug_enabled { " and JSON capture" } else { "" });
                            *agent_guard = Some(new_agent);
                        }
                        Err(e) => {
                            error!("❌ Failed to create Agent: {}", e);
                            return Err(format!("Failed to create Agent: {}", e));
                        }
                    }
                }
                
                // Execute with the agent
                if let Some(ref mut agent) = agent_guard.as_mut() {
                    match agent.execute(&input).await {
                        Ok(result) => Ok(result),
                        Err(e) => Err(format!("Agent execution failed: {}", e)),
                    }
                } else {
                    Err("Agent not initialized".to_string())
                }
            });
            
            // Send result back to UI thread via channel
            let response = match result {
                Ok(agent_result) => AgentResponse::Success(agent_result),
                Err(e) => AgentResponse::Error(e), // e is already a String
            };
            
            if let Err(e) = sender.send(response) {
                error!("Failed to send agent response back to UI: {}", e);
            }
        });
        
        // Note: We don't wait for the result here - it will come back via the channel
        // The UI update loop will handle the response when it arrives
        debug!("Agent execution thread spawned, processing_message remains true until response received");
    }
    
    fn handle_command(&mut self, command: &str) {
        info!("📝 Handling command: {}", command);
        match command {
            "/help" => {
                self.add_message(Message::new_with_agent(
                    MessageRole::System,
                    "Available commands:\n/help - Show this help\n/json - Toggle JSON debug display\n/progress - Show current progress\n/clear - Clear messages".to_string(),
                    "System".to_string(),
                ));
            }
            "/json" => {
                self.show_json_debug = !self.show_json_debug;
                self.add_message(Message::new_with_agent(
                    MessageRole::System,
                    format!("JSON debug display: {}", if self.show_json_debug { "ON" } else { "OFF" }),
                    "System".to_string(),
                ));
            }
            "/progress" => {
                let progress = self.orchestration_state.progress_percentage();
                self.add_message(Message::new_with_agent(
                    MessageRole::System,
                    format!("Progress: {:.1}% ({}/7 sections completed)", 
                            progress, 
                            self.orchestration_state.completed_sections.len()),
                    "System".to_string(),
                ));
            }
            "/clear" => {
                self.messages.clear();
                self.add_message(Message::new_with_agent(
                    MessageRole::System,
                    "Messages cleared.".to_string(),
                    "System".to_string(),
                ));
            }
            _ => {
                self.add_message(Message::new_with_agent(
                    MessageRole::System,
                    format!("Unknown command: {}. Type /help for available commands.", command),
                    "System".to_string(),
                ));
            }
        }
    }
}

impl App for EnterprisePromptBuilderApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut Frame) {
        // Check for agent responses from background threads (non-blocking)
        while let Ok(response) = self.response_receiver.try_recv() {
            let duration = self.processing_start_time.map(|start| start.elapsed()).unwrap_or_default();
            
            match response {
                AgentResponse::Success(agent_result) => {
                    info!("✅ Agent response received in {:?}. Response length: {} chars", 
                          duration, agent_result.response.len());
                    
                    debug!("[DEBUG] Response received in {:?}", duration);
                    debug!("[DEBUG] Response length: {} chars", agent_result.response.len());
                    debug!("[DEBUG] Used tools: {}", agent_result.used_tools);
                    if agent_result.used_tools {
                        debug!("[DEBUG] Tools called: {:?}", agent_result.tools_called);
                    }
                    debug!("[DEBUG] Success: {}", agent_result.success);
                    
                    if agent_result.response.trim().is_empty() {
                        warn!("❌ [ERROR] Empty response received!");
                        warn!("💡 This might be the empty response bug we're debugging");
                        
                        // Add error message to the conversation
                        self.add_message(Message::new_with_agent(
                            MessageRole::Assistant, 
                            "❌ Error: Received empty response from agent".to_string(), 
                            "Agent".to_string()
                        ));
                    } else {
                        // Add the assistant message to the conversation
                        let mut message = Message::new_with_agent(
                            MessageRole::Assistant, 
                            agent_result.response, 
                            "Agent".to_string()
                        );
                        
                        // Add execution details as debug info if debug mode is on
                        if self.debug_mode {
                            let debug_info = format!(
                                "Execution Details:\nCycles: {}\nModel calls: {}\nTool executions: {}\nDuration: {:?}\nUsed tools: {}\nSuccess: {}",
                                agent_result.execution.cycles,
                                agent_result.execution.model_calls,
                                agent_result.execution.tool_executions,
                                agent_result.duration,
                                agent_result.used_tools,
                                agent_result.success
                            );
                            message = message.with_debug(debug_info);
                        }
                        
                        self.add_message(message);
                    }
                    
                    // Reset processing state
                    self.processing_message = false;
                    self.last_processing_time = Some(duration);
                    self.scroll_to_bottom = true;
                    debug!("Set processing_message = false, scroll_to_bottom = true");
                }
                AgentResponse::Error(error) => {
                    error!("❌ Agent processing failed after {:?}: {}", duration, error);
                    
                    // Create a user-friendly error message
                    let error_text = if error.contains("ExpiredTokenException") {
                        "⚠️ AWS credentials have expired. Please refresh your AWS credentials and try again.".to_string()
                    } else if error.contains("UnknownServiceError") {
                        "⚠️ AWS service error. Please check your AWS configuration and try again.".to_string()
                    } else if error.contains("timeout") || error.contains("Timeout") {
                        "⚠️ Request timed out. The model took too long to respond. Please try again.".to_string()
                    } else {
                        format!("⚠️ Error processing message: {}", error)
                    };
                    
                    self.add_message(Message::new_with_agent(
                        MessageRole::System,
                        error_text,
                        "System".to_string(),
                    ));
                    
                    // Reset processing state
                    self.processing_message = false;
                    self.last_processing_time = Some(duration);
                    self.scroll_to_bottom = true;
                    debug!("Set processing_message = false after error");
                }
                AgentResponse::JsonDebug(json_data) => {
                    debug!("📊 Received JSON debug data: {:?}", json_data.json_type);
                    
                    // Only add JSON debug messages if JSON debug mode is enabled
                    if self.show_json_debug {
                        let message_role = match json_data.json_type {
                            JsonDebugType::Request => MessageRole::JsonRequest,
                            JsonDebugType::Response => MessageRole::JsonResponse,
                        };
                        
                        let header = match json_data.json_type {
                            JsonDebugType::Request => "Model Request JSON",
                            JsonDebugType::Response => "Model Response JSON",
                        };
                        
                        // Create a message with formatted JSON content
                        let mut message = Message::new_with_agent(
                            message_role,
                            json_data.json_content,
                            "JsonCapture".to_string(),
                        );
                        
                        // Set a custom summary for the header
                        message.summary = Some(header.to_string());
                        
                        self.add_message(message);
                        self.scroll_to_bottom = true;
                    }
                }
            }
        }
        
        // Set larger default font size
        ctx.style_mut(|style| {
            style.text_styles.insert(
                egui::TextStyle::Body,
                egui::FontId::new(14.0, egui::FontFamily::Proportional),
            );
            style.text_styles.insert(
                egui::TextStyle::Button,
                egui::FontId::new(14.0, egui::FontFamily::Proportional),
            );
            style.text_styles.insert(
                egui::TextStyle::Heading,
                egui::FontId::new(18.0, egui::FontFamily::Proportional),
            );
            style.text_styles.insert(
                egui::TextStyle::Monospace,
                egui::FontId::new(12.0, egui::FontFamily::Monospace),
            );
        });
        
        // Set theme
        if self.dark_mode {
            ctx.set_visuals(egui::Visuals::dark());
        } else {
            ctx.set_visuals(egui::Visuals::light());
        }
        
        // Direct message processing - no channels needed like example 003!
        
        // Update message count stats for debug panel
        self.update_message_stats();
        
        // Monitor for option changes and inject debug/JSON info
        if self.debug_mode != self.prev_debug_mode || self.show_json_debug != self.prev_json_debug {
            self.update_debug_display();
            self.prev_debug_mode = self.debug_mode;
            self.prev_json_debug = self.show_json_debug;
        }
        
        // Handle delayed auto-scroll after new messages (500ms delay)
        if let Some(last_msg_time) = self.last_message_time {
            if last_msg_time.elapsed() >= std::time::Duration::from_millis(500) {
                if self.auto_scroll && !self.scroll_to_bottom {
                    self.scroll_to_bottom = true;
                    trace!("Delayed auto-scroll triggered after 500ms");
                }
                self.last_message_time = None; // Clear the timer
            } else {
                // Keep requesting repaints until the delay passes
                ctx.request_repaint_after(std::time::Duration::from_millis(10));
            }
        }
        
        // Menu bar
        egui::TopBottomPanel::top("menu_bar").show(ctx, |ui| {
            egui::menu::bar(ui, |ui| {
                ui.menu_button("Options", |ui| {
                    ui.checkbox(&mut self.show_json_debug, "JSON Debug");
                    ui.checkbox(&mut self.show_debug_panel, "📊 Debug Panel");
                    ui.checkbox(&mut self.auto_scroll, "Auto Scroll");
                    ui.add_space(4.0);
                    ui.label("Theme:");
                    ui.radio_value(&mut self.dark_mode, true, "🌙 Dark Mode");
                    ui.radio_value(&mut self.dark_mode, false, "Light Mode");
                    ui.add_space(4.0);
                    if ui.button("Clear Messages").clicked() {
                        self.messages.clear();
                        self.add_message(Message::new_with_agent(
                            MessageRole::System,
                            "Messages cleared.".to_string(),
                            "System".to_string(),
                        ));
                        ui.close_menu();
                    }
                    ui.add_space(4.0);
                    if ui.button("📤 Expand All").clicked() {
                        // Set all visible messages to expanded in persistent state
                        for message in &self.messages {
                            let should_include = match message.role {
                                MessageRole::JsonRequest | MessageRole::JsonResponse => self.show_json_debug,
                                _ => true,
                            };
                            if should_include {
                                self.message_expand_states.insert(message.id.clone(), true);
                            }
                        }
                        self.force_expand_all = true;
                        ui.close_menu();
                    }
                    if ui.button("📥 Collapse All").clicked() {
                        // Set all visible messages to collapsed in persistent state
                        for message in &self.messages {
                            let should_include = match message.role {
                                MessageRole::JsonRequest | MessageRole::JsonResponse => self.show_json_debug,
                                _ => true,
                            };
                            if should_include {
                                self.message_expand_states.insert(message.id.clone(), false);
                            }
                        }
                        self.force_collapse_all = true;
                        ui.close_menu();
                    }
                });
                ui.menu_button("Help", |ui| {
                    ui.label("Available commands:");
                    ui.monospace("/help - Show commands");
                    ui.monospace("/json - Toggle JSON display");
                    ui.monospace("/progress - Show progress");
                    ui.monospace("/clear - Clear messages");
                });
                
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    // Removed progress indicator as requested
                    ui.label(""); // Empty space for layout
                });
            });
        });
        
        // Input panel at bottom - make it taller (8 lines)
        egui::TopBottomPanel::bottom("input_panel").min_height(120.0).show(ctx, |ui| {
            ui.horizontal(|ui| {
                let text_edit = TextEdit::multiline(&mut self.input_text)
                    .desired_rows(8)
                    .desired_width(f32::INFINITY)
                    .hint_text("Type your message here... (use /help for commands)\nPress Enter to send, Shift+Enter for new line");
                
                let response = ui.add(text_edit);
                
                ui.vertical(|ui| {
                    // Send button
                    let send_enabled = !self.processing_message && !self.input_text.trim().is_empty();
                    ui.add_enabled_ui(send_enabled, |ui| {
                        if ui.button("Send").clicked() {
                            let input = std::mem::take(&mut self.input_text);
                            self.process_user_input(input);
                        }
                    });
                    
                    // Stop button - always visible but only enabled when processing
                    if self.processing_message {
                        if ui.button("🛑 Stop").clicked() {
                            // Cancel current processing
                            self.processing_message = false;
                            self.add_message(Message::new_with_agent(
                                MessageRole::System,
                                "Processing stopped by user.".to_string(),
                                "System".to_string(),
                            ));
                        }
                    } else {
                        ui.add_enabled_ui(false, |ui| {
                            let _ = ui.button("🛑 Stop");
                        });
                    }
                    
                    // Clear input button
                    if ui.button("🗑 Clear").clicked() {
                        self.input_text.clear();
                    }
                });
                
                // Handle Enter to send, Shift+Enter for new line
                if response.has_focus() {
                    if ctx.input(|i| i.key_pressed(egui::Key::Enter) && !i.modifiers.shift) {
                        // Enter without shift: send message
                        let input = std::mem::take(&mut self.input_text);
                        self.process_user_input(input);
                    }
                    // Shift+Enter is handled automatically by TextEdit::multiline for new lines
                }
            });
        });
        
        // Debug panel (optional)
        if self.show_debug_panel {
            egui::SidePanel::left("debug_panel")
                .default_width(300.0)
                .show(ctx, |ui| {
                    self.render_debug_panel(ui);
                });
        }
        
        // Main message window
        egui::CentralPanel::default().show(ctx, |ui| {
            let scroll_area = ScrollArea::vertical()
                .id_source("message_scroll")
                .auto_shrink([false; 2])
                .stick_to_bottom(self.scroll_to_bottom);
                
            let scroll_response = scroll_area.show(ui, |ui| {
                if self.messages.is_empty() {
                    ui.centered_and_justified(|ui| {
                        ui.label("No messages yet. Start typing below!");
                    });
                    return;
                }
                
                // Collect messages to render to avoid borrow checker issues
                let messages_to_render: Vec<(usize, Message, bool)> = self.messages.iter().enumerate()
                    .filter_map(|(i, message)| {
                        let should_render = match message.role {
                            MessageRole::JsonRequest | MessageRole::JsonResponse => self.show_json_debug,
                            _ => true,
                        };
                        if should_render {
                            Some((i, message.clone(), i < self.messages.len() - 1))
                        } else {
                            None
                        }
                    })
                    .collect();
                
                // Render the collected messages
                for (i, message, add_space) in messages_to_render {
                    self.render_message(ui, &message, i);
                    if add_space {
                        ui.add_space(8.0);
                    }
                }
                
                // Show processing spinner as a temporary message node when processing
                if self.processing_message {
                    ui.add_space(8.0);
                    ui.horizontal(|ui| {
                        ui.spinner();
                        ui.label(RichText::new("⚡ ConversationAgent is thinking...").color(Color32::GRAY));
                    });
                }
            });
            
            // Reset expand/collapse flags after they've been applied to all messages
            if self.force_expand_all {
                self.force_expand_all = false;
            }
            if self.force_collapse_all {
                self.force_collapse_all = false;
            }
            
            // Reset the scroll flag only if we successfully scrolled or if auto-scroll is disabled
            if self.scroll_to_bottom {
                // Check if we actually scrolled to bottom by examining scroll response
                let scrolled_to_bottom = scroll_response.state.offset.y >= scroll_response.content_size.y - scroll_response.inner_rect.height() - 1.0;
                if scrolled_to_bottom || !self.auto_scroll {
                    self.scroll_to_bottom = false;
                } else {
                    // Keep the flag set and request another repaint to try again
                    ctx.request_repaint();
                }
            }
        });
    }
}

impl EnterprisePromptBuilderApp {
    fn render_message(&mut self, ui: &mut egui::Ui, message: &Message, index: usize) {
        let agent_prefix = if let Some(ref agent_source) = message.agent_source {
            format!("({}) ", agent_source)
        } else {
            String::new()
        };
        
        let local_time = message.timestamp.with_timezone(&Local);
        let header_text = format!(
            "{} {} - {}{}",
            message.role.icon(),
            local_time.format("%H:%M:%S"),
            agent_prefix,
            message.summary.as_ref().unwrap_or(&"Message".to_string())
        );
        
        // (Removed unused is_assistant variable)
        
        let mut header = CollapsingHeader::new(RichText::new(header_text).color(message.role.color(self.dark_mode)))
            .id_source(&message.id);
            
        // Determine open state based on force flags, persistent state, or defaults
        if self.force_expand_all {
            header = header.open(Some(true));
            self.message_expand_states.insert(message.id.clone(), true);
        } else if self.force_collapse_all {
            header = header.open(Some(false));
            self.message_expand_states.insert(message.id.clone(), false);
        } else if let Some(&is_open) = self.message_expand_states.get(&message.id) {
            // Use persistent state if available
            header = header.open(Some(is_open));
        } else {
            // Assistant messages start open by default, JSON messages always start closed
            let default_open = match message.role {
                MessageRole::Assistant => true,
                MessageRole::JsonRequest | MessageRole::JsonResponse => false, // JSON always closed by default
                _ => false,
            };
            header = header.default_open(default_open);
            self.message_expand_states.insert(message.id.clone(), default_open);
        }
        
        let header_response = header
            .show(ui, |ui| {
                // JSON messages get special formatting
                let is_json_message = matches!(message.role, MessageRole::JsonRequest | MessageRole::JsonResponse);
                
                if is_json_message {
                    // JSON content with syntax highlighting via color
                    ui.style_mut().visuals.override_text_color = Some(message.role.color(self.dark_mode));
                }
                
                // Message content - split by lines for better display
                for line in message.content.lines() {
                    if line.trim().is_empty() {
                        ui.add_space(6.0); // Zen whitespace instead of separator
                    } else {
                        ui.monospace(line);
                    }
                }
                
                // Reset text color override
                if is_json_message {
                    ui.style_mut().visuals.override_text_color = None;
                }
                
                // Debug information if available and debug mode is on
                if self.debug_mode && message.debug_info.is_some() {
                    ui.add_space(8.0); // Zen whitespace instead of separator
                    ui.label("Debug Information:");
                    if let Some(debug_info) = &message.debug_info {
                        ui.monospace(debug_info);
                    }
                }
                
                // Nested messages
                if !message.nested_messages.is_empty() {
                    ui.add_space(8.0); // Zen whitespace instead of separator
                    ui.label(format!("Nested Messages ({})", message.nested_messages.len()));
                    for (j, nested_msg) in message.nested_messages.iter().enumerate() {
                        ui.indent(format!("nested_{}_{}", message.id, j), |ui| {
                            self.render_message(ui, nested_msg, index * 1000 + j);
                        });
                    }
                }
            });
        
        // Capture user interactions with individual message expand/collapse
        if header_response.header_response.clicked() {
            // User manually clicked the header - toggle the state
            let current_state = self.message_expand_states.get(&message.id).copied().unwrap_or(false);
            self.message_expand_states.insert(message.id.clone(), !current_state);
            debug!("User manually toggled message {}: {} -> {}", message.id, current_state, !current_state);
        }
    }
    
    fn update_message_stats(&mut self) {
        let mut user_count = 0;
        let mut assistant_count = 0;
        let mut system_count = 0;
        let mut debug_count = 0;
        
        for message in &self.messages {
            match message.role {
                MessageRole::User => user_count += 1,
                MessageRole::Assistant => assistant_count += 1,
                MessageRole::System => system_count += 1,
                MessageRole::Debug => debug_count += 1,
                MessageRole::JsonRequest | MessageRole::JsonResponse => debug_count += 1, // Count JSON as debug messages
            }
        }
        
        // Only log if stats actually changed
        if self.message_count_stats != (user_count, assistant_count, system_count, debug_count) {
            debug!("Message stats updated: User={}, Assistant={}, System={}, Debug={}", 
                   user_count, assistant_count, system_count, debug_count);
            self.message_count_stats = (user_count, assistant_count, system_count, debug_count);
        }
    }
    
    fn render_debug_panel(&mut self, ui: &mut egui::Ui) {
        ui.heading("🔧 Debug Panel");
        ui.add_space(8.0);
        
        // Processing Status
        ui.label(egui::RichText::new("Processing Status").strong());
        ui.monospace(format!("Processing: {}", if self.processing_message { "🔄 Active" } else { "⏸️ Idle" }));
        
        if let Some(duration) = self.last_processing_time {
            ui.monospace(format!("Last processing: {:?}", duration));
        }
        
        ui.add_space(8.0);
        
        // Message Statistics
        ui.label(egui::RichText::new("Message Statistics").strong());
        let (user, assistant, system, debug) = self.message_count_stats;
        ui.monospace(format!("👤 User: {}", user));
        ui.monospace(format!("⚡ Assistant: {}", assistant));
        ui.monospace(format!("ℹ System: {}", system));
        ui.monospace(format!("🔧 Debug: {}", debug));
        ui.monospace(format!("📊 Total: {}", self.messages.len()));
        
        ui.add_space(8.0);
        
        // UI State
        ui.label(egui::RichText::new("UI State").strong());
        ui.monospace(format!("Theme: {}", if self.dark_mode { "🌙 Dark" } else { "☀️ Light" }));
        ui.monospace(format!("Auto-scroll: {}", if self.auto_scroll { "✅ On" } else { "❌ Off" }));
        ui.monospace(format!("Debug mode: {}", if self.debug_mode { "✅ On" } else { "❌ Off" }));
        ui.monospace(format!("JSON debug: {}", if self.show_json_debug { "✅ On" } else { "❌ Off" }));
        
        ui.add_space(8.0);
        
        // Agent State
        ui.label(egui::RichText::new("Agent State").strong());
        ui.monospace("ConversationAgent: 🟢 Ready");
        ui.monospace("Model: Claude 3.5 Haiku");
        ui.monospace("Tools: coordinate_prompt_building");
        
        ui.add_space(8.0);
        
        // Telemetry Status
        ui.label(egui::RichText::new("Telemetry Status").strong());
        ui.monospace("Service: promptapp-conversation-agent");
        ui.monospace("OTEL Endpoint: localhost:4319 ✅");
        ui.monospace("Spans: Auto-instrumented by Stood");
        ui.monospace("Jaeger UI: localhost:16686");
        
        ui.add_space(8.0);
        
        // Controls
        ui.label(egui::RichText::new("Debug Controls").strong());
        if ui.button("📊 Refresh Stats").clicked() {
            self.update_message_stats();
            info!("Debug stats manually refreshed");
        }
        
        if ui.button("🧹 Force UI Repaint").clicked() {
            debug!("Manual UI repaint requested");
        }

        if ui.button("🔍 Open Jaeger UI").clicked() {
            info!("Opening Jaeger UI in browser");
            #[cfg(target_os = "macos")]
            let _ = std::process::Command::new("open").arg("http://localhost:16686").spawn();
            #[cfg(target_os = "linux")]
            let _ = std::process::Command::new("xdg-open").arg("http://localhost:16686").spawn();
            #[cfg(target_os = "windows")]
            let _ = std::process::Command::new("start").arg("http://localhost:16686").spawn();
        }
        
        ui.add_space(8.0);
        
        // Recent Debug Info
        ui.label(egui::RichText::new("Recent Activity").strong());
        ui.monospace("See terminal for full debug logs");
        if self.processing_message {
            ui.monospace("🔄 Processing user input...");
        } else if !self.messages.is_empty() {
            if let Some(last_msg) = self.messages.back() {
                ui.monospace(format!("Last: {:?} from {:?}", 
                    last_msg.role, 
                    last_msg.agent_source.as_deref().unwrap_or("unknown")));
            }
        }
    }
    
    fn update_debug_display(&mut self) {
        // Add debug/JSON info to existing messages based on current settings
        for message in &mut self.messages {
            if self.debug_mode && message.debug_info.is_none() {
                // Add debug information for messages that don't have it
                message.debug_info = Some(format!(
                    "Message ID: {}\nRole: {:?}\nLength: {} chars\nTimestamp: {}", 
                    format!("msg_{}", message.timestamp.timestamp()), 
                    message.role,
                    message.content.len(),
                    message.timestamp.with_timezone(&Local).format("%Y-%m-%d %H:%M:%S")
                ));
            } else if !self.debug_mode {
                // Remove debug info if debug mode is turned off
                message.debug_info = None;
            }
            
            // Handle JSON debug information - show both message metadata and model interaction JSON
            if self.show_json_debug {
                let basic_json_info = format!(
                    "Message JSON:\n{{\n  \"role\": \"{:?}\",\n  \"timestamp\": \"{}\",\n  \"content_length\": {},\n  \"agent_source\": \"{}\"\n}}", 
                    message.role,
                    message.timestamp.to_rfc3339(),
                    message.content.len(),
                    message.agent_source.as_deref().unwrap_or("unknown")
                );
                
                if let Some(ref mut debug_info) = message.debug_info {
                    if !debug_info.contains("Message JSON:") {
                        debug_info.push_str(&format!("\n\n{}", basic_json_info));
                    }
                } else if self.debug_mode {
                    message.debug_info = Some(basic_json_info);
                } else {
                    // If only JSON debug is on but not debug mode, still show JSON
                    message.debug_info = Some(basic_json_info);
                }
            }
        }
    }
}

/// Custom callback handler that captures model interaction JSON data
#[derive(Debug)]
pub struct JsonCaptureHandler {
    sender: mpsc::Sender<AgentResponse>,
}

impl JsonCaptureHandler {
    pub fn new(sender: mpsc::Sender<AgentResponse>) -> Self {
        Self { sender }
    }
}

#[async_trait]
impl CallbackHandler for JsonCaptureHandler {
    async fn handle_event(&self, event: CallbackEvent) -> Result<(), CallbackError> {
        match event {
            CallbackEvent::ModelStart { provider, model_id, messages, tools_available } => {
                debug!("📤 Capturing model request JSON");
                
                // Create JSON representation of the request
                let request_json = serde_json::json!({
                    "type": "model_request",
                    "provider": format!("{:?}", provider),
                    "model_id": model_id,
                    "timestamp": Utc::now().to_rfc3339(),
                    "messages": messages,
                    "tools_available": tools_available,
                });
                
                let json_data = JsonDebugData {
                    json_type: JsonDebugType::Request,
                    json_content: serde_json::to_string_pretty(&request_json)
                        .unwrap_or_else(|_| "Error serializing request JSON".to_string()),
                    timestamp: Utc::now(),
                };
                
                // Send to UI thread
                if let Err(e) = self.sender.send(AgentResponse::JsonDebug(json_data)) {
                    error!("Failed to send JSON request data to UI: {}", e);
                }
            }
            CallbackEvent::ModelComplete { response, stop_reason, duration, tokens } => {
                debug!("📥 Capturing model response JSON");
                
                // Create JSON representation of the response
                let response_json = serde_json::json!({
                    "type": "model_response",
                    "timestamp": Utc::now().to_rfc3339(),
                    "response": response,
                    "stop_reason": format!("{:?}", stop_reason),
                    "duration_ms": duration.as_millis(),
                    "tokens": tokens.map(|t| serde_json::json!({
                        "input_tokens": t.input_tokens,
                        "output_tokens": t.output_tokens,
                        "total_tokens": t.total_tokens,
                    })),
                });
                
                let json_data = JsonDebugData {
                    json_type: JsonDebugType::Response,
                    json_content: serde_json::to_string_pretty(&response_json)
                        .unwrap_or_else(|_| "Error serializing response JSON".to_string()),
                    timestamp: Utc::now(),
                };
                
                // Send to UI thread
                if let Err(e) = self.sender.send(AgentResponse::JsonDebug(json_data)) {
                    error!("Failed to send JSON response data to UI: {}", e);
                }
            }
            _ => {
                // Ignore other events - we only care about model interactions
            }
        }
        Ok(())
    }
}


// ============================================================================
// SIMPLIFIED TOOLS
// ============================================================================

/// Tool for structured enterprise prompt building with internal coordinator state
#[tool]
/// Coordinate the enterprise prompt building process through structured conversation
async fn coordinate_prompt_building(message: String) -> Result<String, String> {
    // Simplified coordination - in a full implementation, this would manage
    // the complex coordinator state and task agents
    Ok(format!("Coordinating prompt building for: {}", message))
}

// ============================================================================
// MAIN FUNCTION
// ============================================================================

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Configure OTEL environment variables for existing collector
    std::env::set_var("OTEL_EXPORTER_OTLP_ENDPOINT", "http://localhost:4319");
    std::env::set_var("OTEL_SERVICE_NAME", "promptapp");
    std::env::set_var("OTEL_SERVICE_VERSION", "1.0.0");
    std::env::set_var("OTEL_RESOURCE_ATTRIBUTES", "environment=development,team=ai-agents,example=024");
    
    // Initialize comprehensive DEBUG logging to stderr
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::from_default_env()
                .add_directive("debug".parse().unwrap())
                .add_directive("stood=debug".parse().unwrap())
                .add_directive("enterprise_prompt_builder=trace".parse().unwrap())
        )
        .with_writer(std::io::stderr)
        .with_ansi(true)
        .with_file(true)
        .with_line_number(true)
        .with_thread_ids(true)
        .with_target(true)
        .init();
    
    info!("🚀 Starting Enterprise Prompt Builder with DEBUG logging + OTEL telemetry");
    info!("📊 OTEL configured: endpoint=localhost:4319, service=promptapp");
    info!("🔍 Jaeger UI available at: http://localhost:16686");
    debug!("Debug logging enabled for comprehensive tracing");
    
    // Check AWS credentials
    let has_aws = std::env::var("AWS_ACCESS_KEY_ID").is_ok()
        || std::env::var("AWS_PROFILE").is_ok()
        || std::env::var("AWS_ROLE_ARN").is_ok();

    if !has_aws {
        eprintln!("⚠️ No AWS credentials found.");
        eprintln!("GUI will start anyway - agent creation will be attempted on first message.");
        eprintln!("To set up AWS credentials:");
        eprintln!("   • Set AWS_ACCESS_KEY_ID and AWS_SECRET_ACCESS_KEY");
        eprintln!("   • Or set AWS_PROFILE to use AWS credentials file");
        eprintln!("   • Or configure IAM role with AWS_ROLE_ARN");
    }

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1200.0, 800.0])
            .with_title("Enterprise Prompt Builder"),
        ..Default::default()
    };

    eframe::run_native(
        "Enterprise Prompt Builder",
        options,
        Box::new(|cc| Ok(Box::new(EnterprisePromptBuilderApp::new(cc)))),
    )
    .map_err(|e| Box::new(e) as Box<dyn std::error::Error>)
}

