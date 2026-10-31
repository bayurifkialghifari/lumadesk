use std::{
    collections::HashSet,
    fs::{self, File, OpenOptions},
    io::{self, BufReader, BufWriter, Read, Write},
    path::{Path, PathBuf},
    sync::atomic::{AtomicU64, Ordering},
    time::Duration,
};

use lumadesk_core::{ImportOutcome, MediaFormat};
use reqwest::{Url, blocking::Client};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use tauri::State;
use thiserror::Error;

use crate::{
    command_error::CommandError,
    state::{AppState, with_library},
};

const CATALOG_URL: &str =
    "https://raw.githubusercontent.com/bayurifkialghifari/lumadesk/main/gallery/catalog.v1.json";
const CATALOG_SCHEMA_VERSION: u32 = 1;
const CATALOG_CACHE_MAX_AGE: Duration = Duration::from_secs(6 * 60 * 60);
const CATALOG_MAX_BYTES: u64 = 1024 * 1024;
const ASSET_MAX_BYTES: u64 = 100 * 1024 * 1024;
const NETWORK_TIMEOUT: Duration = Duration::from_secs(2 * 60);
const CONNECT_TIMEOUT: Duration = Duration::from_secs(15);
const BUNDLED_CATALOG: &[u8] = include_bytes!("../../gallery/catalog.v1.json");
const NEON_ORBIT: &[u8] = include_bytes!("../../src/assets/gallery/neon-orbit.webp");
const PIXEL_BLOOM: &[u8] = include_bytes!("../../src/assets/gallery/pixel-bloom.gif");
const MOON_JELLY: &[u8] = include_bytes!("../../src/assets/gallery/moon-jelly.png");
const SIGNAL_GHOST: &[u8] = include_bytes!("../../src/assets/gallery/signal-ghost.webp");
static TEMPORARY_SEQUENCE: AtomicU64 = AtomicU64::new(0);

#[derive(Debug)]
pub(crate) struct GalleryState {
    root: PathBuf,
    operations: async_lock::Mutex<()>,
}

