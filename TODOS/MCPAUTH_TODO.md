# MCP OAuth 2.1 Authentication Implementation Plan

## Overview

This document outlines the implementation plan for adding OAuth 2.1 authentication to the Stood MCP client library, based on the official Model Context Protocol specification and security best practices.

## Official Documentation References

### Primary Sources
- **MCP Authorization Specification**: https://modelcontextprotocol.io/specification/draft/basic/authorization
- **MCP Security Best Practices**: https://modelcontextprotocol.io/specification/draft/basic/security_best_practices  
- **Main MCP Specification**: https://modelcontextprotocol.io/specification/2025-03-26
- **MCP Introduction**: https://modelcontextprotocol.io/introduction

### Key Standards Referenced
- OAuth 2.1 IETF Draft Specification
- RFC 8414 - OAuth 2.0 Authorization Server Metadata
- RFC 9728 - OAuth 2.0 Protected Resource Metadata
- OAuth 2.0 Dynamic Client Registration Protocol

## Current State Analysis

### Existing MCP Infrastructure in Stood

**Transport Layer** (`src/mcp/transport.rs`):
- ✅ **WebSocket Transport**: Supports custom headers including `Authorization`
- ✅ **Stdio Transport**: Local process communication (no auth needed)
- ✅ **Transport Abstraction**: `MCPTransport` trait supports extension
- ⚠️ **HTTP+SSE Transport**: Not implemented (required for full OAuth support)

**Current Auth Support**:
```rust
// Basic header support in WebSocketConfig
pub headers: std::collections::HashMap<String, String>,

// Example usage:
WebSocketConfig {
    headers: [("Authorization".to_string(), "Bearer token".to_string())].into(),
    // ...
}
```

**Client Implementation** (`src/mcp/client.rs`):
- ✅ Session management and capability negotiation
- ✅ Request/response handling with timeout support
- ✅ Tool discovery and execution
- ❌ No OAuth flow implementation
- ❌ No token refresh mechanisms

**Missing Infrastructure**:
- HTTP+SSE transport implementation
- OAuth 2.1 client implementation
- Token storage and refresh logic
- Authorization server discovery
- Dynamic client registration

## MCP OAuth 2.1 Requirements

### Core Requirements from Specification

1. **OAuth 2.1 Compliance**
   - MUST implement OAuth 2.1 draft specification
   - MUST support PKCE (Proof Key for Code Exchange)
   - MUST validate exact redirect URIs
   - MUST use HTTPS for all authorization endpoints

2. **Server Discovery**
   - MUST implement OAuth 2.0 Protected Resource Metadata (RFC 9728)
   - MUST use OAuth 2.0 Authorization Server Metadata (RFC 8414) for discovery
   - SHOULD support OAuth 2.0 Dynamic Client Registration Protocol

3. **Token Management**
   - Access tokens MUST be included in Authorization header
   - Tokens MUST NOT be included in URI query string
   - MUST validate token audience to prevent confused deputy attacks
   - SHOULD issue short-lived access tokens
   - MUST rotate refresh tokens for public clients

4. **Security Requirements**
   - All authorization endpoints MUST be served over HTTPS
   - MUST prevent token confusion attacks
   - MUST obtain user consent for each dynamically registered client (for proxy servers)
   - MUST NOT accept tokens not explicitly issued for the MCP server

5. **Error Handling**
   - 401 Unauthorized: Token invalid/required → initiate OAuth flow
   - 403 Forbidden: Insufficient permissions
   - 400 Bad Request: Malformed authorization request

## Implementation Plan

### Phase 1: HTTP+SSE Transport Foundation

**Priority: HIGH** - Required for OAuth-enabled MCP servers

**Tasks:**
1. **Create `HttpSseTransport` struct**
   - Implement `MCPTransport` trait
   - HTTP client for sending requests
   - SSE client for receiving real-time messages
   - Message serialization/deserialization

2. **HTTP+SSE Configuration**
   ```rust
   pub struct HttpSseConfig {
       pub base_url: String,
       pub headers: HashMap<String, String>,
       pub connect_timeout_ms: u64,
       pub request_timeout_ms: u64,
       pub max_message_size: Option<usize>,
   }
   ```

3. **Integration with TransportFactory**
   ```rust
   impl TransportFactory {
       pub fn http_sse(config: HttpSseConfig) -> Box<dyn MCPTransport> {
           Box::new(HttpSseTransport::new(config))
       }
   }
   ```

**Files to Create/Modify:**
- `src/mcp/transport.rs` - Add HttpSseTransport implementation
- `Cargo.toml` - Add HTTP/SSE dependencies (reqwest, eventsource-stream)

**Estimated Effort:** 2-3 days

### Phase 2: OAuth 2.1 Core Infrastructure

