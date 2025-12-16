use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Config {
    pub feed_url: String,
    #[serde(default)]
    pub public_ed_key: Option<String>,
    #[serde(default = "default_true")]
    pub automatically_checks_for_updates: bool,
    #[serde(default)]
    pub automatically_downloads_updates: bool,
    #[serde(default = "default_interval")]
    pub update_check_interval: u64,
}

fn default_true() -> bool {
    true
}

fn default_interval() -> u64 {
    86400
}

impl Default for Config {
    fn default() -> Self {
        Self {
            feed_url: String::new(),
            public_ed_key: None,
            automatically_checks_for_updates: true,
            automatically_downloads_updates: false,
            update_check_interval: 86400,
        }
    }
}

impl Config {
    pub fn validate(&self) -> Result<(), ConfigError> {
        if self.feed_url.is_empty() {
            return Err(ConfigError::MissingFeedUrl);
        }

        if url::Url::parse(&self.feed_url).is_err() {
            return Err(ConfigError::InvalidFeedUrl(self.feed_url.clone()));
        }

        #[cfg(not(debug_assertions))]
        if self.public_ed_key.is_none() {
            return Err(ConfigError::MissingPublicKey);
        }

        Ok(())
    }
}

#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    #[error("Missing required field: feedUrl")]
    MissingFeedUrl,

    #[error("Invalid feed URL: {0}")]
    InvalidFeedUrl(String),

    #[error("Missing required field: publicEdKey (required in release builds)")]
    MissingPublicKey,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_values() {
        let config = Config::default();
        assert!(config.automatically_checks_for_updates);
        assert!(!config.automatically_downloads_updates);
        assert_eq!(config.update_check_interval, 86400);
    }

    #[test]
    fn test_validate_missing_feed_url() {
        let config = Config::default();
        let result = config.validate();
        assert!(matches!(result, Err(ConfigError::MissingFeedUrl)));
    }

    #[test]
    fn test_validate_invalid_feed_url() {
        let config = Config {
            feed_url: "not-a-valid-url".to_string(),
            ..Default::default()
        };
        let result = config.validate();
        assert!(matches!(result, Err(ConfigError::InvalidFeedUrl(_))));
    }

    #[test]
    fn test_validate_valid_config() {
        let config = Config {
            feed_url: "https://example.com/appcast.xml".to_string(),
            public_ed_key: Some("test-key".to_string()),
            ..Default::default()
        };
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_deserialize_from_json() {
        let json = r#"{
            "feedUrl": "https://example.com/appcast.xml",
            "publicEdKey": "base64key",
            "automaticallyChecksForUpdates": false,
            "updateCheckInterval": 3600
        }"#;

        let config: Config = serde_json::from_str(json).unwrap();
        assert_eq!(config.feed_url, "https://example.com/appcast.xml");
        assert_eq!(config.public_ed_key, Some("base64key".to_string()));
        assert!(!config.automatically_checks_for_updates);
        assert_eq!(config.update_check_interval, 3600);
    }
}
