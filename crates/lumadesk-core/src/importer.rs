use std::{
    fs::{self, File, OpenOptions},
    io::{BufReader, BufWriter, Read, Write},
    path::{Path, PathBuf},
};

use scopeguard::{ScopeGuard, guard};
use sha2::{Digest, Sha256};

use crate::{
    ApiError, AppSettings, CoreError, CreatorChanges, CreatorChangesInput, DeleteReport,
    DesktopScene, DesktopSceneId, DesktopSceneItem, DesktopSceneSummary, EditRecipe,
    EditRecipePatch, FrameBatch, FrameBatchRequest, ImportFailure, ImportOutcome, ImportReport,
    LibraryQuery, MediaId, MediaItem, MediaPage, MediaVariant, NewOverlay, OverlayBatchPatch,
    OverlayId, OverlayInstance, OverlayPatch, OverlayReplacement, SpriteSequence,
    SpriteSequenceInput, VariantKind,
    database::Database,
    domain::{NewMediaItem, display_name},
    frame_cache::FrameCache,
    media,
};

#[derive(Debug)]
pub struct Library {
    database: Database,
    data_root: PathBuf,
    media_root: PathBuf,
    frame_cache: FrameCache,
}

impl Library {
    /// Opens or creates a library inside the supplied application data directory.
    ///
    /// # Errors
    /// Returns an error when directories, the `SQLite` database, or migrations cannot be prepared.
    pub fn open(data_root: impl AsRef<Path>) -> Result<Self, CoreError> {
        let data_root = data_root.as_ref().to_path_buf();
        let media_root = data_root.join("media");
        fs::create_dir_all(&media_root)?;
        fs::create_dir_all(data_root.join("thumbnails"))?;
        let frame_cache = FrameCache::open(data_root.join("frames"))?;
        let database = Database::open(&data_root.join("lumadesk.sqlite3"))?;
        let library = Self {
            database,
            data_root,
            media_root,
            frame_cache,
        };
        library.remove_stale_imports()?;
        Ok(library)
    }

    /// Lists catalog items using search, filter, and pagination options.
    ///
    /// # Errors
    /// Returns an error when catalog data cannot be read or contains invalid stored values.
    pub fn list_media(&self, query: &LibraryQuery) -> Result<MediaPage, CoreError> {
        self.database.list_media(query)
    }

    /// Imports multiple paths and keeps per-file failures inside one report.
    pub fn import_paths<I, P>(&mut self, paths: I) -> ImportReport
    where
        I: IntoIterator<Item = P>,
        P: AsRef<Path>,
    {
        let mut report = ImportReport::default();
        for path in paths {
            let path = path.as_ref();
            match self.import_file(path) {
                Ok(outcome) => report.outcomes.push(outcome),
                Err(error) => {
                    let api_error = ApiError::from(&error);
                    report.failures.push(ImportFailure {
                        path: path.to_string_lossy().into_owned(),
                        code: api_error.code,
                        message: api_error.message,
                    });
                }
            }
        }
        report
    }

    /// Imports one media file into the managed store.
    ///
    /// # Errors
    /// Returns an error for unreadable, corrupt, unsupported, or unstorable media.
    pub fn import_file(&mut self, source: &Path) -> Result<ImportOutcome, CoreError> {
        let source_metadata = source.metadata()?;
        if !source_metadata.is_file() {
            return Err(CoreError::InvalidInput(
                "Choose a regular media file.".into(),
            ));
        }
        if source_metadata.len() == 0 {
            return Err(CoreError::InvalidMedia("The file is empty.".into()));
        }

        let temporary = self
            .media_root
            .join(format!(".import-{}.part", uuid::Uuid::new_v4()));
        let mut cleanup = guard(temporary.clone(), |path| drop(fs::remove_file(path)));
        let content_hash = copy_and_hash(source, &temporary)?;
        if let Some(item) = self.database.media_by_hash(&content_hash)? {
            return Ok(ImportOutcome::Duplicate { item });
        }

        let metadata = media::inspect(&temporary)?;
        let id = MediaId::new();
        let managed_path = format!("{}.{}", id, metadata.format.extension());
        let final_path = self.media_root.join(&managed_path);
        fs::rename(&temporary, &final_path)?;
        *cleanup = final_path;

        let item = self.database.insert_media(&NewMediaItem {
            id,
            content_hash,
            display_name: display_name(source),
            managed_path,
            asset_kind: metadata.asset_kind,
            media_format: metadata.format,
            width: metadata.width,
            height: metadata.height,
            frame_count: metadata.frame_count,
            duration_ms: metadata.duration_ms,
        })?;
        drop(ScopeGuard::into_inner(cleanup));
        Ok(ImportOutcome::Imported { item })
    }

