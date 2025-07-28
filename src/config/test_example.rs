//! Test example configurations
//! This module provides tests to verify that the example configuration files work correctly.

#[cfg(test)]
mod tests {
    use crate::config::StoodConfig;
    use std::path::Path;

    #[test]
    fn test_example_toml_config() {
        let config_path = Path::new("docs/sample-config/example-config.toml");
        if config_path.exists() {
            let config =
                StoodConfig::from_file(config_path).expect("Failed to load example TOML config");
            config
                .validate()
                .expect("Example TOML config validation failed");

            // Verify some key values
            assert_eq!(config.bedrock.region, "us-west-2");
            assert_eq!(
                config.bedrock.default_model,
                "us.anthropic.claude-3-5-haiku-20241022-v1:0"
            );
            assert!(config.telemetry.enabled);
            assert_eq!(config.telemetry.service_name, "stood-production-agent");
            assert!(config.features.context_window_management);
        }
    }

    #[test]
    fn test_example_yaml_config() {
        let config_path = Path::new("tests/sample-config/example-config.yaml");
        if config_path.exists() {
            let config =
                StoodConfig::from_file(config_path).expect("Failed to load example YAML config");
            config
                .validate()
                .expect("Example YAML config validation failed");

            // Verify some key values
            assert_eq!(config.bedrock.region, "us-east-1");
            assert_eq!(
                config.bedrock.default_model,
                "us.anthropic.claude-3-5-haiku-20241022-v1:0"
            );
            assert!(config.telemetry.enabled);
            assert_eq!(config.telemetry.service_name, "stood-development-agent");
            assert!(config.features.tool_hot_reload);
        }
    }

    #[test]
    fn test_config_from_env() {
        // Clean environment first
        std::env::remove_var("AWS_REGION");
        std::env::remove_var("STOOD_TEMPERATURE");

        // Test environment variable override functionality
        std::env::set_var("AWS_REGION", "eu-central-1");
        std::env::set_var("STOOD_TEMPERATURE", "0.9");

        // Test loading from environment directly
        let env_config = StoodConfig::from_env().expect("Environment loading failed");
        assert_eq!(env_config.bedrock.region, "eu-central-1");
        assert_eq!(env_config.agent.temperature, 0.9);

        // Clean up
        std::env::remove_var("AWS_REGION");
        std::env::remove_var("STOOD_TEMPERATURE");
    }
}
