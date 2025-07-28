//! Core type definitions for the Stood agent library.
//!
//! This module contains the fundamental data structures used throughout
//! the library for agent communication, tool execution, and response handling.

use uuid::Uuid;

pub mod agent;
pub mod content;
pub mod messages;
pub mod tools;

// Re-export commonly used types
pub use agent::*;
pub use content::*;
pub use messages::*;
pub use tools::*;

/// A unique identifier for tracking conversations, tool calls, and events
pub type RequestId = Uuid;

/// A unique identifier for agent instances
pub type AgentId = String;

/// Generic result type for the Stood library
pub type StoodResult<T> = Result<T, crate::StoodError>;