    /// Changes the catalog label without renaming the managed file.
    ///
    /// # Errors
    /// Returns an error for an invalid name, missing item, or failed database write.
    pub fn rename_media(&mut self, id: MediaId, name: &str) -> Result<MediaItem, CoreError> {
        self.database.rename_media(id, name)
    }

    /// Changes an item's favorite state.
    ///
    /// # Errors
    /// Returns an error when the item is missing or the database cannot be updated.
    pub fn set_favorite(&mut self, id: MediaId, favorite: bool) -> Result<MediaItem, CoreError> {
        self.database.set_favorite(id, favorite)
    }

    /// Replaces every tag attached to an item.
    ///
    /// # Errors
    /// Returns an error for invalid tags, a missing item, or a failed database transaction.
    pub fn replace_tags(&mut self, id: MediaId, tags: &[String]) -> Result<MediaItem, CoreError> {
        self.database.replace_tags(id, tags)
    }

    /// Deletes an item, its overlays, managed media, and regenerable cache.
    ///
    /// # Errors
    /// Returns an error when the item is missing or managed data cannot be removed safely.
    pub fn delete_media(&mut self, id: MediaId) -> Result<DeleteReport, CoreError> {
        let item = self.database.media_by_id(id)?;
        let managed = self.resolve_managed_path(&item.managed_path)?;
        let quarantine = self.media_root.join(format!(".delete-{id}"));
        let was_moved = if managed.exists() {
            fs::rename(&managed, &quarantine)?;
            true
        } else {
            false
        };
        let report = match self.database.delete_media_record(id) {
            Ok(report) => report,
            Err(error) => {
                if was_moved {
                    drop(fs::rename(&quarantine, &managed));
                }
                return Err(error);
            }
        };
        if was_moved {
            fs::remove_file(quarantine)?;
        }
        self.remove_cache_for(id)?;
        self.remove_mask_directories(id)?;
        Ok(report)
    }

    /// Returns one catalog item by its internal identifier.
    ///
    /// # Errors
    /// Returns an error when the item is missing or stored data is invalid.
    pub fn media_item(&self, id: MediaId) -> Result<MediaItem, CoreError> {
        self.database.media_by_id(id)
    }

    /// Returns the non-destructive edit recipe for an asset.
    ///
    /// # Errors
    /// Returns an error when the asset or stored recipe is unavailable.
    pub fn edit_recipe(&self, id: MediaId) -> Result<EditRecipe, CoreError> {
        self.database.edit_recipe(id)
    }

    /// Applies validated, non-destructive edits without changing source bytes.
    ///
    /// # Errors
    /// Returns an error when values are invalid or persistence fails.
    pub fn update_edit_recipe(
        &mut self,
        id: MediaId,
        patch: &EditRecipePatch,
    ) -> Result<EditRecipe, CoreError> {
        self.database.update_edit_recipe(id, patch)
    }

    /// Saves a complete Creator Studio edit as one database transaction.
    ///
    /// # Errors
    /// Returns an error when the recipe or sprite configuration is invalid or cannot be stored.
    pub fn save_creator_changes(
        &mut self,
        input: &CreatorChangesInput,
    ) -> Result<CreatorChanges, CoreError> {
        let current = self.database.sprite_sequence(input.media_id)?;
        let sprite_changed = input.sprite_sequence.as_ref().map_or_else(
            || input.clear_sprite_sequence && current.is_some(),
            |next| {
                current.as_ref().is_none_or(|stored| {
                    stored.columns != next.columns
                        || stored.rows != next.rows
                        || stored.frame_count != next.frame_count
                        || stored.frame_duration_ms != next.frame_duration_ms
                        || stored.playback_mode != next.playback_mode
                })
            },
        );
        let saved = self.database.save_creator_changes(input)?;
        if sprite_changed {
            self.frame_cache.remove(input.media_id)?;
            self.remove_segmentation_variants(input.media_id)?;
        }
        Ok(saved)
    }

