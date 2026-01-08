//! Logging utilities with credential protection
//!
//! This module provides utilities to safely log information while protecting
//! sensitive credentials from accidental exposure in logs.

/// Obscures a credential string by showing only the first few characters
///
/// This function helps prevent accidental credential exposure in logs by
/// showing only the first 5 characters followed by asterisks.
///
/// # Examples
///
/// ```rust
/// use stood::utils::logging::obscure_credential;
///
/// let credential = "AKIA2PP6SBMCSVNYUNVK";
/// let obscured = obscure_credential(credential);
/// assert_eq!(obscured, "AKIA2***");
/// ```
pub fn obscure_credential(credential: &str) -> String {
    let char_count = credential.chars().count();
    if char_count <= 5 {
        "*".repeat(char_count)
    } else {
        format!("{}***", truncate_string(credential, 5))
    }
}

/// Safely logs an AWS access key ID by obscuring most of the key
///
/// This is specifically designed for AWS access key IDs which typically
/// start with "AKIA" or "ASIA".
///
/// # Examples
///
/// ```rust
/// use stood::utils::logging::safe_log_aws_key;
///
/// let key_id = "AKIA2PP6SBMCSVNYUNVK";
/// tracing::info!("Using access key: {}", safe_log_aws_key(key_id));
/// // Logs: "Using access key: AKIA2***"
/// ```
pub fn safe_log_aws_key(access_key_id: &str) -> String {
    obscure_credential(access_key_id)
}

/// Safely truncates a string to a maximum number of characters, respecting UTF-8 boundaries
///
/// This function prevents panics when truncating strings that contain multi-byte UTF-8 characters
/// like emojis. It truncates at character boundaries rather than byte boundaries.
///
/// # Arguments
///
/// * `s` - The string to truncate
/// * `max_chars` - Maximum number of characters (not bytes) to keep
///
/// # Examples
///
/// ```rust
/// use stood::utils::logging::truncate_string;
///
/// let text = "Hello 👋 World!";
/// assert_eq!(truncate_string(text, 10), "Hello 👋 Wo");
///
/// let text = "Short";
/// assert_eq!(truncate_string(text, 100), "Short");
/// ```
pub fn truncate_string(s: &str, max_chars: usize) -> String {
    s.chars().take(max_chars).collect()
}

/// Detects if a string looks like a credential and obscures it
///
/// This function attempts to detect various credential patterns and obscure them.
/// It's useful for sanitizing log output that might contain credentials.
pub fn sanitize_for_logging(input: &str) -> String {
    // Note: Future enhancement could use regex patterns for more comprehensive detection

    let mut result = input.to_string();

    // Simple pattern matching without regex dependency for now
    // AWS Access Keys pattern
    if let Some(start) = result.find("AKIA") {
        if result.len() >= start + 20 {
            let end = start + 20;
            let replacement = obscure_credential(&result[start..end]);
            result.replace_range(start..end, &replacement);
        }
    }

    if let Some(start) = result.find("ASIA") {
        if result.len() >= start + 20 {
            let end = start + 20;
            let replacement = obscure_credential(&result[start..end]);
            result.replace_range(start..end, &replacement);
        }
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_obscure_credential() {
        assert_eq!(obscure_credential("AKIA2PP6SBMCSVNYUNVK"), "AKIA2***");
        assert_eq!(obscure_credential("short"), "*****");
        assert_eq!(obscure_credential(""), "");
        assert_eq!(obscure_credential("a"), "*");
    }

    #[test]
    fn test_safe_log_aws_key() {
        let key = "AKIA2PP6SBMCSVNYUNVK";
        assert_eq!(safe_log_aws_key(key), "AKIA2***");
    }

    #[test]
    fn test_sanitize_for_logging() {
        let input = "Using access_key_id: AKIA2PP6SBMCSVNYUNVK for requests";
        let sanitized = sanitize_for_logging(input);
        assert!(sanitized.contains("AKIA2***"));
        assert!(!sanitized.contains("AKIA2PP6SBMCSVNYUNVK"));
    }

    #[test]
    fn test_truncate_string() {
        // Test basic truncation
        assert_eq!(truncate_string("Hello World", 5), "Hello");

        // Test with emojis (multi-byte characters)
        assert_eq!(truncate_string("Hello 👋 World!", 10), "Hello 👋 Wo");
        assert_eq!(truncate_string("👋👋👋👋👋", 3), "👋👋👋");

        // Test string shorter than limit
        assert_eq!(truncate_string("Short", 100), "Short");

        // Test empty string
        assert_eq!(truncate_string("", 10), "");

        // Test with various multi-byte characters
        assert_eq!(truncate_string("日本語テキスト", 3), "日本語");
    }

    #[test]
    fn test_obscure_credential_with_emojis() {
        // Test that obscure_credential handles multi-byte characters safely
        assert_eq!(obscure_credential("👋👋👋👋👋👋"), "👋👋👋👋👋***");
        assert_eq!(obscure_credential("😀😃😄"), "***");
    }
}