**Priority: HIGH** - Core authentication framework

**Tasks:**
1. **OAuth Configuration**
   ```rust
   pub struct OAuthConfig {
       pub client_id: String,
       pub client_secret: Option<String>, // None for public clients
       pub authorization_server: String,
       pub redirect_uri: String,
       pub scope: Vec<String>,
       pub pkce_enabled: bool, // MUST be true
   }
   ```

2. **Token Storage**
   ```rust
   pub struct TokenStore {
       access_token: Option<String>,
       refresh_token: Option<String>,
       expires_at: Option<SystemTime>,
       token_type: String,
       scope: Vec<String>,
   }
   ```

3. **OAuth Client Implementation**
   ```rust
   pub struct OAuthClient {
       config: OAuthConfig,
       token_store: Arc<Mutex<TokenStore>>,
       http_client: reqwest::Client,
   }
   
   impl OAuthClient {
       pub async fn authorize(&self) -> Result<String, OAuthError>;
       pub async fn exchange_code(&self, code: String, code_verifier: String) -> Result<(), OAuthError>;
       pub async fn refresh_token(&self) -> Result<(), OAuthError>;
       pub async fn get_valid_token(&self) -> Result<String, OAuthError>;
   }
   ```

**Files to Create:**
- `src/mcp/auth/mod.rs` - OAuth module
- `src/mcp/auth/oauth.rs` - OAuth client implementation
- `src/mcp/auth/token_store.rs` - Token storage and management
- `src/mcp/auth/errors.rs` - OAuth-specific error types

**Dependencies to Add:**
```toml
oauth2 = "4.4"
url = "2.4"
base64 = "0.21"
sha2 = "0.10"
rand = "0.8"
```

**Estimated Effort:** 4-5 days

### Phase 3: Server Discovery and Metadata

**Priority: MEDIUM** - Standards compliance

**Tasks:**
1. **Protected Resource Metadata Client**
   ```rust
   pub struct ProtectedResourceMetadata {
       pub resource: String,
       pub authorization_servers: Vec<String>,
       pub scopes_supported: Option<Vec<String>>,
       pub bearer_methods_supported: Option<Vec<String>>,
   }
   
   pub async fn discover_protected_resource(url: &str) -> Result<ProtectedResourceMetadata, DiscoveryError>;
   ```

2. **Authorization Server Metadata Client**
   ```rust
   pub struct AuthorizationServerMetadata {
       pub issuer: String,
       pub authorization_endpoint: String,
       pub token_endpoint: String,
       pub registration_endpoint: Option<String>,
       pub scopes_supported: Option<Vec<String>>,
       pub grant_types_supported: Vec<String>,
       pub pkce_code_challenge_methods_supported: Vec<String>,
   }
   
   pub async fn discover_authorization_server(issuer: &str) -> Result<AuthorizationServerMetadata, DiscoveryError>;
   ```

3. **Dynamic Client Registration**
   ```rust
   pub struct ClientRegistrationRequest {
       pub client_name: String,
       pub redirect_uris: Vec<String>,
       pub grant_types: Vec<String>,
       pub token_endpoint_auth_method: String,
   }
   
   pub async fn register_client(
       registration_endpoint: &str,
       request: ClientRegistrationRequest
   ) -> Result<ClientRegistrationResponse, RegistrationError>;
   ```

**Files to Create:**
- `src/mcp/auth/discovery.rs` - Server discovery implementations
- `src/mcp/auth/registration.rs` - Dynamic client registration

**Estimated Effort:** 2-3 days

### Phase 4: MCP Client Integration

**Priority: HIGH** - Integration with existing client

**Tasks:**
1. **Enhanced MCPClientConfig**
   ```rust
   pub struct MCPClientConfig {
       // ... existing fields ...
       pub oauth_config: Option<OAuthConfig>,
       pub auto_refresh_tokens: bool,
       pub token_refresh_threshold_seconds: u64,
   }
   ```

2. **Automatic Token Management in MCPClient**
   ```rust
   impl MCPClient {
       async fn ensure_valid_token(&self) -> Result<(), MCPOperationError>;
       async fn handle_auth_error(&self, error: &MCPOperationError) -> Result<(), MCPOperationError>;
       
       // Modify existing methods to handle OAuth
       async fn connect_with_oauth(&mut self) -> Result<(), MCPOperationError>;
       async fn send_authenticated_request(&self, request: MCPRequest) -> Result<MCPResponse, MCPOperationError>;
   }
   ```

3. **Transport Header Injection**
   - Modify transport implementations to inject Authorization headers
   - Handle token refresh on 401 responses
   - Implement retry logic for authentication failures

**Files to Modify:**
- `src/mcp/client.rs` - Integrate OAuth into client
- `src/mcp/transport.rs` - Add OAuth header support
- `src/mcp/types.rs` - Add OAuth-related types if needed

