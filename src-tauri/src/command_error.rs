use std::fmt::Display;

use lumadesk_core::{ApiError, CoreError, ErrorCode};
use serde::Serialize;

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct CommandError {
    code: ErrorCode,
    message: String,
}

impl CommandError {
    pub(crate) fn unavailable() -> Self {
        Self {
            code: ErrorCode::Database,
            message: "Local data is temporarily unavailable. Try again shortly.".into(),
        }
    }

    pub(crate) fn overlay(error: &tauri::Error) -> Self {
        Self {
            code: ErrorCode::Io,
            message: format!("The desktop overlay could not be updated: {error}"),
        }
    }

    pub(crate) fn native(action: &str, error: impl Display) -> Self {
        Self {
            code: ErrorCode::Io,
            message: format!("{action}: {error}"),
        }
    }

    pub(crate) fn invalid_input(message: impl Into<String>) -> Self {
        Self {
            code: ErrorCode::InvalidInput,
            message: message.into(),
        }
    }

    pub(crate) fn invalid_data(message: impl Into<String>) -> Self {
        Self {
            code: ErrorCode::InvalidData,
            message: message.into(),
        }
    }
}

impl From<CoreError> for CommandError {
    fn from(error: CoreError) -> Self {
        let error = ApiError::from(&error);
        Self {
            code: error.code,
            message: error.message,
        }
    }
}
