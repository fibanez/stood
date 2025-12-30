//! AWS Authentication for CloudWatch Gen AI Observability
//!
//! This module provides AWS credential resolution and SigV4 request signing
//! for the X-Ray OTLP traces endpoint.

use super::AwsCredentialSource;
use aws_credential_types::provider::ProvideCredentials;
use aws_credential_types::Credentials;
use aws_sigv4::http_request::{sign, SignableBody, SignableRequest, SigningSettings};
use aws_sigv4::sign::v4;
use aws_smithy_runtime_api::client::identity::Identity;
use std::time::SystemTime;

/// Error during AWS authentication
#[derive(Debug, thiserror::Error)]
pub enum AuthError {
    /// Credentials not found or invalid
    #[error("Credential error: {0}")]
    Credentials(String),

    /// Signing failed
    #[error("Signing error: {0}")]
    Signing(String),

    /// Configuration error
    #[error("Configuration error: {0}")]
    Configuration(String),
}

/// Provider for AWS credentials
///
/// Resolves credentials from various sources based on `AwsCredentialSource`.
#[derive(Debug, Clone)]
pub struct AwsCredentialsProvider {
    source: AwsCredentialSource,
    region: String,
}

impl AwsCredentialsProvider {
    /// Create a new credentials provider
    pub fn new(source: AwsCredentialSource, region: impl Into<String>) -> Self {
        Self {
            source,
            region: region.into(),
        }
    }

    /// Get the AWS region
    pub fn region(&self) -> &str {
        &self.region
    }

    /// Resolve credentials from the configured source
    pub async fn resolve(&self) -> Result<Credentials, AuthError> {
        match &self.source {
            AwsCredentialSource::Environment => self.load_from_environment().await,
            AwsCredentialSource::Profile(profile) => self.load_from_profile(profile).await,
            AwsCredentialSource::IamRole => self.load_from_iam_role().await,
            AwsCredentialSource::Explicit {
                access_key_id,
                secret_access_key,
                session_token,
            } => Ok(Credentials::new(
                access_key_id.clone(),
                secret_access_key.clone(),
                session_token.clone(),
                None, // expiration
                "explicit",
            )),
        }
    }

    /// Load credentials from environment variables
    async fn load_from_environment(&self) -> Result<Credentials, AuthError> {
        // Use the AWS SDK's default credential chain which checks:
        // 1. Environment variables (AWS_ACCESS_KEY_ID, AWS_SECRET_ACCESS_KEY)
        // 2. Shared credentials file (~/.aws/credentials)
        // 3. IAM role (if running on AWS)
        let config = aws_config::load_defaults(aws_config::BehaviorVersion::latest()).await;

        config
            .credentials_provider()
            .ok_or_else(|| {
                AuthError::Credentials("No credentials provider configured".to_string())
            })?
            .provide_credentials()
            .await
            .map_err(|e| AuthError::Credentials(e.to_string()))
    }

    /// Load credentials from a specific AWS profile
    async fn load_from_profile(&self, profile: &str) -> Result<Credentials, AuthError> {
        let config = aws_config::defaults(aws_config::BehaviorVersion::latest())
            .profile_name(profile)
            .load()
            .await;

        config
            .credentials_provider()
            .ok_or_else(|| {
                AuthError::Credentials(format!(
                    "Profile '{}' not found or has no credentials",
                    profile
                ))
            })?
            .provide_credentials()
            .await
            .map_err(|e| AuthError::Credentials(e.to_string()))
    }

    /// Load credentials from IAM role (EC2/ECS/Lambda)
    async fn load_from_iam_role(&self) -> Result<Credentials, AuthError> {
        // The default credential chain will automatically use IMDS for EC2,
        // ECS container credentials, or Lambda execution role
        self.load_from_environment().await
    }
}

