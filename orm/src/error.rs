use std::fmt::Display;

use thiserror::*;

#[derive(Error, Debug)]
pub enum SerdeError {
    // One or more variants that can be created by data structures through the
    // `ser::Error` and `de::Error` traits. For example the Serialize impl for
    // Mutex<T> might return an error because the mutex is poisoned, or the
    // Deserialize impl for a struct may return an error because a required
    // field is missing.
    #[error("{0}")]
    Message(String),

    #[error("expect attribute #[serde(with = \"serde_bytes\") for Vec<u8> or &[u8]")]
    SerdeBytes,
}

impl serde::ser::Error for SerdeError {
    fn custom<T>(msg: T) -> Self
    where
        T: std::fmt::Display,
    {
        SerdeError::Message(msg.to_string())
    }
}

impl serde::de::Error for SerdeError {
    fn custom<T: Display>(msg: T) -> Self {
        SerdeError::Message(msg.to_string())
    }
}

pub type Result<Output> = std::result::Result<Output, SerdeError>;
