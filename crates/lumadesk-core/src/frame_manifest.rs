use std::{
    fs::File,
    io::{BufReader, BufWriter, ErrorKind, Write},
    path::{Path, PathBuf},
};

use serde::{Deserialize, Serialize};

use crate::{CoreError, MediaItem};

const CACHE_VERSION: u32 = 1;

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct FrameManifest {
    version: u32,
    content_hash: String,
    pub(super) width: u32,
    pub(super) height: u32,
    pub(super) total_duration_ms: u64,
    pub(super) durations_ms: Vec<u64>,
}

impl FrameManifest {
    pub(super) fn from_durations(
        item: &MediaItem,
        durations_ms: Vec<u64>,
    ) -> Result<Self, CoreError> {
        let total_duration_ms = durations_ms.iter().try_fold(0_u64, |total, value| {
            total.checked_add(*value).ok_or_else(|| {
                CoreError::InvalidMedia("The animation duration is too long.".into())
            })
        })?;
        let manifest = Self {
            version: CACHE_VERSION,
            content_hash: item.content_hash.clone(),
            width: item.width,
            height: item.height,
            total_duration_ms,
            durations_ms,
        };
        if !manifest.matches_item(item)? {
            return Err(CoreError::InvalidData(
                "Decoded frames do not match the media metadata.".into(),
            ));
        }
        Ok(manifest)
    }

    pub(super) fn read(cache_dir: &Path) -> Result<Option<Self>, CoreError> {
        let file = match File::open(cache_dir.join("manifest.json")) {
            Ok(file) => file,
            Err(error) if error.kind() == ErrorKind::NotFound => return Ok(None),
            Err(error) => return Err(error.into()),
        };
        Ok(serde_json::from_reader(BufReader::new(file)).ok())
    }

    pub(super) fn write(&self, cache_dir: &Path) -> Result<(), CoreError> {
        let file = File::create(cache_dir.join("manifest.json"))?;
        let mut writer = BufWriter::new(file);
        serde_json::to_writer(&mut writer, self).map_err(|error| {
            CoreError::InvalidData(format!("The frame manifest could not be saved: {error}"))
        })?;
        writer.flush()?;
        writer.get_ref().sync_all()?;
        Ok(())
    }

    pub(super) fn is_valid_for(
        &self,
        cache_dir: &Path,
        item: &MediaItem,
    ) -> Result<bool, CoreError> {
        if !self.matches_item(item)? {
            return Ok(false);
        }
        for index in 0..item.frame_count {
            if !frame_path(cache_dir, index).is_file() {
                return Ok(false);
            }
        }
        Ok(true)
    }

    fn matches_item(&self, item: &MediaItem) -> Result<bool, CoreError> {
        let frame_count = u32::try_from(self.durations_ms.len())
            .map_err(|_| CoreError::InvalidData("The cached frame count is invalid.".into()))?;
        Ok(self.version == CACHE_VERSION
            && self.content_hash == item.content_hash
            && self.width == item.width
            && self.height == item.height
            && self.total_duration_ms == item.duration_ms
            && frame_count == item.frame_count)
    }
}

pub(super) fn frame_path(cache_dir: &Path, index: u32) -> PathBuf {
    cache_dir.join(format!("{index:08}.png"))
}