impl GalleryState {
    pub(crate) fn open(data_root: &Path) -> Result<Self, io::Error> {
        let root = data_root.join("gallery");
        fs::create_dir_all(root.join("previews"))?;
        fs::create_dir_all(root.join("downloads"))?;
        fs::create_dir_all(root.join("imports"))?;
        Ok(Self {
            root,
            operations: async_lock::Mutex::new(()),
        })
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub(crate) struct GalleryCatalog {
    schema_version: u32,
    revision: u64,
    items: Vec<GalleryItem>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub(crate) struct GalleryItem {
    id: String,
    title: String,
    description: String,
    author: String,
    license_name: String,
    license_url: String,
    source_url: String,
    tags: Vec<String>,
    preview: GalleryAsset,
    download: GalleryAsset,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub(crate) struct GalleryAsset {
    url: String,
    sha256: String,
    size_bytes: u64,
    media_format: MediaFormat,
}

#[derive(Clone, Copy, Debug, Serialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum GalleryCatalogSource {
    Network,
    Cache,
    Bundled,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct GalleryCatalogResponse {
    schema_version: u32,
    revision: u64,
    items: Vec<GalleryItem>,
    source: GalleryCatalogSource,
    installed_hashes: Vec<String>,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub(crate) struct GalleryItemInput {
    item_id: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct GalleryPreview {
    item_id: String,
    path: String,
}

#[derive(Debug, Error)]
enum GalleryError {
    #[error("file operation failed: {0}")]
    Io(#[from] io::Error),
    #[error("network request failed: {0}")]
    Network(#[from] reqwest::Error),
    #[error("catalog JSON is invalid: {0}")]
    Json(#[from] serde_json::Error),
    #[error("invalid gallery data: {0}")]
    InvalidData(String),
    #[error("download verification failed for {0}")]
    Integrity(String),
    #[error("invalid gallery request: {0}")]
    InvalidInput(String),
}

#[derive(Debug)]
struct LoadedCatalog {
    catalog: GalleryCatalog,
    source: GalleryCatalogSource,
}

/// Loads the versioned Gallery catalog with network, disk-cache, and bundled fallbacks.
#[tauri::command]
pub(crate) async fn fetch_gallery_catalog(
    gallery: State<'_, GalleryState>,
    app: State<'_, AppState>,
    refresh: bool,
) -> Result<GalleryCatalogResponse, CommandError> {
    let loaded = {
        let _operation = gallery.operations.lock().await;
        let root = gallery.root.clone();
        tauri::async_runtime::spawn_blocking(move || load_catalog(&root, refresh))
            .await
            .map_err(|_| CommandError::unavailable())?
            .map_err(command_error)?
    };
    let candidate_hashes = loaded
        .catalog
        .items
        .iter()
        .map(|item| item.download.sha256.clone())
        .collect::<Vec<_>>();
    let installed_hashes = with_library(app, move |library| {
        library.existing_content_hashes(&candidate_hashes)
    })
    .await?;
    Ok(GalleryCatalogResponse {
        schema_version: loaded.catalog.schema_version,
        revision: loaded.catalog.revision,
        items: loaded.catalog.items,
        source: loaded.source,
        installed_hashes,
    })
}

/// Produces a verified local preview path accepted by the Tauri asset protocol.
#[tauri::command]
pub(crate) async fn cache_gallery_preview(
    gallery: State<'_, GalleryState>,
    input: GalleryItemInput,
) -> Result<GalleryPreview, CommandError> {
    let _operation = gallery.operations.lock().await;
    let root = gallery.root.clone();
    tauri::async_runtime::spawn_blocking(move || {
        let loaded = load_catalog(&root, false)?;
        let item = find_item(&loaded.catalog, &input.item_id)?;
        let path = cache_asset(&root, "previews", &item.id, &item.preview)?;
        Ok(GalleryPreview {
            item_id: item.id,
            path: path.to_string_lossy().into_owned(),
        })
    })
    .await
    .map_err(|_| CommandError::unavailable())?
    .map_err(command_error)
}

/// Downloads and imports one verified Gallery item into the managed collection.
#[tauri::command]
pub(crate) async fn import_gallery_item(
    gallery: State<'_, GalleryState>,
    app: State<'_, AppState>,
    input: GalleryItemInput,
) -> Result<ImportOutcome, CommandError> {
    let (path, title, tags) = {
        let _operation = gallery.operations.lock().await;
        let root = gallery.root.clone();
        tauri::async_runtime::spawn_blocking(move || {
            let loaded = load_catalog(&root, false)?;
            let item = find_item(&loaded.catalog, &input.item_id)?;
            let path = cache_asset(&root, "downloads", &item.id, &item.download)?;
            Ok::<_, GalleryError>((path, item.title, item.tags))
        })
        .await
        .map_err(|_| CommandError::unavailable())?
        .map_err(command_error)?
    };
    with_library(app, move |library| {
        library.import_file_with_metadata(&path, &title, &tags)
    })
    .await
}

/// Downloads an HTTPS media URL and imports it through the standard managed-file pipeline.
#[tauri::command]
pub(crate) async fn import_media_url(
    gallery: State<'_, GalleryState>,
    app: State<'_, AppState>,
    url: String,
    name: Option<String>,
) -> Result<ImportOutcome, CommandError> {
    let (temporary, display_name) = {
        let _operation = gallery.operations.lock().await;
        let root = gallery.root.clone();
        tauri::async_runtime::spawn_blocking(move || download_direct_url(&root, &url, name))
            .await
            .map_err(|_| CommandError::unavailable())?
            .map_err(command_error)?
    };
    let import_path = temporary.clone();
    let outcome = with_library(app, move |library| {
        library.import_file_with_metadata(&import_path, &display_name, &[])
    })
    .await;
    drop(fs::remove_file(temporary));
    outcome
}

fn load_catalog(root: &Path, refresh: bool) -> Result<LoadedCatalog, GalleryError> {
    let cache_path = root.join("catalog.v1.json");
    if !refresh
        && cache_is_fresh(&cache_path)
        && let Ok(catalog) = read_catalog_file(&cache_path)
    {
        return Ok(LoadedCatalog {
            catalog,
            source: GalleryCatalogSource::Cache,
        });
    }

    if let Ok(bytes) = download_bytes(CATALOG_URL, CATALOG_MAX_BYTES)
        && let Ok(catalog) = parse_catalog(&bytes)
    {
        write_atomic(&cache_path, &bytes)?;
        return Ok(LoadedCatalog {
            catalog,
            source: GalleryCatalogSource::Network,
        });
    }

    if let Ok(catalog) = read_catalog_file(&cache_path) {
        return Ok(LoadedCatalog {
            catalog,
            source: GalleryCatalogSource::Cache,
        });
    }

    Ok(LoadedCatalog {
        catalog: parse_catalog(BUNDLED_CATALOG)?,
        source: GalleryCatalogSource::Bundled,
    })
}

fn read_catalog_file(path: &Path) -> Result<GalleryCatalog, GalleryError> {
    let metadata = path.metadata()?;
    if metadata.len() > CATALOG_MAX_BYTES {
        return Err(GalleryError::InvalidData(
            "the cached catalog is too large".into(),
        ));
    }
    parse_catalog(&fs::read(path)?)
}

fn parse_catalog(bytes: &[u8]) -> Result<GalleryCatalog, GalleryError> {
    let catalog: GalleryCatalog = serde_json::from_slice(bytes)?;
    validate_catalog(&catalog)?;
    Ok(catalog)
}

fn validate_catalog(catalog: &GalleryCatalog) -> Result<(), GalleryError> {
    if catalog.schema_version != CATALOG_SCHEMA_VERSION {
        return Err(GalleryError::InvalidData(format!(
            "unsupported schema version {}",
            catalog.schema_version
        )));
    }
    if catalog.revision == 0 {
        return Err(GalleryError::InvalidData(
            "catalog revision must be positive".into(),
        ));
    }
    if catalog.items.is_empty() || catalog.items.len() > 500 {
        return Err(GalleryError::InvalidData(
            "catalog must contain 1–500 items".into(),
        ));
    }
    let mut identifiers = HashSet::new();
    for item in &catalog.items {
        if !valid_identifier(&item.id) || !identifiers.insert(item.id.as_str()) {
            return Err(GalleryError::InvalidData(format!(
                "invalid or duplicate item ID {}",
                item.id
            )));
        }
        validate_text("title", &item.title, 1, 120)?;
        validate_text("description", &item.description, 1, 500)?;
        validate_text("author", &item.author, 1, 120)?;
        validate_text("license name", &item.license_name, 1, 80)?;
        validate_https_url(&item.license_url)?;
        validate_https_url(&item.source_url)?;
        if item.tags.is_empty() || item.tags.len() > 20 {
            return Err(GalleryError::InvalidData(format!(
                "item {} must contain 1–20 tags",
                item.id
            )));
        }
        let mut tags = HashSet::new();
        for tag in &item.tags {
            validate_text("tag", tag, 1, 32)?;
            if !tags.insert(tag.to_lowercase()) {
                return Err(GalleryError::InvalidData(format!(
                    "item {} contains duplicate tags",
                    item.id
                )));
            }
        }
        validate_asset(&item.preview)?;
        validate_asset(&item.download)?;
    }
    Ok(())
}

fn validate_asset(asset: &GalleryAsset) -> Result<(), GalleryError> {
    validate_https_url(&asset.url)?;
    if asset.size_bytes == 0 || asset.size_bytes > ASSET_MAX_BYTES {
        return Err(GalleryError::InvalidData(
            "asset size is outside the supported range".into(),
        ));
    }
    if asset.sha256.len() != 64
        || !asset
            .sha256
            .bytes()
            .all(|value| value.is_ascii_digit() || (b'a'..=b'f').contains(&value))
    {
        return Err(GalleryError::InvalidData(
            "asset SHA-256 must be lowercase hexadecimal".into(),
        ));
    }
    if !matches!(
        asset.media_format,
        MediaFormat::Gif
            | MediaFormat::Apng
            | MediaFormat::Webp
            | MediaFormat::Png
            | MediaFormat::Jpeg
    ) {
        return Err(GalleryError::InvalidData(
            "gallery assets must use a supported image format".into(),
        ));
    }
    Ok(())
}

fn validate_text(
    label: &str,
    value: &str,
    minimum: usize,
    maximum: usize,
) -> Result<(), GalleryError> {
    let length = value.trim().chars().count();
    if length < minimum || length > maximum {
        return Err(GalleryError::InvalidData(format!(
            "{label} must contain {minimum}–{maximum} characters"
        )));
    }
    Ok(())
}

fn valid_identifier(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 80
        && value.split('-').all(|segment| {
            !segment.is_empty()
                && segment
                    .bytes()
                    .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit())
        })
}

fn validate_https_url(value: &str) -> Result<Url, GalleryError> {
    let url = Url::parse(value)
        .map_err(|_| GalleryError::InvalidData("catalog URL is invalid".into()))?;
    if url.scheme() != "https"
        || url.host_str().is_none()
        || !url.username().is_empty()
        || url.password().is_some()
    {
        return Err(GalleryError::InvalidData(
            "gallery URLs must use HTTPS without embedded credentials".into(),
        ));
    }
    Ok(url)
}

fn find_item(catalog: &GalleryCatalog, item_id: &str) -> Result<GalleryItem, GalleryError> {
    catalog
        .items
        .iter()
        .find(|item| item.id == item_id)
        .cloned()
        .ok_or_else(|| GalleryError::InvalidInput("The Gallery item was not found.".into()))
}

fn cache_asset(
    root: &Path,
    directory: &str,
    item_id: &str,
    asset: &GalleryAsset,
) -> Result<PathBuf, GalleryError> {
    let destination = root
        .join(directory)
        .join(format!("{item_id}.{}", asset.media_format.extension()));
    if verify_file(&destination, asset)? {
        return Ok(destination);
    }
    if destination.exists() {
        fs::remove_file(&destination)?;
    }
    if let Some(bytes) = bundled_asset(&asset.sha256) {
        if u64::try_from(bytes.len()).ok() != Some(asset.size_bytes)
            || hash_bytes(bytes) != asset.sha256
        {
            return Err(GalleryError::Integrity(item_id.into()));
        }
        write_atomic(&destination, bytes)?;
    } else {
        download_verified(asset, &destination)?;
    }
    if !verify_file(&destination, asset)? {
        drop(fs::remove_file(&destination));
        return Err(GalleryError::Integrity(item_id.into()));
    }
    Ok(destination)
}

fn download_verified(asset: &GalleryAsset, destination: &Path) -> Result<(), GalleryError> {
    let url = validate_https_url(&asset.url)?;
    let client = network_client()?;
    let mut response = client.get(url).send()?.error_for_status()?;
    validate_final_url(response.url())?;
    if response
        .content_length()
        .is_some_and(|length| length > asset.size_bytes)
    {
        return Err(GalleryError::Integrity(asset.url.clone()));
    }
    let temporary = destination.with_extension(format!("{}.part", asset.media_format.extension()));
    if temporary.exists() {
        fs::remove_file(&temporary)?;
    }
    let output = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&temporary)?;
    let transfer = stream_response(
        &mut response,
        output,
        asset.size_bytes,
        Some((&asset.sha256, asset.size_bytes)),
    );
    if let Err(error) = transfer {
        drop(fs::remove_file(&temporary));
        return Err(error);
    }
    if destination.exists() {
        fs::remove_file(destination)?;
    }
    fs::rename(&temporary, destination)?;
    Ok(())
}

fn download_direct_url(
    root: &Path,
    url: &str,
    name: Option<String>,
) -> Result<(PathBuf, String), GalleryError> {
    let parsed = Url::parse(url)
        .map_err(|_| GalleryError::InvalidInput("Enter a valid HTTPS media URL.".into()))?;
    if parsed.scheme() != "https" || parsed.host_str().is_none() {
        return Err(GalleryError::InvalidInput(
            "Media URLs must use HTTPS.".into(),
        ));
    }
    let display_name = match name {
        Some(value) if !value.trim().is_empty() => value.trim().to_owned(),
        _ => suggested_name(&parsed),
    };
    if display_name.chars().count() > 120 {
        return Err(GalleryError::InvalidInput(
            "The name must contain 1–120 characters.".into(),
        ));
    }
    let client = network_client()?;
    let mut response = client.get(parsed).send()?.error_for_status()?;
    validate_final_url(response.url())?;
    if response
        .content_length()
        .is_some_and(|length| length > ASSET_MAX_BYTES)
    {
        return Err(GalleryError::InvalidInput(
            "The remote media file is larger than 100 MiB.".into(),
        ));
    }
    let sequence = TEMPORARY_SEQUENCE.fetch_add(1, Ordering::Relaxed);
    let destination = root.join("imports").join(format!(
        ".url-import-{}-{sequence}.part",
        std::process::id()
    ));
    let output = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&destination)?;
    if let Err(error) = stream_response(&mut response, output, ASSET_MAX_BYTES, None) {
        drop(fs::remove_file(&destination));
        return Err(error);
    }
    Ok((destination, display_name))
}

fn stream_response(
    response: &mut impl Read,
    output: File,
    maximum_bytes: u64,
    expected: Option<(&str, u64)>,
) -> Result<(), GalleryError> {
    let mut writer = BufWriter::new(output);
    let mut hasher = Sha256::new();
    let mut total = 0_u64;
    let mut buffer = vec![0_u8; 64 * 1024];
    loop {
        let read = response.read(&mut buffer)?;
        if read == 0 {
            break;
        }
        total = total
            .checked_add(
                u64::try_from(read).map_err(|_| {
                    GalleryError::InvalidData("download byte count is invalid".into())
                })?,
            )
            .ok_or_else(|| GalleryError::InvalidData("download byte count overflowed".into()))?;
        if total > maximum_bytes {
            return Err(GalleryError::InvalidInput(
                "The remote media file is larger than 100 MiB.".into(),
            ));
        }
        hasher.update(&buffer[..read]);
        writer.write_all(&buffer[..read])?;
    }
    writer.flush()?;
    writer.get_ref().sync_all()?;
    if let Some((hash, size)) = expected
        && (total != size || format!("{:x}", hasher.finalize()) != hash)
    {
        return Err(GalleryError::Integrity("remote asset".into()));
    }
    Ok(())
}

fn download_bytes(url: &str, maximum_bytes: u64) -> Result<Vec<u8>, GalleryError> {
    let url = validate_https_url(url)?;
    let client = network_client()?;
    let mut response = client.get(url).send()?.error_for_status()?;
    validate_final_url(response.url())?;
    if response
        .content_length()
        .is_some_and(|length| length > maximum_bytes)
    {
        return Err(GalleryError::InvalidData(
            "the remote catalog is too large".into(),
        ));
    }
    let mut bytes = Vec::new();
    let mut buffer = vec![0_u8; 32 * 1024];
    loop {
        let read = response.read(&mut buffer)?;
        if read == 0 {
            break;
        }
        let next = bytes
            .len()
            .checked_add(read)
            .ok_or_else(|| GalleryError::InvalidData("catalog size overflowed".into()))?;
        if u64::try_from(next)
            .ok()
            .is_none_or(|size| size > maximum_bytes)
        {
            return Err(GalleryError::InvalidData(
                "the remote catalog is too large".into(),
            ));
        }
        bytes.extend_from_slice(&buffer[..read]);
    }
    Ok(bytes)
}

fn network_client() -> Result<Client, GalleryError> {
    Ok(Client::builder()
        .connect_timeout(CONNECT_TIMEOUT)
        .timeout(NETWORK_TIMEOUT)
        .redirect(reqwest::redirect::Policy::limited(5))
        .user_agent(concat!("LumaDesk/", env!("CARGO_PKG_VERSION")))
        .build()?)
}

fn validate_final_url(url: &Url) -> Result<(), GalleryError> {
    if url.scheme() != "https" {
        return Err(GalleryError::InvalidData(
            "the remote server redirected to a non-HTTPS URL".into(),
        ));
    }
    Ok(())
}

fn suggested_name(url: &Url) -> String {
    url.path_segments()
        .and_then(Iterator::last)
        .map(|segment| segment.rsplit_once('.').map_or(segment, |(stem, _)| stem))
        .map(|value| value.replace(['-', '_'], " "))
        .filter(|value| !value.trim().is_empty() && value.chars().count() <= 120)
        .unwrap_or_else(|| "Downloaded animation".into())
}

fn verify_file(path: &Path, asset: &GalleryAsset) -> Result<bool, GalleryError> {
    if !path.is_file() || path.metadata()?.len() != asset.size_bytes {
        return Ok(false);
    }
    let mut reader = BufReader::new(File::open(path)?);
    let mut hasher = Sha256::new();
    let mut buffer = vec![0_u8; 64 * 1024];
    loop {
        let read = reader.read(&mut buffer)?;
        if read == 0 {
            break;
        }
        hasher.update(&buffer[..read]);
    }
    Ok(format!("{:x}", hasher.finalize()) == asset.sha256)
}

fn write_atomic(path: &Path, bytes: &[u8]) -> Result<(), GalleryError> {
    let parent = path
        .parent()
        .ok_or_else(|| GalleryError::InvalidData("cache path has no parent".into()))?;
    fs::create_dir_all(parent)?;
    let temporary = path.with_extension("tmp");
    if temporary.exists() {
        fs::remove_file(&temporary)?;
    }
    let output = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&temporary)?;
    let mut writer = BufWriter::new(output);
    writer.write_all(bytes)?;
    writer.flush()?;
    writer.get_ref().sync_all()?;
    drop(writer);
    if path.exists() {
        fs::remove_file(path)?;
    }
    fs::rename(&temporary, path)?;
    Ok(())
}

fn cache_is_fresh(path: &Path) -> bool {
    path.metadata()
        .and_then(|metadata| metadata.modified())
        .and_then(|modified| modified.elapsed())
        .is_ok_and(|age| age <= CATALOG_CACHE_MAX_AGE)
}

fn hash_bytes(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

fn bundled_asset(hash: &str) -> Option<&'static [u8]> {
    match hash {
        "7cb74e7231ebd5969ff519902fee6e40e01f6730806a8741bc9d726005267f7e" => Some(NEON_ORBIT),
        "21ab8cedef0dccc7a466e2d690730d35f512e06d03a5c15e86f91fbe3c03f502" => Some(PIXEL_BLOOM),
        "9b9fb55565335a619f451148810919030bffe230b4d3eb7a69cfe4e5039bd9a4" => Some(MOON_JELLY),
        "6b0e7d0605aaed59c3fd73cfac983f7bcb1308ae21cde5de2f0a50f593449eaa" => Some(SIGNAL_GHOST),
        _ => None,
    }
}

fn command_error(error: GalleryError) -> CommandError {
    match error {
        GalleryError::InvalidInput(message) => CommandError::invalid_input(message),
        GalleryError::InvalidData(message) => CommandError::invalid_data(message),
        GalleryError::Integrity(item) => CommandError::invalid_data(format!(
            "The Gallery file for {item} failed integrity verification. Refresh the Gallery and try again."
        )),
        GalleryError::Network(error) => {
            CommandError::native("The Gallery could not reach its static catalog", error)
        }
        GalleryError::Io(error) => {
            CommandError::native("The Gallery cache could not be updated", error)
        }
        GalleryError::Json(error) => {
            CommandError::invalid_data(format!("The Gallery catalog could not be read: {error}"))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bundled_catalog_and_assets_pass_integrity_validation() -> Result<(), GalleryError> {
        let catalog = parse_catalog(BUNDLED_CATALOG)?;
        assert_eq!(catalog.items.len(), 4);
        for item in catalog.items {
            let bytes = bundled_asset(&item.download.sha256)
                .ok_or_else(|| GalleryError::Integrity(item.id.clone()))?;
            assert_eq!(
                u64::try_from(bytes.len()).ok(),
                Some(item.download.size_bytes)
            );
            assert_eq!(hash_bytes(bytes), item.download.sha256);
        }
        Ok(())
    }

    #[test]
    fn rejects_insecure_urls_and_invalid_identifiers() -> Result<(), GalleryError> {
        assert!(validate_https_url("http://example.com/file.gif").is_err());
        assert!(valid_identifier("pixel-bloom"));
        assert!(!valid_identifier("Pixel Bloom"));
        let url = validate_https_url("https://example.com/neon-orbit.gif")?;
        assert_eq!(suggested_name(&url), "neon orbit");
        Ok(())
    }
}
