use std::{
    collections::HashMap,
    fs::{self, File, OpenOptions},
    io::{self, BufReader, BufWriter, Read, Write},
    path::{Path, PathBuf},
    sync::{Arc, Mutex},
    time::Duration,
};

use image::{DynamicImage, GrayImage, ImageFormat, imageops::FilterType};
use lumadesk_core::{
    FrameBatch, FrameBatchRequest, MediaId, MediaItem, SpriteSequence, VariantKind,
};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use tauri::State;
use thiserror::Error;
use tract_onnx::prelude::*;

use crate::{
    command_error::CommandError,
    state::{AppState, with_library},
};

const MODEL_SIZE: usize = 320;
const DOWNLOAD_TIMEOUT: Duration = Duration::from_secs(30 * 60);
const MODEL_INTEGRITY_MESSAGE: &str = "Model verification failed. Remove it and download it again.";

#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum AiModel {
    Fast,
    Quality,
}

impl AiModel {
    const ALL: [Self; 2] = [Self::Fast, Self::Quality];

    const fn specification(self) -> ModelSpecification {
        match self {
            Self::Fast => ModelSpecification {
                file_name: "u2netp.onnx",
                url: "https://github.com/danielgatis/rembg/releases/download/v0.0.0/u2netp.onnx",
                size_bytes: 4_574_861,
                sha256: "309c8469258dda742793dce0ebea8e6dd393174f89934733ecc8b14c76f4ddd8",
            },
            Self::Quality => ModelSpecification {
                file_name: "u2net.onnx",
                url: "https://github.com/danielgatis/rembg/releases/download/v0.0.0/u2net.onnx",
                size_bytes: 175_997_641,
                sha256: "8d10d2f3bb75ae3b6d527c77944fc5e7dcd94b29809d47a739a7a728a912b491",
            },
        }
    }

    const fn directory_name(self) -> &'static str {
        match self {
            Self::Fast => "fast",
            Self::Quality => "quality",
        }
    }
}

