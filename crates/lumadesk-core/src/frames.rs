use serde::{Deserialize, Serialize};

use crate::MediaId;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FrameBatchRequest {
    pub offset: u32,
    pub limit: u32,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FrameAsset {
    pub index: u32,
    pub duration_ms: u64,
    pub path: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FrameBatch {
    pub media_id: MediaId,
    pub width: u32,
    pub height: u32,
    pub total_frames: u32,
    pub total_duration_ms: u64,
    pub frames: Vec<FrameAsset>,
}
