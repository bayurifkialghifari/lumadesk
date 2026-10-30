use std::{
    collections::HashMap,
    fs,
    path::{Path, PathBuf},
};

use scopeguard::{ScopeGuard, guard};

use crate::{
    CoreError, FrameAsset, FrameBatch, FrameBatchRequest, MediaId, MediaItem, frame_decoder,
    frame_manifest::{FrameManifest, frame_path},
};

const MAX_BATCH_SIZE: u32 = 64;

#[derive(Debug)]
pub(crate) struct FrameCache {
    root: PathBuf,
    manifests: HashMap<MediaId, FrameManifest>,
}

impl FrameCache {
    pub(crate) fn open(root: PathBuf) -> Result<Self, CoreError> {
        fs::create_dir_all(&root)?;
        remove_stale_directories(&root)?;
        Ok(Self {
            root,
            manifests: HashMap::new(),
        })
    }

    pub(crate) fn batch(
        &mut self,
        item: &MediaItem,
        source: &Path,
        request: FrameBatchRequest,
    ) -> Result<FrameBatch, CoreError> {
        validate_request(request)?;
        if !self.manifests.contains_key(&item.id) {
            let manifest = self.load_or_build(item, source)?;
            self.manifests.insert(item.id, manifest);
        }
        let manifest = self
            .manifests
            .get(&item.id)
            .ok_or_else(|| CoreError::InvalidData("The frame manifest is unavailable.".into()))?;
        build_batch(&self.root, item, manifest, request)
    }

    pub(crate) fn remove(&mut self, id: MediaId) -> Result<(), CoreError> {
        self.manifests.remove(&id);
        let path = self.root.join(id.to_string());
        if path.exists() {
            fs::remove_dir_all(path)?;
        }
        Ok(())
    }

    fn load_or_build(&self, item: &MediaItem, source: &Path) -> Result<FrameManifest, CoreError> {
        let cache_dir = self.root.join(item.id.to_string());
        if let Some(manifest) = FrameManifest::read(&cache_dir)?
            && manifest.is_valid_for(&cache_dir, item)?
        {
            return Ok(manifest);
        }
        if cache_dir.exists() {
            fs::remove_dir_all(&cache_dir)?;
        }
        self.build(item, source, &cache_dir)
    }

    fn build(
        &self,
        item: &MediaItem,
        source: &Path,
        cache_dir: &Path,
    ) -> Result<FrameManifest, CoreError> {
        let temporary = self
            .root
            .join(format!(".frames-{}-{}", item.id, uuid::Uuid::new_v4()));
        fs::create_dir(&temporary)?;
        let cleanup = guard(temporary, |path| drop(fs::remove_dir_all(path)));
        let decoded = frame_decoder::decode_to(source, item.media_format, &cleanup)?;
        let manifest = FrameManifest::from_durations(item, decoded.durations_ms)?;
        manifest.write(&cleanup)?;
        fs::rename(&*cleanup, cache_dir)?;
        drop(ScopeGuard::into_inner(cleanup));
        Ok(manifest)
    }
}

fn validate_request(request: FrameBatchRequest) -> Result<(), CoreError> {
    if request.limit == 0 || request.limit > MAX_BATCH_SIZE {
        return Err(CoreError::InvalidInput(format!(
            "Frames per batch must be between 1 and {MAX_BATCH_SIZE}."
        )));
    }
    Ok(())
}

fn build_batch(
    root: &Path,
    item: &MediaItem,
    manifest: &FrameManifest,
    request: FrameBatchRequest,
) -> Result<FrameBatch, CoreError> {
    let offset = usize::try_from(request.offset)
        .map_err(|_| CoreError::InvalidInput("The frame offset is too large.".into()))?;
    let limit = usize::try_from(request.limit)
        .map_err(|_| CoreError::InvalidInput("The frame count is too large.".into()))?;
    let cache_dir = root.join(item.id.to_string());
    let mut frames = Vec::with_capacity(limit.min(manifest.durations_ms.len()));
    for (index, duration_ms) in manifest
        .durations_ms
        .iter()
        .copied()
        .enumerate()
        .skip(offset)
        .take(limit)
    {
        let index = u32::try_from(index)
            .map_err(|_| CoreError::InvalidData("The frame index is invalid.".into()))?;
        frames.push(FrameAsset {
            index,
            duration_ms,
            path: frame_path(&cache_dir, index).to_string_lossy().into_owned(),
        });
    }
    Ok(FrameBatch {
        media_id: item.id,
        width: manifest.width,
        height: manifest.height,
        total_frames: item.frame_count,
        total_duration_ms: manifest.total_duration_ms,
        frames,
    })
}

fn remove_stale_directories(root: &Path) -> Result<(), CoreError> {
    for entry in root.read_dir()? {
        let entry = entry?;
        if entry.file_name().to_string_lossy().starts_with(".frames-") {
            let path = entry.path();
            if entry.file_type()?.is_dir() {
                fs::remove_dir_all(path)?;
            } else {
                fs::remove_file(path)?;
            }
        }
    }
    Ok(())
}
