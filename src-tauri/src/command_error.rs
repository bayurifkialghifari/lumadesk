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
