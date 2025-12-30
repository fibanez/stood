//! Session management for multi-turn conversations
//!
//! This module provides session and conversation tracking for agent interactions.
//! Sessions group related spans under a common conversation ID.

use super::genai::attrs;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use uuid::Uuid;

/// A conversation session for grouping related spans
///
/// Sessions track multi-turn conversations and provide consistent
/// conversation IDs across spans.
#[derive(Debug, Clone)]
pub struct Session {
    /// Unique session identifier
    id: String,
    /// Optional conversation ID (may be provided by user)
    conversation_id: String,
    /// Agent name associated with this session
    agent_name: Option<String>,
    /// Agent ID associated with this session
    agent_id: Option<String>,
    /// Session metadata
    metadata: HashMap<String, String>,
    /// Number of turns in this session
    turn_count: u32,
    /// Creation timestamp (Unix epoch millis)
    created_at: u64,
}

impl Session {
    /// Create a new session
    pub fn new() -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            conversation_id: Uuid::new_v4().to_string(),
            agent_name: None,
            agent_id: None,
            metadata: HashMap::new(),
            turn_count: 0,
            created_at: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_millis() as u64,
        }
    }

    /// Create a session with a specific conversation ID
    pub fn with_conversation_id(conversation_id: impl Into<String>) -> Self {
        let mut session = Self::new();
        session.conversation_id = conversation_id.into();
        session
    }

    /// Create a session with a specific session ID and conversation ID
    ///
    /// Use this when your application needs to control the session ID
    /// (e.g., to correlate with an external session or revisit traces later).
    ///
    /// # Example
    /// ```
    /// use stood::telemetry::Session;
    ///
    /// // Use your app's session ID for traceability
    /// let session = Session::with_ids("app-session-123", "conversation-456");
    /// ```
    pub fn with_ids(session_id: impl Into<String>, conversation_id: impl Into<String>) -> Self {
        Self {
            id: session_id.into(),
            conversation_id: conversation_id.into(),
            agent_name: None,
            agent_id: None,
            metadata: HashMap::new(),
            turn_count: 0,
            created_at: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_millis() as u64,
        }
    }

    /// Create a session with a specific session ID (conversation_id will match)
    ///
    /// Use this when you want the same ID for both session and conversation.
    pub fn with_session_id(session_id: impl Into<String>) -> Self {
        let id = session_id.into();
        Self::with_ids(id.clone(), id)
    }

    /// Get the session ID
    pub fn id(&self) -> &str {
        &self.id
    }

    /// Get the conversation ID
    pub fn conversation_id(&self) -> &str {
        &self.conversation_id
    }

    /// Set the agent name
    pub fn set_agent_name(&mut self, name: impl Into<String>) {
        self.agent_name = Some(name.into());
    }

    /// Get the agent name
    pub fn agent_name(&self) -> Option<&str> {
        self.agent_name.as_deref()
    }

    /// Set the agent ID
    pub fn set_agent_id(&mut self, id: impl Into<String>) {
        self.agent_id = Some(id.into());
    }

    /// Get the agent ID
    pub fn agent_id(&self) -> Option<&str> {
        self.agent_id.as_deref()
    }

    /// Add metadata to the session
    pub fn set_metadata(&mut self, key: impl Into<String>, value: impl Into<String>) {
        self.metadata.insert(key.into(), value.into());
    }

    /// Get session metadata
    pub fn metadata(&self) -> &HashMap<String, String> {
        &self.metadata
    }

    /// Get specific metadata value
    pub fn get_metadata(&self, key: &str) -> Option<&str> {
        self.metadata.get(key).map(|s| s.as_str())
    }

    /// Increment the turn count and return the new value
    pub fn next_turn(&mut self) -> u32 {
        self.turn_count += 1;
        self.turn_count
    }

    /// Get the current turn count
    pub fn turn_count(&self) -> u32 {
        self.turn_count
    }

    /// Get creation timestamp
    pub fn created_at(&self) -> u64 {
        self.created_at
    }

    /// Get span attributes for this session
    pub fn as_attributes(&self) -> Vec<(&'static str, String)> {
        let mut attrs_vec = vec![(attrs::CONVERSATION_ID, self.conversation_id.clone())];

        if let Some(ref name) = self.agent_name {
            attrs_vec.push((attrs::AGENT_NAME, name.clone()));
        }
        if let Some(ref id) = self.agent_id {
            attrs_vec.push((attrs::AGENT_ID, id.clone()));
        }

        attrs_vec
    }
}

