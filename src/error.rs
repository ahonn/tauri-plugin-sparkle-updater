use serde::{ser::Serializer, Serialize};

use crate::config::ConfigError;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    Io(#[from] std::io::Error),

    #[error("Configuration error: {0}")]
    Config(#[from] ConfigError),

    #[error("Invalid feed URL: {0}")]
    InvalidFeedUrl(String),

    #[error("Sparkle initialization failed: {0}")]
    SparkleInit(String),

    #[error("Updater not ready")]
    UpdaterNotReady,
}

impl Serialize for Error {
  fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
  where
    S: Serializer,
  {
    serializer.serialize_str(self.to_string().as_ref())
  }
}
