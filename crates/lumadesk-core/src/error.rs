use std::io;

use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ErrorCode {
    Io,
    Database,
    UnsupportedMedia,
    InvalidMedia,
    InvalidInput,
    InvalidData,
    NotFound,
    SchemaTooNew,
}

impl ErrorCode {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Io => "io",
            Self::Database => "database",
            Self::UnsupportedMedia => "unsupported_media",
            Self::InvalidMedia => "invalid_media",
            Self::InvalidInput => "invalid_input",
            Self::InvalidData => "invalid_data",
            Self::NotFound => "not_found",
            Self::SchemaTooNew => "schema_too_new",
        }
    }
}

#[derive(Debug, Error)]
pub enum CoreError {
    #[error("file operation failed: {0}")]
    Io(#[from] io::Error),
    #[error("database operation failed: {0}")]
    Database(#[from] rusqlite::Error),
    #[error("unsupported media format")]
    UnsupportedMedia,
    #[error("corrupt or unreadable media: {0}")]
    InvalidMedia(String),
    #[error("invalid input: {0}")]
    InvalidInput(String),
    #[error("invalid local data: {0}")]
    InvalidData(String),
    #[error("{entity} not found: {id}")]
    NotFound { entity: &'static str, id: String },
    #[error("database version {found} is newer than supported version {supported}")]
    SchemaTooNew { found: u32, supported: u32 },
}

impl CoreError {
    #[must_use]
    pub const fn code(&self) -> ErrorCode {
        match self {
            Self::Io(_) => ErrorCode::Io,
            Self::Database(_) => ErrorCode::Database,
            Self::UnsupportedMedia => ErrorCode::UnsupportedMedia,
            Self::InvalidMedia(_) => ErrorCode::InvalidMedia,
            Self::InvalidInput(_) => ErrorCode::InvalidInput,
            Self::InvalidData(_) => ErrorCode::InvalidData,
            Self::NotFound { .. } => ErrorCode::NotFound,
            Self::SchemaTooNew { .. } => ErrorCode::SchemaTooNew,
        }
    }

    #[must_use]
    pub fn user_message(&self) -> String {
        match self {
            Self::Io(_) => "The file could not be read or saved.".into(),
            Self::Database(_) => "Local data could not be updated.".into(),
            Self::UnsupportedMedia => "This file format is not supported.".into(),
            Self::InvalidMedia(_) => "The media file is corrupt or unreadable.".into(),
            Self::InvalidInput(reason) => reason.clone(),
            Self::InvalidData(_) => "Local data is corrupt and could not be read.".into(),
            Self::NotFound { entity, .. } => format!("{entity} was not found."),
            Self::SchemaTooNew { .. } => {
                "This data was created by a newer version of LumaDesk. Update the app before opening it."
                    .into()
            }
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ApiError {
    pub code: ErrorCode,
    pub message: String,
}

impl From<&CoreError> for ApiError {
    fn from(error: &CoreError) -> Self {
        Self {
            code: error.code(),
            message: error.user_message(),
        }
    }
}