impl Default for Session {
    fn default() -> Self {
        Self::new()
    }
}

/// Manager for tracking multiple sessions
///
/// The session manager maintains active sessions and provides
/// lookup by conversation ID.
#[derive(Debug, Clone)]
pub struct SessionManager {
    sessions: Arc<Mutex<HashMap<String, Session>>>,
    /// Maximum number of sessions to keep (LRU eviction)
    max_sessions: usize,
}

impl SessionManager {
    /// Create a new session manager
    pub fn new() -> Self {
        Self {
            sessions: Arc::new(Mutex::new(HashMap::new())),
            max_sessions: 1000,
        }
    }

    /// Create a session manager with a custom max sessions limit
    pub fn with_max_sessions(max_sessions: usize) -> Self {
        Self {
            sessions: Arc::new(Mutex::new(HashMap::new())),
            max_sessions,
        }
    }

    /// Create a new session and register it
    pub fn create_session(&self) -> Session {
        let session = Session::new();
        self.register_session(session.clone());
        session
    }

    /// Create a session with a specific conversation ID
    pub fn create_session_with_id(&self, conversation_id: impl Into<String>) -> Session {
        let session = Session::with_conversation_id(conversation_id);
        self.register_session(session.clone());
        session
    }

    /// Register an existing session
    pub fn register_session(&self, session: Session) {
        let mut sessions = self.sessions.lock().unwrap();

        // Simple eviction if at capacity
        if sessions.len() >= self.max_sessions {
            // Remove oldest session (by creation time)
            if let Some(oldest_id) = sessions
                .iter()
                .min_by_key(|(_, s)| s.created_at)
                .map(|(id, _)| id.clone())
            {
                sessions.remove(&oldest_id);
            }
        }

        sessions.insert(session.conversation_id().to_string(), session);
    }

    /// Get a session by conversation ID
    pub fn get_session(&self, conversation_id: &str) -> Option<Session> {
        let sessions = self.sessions.lock().unwrap();
        sessions.get(conversation_id).cloned()
    }

    /// Get or create a session for a conversation ID
    pub fn get_or_create_session(&self, conversation_id: &str) -> Session {
        if let Some(session) = self.get_session(conversation_id) {
            session
        } else {
            self.create_session_with_id(conversation_id)
        }
    }

    /// Update a session
    pub fn update_session(&self, session: Session) {
        let mut sessions = self.sessions.lock().unwrap();
        sessions.insert(session.conversation_id().to_string(), session);
    }

    /// Remove a session
    pub fn remove_session(&self, conversation_id: &str) -> Option<Session> {
        let mut sessions = self.sessions.lock().unwrap();
        sessions.remove(conversation_id)
    }

    /// Get the number of active sessions
    pub fn session_count(&self) -> usize {
        let sessions = self.sessions.lock().unwrap();
        sessions.len()
    }

    /// Clear all sessions
    pub fn clear(&self) {
        let mut sessions = self.sessions.lock().unwrap();
        sessions.clear();
    }
}