/// Sign an HTTP request with AWS SigV4 and return the signed headers
///
/// # Arguments
/// * `method` - HTTP method (e.g., "POST")
/// * `uri` - Full URI including query string
/// * `existing_headers` - Headers to include in signature
/// * `body` - Request body
/// * `credentials` - AWS credentials
/// * `region` - AWS region
/// * `service` - AWS service name (e.g., "xray")
///
/// # Returns
/// A vector of (header_name, header_value) pairs including auth headers
pub fn sign_request(
    method: &str,
    uri: &str,
    existing_headers: &[(String, String)],
    body: &[u8],
    credentials: &Credentials,
    region: &str,
    service: &str,
) -> Result<Vec<(String, String)>, AuthError> {
    // Convert credentials to Identity
    let identity: Identity = credentials.clone().into();

    let signing_settings = SigningSettings::default();
    let signing_params = v4::SigningParams::builder()
        .identity(&identity)
        .region(region)
        .name(service)
        .time(SystemTime::now())
        .settings(signing_settings)
        .build()
        .map_err(|e| AuthError::Signing(format!("Failed to build signing params: {}", e)))?;

    // Convert headers to the format needed by SignableRequest
    let header_pairs: Vec<(&str, &str)> = existing_headers
        .iter()
        .map(|(k, v)| (k.as_str(), v.as_str()))
        .collect();

    let signable_request = SignableRequest::new(
        method,
        uri,
        header_pairs.into_iter(),
        SignableBody::Bytes(body),
    )
    .map_err(|e| AuthError::Signing(format!("Failed to create signable request: {}", e)))?;

    let (signing_instructions, _signature) = sign(signable_request, &signing_params.into())
        .map_err(|e| AuthError::Signing(format!("Failed to sign request: {}", e)))?
        .into_parts();

    // Build the final headers list
    let mut result_headers: Vec<(String, String)> = existing_headers.to_vec();

    // Add signing headers using the headers() method
    for (name, value) in signing_instructions.headers() {
        result_headers.push((name.to_string(), value.to_string()));
    }

    Ok(result_headers)
}

/// X-Ray OTLP endpoint URL for a region
pub fn xray_otlp_endpoint(region: &str) -> String {
    format!("https://xray.{}.amazonaws.com/v1/traces", region)
}

/// CloudWatch Logs endpoint URL for a region
pub fn cloudwatch_logs_endpoint(region: &str) -> String {
    format!("https://logs.{}.amazonaws.com/", region)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_xray_endpoint_format() {
        assert_eq!(
            xray_otlp_endpoint("us-east-1"),
            "https://xray.us-east-1.amazonaws.com/v1/traces"
        );
        assert_eq!(
            xray_otlp_endpoint("eu-west-1"),
            "https://xray.eu-west-1.amazonaws.com/v1/traces"
        );
    }

    #[test]
    fn test_explicit_credentials() {
        let provider = AwsCredentialsProvider::new(
            AwsCredentialSource::Explicit {
                access_key_id: "AKIAIOSFODNN7EXAMPLE".to_string(),
                secret_access_key: "wJalrXUtnFEMI/K7MDENG/bPxRfiCYEXAMPLEKEY".to_string(),
                session_token: None,
            },
            "us-east-1",
        );

        assert_eq!(provider.region(), "us-east-1");
    }

    #[tokio::test]
    async fn test_explicit_credentials_resolve() {
        let provider = AwsCredentialsProvider::new(
            AwsCredentialSource::Explicit {
                access_key_id: "AKIAIOSFODNN7EXAMPLE".to_string(),
                secret_access_key: "wJalrXUtnFEMI/K7MDENG/bPxRfiCYEXAMPLEKEY".to_string(),
                session_token: Some("session-token".to_string()),
            },
            "us-west-2",
        );

        let creds = provider.resolve().await.unwrap();
        assert_eq!(creds.access_key_id(), "AKIAIOSFODNN7EXAMPLE");
        assert_eq!(
            creds.secret_access_key(),
            "wJalrXUtnFEMI/K7MDENG/bPxRfiCYEXAMPLEKEY"
        );
        assert_eq!(creds.session_token(), Some("session-token"));
    }
}
