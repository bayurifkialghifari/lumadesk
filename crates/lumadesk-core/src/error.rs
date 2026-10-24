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
    #[error("operasi berkas gagal: {0}")]
    Io(#[from] io::Error),
    #[error("operasi database gagal: {0}")]
    Database(#[from] rusqlite::Error),
    #[error("format media tidak didukung")]
    UnsupportedMedia,
    #[error("media rusak atau tidak dapat dibaca: {0}")]
    InvalidMedia(String),
    #[error("input tidak valid: {0}")]
    InvalidInput(String),
    #[error("data lokal tidak valid: {0}")]
    InvalidData(String),
    #[error("{entity} tidak ditemukan: {id}")]
    NotFound { entity: &'static str, id: String },
    #[error("versi database {found} lebih baru dari versi yang didukung {supported}")]
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
            Self::Io(_) => "Berkas tidak dapat dibaca atau disimpan.".into(),
            Self::Database(_) => "Data lokal tidak dapat diperbarui.".into(),
            Self::UnsupportedMedia => "Format berkas ini belum didukung.".into(),
            Self::InvalidMedia(_) => "Berkas media rusak atau tidak dapat dibaca.".into(),
            Self::InvalidInput(reason) => reason.clone(),
            Self::InvalidData(_) => "Data lokal rusak dan tidak dapat dibaca.".into(),
            Self::NotFound { entity, .. } => format!("{entity} tidak ditemukan."),
            Self::SchemaTooNew { .. } => {
                "Data dibuat oleh LumaDesk versi lebih baru. Perbarui aplikasi sebelum membukanya."
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