    /// Returns sprite-sheet playback metadata when configured.
    ///
    /// # Errors
    /// Returns an error when the asset or stored metadata is invalid.
    pub fn sprite_sequence(&self, id: MediaId) -> Result<Option<SpriteSequence>, CoreError> {
        self.database.sprite_sequence(id)
    }

    /// Creates or replaces sprite-sheet playback metadata.
    ///
    /// # Errors
    /// Returns an error when the grid is invalid or persistence fails.
    pub fn set_sprite_sequence(
        &mut self,
        sequence: &SpriteSequenceInput,
    ) -> Result<SpriteSequence, CoreError> {
        let saved = self.database.set_sprite_sequence(sequence)?;
        self.frame_cache.remove(sequence.media_id)?;
        self.remove_segmentation_variants(sequence.media_id)?;
        Ok(saved)
    }

    /// Removes sprite-sheet playback metadata and restores still-image behavior.
    ///
    /// # Errors
    /// Returns an error when the asset is missing or persistence fails.
    pub fn clear_sprite_sequence(&mut self, id: MediaId) -> Result<(), CoreError> {
        self.database.clear_sprite_sequence(id)?;
        self.frame_cache.remove(id)?;
        self.remove_segmentation_variants(id)
    }

    /// Lists generated derivatives while keeping the imported source immutable.
    ///
    /// # Errors
    /// Returns an error when the asset or stored variant metadata is invalid.
    pub fn media_variants(&self, id: MediaId) -> Result<Vec<MediaVariant>, CoreError> {
        self.database.media_variants(id)
    }

    /// Registers a generated derivative using a path relative to the application data directory.
    ///
    /// # Errors
    /// Returns an error for invalid metadata, a missing source item, or failed persistence.
    pub fn upsert_media_variant(
        &mut self,
        id: MediaId,
        kind: VariantKind,
        relative_path: &str,
        width: u32,
        height: u32,
    ) -> Result<MediaVariant, CoreError> {
        self.database
            .upsert_media_variant(id, kind, relative_path, width, height)
    }

    /// Removes generated derivatives of one kind for a media item.
    ///
    /// # Errors
    /// Returns an error when the source item is missing or variant metadata cannot be removed.
    pub fn remove_media_variants(
        &mut self,
        id: MediaId,
        kind: VariantKind,
    ) -> Result<Vec<String>, CoreError> {
        self.database.remove_media_variants(id, kind)
    }

    /// Removes generated derivatives whose relative paths begin with the supplied prefix.
    ///
    /// # Errors
    /// Returns an error for an unsafe prefix or failed persistence.
    pub fn remove_variants_by_prefix(
        &mut self,
        kind: VariantKind,
        relative_prefix: &str,
    ) -> Result<Vec<String>, CoreError> {
        self.database
            .remove_variants_by_prefix(kind, relative_prefix)
    }

    /// Resolves an internal media identifier to its managed local path.
    ///
    /// # Errors
    /// Returns an error when the item is missing or its stored relative path is invalid.
    pub fn managed_media_path(&self, id: MediaId) -> Result<PathBuf, CoreError> {
        let item = self.database.media_by_id(id)?;
        self.resolve_managed_path(&item.managed_path)
    }

    /// Decodes media into the persistent frame cache and returns one bounded frame batch.
    ///
    /// # Errors
    /// Returns an error when media is missing, cache data cannot be written, or decoding fails.
    pub fn frame_batch(
        &mut self,
        id: MediaId,
        request: FrameBatchRequest,
    ) -> Result<FrameBatch, CoreError> {
        let item = self.database.media_by_id(id)?;
        let source = self.resolve_managed_path(&item.managed_path)?;
        let sequence = self.database.sprite_sequence(id)?;
        self.frame_cache
            .batch(&item, &source, sequence.as_ref(), request)
    }

    /// Lists every persisted desktop overlay.
    ///
    /// # Errors
    /// Returns an error when overlay records cannot be read or decoded.
    pub fn list_overlays(&self) -> Result<Vec<OverlayInstance>, CoreError> {
        self.database.list_overlays()
    }

    /// Returns one persisted desktop overlay by its internal identifier.
    ///
    /// # Errors
    /// Returns an error when the overlay is missing or its stored data is invalid.
    pub fn overlay(&self, id: OverlayId) -> Result<OverlayInstance, CoreError> {
        self.database.overlay_by_id(id)
    }

