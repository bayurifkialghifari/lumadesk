//! Local-first domain, persistence, and media import for `LumaDesk`.

mod catalog;
mod database;
mod domain;
mod editor;
mod error;
mod frame_cache;
mod frame_decoder;
mod frame_manifest;
mod frames;
mod importer;
mod media;
mod overlays;
mod scenes;
mod settings;
mod validation;

pub use database::SCHEMA_VERSION;
pub use domain::{
    AppSettings, AssetKind, BackgroundRemovalMode, CreatorChanges, CreatorChangesInput, CropRect,
    DeleteReport, DesktopScene, DesktopSceneId, DesktopSceneItem, DesktopSceneSummary, EditRecipe,
    EditRecipePatch, ImportFailure, ImportOutcome, ImportReport, LibraryQuery, MediaFormat,
    MediaId, MediaItem, MediaPage, MediaVariant, MonitorDescriptor, NewOverlay, OverlayBatchPatch,
    OverlayId, OverlayInstance, OverlayPatch, OverlayReplacement, SpritePlaybackMode,
    SpriteSequence, SpriteSequenceInput, VariantId, VariantKind,
};
pub use error::{ApiError, CoreError, ErrorCode};
pub use frames::{FrameAsset, FrameBatch, FrameBatchRequest};
pub use importer::Library;
