use std::{fmt, path::Path};

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

macro_rules! uuid_id {
    ($name:ident) => {
        #[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, Serialize, Deserialize)]
        #[serde(transparent)]
        pub struct $name(pub Uuid);

        impl $name {
            #[must_use]
            pub fn new() -> Self {
                Self(Uuid::new_v4())
            }
        }

        impl Default for $name {
            fn default() -> Self {
                Self::new()
            }
        }

        impl fmt::Display for $name {
            fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
                self.0.fmt(formatter)
            }
        }
    };
}

uuid_id!(MediaId);
uuid_id!(VariantId);
uuid_id!(OverlayId);

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AssetKind {
    Still,
    Animation,
    SpriteSheet,
    Video,
    Audio,
}

impl AssetKind {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Still => "still",
            Self::Animation => "animation",
            Self::SpriteSheet => "sprite_sheet",
            Self::Video => "video",
            Self::Audio => "audio",
        }
    }

    pub(crate) fn parse(value: &str) -> Option<Self> {
        match value {
            "still" => Some(Self::Still),
            "animation" => Some(Self::Animation),
            "sprite_sheet" => Some(Self::SpriteSheet),
            "video" => Some(Self::Video),
            "audio" => Some(Self::Audio),
            _ => None,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MediaFormat {
    Gif,
    Apng,
    Webp,
    Png,
    Jpeg,
    Mp4,
    Webm,
    Mp3,
    Wav,
    Ogg,
}

impl MediaFormat {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Gif => "gif",
            Self::Apng => "apng",
            Self::Webp => "webp",
            Self::Png => "png",
            Self::Jpeg => "jpeg",
            Self::Mp4 => "mp4",
            Self::Webm => "webm",
            Self::Mp3 => "mp3",
            Self::Wav => "wav",
            Self::Ogg => "ogg",
        }
    }

    #[must_use]
    pub const fn extension(self) -> &'static str {
        match self {
            Self::Gif => "gif",
            Self::Apng | Self::Png => "png",
            Self::Webp => "webp",
            Self::Jpeg => "jpg",
            Self::Mp4 => "mp4",
            Self::Webm => "webm",
            Self::Mp3 => "mp3",
            Self::Wav => "wav",
            Self::Ogg => "ogg",
        }
    }

    pub(crate) fn parse(value: &str) -> Option<Self> {
        match value {
            "gif" => Some(Self::Gif),
            "apng" => Some(Self::Apng),
            "webp" => Some(Self::Webp),
            "png" => Some(Self::Png),
            "jpeg" => Some(Self::Jpeg),
            "mp4" => Some(Self::Mp4),
            "webm" => Some(Self::Webm),
            "mp3" => Some(Self::Mp3),
            "wav" => Some(Self::Wav),
            "ogg" => Some(Self::Ogg),
            _ => None,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MediaItem {
    pub id: MediaId,
    pub content_hash: String,
    pub display_name: String,
    pub managed_path: String,
    pub asset_kind: AssetKind,
    pub media_format: MediaFormat,
    pub width: u32,
    pub height: u32,
    pub frame_count: u32,
    pub duration_ms: u64,
    pub is_favorite: bool,
    pub tags: Vec<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Clone, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct LibraryQuery {
    pub search: Option<String>,
    pub asset_kind: Option<AssetKind>,
    pub media_format: Option<MediaFormat>,
    pub favorite_only: bool,
    pub tag: Option<String>,
    pub limit: Option<u32>,
    pub offset: u32,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MediaPage {
    pub items: Vec<MediaItem>,
    pub total: u64,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BackgroundRemovalMode {
    None,
    Edge,
    AiFast,
    AiQuality,
}

impl BackgroundRemovalMode {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::None => "none",
            Self::Edge => "edge",
            Self::AiFast => "ai_fast",
            Self::AiQuality => "ai_quality",
        }
    }

    pub(crate) fn parse(value: &str) -> Option<Self> {
        match value {
            "none" => Some(Self::None),
            "edge" => Some(Self::Edge),
            "ai_fast" => Some(Self::AiFast),
            "ai_quality" => Some(Self::AiQuality),
            _ => None,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CropRect {
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EditRecipe {
    pub media_id: MediaId,
    pub crop: Option<CropRect>,
    pub rotation_degrees: i16,
    pub flip_horizontal: bool,
    pub background_removal: BackgroundRemovalMode,
    pub brightness: f64,
    pub contrast: f64,
    pub saturation: f64,
    pub updated_at: DateTime<Utc>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EditRecipePatch {
    pub crop: Option<Option<CropRect>>,
    pub rotation_degrees: Option<i16>,
    pub flip_horizontal: Option<bool>,
    pub background_removal: Option<BackgroundRemovalMode>,
    pub brightness: Option<f64>,
    pub contrast: Option<f64>,
    pub saturation: Option<f64>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SpritePlaybackMode {
    Loop,
    Once,
    PingPong,
}

impl SpritePlaybackMode {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Loop => "loop",
            Self::Once => "once",
            Self::PingPong => "ping_pong",
        }
    }

    pub(crate) fn parse(value: &str) -> Option<Self> {
        match value {
            "loop" => Some(Self::Loop),
            "once" => Some(Self::Once),
            "ping_pong" => Some(Self::PingPong),
            _ => None,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SpriteSequence {
    pub media_id: MediaId,
    pub columns: u32,
    pub rows: u32,
    pub frame_count: u32,
    pub frame_duration_ms: u32,
    pub playback_mode: SpritePlaybackMode,
    pub updated_at: DateTime<Utc>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SpriteSequenceInput {
    pub media_id: MediaId,
    pub columns: u32,
    pub rows: u32,
    pub frame_count: u32,
    pub frame_duration_ms: u32,
    pub playback_mode: SpritePlaybackMode,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum VariantKind {
    Preview,
    Export,
    Proxy,
    SegmentationMask,
}

impl VariantKind {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Preview => "preview",
            Self::Export => "export",
            Self::Proxy => "proxy",
            Self::SegmentationMask => "segmentation_mask",
        }
    }

    pub(crate) fn parse(value: &str) -> Option<Self> {
        match value {
            "preview" => Some(Self::Preview),
            "export" => Some(Self::Export),
            "proxy" => Some(Self::Proxy),
            "segmentation_mask" => Some(Self::SegmentationMask),
            _ => None,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MediaVariant {
    pub id: VariantId,
    pub media_id: MediaId,
    pub kind: VariantKind,
    pub relative_path: String,
    pub width: u32,
    pub height: u32,
    pub created_at: DateTime<Utc>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[expect(
    clippy::struct_excessive_bools,
    reason = "independent persisted overlay controls are represented as booleans at the command boundary"
)]
pub struct OverlayInstance {
    pub id: OverlayId,
    pub media_id: MediaId,
    pub monitor_id: Option<String>,
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub opacity: f64,
    pub speed: f64,
    pub flip_horizontal: bool,
    pub remove_background: bool,
    pub is_playing: bool,
    pub z_order: i32,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NewOverlay {
    pub media_id: MediaId,
    pub monitor_id: Option<String>,
    pub x: f64,
    pub y: f64,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct OverlayPatch {
    pub monitor_id: Option<Option<String>>,
    pub x: Option<f64>,
    pub y: Option<f64>,
    pub width: Option<f64>,
    pub opacity: Option<f64>,
    pub speed: Option<f64>,
    pub flip_horizontal: Option<bool>,
    pub remove_background: Option<bool>,
    pub is_playing: Option<bool>,
    pub z_order: Option<i32>,
    pub is_active: Option<bool>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct AppSettings {
    pub theme: String,
    pub launch_at_login: bool,
    pub restore_overlays: bool,
    pub close_to_tray: bool,
    pub collection_view: String,
    pub default_overlay_width: u32,
    pub default_opacity: f64,
    pub default_speed: f64,
    pub cache_limit_mb: u32,
    pub ai_model: String,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            theme: "dark".into(),
            launch_at_login: false,
            restore_overlays: true,
            close_to_tray: true,
            collection_view: "grid".into(),
            default_overlay_width: 240,
            default_opacity: 1.0,
            default_speed: 1.0,
            cache_limit_mb: 512,
            ai_model: "none".into(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "status", rename_all = "snake_case")]
pub enum ImportOutcome {
    Imported { item: MediaItem },
    Duplicate { item: MediaItem },
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ImportFailure {
    pub path: String,
    pub code: crate::ErrorCode,
    pub message: String,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ImportReport {
    pub outcomes: Vec<ImportOutcome>,
    pub failures: Vec<ImportFailure>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DeleteReport {
    pub media_id: MediaId,
    pub overlays_removed: u32,
}

#[derive(Clone, Debug)]
pub(crate) struct NewMediaItem {
    pub id: MediaId,
    pub content_hash: String,
    pub display_name: String,
    pub managed_path: String,
    pub asset_kind: AssetKind,
    pub media_format: MediaFormat,
    pub width: u32,
    pub height: u32,
    pub frame_count: u32,
    pub duration_ms: u64,
}

pub(crate) fn display_name(path: &Path) -> String {
    path.file_stem()
        .and_then(|name| name.to_str())
        .filter(|name| !name.trim().is_empty())
        .map_or_else(|| "Untitled animation".into(), str::to_owned)
}
