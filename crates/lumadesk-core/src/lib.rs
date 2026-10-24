//! Local-first domain, persistence, and media import for `LumaDesk`.

mod catalog;
mod database;
mod domain;
mod error;
mod importer;
mod media;
mod overlays;
mod settings;
mod validation;

pub use database::SCHEMA_VERSION;
pub use domain::{
    AppSettings, DeleteReport, ImportFailure, ImportOutcome, ImportReport, LibraryQuery, MediaId,
    MediaItem, MediaKind, MediaPage, NewOverlay, OverlayId, OverlayInstance, OverlayPatch,
};
pub use error::{ApiError, CoreError, ErrorCode};
pub use importer::Library;