impl Default for SessionManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_session_creation() {
        let session = Session::new();
        assert!(!session.id().is_empty());
        assert!(!session.conversation_id().is_empty());
        assert_eq!(session.turn_count(), 0);
    }

    #[test]
    fn test_session_with_conversation_id() {
        let session = Session::with_conversation_id("my-conversation-123");
        assert_eq!(session.conversation_id(), "my-conversation-123");
    }

    #[test]
    fn test_session_turn_tracking() {
        let mut session = Session::new();
        assert_eq!(session.turn_count(), 0);

        assert_eq!(session.next_turn(), 1);
        assert_eq!(session.next_turn(), 2);
        assert_eq!(session.next_turn(), 3);
        assert_eq!(session.turn_count(), 3);
    }

    #[test]
    fn test_session_agent_info() {
        let mut session = Session::new();
        session.set_agent_name("my-agent");
        session.set_agent_id("agent-456");

        assert_eq!(session.agent_name(), Some("my-agent"));
        assert_eq!(session.agent_id(), Some("agent-456"));
    }

    #[test]
    fn test_session_metadata() {
        let mut session = Session::new();
        session.set_metadata("user_id", "user-123");
        session.set_metadata("source", "api");

        assert_eq!(session.get_metadata("user_id"), Some("user-123"));
        assert_eq!(session.get_metadata("source"), Some("api"));
        assert_eq!(session.get_metadata("missing"), None);
    }

    #[test]
    fn test_session_as_attributes() {
        let mut session = Session::new();
        session.set_agent_name("test-agent");
        session.set_agent_id("agent-789");

        let attrs = session.as_attributes();
        assert!(attrs.iter().any(|(k, _)| *k == attrs::CONVERSATION_ID));
        assert!(attrs.iter().any(|(k, _)| *k == attrs::AGENT_NAME));
        assert!(attrs.iter().any(|(k, _)| *k == attrs::AGENT_ID));
    }

    #[test]
    fn test_session_manager_create() {
        let manager = SessionManager::new();
        let session = manager.create_session();

        assert!(manager.get_session(session.conversation_id()).is_some());
        assert_eq!(manager.session_count(), 1);
    }

    #[test]
    fn test_session_manager_get_or_create() {
        let manager = SessionManager::new();

        // First call creates
        let session1 = manager.get_or_create_session("conv-123");
        assert_eq!(session1.conversation_id(), "conv-123");
        assert_eq!(manager.session_count(), 1);

        // Second call retrieves
        let session2 = manager.get_or_create_session("conv-123");
        assert_eq!(session2.id(), session1.id());
        assert_eq!(manager.session_count(), 1);
    }

    #[test]
    fn test_session_manager_remove() {
        let manager = SessionManager::new();
        let session = manager.create_session();
        let conv_id = session.conversation_id().to_string();

        assert_eq!(manager.session_count(), 1);
        let removed = manager.remove_session(&conv_id);
        assert!(removed.is_some());
        assert_eq!(manager.session_count(), 0);
    }

    #[test]
    fn test_session_manager_eviction() {
        let manager = SessionManager::with_max_sessions(3);

        // Create 4 sessions, first one should be evicted
        let _s1 = manager.create_session_with_id("conv-1");
        std::thread::sleep(std::time::Duration::from_millis(1)); // Ensure different timestamps
        let _s2 = manager.create_session_with_id("conv-2");
        std::thread::sleep(std::time::Duration::from_millis(1));
        let _s3 = manager.create_session_with_id("conv-3");
        std::thread::sleep(std::time::Duration::from_millis(1));
        let _s4 = manager.create_session_with_id("conv-4");

        assert_eq!(manager.session_count(), 3);
        // conv-1 should have been evicted (oldest)
        assert!(manager.get_session("conv-1").is_none());
        assert!(manager.get_session("conv-4").is_some());
    }

    #[test]
    fn test_session_manager_clear() {
        let manager = SessionManager::new();
        manager.create_session();
        manager.create_session();
        manager.create_session();

        assert_eq!(manager.session_count(), 3);
        manager.clear();
        assert_eq!(manager.session_count(), 0);
    }
}