    /// Creates a persisted overlay using the current default settings.
    ///
    /// # Errors
    /// Returns an error for invalid coordinates, missing media, or failed persistence.
    pub fn create_overlay(&mut self, input: &NewOverlay) -> Result<OverlayInstance, CoreError> {
        self.database.create_overlay(input)
    }

    /// Applies validated changes to an existing overlay.
    ///
    /// # Errors
    /// Returns an error for invalid values, a missing overlay, or failed persistence.
    pub fn update_overlay(
        &mut self,
        id: OverlayId,
        patch: &OverlayPatch,
    ) -> Result<OverlayInstance, CoreError> {
        self.database.update_overlay(id, patch)
    }

    /// Applies one patch to every persisted overlay in a single database statement.
    ///
    /// # Errors
    /// Returns an error when overlay records cannot be updated or decoded.
    pub fn patch_all_overlays(
        &mut self,
        patch: &OverlayBatchPatch,
    ) -> Result<Vec<OverlayInstance>, CoreError> {
        self.database.patch_all_overlays(patch)
    }

    /// Copies an overlay's controls to a new persisted instance at another position.
    ///
    /// # Errors
    /// Returns an error for invalid coordinates, a missing overlay, or failed persistence.
    pub fn duplicate_overlay(
        &mut self,
        id: OverlayId,
        x: f64,
        y: f64,
    ) -> Result<OverlayInstance, CoreError> {
        self.database.duplicate_overlay(id, x, y)
    }

    /// Removes a persisted overlay.
    ///
    /// # Errors
    /// Returns an error when the overlay is missing or cannot be deleted.
    pub fn remove_overlay(&mut self, id: OverlayId) -> Result<(), CoreError> {
        self.database.remove_overlay(id)
    }

    /// Replaces the entire persisted desktop layout atomically.
    ///
    /// # Errors
    /// Returns an error when any replacement is invalid, references missing media, or cannot be saved.
    pub fn replace_overlays(
        &mut self,
        replacements: &[OverlayReplacement],
    ) -> Result<Vec<OverlayInstance>, CoreError> {
        self.database.replace_overlays(replacements)
    }

    /// Restores exact persisted overlay rows after a failed native synchronization.
    ///
    /// # Errors
    /// Returns an error when the previous layout cannot be restored.
    pub fn restore_overlays(
        &mut self,
        overlays: &[OverlayInstance],
    ) -> Result<Vec<OverlayInstance>, CoreError> {
        self.database.restore_overlays(overlays)
    }

    /// Lists saved desktop scene summaries.
    ///
    /// # Errors
    /// Returns an error when scene metadata cannot be read.
    pub fn list_desktop_scenes(&self) -> Result<Vec<DesktopSceneSummary>, CoreError> {
        self.database.list_desktop_scenes()
    }

    /// Returns one saved desktop scene and all of its snapshot items.
    ///
    /// # Errors
    /// Returns an error when the scene is missing or invalid.
    pub fn desktop_scene(&self, id: DesktopSceneId) -> Result<DesktopScene, CoreError> {
        self.database.desktop_scene(id)
    }

    /// Creates a named scene from an explicit snapshot.
    ///
    /// # Errors
    /// Returns an error for an invalid or duplicate name, invalid items, or failed persistence.
    pub fn create_desktop_scene(
        &mut self,
        name: &str,
        items: &[DesktopSceneItem],
    ) -> Result<DesktopScene, CoreError> {
        self.database.create_desktop_scene(name, items)
    }

    /// Replaces a scene snapshot without changing its name.
    ///
    /// # Errors
    /// Returns an error for a missing scene, invalid items, or failed persistence.
    pub fn update_desktop_scene(
        &mut self,
        id: DesktopSceneId,
        items: &[DesktopSceneItem],
    ) -> Result<DesktopScene, CoreError> {
        self.database.update_desktop_scene(id, items)
    }

    /// Renames a scene while preserving its snapshot.
    ///
    /// # Errors
    /// Returns an error for a missing scene, invalid or duplicate name, or failed persistence.
    pub fn rename_desktop_scene(
        &mut self,
        id: DesktopSceneId,
        name: &str,
    ) -> Result<DesktopScene, CoreError> {
        self.database.rename_desktop_scene(id, name)
    }