#[derive(Clone, Copy, Debug)]
struct ModelSpecification {
    file_name: &'static str,
    url: &'static str,
    size_bytes: u64,
    sha256: &'static str,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum AiModelAvailability {
    Missing,
    Installed,
    Downloading,
    Removing,
    Error,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct AiModelStatus {
    model: AiModel,
    status: AiModelAvailability,
    size_bytes: u64,
    installed_bytes: u64,
    message: Option<String>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct AiMaskAsset {
    index: u32,
    path: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct AiMaskBatch {
    media_id: MediaId,
    model: AiModel,
    masks: Vec<AiMaskAsset>,
}

#[derive(Clone, Debug)]
enum TransientStatus {
    Downloading,
    Removing,
    Error(String),
}

#[derive(Debug)]
struct AiModelManager {
    data_root: PathBuf,
    model_root: PathBuf,
    statuses: Mutex<HashMap<AiModel, TransientStatus>>,
    runners: Mutex<HashMap<AiModel, Arc<TypedRunnableModel>>>,
    inference_lock: Mutex<()>,
}

#[derive(Debug)]
pub(crate) struct AiState {
    manager: Arc<AiModelManager>,
    generated_media: async_lock::Mutex<()>,
}

impl AiState {
    pub(crate) fn open(data_root: &Path) -> Result<Self, io::Error> {
        let model_root = data_root.join("models");
        fs::create_dir_all(&model_root)?;
        fs::create_dir_all(data_root.join("masks"))?;
        Ok(Self {
            manager: Arc::new(AiModelManager {
                data_root: data_root.to_path_buf(),
                model_root,
                statuses: Mutex::new(HashMap::new()),
                runners: Mutex::new(HashMap::new()),
                inference_lock: Mutex::new(()),
            }),
            generated_media: async_lock::Mutex::new(()),
        })
    }

    pub(crate) async fn lock_generated_media(&self) -> async_lock::MutexGuard<'_, ()> {
        self.generated_media.lock().await
    }
}

#[derive(Debug, Error)]
enum AiError {
    #[error("file operation failed: {0}")]
    Io(#[from] io::Error),
    #[error("image processing failed: {0}")]
    Image(#[from] image::ImageError),
    #[error("model download failed: {0}")]
    Network(#[from] reqwest::Error),
    #[error("the selected local AI model is not installed")]
    MissingModel,
    #[error("the model file failed its integrity check")]
    Integrity,
    #[error("local AI inference failed: {0}")]
    Inference(String),
    #[error("another operation is already running for this local AI model")]
    OperationInProgress,
    #[error("local AI state is temporarily unavailable")]
    StateUnavailable,
}

impl AiModelManager {
    fn statuses(&self) -> Result<Vec<AiModelStatus>, AiError> {
        let transient = self
            .statuses
            .lock()
            .map_err(|_| AiError::StateUnavailable)?
            .clone();
        AiModel::ALL
            .into_iter()
            .map(|model| {
                let specification = model.specification();
                let (installed_bytes, status, message) = match transient.get(&model) {
                    Some(TransientStatus::Downloading) => (
                        installed_file_size(&self.model_path(model)),
                        AiModelAvailability::Downloading,
                        None,
                    ),
                    Some(TransientStatus::Removing) => (
                        installed_file_size(&self.model_path(model)),
                        AiModelAvailability::Removing,
                        None,
                    ),
                    Some(TransientStatus::Error(message)) => (
                        installed_file_size(&self.model_path(model)),
                        AiModelAvailability::Error,
                        Some(message.clone()),
                    ),
                    None => disk_model_status(&self.model_path(model), specification)?,
                };
                Ok(AiModelStatus {
                    model,
                    status,
                    size_bytes: specification.size_bytes,
                    installed_bytes,
                    message,
                })
            })
            .collect()
    }

    fn download(&self, model: AiModel) -> Result<(), AiError> {
        {
            let mut statuses = self
                .statuses
                .lock()
                .map_err(|_| AiError::StateUnavailable)?;
            if matches!(
                statuses.get(&model),
                Some(TransientStatus::Downloading | TransientStatus::Removing)
            ) {
                return Err(AiError::OperationInProgress);
            }
            statuses.insert(model, TransientStatus::Downloading);
        }

        let result = self.download_inner(model);
        let mut statuses = self
            .statuses
            .lock()
            .map_err(|_| AiError::StateUnavailable)?;
        match &result {
            Ok(()) => {
                statuses.remove(&model);
            }
            Err(error) => {
                statuses.insert(model, TransientStatus::Error(error.to_string()));
            }
        }
        result
    }

    fn download_inner(&self, model: AiModel) -> Result<(), AiError> {
        let specification = model.specification();
        let destination = self.model_path(model);
        if destination.is_file() && verify_file(&destination, specification)? {
            return Ok(());
        }
        let temporary = destination.with_extension("onnx.part");
        if temporary.exists() {
            fs::remove_file(&temporary)?;
        }

        let client = reqwest::blocking::Client::builder()
            .timeout(DOWNLOAD_TIMEOUT)
            .user_agent("LumaDesk model manager")
            .build()?;
        let mut response = client.get(specification.url).send()?.error_for_status()?;
        let output = OpenOptions::new()
            .create_new(true)
            .write(true)
            .open(&temporary)?;
        let mut writer = BufWriter::new(output);
        let mut hasher = Sha256::new();
        let mut total = 0_u64;
        let mut buffer = vec![0_u8; 64 * 1_024];
        let transfer = (|| -> Result<(), AiError> {
            loop {
                let read = response.read(&mut buffer)?;
                if read == 0 {
                    break;
                }
                total = total
                    .checked_add(u64::try_from(read).map_err(|_| AiError::Integrity)?)
                    .ok_or(AiError::Integrity)?;
                if total > specification.size_bytes {
                    return Err(AiError::Integrity);
                }
                hasher.update(&buffer[..read]);
                writer.write_all(&buffer[..read])?;
            }
            writer.flush()?;
            writer.get_ref().sync_all()?;
            if total != specification.size_bytes
                || format!("{:x}", hasher.finalize()) != specification.sha256
            {
                return Err(AiError::Integrity);
            }
            Ok(())
        })();
        drop(writer);
        if let Err(error) = transfer {
            drop(fs::remove_file(&temporary));
            return Err(error);
        }
        let finalize = (|| -> Result<(), AiError> {
            let _inference = self
                .inference_lock
                .lock()
                .map_err(|_| AiError::StateUnavailable)?;
            if destination.exists() {
                fs::remove_file(&destination)?;
            }
            fs::rename(&temporary, &destination)?;
            self.runners
                .lock()
                .map_err(|_| AiError::StateUnavailable)?
                .remove(&model);
            Ok(())
        })();
        if finalize.is_err() && temporary.exists() {
            drop(fs::remove_file(temporary));
        }
        finalize
    }

    fn remove(&self, model: AiModel) -> Result<(), AiError> {
        {
            let mut statuses = self
                .statuses
                .lock()
                .map_err(|_| AiError::StateUnavailable)?;
            if matches!(
                statuses.get(&model),
                Some(TransientStatus::Downloading | TransientStatus::Removing)
            ) {
                return Err(AiError::OperationInProgress);
            }
            statuses.insert(model, TransientStatus::Removing);
        }

        let result = self.remove_inner(model);
        let mut statuses = self
            .statuses
            .lock()
            .map_err(|_| AiError::StateUnavailable)?;
        match &result {
            Ok(()) => {
                statuses.remove(&model);
            }
            Err(error) => {
                statuses.insert(model, TransientStatus::Error(error.to_string()));
            }
        }
        result
    }

    fn remove_inner(&self, model: AiModel) -> Result<(), AiError> {
        let _inference = self
            .inference_lock
            .lock()
            .map_err(|_| AiError::StateUnavailable)?;
        for path in [
            self.model_path(model),
            self.model_path(model).with_extension("onnx.part"),
        ] {
            if path.exists() {
                fs::remove_file(path)?;
            }
        }
        let masks = self.mask_model_root(model);
        if masks.exists() {
            fs::remove_dir_all(&masks)?;
        }
        fs::create_dir_all(masks)?;
        self.runners
            .lock()
            .map_err(|_| AiError::StateUnavailable)?
            .remove(&model);
        Ok(())
    }

    fn generate_masks(
        &self,
        model: AiModel,
        media_id: MediaId,
        source_key: &str,
        batch: &FrameBatch,
    ) -> Result<Vec<GeneratedMask>, AiError> {
        if matches!(
            self.statuses
                .lock()
                .map_err(|_| AiError::StateUnavailable)?
                .get(&model),
            Some(TransientStatus::Removing)
        ) {
            return Err(AiError::OperationInProgress);
        }
        let _inference = self
            .inference_lock
            .lock()
            .map_err(|_| AiError::StateUnavailable)?;
        let runner = self.runner(model)?;
        let output_root = self
            .mask_model_root(model)
            .join(media_id.to_string())
            .join(source_key);
        fs::create_dir_all(&output_root)?;
        batch
            .frames
            .iter()
            .map(|frame| {
                let relative_path = format!(
                    "masks/{}/{}/{}/{:08}.png",
                    model.directory_name(),
                    media_id,
                    source_key,
                    frame.index
                );
                let output = self.data_root.join(&relative_path);
                if !output.is_file() {
                    infer_mask(&runner, Path::new(&frame.path), &output)?;
                }
                let dimensions = image::image_dimensions(&output)?;
                Ok(GeneratedMask {
                    index: frame.index,
                    path: output.to_string_lossy().into_owned(),
                    relative_path,
                    width: dimensions.0,
                    height: dimensions.1,
                })
            })
            .collect()
    }

    fn runner(&self, model: AiModel) -> Result<Arc<TypedRunnableModel>, AiError> {
        if let Some(runner) = self
            .runners
            .lock()
            .map_err(|_| AiError::StateUnavailable)?
            .get(&model)
            .cloned()
        {
            return Ok(runner);
        }
        let specification = model.specification();
        let path = self.model_path(model);
        if !path.is_file() {
            return Err(AiError::MissingModel);
        }
        if !verify_file(&path, specification)? {
            return Err(AiError::Integrity);
        }
        let runner = tract_onnx::onnx()
            .model_for_path(path)
            .and_then(|model| {
                model.with_input_fact(0, f32::fact([1, 3, MODEL_SIZE, MODEL_SIZE]).into())
            })
            .and_then(|model| model.into_optimized())
            .and_then(|model| model.into_runnable())
            .map_err(|error| AiError::Inference(error.to_string()))?;
        self.runners
            .lock()
            .map_err(|_| AiError::StateUnavailable)?
            .insert(model, Arc::clone(&runner));
        Ok(runner)
    }

    fn model_path(&self, model: AiModel) -> PathBuf {
        self.model_root.join(model.specification().file_name)
    }

    fn mask_model_root(&self, model: AiModel) -> PathBuf {
        self.data_root.join("masks").join(model.directory_name())
    }
}

#[derive(Clone, Debug)]
struct GeneratedMask {
    index: u32,
    path: String,
    relative_path: String,
    width: u32,
    height: u32,
}

#[tauri::command]
pub(crate) async fn list_ai_models(
    state: State<'_, AiState>,
) -> Result<Vec<AiModelStatus>, CommandError> {
    let manager = Arc::clone(&state.manager);
    tauri::async_runtime::spawn_blocking(move || manager.statuses().map_err(command_error))
        .await
        .map_err(|_| CommandError::unavailable())?
}

#[tauri::command]
pub(crate) async fn download_ai_model(
    state: State<'_, AiState>,
    model: AiModel,
) -> Result<AiModelStatus, CommandError> {
    let manager = Arc::clone(&state.manager);
    tauri::async_runtime::spawn_blocking(move || {
        manager.download(model).map_err(command_error)?;
        manager
            .statuses()
            .map_err(command_error)?
            .into_iter()
            .find(|status| status.model == model)
            .ok_or_else(|| CommandError::invalid_data("The model status could not be read."))
    })
    .await
    .map_err(|_| CommandError::unavailable())?
}

#[tauri::command]
pub(crate) async fn remove_ai_model(
    ai_state: State<'_, AiState>,
    app_state: State<'_, AppState>,
    model: AiModel,
) -> Result<AiModelStatus, CommandError> {
    let generated_media = ai_state.lock_generated_media().await;
    let manager = Arc::clone(&ai_state.manager);
    tauri::async_runtime::spawn_blocking(move || manager.remove(model).map_err(command_error))
        .await
        .map_err(|_| CommandError::unavailable())??;
    let prefix = format!("masks/{}/", model.directory_name());
    with_library(app_state, move |library| {
        library.remove_variants_by_prefix(VariantKind::SegmentationMask, &prefix)?;
        Ok(())
    })
    .await?;
    drop(generated_media);
    list_model_status(ai_state, model).await
}

#[tauri::command]
pub(crate) async fn get_ai_mask_batch(
    ai_state: State<'_, AiState>,
    app_state: State<'_, AppState>,
    id: MediaId,
    model: AiModel,
    offset: u32,
    limit: u32,
) -> Result<AiMaskBatch, CommandError> {
    let _generated_media = ai_state.lock_generated_media().await;
    let (item, sequence, batch) = with_library(app_state.clone(), move |library| {
        Ok((
            library.media_item(id)?,
            library.sprite_sequence(id)?,
            library.frame_batch(id, FrameBatchRequest { offset, limit })?,
        ))
    })
    .await?;
    let source_key = source_key(&item, sequence.as_ref());
    let manager = Arc::clone(&ai_state.manager);
    let masks = tauri::async_runtime::spawn_blocking(move || {
        manager
            .generate_masks(model, id, &source_key, &batch)
            .map_err(command_error)
    })
    .await
    .map_err(|_| CommandError::unavailable())??;
    let records = masks.clone();
    with_library(app_state, move |library| {
        for mask in records {
            library.upsert_media_variant(
                id,
                VariantKind::SegmentationMask,
                &mask.relative_path,
                mask.width,
                mask.height,
            )?;
        }
        Ok(())
    })
    .await?;
    Ok(AiMaskBatch {
        media_id: id,
        model,
        masks: masks
            .into_iter()
            .map(|mask| AiMaskAsset {
                index: mask.index,
                path: mask.path,
            })
            .collect(),
    })
}

async fn list_model_status(
    state: State<'_, AiState>,
    model: AiModel,
) -> Result<AiModelStatus, CommandError> {
    let manager = Arc::clone(&state.manager);
    tauri::async_runtime::spawn_blocking(move || {
        manager
            .statuses()
            .map_err(command_error)?
            .into_iter()
            .find(|status| status.model == model)
            .ok_or_else(|| CommandError::invalid_data("The model status could not be read."))
    })
    .await
    .map_err(|_| CommandError::unavailable())?
}

fn verify_file(path: &Path, specification: ModelSpecification) -> Result<bool, AiError> {
    if path.metadata()?.len() != specification.size_bytes {
        return Ok(false);
    }
    let mut reader = BufReader::new(File::open(path)?);
    let mut hasher = Sha256::new();
    let mut buffer = vec![0_u8; 64 * 1_024];
    loop {
        let read = reader.read(&mut buffer)?;
        if read == 0 {
            break;
        }
        hasher.update(&buffer[..read]);
    }
    Ok(format!("{:x}", hasher.finalize()) == specification.sha256)
}

fn installed_file_size(path: &Path) -> u64 {
    path.metadata().map_or(0, |metadata| metadata.len())
}

fn disk_model_status(
    path: &Path,
    specification: ModelSpecification,
) -> Result<(u64, AiModelAvailability, Option<String>), AiError> {
    let metadata = match path.metadata() {
        Ok(metadata) => metadata,
        Err(error) if error.kind() == io::ErrorKind::NotFound => {
            return Ok((0, AiModelAvailability::Missing, None));
        }
        Err(error) => return Err(error.into()),
    };
    let installed_bytes = metadata.len();
    if metadata.is_file() && verify_file(path, specification).unwrap_or(false) {
        return Ok((installed_bytes, AiModelAvailability::Installed, None));
    }
    Ok((
        installed_bytes,
        AiModelAvailability::Error,
        Some(MODEL_INTEGRITY_MESSAGE.into()),
    ))
}

fn infer_mask(
    runner: &Arc<TypedRunnableModel>,
    source: &Path,
    output: &Path,
) -> Result<(), AiError> {
    let image = image::open(source)?;
    let original_width = image.width();
    let original_height = image.height();
    let resized = image
        .resize_exact(
            u32::try_from(MODEL_SIZE).map_err(|_| AiError::Integrity)?,
            u32::try_from(MODEL_SIZE).map_err(|_| AiError::Integrity)?,
            FilterType::Triangle,
        )
        .to_rgb8();
    let means = [0.485_f32, 0.456, 0.406];
    let deviations = [0.229_f32, 0.224, 0.225];
    let mut input = vec![0.0_f32; 3 * MODEL_SIZE * MODEL_SIZE];
    for channel in 0..3 {
        for y in 0..MODEL_SIZE {
            for x in 0..MODEL_SIZE {
                let pixel = resized.get_pixel(
                    u32::try_from(x).map_err(|_| AiError::Integrity)?,
                    u32::try_from(y).map_err(|_| AiError::Integrity)?,
                );
                let value = f32::from(pixel[channel]) / 255.0;
                input[channel * MODEL_SIZE * MODEL_SIZE + y * MODEL_SIZE + x] =
                    (value - means[channel]) / deviations[channel];
            }
        }
    }
    let tensor = Tensor::from_shape(&[1, 3, MODEL_SIZE, MODEL_SIZE], &input)
        .map_err(|error| AiError::Inference(error.to_string()))?;
    let outputs = runner
        .run(tvec!(tensor.into_tvalue()))
        .map_err(|error| AiError::Inference(error.to_string()))?;
    let values = outputs
        .first()
        .ok_or_else(|| AiError::Inference("the model returned no output".into()))?
        .to_plain_array_view::<f32>()
        .map_err(|error| AiError::Inference(error.to_string()))?;
    let (minimum, maximum) = values.iter().fold(
        (f32::INFINITY, f32::NEG_INFINITY),
        |(minimum, maximum), value| (minimum.min(*value), maximum.max(*value)),
    );
    if !minimum.is_finite() || !maximum.is_finite() {
        return Err(AiError::Inference(
            "the model returned invalid mask values".into(),
        ));
    }
    let range = maximum - minimum;
    let bytes = values
        .iter()
        .map(|value| {
            let normalized = if range > f32::EPSILON {
                (*value - minimum) / range
            } else {
                value.clamp(0.0, 1.0)
            };
            alpha_byte(normalized)
        })
        .collect::<Vec<_>>();
    let model_size = u32::try_from(MODEL_SIZE).map_err(|_| AiError::Integrity)?;
    let mask = GrayImage::from_raw(model_size, model_size, bytes)
        .ok_or_else(|| AiError::Inference("the model returned an invalid mask shape".into()))?;
    let mask =
        image::imageops::resize(&mask, original_width, original_height, FilterType::Lanczos3);
    save_mask_atomically(&mask, output)
}

#[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
fn alpha_byte(value: f32) -> u8 {
    (value.clamp(0.0, 1.0) * 255.0).round() as u8
}

fn save_mask_atomically(mask: &GrayImage, output: &Path) -> Result<(), AiError> {
    let parent = output
        .parent()
        .ok_or_else(|| AiError::Inference("the mask path has no parent directory".into()))?;
    fs::create_dir_all(parent)?;
    let temporary = output.with_extension("png.part");
    if temporary.exists() {
        fs::remove_file(&temporary)?;
    }
    DynamicImage::ImageLuma8(mask.clone()).save_with_format(&temporary, ImageFormat::Png)?;
    File::open(&temporary)?.sync_all()?;
    if output.exists() {
        fs::remove_file(output)?;
    }
    fs::rename(temporary, output)?;
    Ok(())
}

fn source_key(item: &MediaItem, sequence: Option<&SpriteSequence>) -> String {
    let mut hasher = Sha256::new();
    hasher.update(item.content_hash.as_bytes());
    if let Some(sequence) = sequence {
        hasher.update(sequence.columns.to_le_bytes());
        hasher.update(sequence.rows.to_le_bytes());
        hasher.update(sequence.frame_count.to_le_bytes());
        hasher.update(sequence.frame_duration_ms.to_le_bytes());
        hasher.update(sequence.playback_mode.as_str().as_bytes());
    }
    format!("{:x}", hasher.finalize())
}

fn command_error(error: AiError) -> CommandError {
    match error {
        AiError::MissingModel => CommandError::invalid_input(
            "Download the selected local AI model in Settings before using it.",
        ),
        AiError::OperationInProgress => CommandError::invalid_input(
            "Another operation is already running for this local AI model.",
        ),
        AiError::Integrity => CommandError::invalid_data(
            "The local AI model failed verification. Remove it and download it again.",
        ),
        AiError::Inference(message) => CommandError::invalid_data(format!(
            "The local AI model could not process this frame: {message}"
        )),
        AiError::Io(error) => CommandError::native("Local AI files could not be updated", error),
        AiError::Image(error) => {
            CommandError::native("The frame could not be processed by local AI", error)
        }
        AiError::Network(error) => {
            CommandError::native("The local AI model could not be downloaded", error)
        }
        AiError::StateUnavailable => CommandError::unavailable(),
    }
}

#[cfg(test)]
mod tests {
    use std::fs;

    use lumadesk_core::{
        AssetKind, MediaFormat, MediaId, MediaItem, SpritePlaybackMode, SpriteSequence,
    };

    use super::{
        AiModelAvailability, ModelSpecification, disk_model_status, source_key, verify_file,
    };

    fn item() -> MediaItem {
        MediaItem {
            id: MediaId::new(),
            content_hash: "a".repeat(64),
            display_name: "Fixture".into(),
            managed_path: "fixture.png".into(),
            asset_kind: AssetKind::Still,
            media_format: MediaFormat::Png,
            width: 8,
            height: 8,
            frame_count: 1,
            duration_ms: 0,
            is_favorite: false,
            tags: Vec::new(),
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        }
    }

    #[test]
    fn sprite_metadata_changes_the_mask_cache_key() {
        let item = item();
        let first = SpriteSequence {
            media_id: item.id,
            columns: 2,
            rows: 2,
            frame_count: 4,
            frame_duration_ms: 80,
            playback_mode: SpritePlaybackMode::Loop,
            updated_at: chrono::Utc::now(),
        };
        let second = SpriteSequence {
            columns: 4,
            ..first.clone()
        };

        assert_ne!(
            source_key(&item, Some(&first)),
            source_key(&item, Some(&second))
        );
        assert_ne!(source_key(&item, None), source_key(&item, Some(&first)));
    }

    #[test]
    fn model_status_requires_size_and_hash_verification() -> Result<(), Box<dyn std::error::Error>>
    {
        let temp = tempfile::TempDir::new()?;
        let path = temp.path().join("fixture.onnx");
        let specification = ModelSpecification {
            file_name: "fixture.onnx",
            url: "https://example.invalid/fixture.onnx",
            size_bytes: 8,
            sha256: "1c34f88707b55e6104c4eb20e71ffa3d33e414b71ef689a15fad0640d0ac58cb",
        };

        fs::write(&path, b"verified")?;
        assert!(verify_file(&path, specification)?);
        assert_eq!(
            disk_model_status(&path, specification)?.1,
            AiModelAvailability::Installed
        );

        fs::write(&path, b"tampered")?;
        let (installed_bytes, availability, message) = disk_model_status(&path, specification)?;
        assert_eq!(installed_bytes, 8);
        assert_eq!(availability, AiModelAvailability::Error);
        assert!(message.is_some());

        fs::remove_file(&path)?;
        assert_eq!(
            disk_model_status(&path, specification)?.1,
            AiModelAvailability::Missing
        );
        Ok(())
    }
}
