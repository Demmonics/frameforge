use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct SerializableError {
    pub code: String,
    pub message: String,
    pub details: Option<String>,
    pub recoverable: bool,
}

#[derive(Debug, Error)]
#[non_exhaustive]
pub enum FrameForgeError {
    #[error("Required media binary '{tool}' was not found ({searched})")]
    BinaryNotFound { tool: String, searched: String },

    #[error("Invalid input path '{path}': {reason}")]
    InvalidInputPath { path: String, reason: String },

    #[error("FFprobe failed to inspect '{path}': {summary}")]
    ProbeFailed {
        path: String,
        exit_code: Option<i32>,
        summary: String,
        raw_stderr: String,
    },

    #[error("Unsupported media '{path}': {reason}")]
    UnsupportedMedia { path: String, reason: String },

    #[error("Corrupt or encrypted media '{path}': {reason}")]
    EncryptedOrCorruptMedia { path: String, reason: String },

    #[error("Incompatible conversion settings: {summary}")]
    IncompatibleConversion {
        summary: String,
        reasons: Vec<String>,
    },

    #[error("Validation error for '{field}': {message}")]
    ValidationError { field: String, message: String },

    #[error("Output file already exists at '{path}'")]
    OutputConflict { path: String },

    #[error(
        "Insufficient disk space at '{path}': need {required_bytes} bytes, {available_bytes} bytes available"
    )]
    InsufficientDiskSpace {
        required_bytes: u64,
        available_bytes: u64,
        path: String,
    },

    #[error("FFmpeg conversion failed: {summary}")]
    ConversionFailed {
        exit_code: Option<i32>,
        summary: String,
        raw_stderr: String,
    },

    #[error("Conversion job was cancelled")]
    Cancelled,

    #[error("I/O error during {context}: {source}")]
    Io {
        context: String,
        #[source]
        source: std::io::Error,
    },

    #[error("Internal error: {0}")]
    Internal(String),
}

impl FrameForgeError {
    /// Returns the stable machine-readable error code for IPC responses.
    #[must_use]
    pub fn code(&self) -> &'static str {
        match self {
            Self::BinaryNotFound { .. } => "BINARY_NOT_FOUND",
            Self::InvalidInputPath { .. } => "INVALID_INPUT_PATH",
            Self::ProbeFailed { .. } => "PROBE_FAILED",
            Self::UnsupportedMedia { .. } => "UNSUPPORTED_MEDIA",
            Self::EncryptedOrCorruptMedia { .. } => "CORRUPT_OR_ENCRYPTED_MEDIA",
            Self::IncompatibleConversion { .. } => "INCOMPATIBLE_CONVERSION",
            Self::ValidationError { .. } => "VALIDATION_ERROR",
            Self::OutputConflict { .. } => "OUTPUT_CONFLICT",
            Self::InsufficientDiskSpace { .. } => "INSUFFICIENT_DISK_SPACE",
            Self::ConversionFailed { .. } => "CONVERSION_FAILED",
            Self::Cancelled => "CANCELLED",
            Self::Io { .. } => "IO_ERROR",
            Self::Internal(_) => "INTERNAL_ERROR",
        }
    }

    /// Converts this structured error into a frontend-serializable payload.
    #[must_use]
    pub fn to_serializable(&self) -> SerializableError {
        match self {
            Self::BinaryNotFound { .. } => SerializableError {
                code: self.code().to_string(),
                message: self.to_string(),
                details: Some(
                    "Verify FFmpeg and FFprobe binaries are installed on PATH or configured in Settings."
                        .to_string(),
                ),
                recoverable: true,
            },
            Self::ProbeFailed { raw_stderr, .. } | Self::ConversionFailed { raw_stderr, .. } => {
                SerializableError {
                    code: self.code().to_string(),
                    message: self.to_string(),
                    details: if raw_stderr.trim().is_empty() {
                        None
                    } else {
                        Some(raw_stderr.clone())
                    },
                    recoverable: true,
                }
            }
            Self::IncompatibleConversion { reasons, .. } => SerializableError {
                code: self.code().to_string(),
                message: self.to_string(),
                details: Some(reasons.join("\n")),
                recoverable: true,
            },
            _ => SerializableError {
                code: self.code().to_string(),
                message: self.to_string(),
                details: None,
                recoverable: !matches!(self, Self::Internal(_)),
            },
        }
    }
}

pub type Result<T> = std::result::Result<T, FrameForgeError>;