    /// Duplicates a scene with a generated unique name.
    ///
    /// # Errors
    /// Returns an error when the source is missing or the copy cannot be saved.
    pub fn duplicate_desktop_scene(
        &mut self,
        id: DesktopSceneId,
    ) -> Result<DesktopScene, CoreError> {
        self.database.duplicate_desktop_scene(id)
    }

    /// Deletes one saved scene.
    ///
    /// # Errors
    /// Returns an error when the scene is missing or cannot be deleted.
    pub fn delete_desktop_scene(&mut self, id: DesktopSceneId) -> Result<(), CoreError> {
        self.database.delete_desktop_scene(id)
    }

    /// Reads typed application settings, falling back to explicit defaults.
    ///
    /// # Errors
    /// Returns an error when stored settings are malformed or unavailable.
    pub fn get_settings(&self) -> Result<AppSettings, CoreError> {
        self.database.get_settings()
    }

    /// Validates and persists all application settings.
    ///
    /// # Errors
    /// Returns an error for invalid values or a failed database write.
    pub fn update_settings(&mut self, settings: &AppSettings) -> Result<AppSettings, CoreError> {
        self.database.update_settings(settings)
    }

    /// Removes generated thumbnails and decoded frames without touching imported media.
    ///
    /// # Errors
    /// Returns an error when generated cache files cannot be removed or recreated.
    pub fn clear_generated_cache(&mut self) -> Result<(), CoreError> {
        let thumbnails = self.data_root.join("thumbnails");
        if thumbnails.exists() {
            fs::remove_dir_all(&thumbnails)?;
        }
        fs::create_dir_all(thumbnails)?;
        self.frame_cache.clear_all()?;
        let masks = self.data_root.join("masks");
        if masks.exists() {
            fs::remove_dir_all(&masks)?;
        }
        fs::create_dir_all(masks)?;
        self.database
            .remove_variants_by_prefix(VariantKind::SegmentationMask, "masks/")?;
        Ok(())
    }

    fn resolve_managed_path(&self, managed_path: &str) -> Result<PathBuf, CoreError> {
        let path = Path::new(managed_path);
        if path.components().count() != 1 || path.file_name().is_none() {
            return Err(CoreError::InvalidData(
                "The managed media location is invalid.".into(),
            ));
        }
        Ok(self.media_root.join(path))
    }

    fn remove_cache_for(&mut self, id: MediaId) -> Result<(), CoreError> {
        let thumbnail_path = self.data_root.join("thumbnails").join(id.to_string());
        if thumbnail_path.exists() {
            fs::remove_dir_all(thumbnail_path)?;
        }
        self.frame_cache.remove(id)?;
        Ok(())
    }

    fn remove_segmentation_variants(&mut self, id: MediaId) -> Result<(), CoreError> {
        self.database
            .remove_media_variants(id, VariantKind::SegmentationMask)?;
        self.remove_mask_directories(id)
    }

    fn remove_mask_directories(&self, id: MediaId) -> Result<(), CoreError> {
        for model in ["fast", "quality"] {
            let path = self
                .data_root
                .join("masks")
                .join(model)
                .join(id.to_string());
            if path.exists() {
                fs::remove_dir_all(path)?;
            }
        }
        Ok(())
    }

    fn remove_stale_imports(&self) -> Result<(), CoreError> {
        for entry in self.media_root.read_dir()? {
            let entry = entry?;
            let name = entry.file_name();
            let name = name.to_string_lossy();
            if name.starts_with(".import-") || name.starts_with(".delete-") {
                let path = entry.path();
                if path.is_file() {
                    fs::remove_file(path)?;
                }
            }
        }
        Ok(())
    }
}

fn copy_and_hash(source: &Path, destination: &Path) -> Result<String, CoreError> {
    let mut reader = BufReader::new(File::open(source)?);
    let output = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(destination)?;
    let mut writer = BufWriter::new(output);
    let mut hasher = Sha256::new();
    let mut buffer = vec![0_u8; 64 * 1_024].into_boxed_slice();
    loop {
        let read = reader.read(&mut buffer)?;
        if read == 0 {
            break;
        }
        hasher.update(&buffer[..read]);
        writer.write_all(&buffer[..read])?;
    }
    writer.flush()?;
    writer.get_ref().sync_all()?;
    Ok(format!("{:x}", hasher.finalize()))
}