**Estimated Effort:** 3-4 days

### Phase 5: Security Hardening

**Priority: HIGH** - Security compliance

**Tasks:**
1. **Audience Validation**
   - Validate token audience claims
   - Prevent confused deputy attacks
   - Implement strict token validation

2. **Secure Token Storage**
   - Implement secure credential storage
   - Support for OS keychain integration (optional)
   - Memory protection for sensitive data

3. **Certificate Validation**
   - Strict TLS certificate validation
   - Custom CA support
   - Certificate pinning (optional)

4. **Rate Limiting and Abuse Prevention**
   - Implement OAuth request rate limiting
   - Exponential backoff for auth failures
   - Abuse detection and prevention

**Files to Create/Modify:**
- `src/mcp/auth/security.rs` - Security utilities
- `src/mcp/auth/validation.rs` - Token and request validation
- Platform-specific credential storage modules

**Estimated Effort:** 3-4 days

### Phase 6: Testing and Documentation

**Priority: MEDIUM** - Quality assurance

**Tasks:**
1. **Unit Tests**
   - OAuth flow testing with mock servers
   - Token refresh and expiration handling
   - Error condition testing

2. **Integration Tests**
   - Real OAuth server testing
   - MCP server integration with OAuth
   - End-to-end authentication flows

3. **Security Testing**
   - Token validation testing
   - PKCE implementation verification
   - Confused deputy attack prevention

4. **Documentation and Examples**
   - OAuth configuration guide
   - Integration examples
   - Security best practices guide

**Files to Create:**
- `tests/oauth_tests.rs` - OAuth integration tests
- `examples/oauth_mcp_client.rs` - OAuth example
- `docs/guides/oauth_authentication.md` - Documentation

**Estimated Effort:** 2-3 days

## Implementation Order and Dependencies

### Critical Path
1. **HTTP+SSE Transport** (Phase 1) - Required for OAuth-enabled servers
2. **OAuth Core** (Phase 2) - Authentication framework foundation
3. **MCP Integration** (Phase 4) - Client-level OAuth support
4. **Security Hardening** (Phase 5) - Production-ready security

### Optional/Parallel Work
- **Server Discovery** (Phase 3) - Can be implemented in parallel with Phase 4
- **Testing and Documentation** (Phase 6) - Ongoing throughout implementation

## Risk Assessment and Mitigation

### High Risk Items
1. **OAuth 2.1 Specification Compliance**
   - *Risk*: Misimplementation of OAuth flows
   - *Mitigation*: Use established OAuth libraries (`oauth2` crate), extensive testing

2. **Token Security**
   - *Risk*: Token exposure or storage vulnerabilities
   - *Mitigation*: Secure storage practices, memory protection, audit trails

3. **Confused Deputy Attacks**
   - *Risk*: Token misuse across different services
   - *Mitigation*: Strict audience validation, token scoping

### Medium Risk Items
1. **HTTP+SSE Implementation Complexity**
   - *Risk*: Transport reliability issues
   - *Mitigation*: Leverage existing HTTP/SSE libraries, comprehensive testing

2. **Backward Compatibility**
   - *Risk*: Breaking existing non-OAuth usage
   - *Mitigation*: Optional OAuth configuration, graceful fallbacks

## Success Criteria

### Functional Requirements
- [ ] OAuth 2.1 authorization code flow with PKCE
- [ ] Automatic token refresh
- [ ] HTTP+SSE transport for OAuth-enabled MCP servers
- [ ] Server discovery and dynamic client registration
- [ ] Integration with existing MCP client API

### Security Requirements
- [ ] Secure token storage and handling
- [ ] HTTPS enforcement for all OAuth endpoints
- [ ] Token audience validation
- [ ] Protection against confused deputy attacks

### Quality Requirements
- [ ] Comprehensive test coverage (>90%)
- [ ] Documentation and examples
- [ ] Performance impact <10% for OAuth-enabled clients
- [ ] Backward compatibility with existing non-OAuth usage

## Estimated Timeline

**Total Estimated Effort:** 16-22 days

**Development Phase:** 3-4 weeks  
**Testing & Documentation:** 1 week  
**Security Review & Hardening:** 1 week

**Target Completion:** 5-6 weeks from start

## Next Steps

1. **Review and Approve Plan** - Stakeholder review of implementation approach
2. **Dependency Analysis** - Confirm HTTP/SSE and OAuth library choices
3. **Phase 1 Implementation** - Start with HTTP+SSE transport foundation
4. **Iterative Development** - Implement phases incrementally with testing

---

*Last Updated: January 9, 2025*  
*Specification Version: MCP 2025-03-26*  
*Author: Claude Code Assistant*