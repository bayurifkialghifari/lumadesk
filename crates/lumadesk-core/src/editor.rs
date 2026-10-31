use std::path::{Component, Path};

use chrono::{DateTime, Utc};
use rusqlite::{Connection, OptionalExtension, params};

use crate::{
    AssetKind, BackgroundRemovalMode, CoreError, CreatorChanges, CreatorChangesInput, CropRect,
    EditRecipe, EditRecipePatch, MediaId, MediaItem, MediaVariant, SpritePlaybackMode,
    SpriteSequence, SpriteSequenceInput, VariantId, VariantKind, database::Database,
};

impl Database {
    pub(crate) fn edit_recipe(&self, media_id: MediaId) -> Result<EditRecipe, CoreError> {
        self.media_by_id(media_id)?;
        let stored = self
            .connection
            .query_row(
                "SELECT crop_x, crop_y, crop_width, crop_height, rotation_degrees,
                        flip_horizontal, background_removal, brightness, contrast, saturation,
                        updated_at
                 FROM edit_recipes WHERE media_id = ?1",
                [media_id.to_string()],
                |row| {
                    Ok((
                        row.get::<_, Option<f64>>(0)?,
                        row.get::<_, Option<f64>>(1)?,
                        row.get::<_, Option<f64>>(2)?,
                        row.get::<_, Option<f64>>(3)?,
                        row.get::<_, i16>(4)?,
                        row.get::<_, bool>(5)?,
                        row.get::<_, String>(6)?,
                        row.get::<_, f64>(7)?,
                        row.get::<_, f64>(8)?,
                        row.get::<_, f64>(9)?,
                        row.get::<_, i64>(10)?,
                    ))
                },
            )
            .optional()?
            .ok_or_else(|| CoreError::InvalidData("The edit recipe is missing.".into()))?;
        let (
            crop_x,
            crop_y,
            crop_width,
            crop_height,
            rotation_degrees,
            flip_horizontal,
            background_removal,
            brightness,
            contrast,
            saturation,
            updated_at,
        ) = stored;
        Ok(EditRecipe {
            media_id,
            crop: crop_from_columns(crop_x, crop_y, crop_width, crop_height)?,
            rotation_degrees,
            flip_horizontal,
            background_removal: BackgroundRemovalMode::parse(&background_removal)
                .ok_or_else(|| invalid_data("background-removal mode"))?,
            brightness,
            contrast,
            saturation,
            updated_at: timestamp(updated_at)?,
        })
    }

    pub(crate) fn update_edit_recipe(
        &mut self,
        media_id: MediaId,
        patch: &EditRecipePatch,
    ) -> Result<EditRecipe, CoreError> {
        let current = self.edit_recipe(media_id)?;
        let values = RecipeValues::merge(&current, patch)?;
        let item = self.media_by_id(media_id)?;
        let sequence = self.sprite_sequence(media_id)?;
        let (width, height) = effective_dimensions(&item, sequence.as_ref());
        validate_crop_bounds(values.crop, width, height)?;
        write_edit_recipe(
            &self.connection,
            media_id,
            values,
            Utc::now().timestamp_millis(),
        )?;
        self.edit_recipe(media_id)
    }

    pub(crate) fn save_creator_changes(
        &mut self,
        input: &CreatorChangesInput,
    ) -> Result<CreatorChanges, CoreError> {
        if input.sprite_sequence.is_some() && input.clear_sprite_sequence {
            return Err(CoreError::InvalidInput(
                "Choose either sprite playback or source playback.".into(),
            ));
        }
        let current_recipe = self.edit_recipe(input.media_id)?;
        let recipe = RecipeValues::merge(&current_recipe, &input.recipe)?;
        let item = self.media_by_id(input.media_id)?;
        let current_sequence = self.sprite_sequence(input.media_id)?;
        let next_sequence = if let Some(sequence) = input.sprite_sequence.as_ref() {
            if sequence.media_id != input.media_id {
                return Err(CoreError::InvalidInput(
                    "The sprite sequence belongs to another media item.".into(),
                ));
            }
            validate_sprite_sequence(&item, sequence)?;
            Some(sequence.clone())
        } else if input.clear_sprite_sequence {
            None
        } else {
            current_sequence
                .as_ref()
                .map(|sequence| SpriteSequenceInput {
                    media_id: sequence.media_id,
                    columns: sequence.columns,
                    rows: sequence.rows,
                    frame_count: sequence.frame_count,
                    frame_duration_ms: sequence.frame_duration_ms,
                    playback_mode: sequence.playback_mode,
                })
        };
        let (width, height) = effective_input_dimensions(&item, next_sequence.as_ref());
        validate_crop_bounds(recipe.crop, width, height)?;

        let now = Utc::now().timestamp_millis();
        let transaction = self.connection.transaction()?;
        write_edit_recipe(&transaction, input.media_id, recipe, now)?;
        if let Some(sequence) = next_sequence.as_ref() {
            let duration_ms = validate_sprite_sequence(&item, sequence)?;
            write_sprite_sequence(&transaction, sequence, duration_ms, now)?;
        } else if input.clear_sprite_sequence && current_sequence.is_some() {
            clear_sprite_sequence(&transaction, input.media_id, now)?;
        }
        transaction.commit()?;

        Ok(CreatorChanges {
            recipe: self.edit_recipe(input.media_id)?,
            sprite_sequence: self.sprite_sequence(input.media_id)?,
        })
    }

    pub(crate) fn sprite_sequence(
        &self,
        media_id: MediaId,
    ) -> Result<Option<SpriteSequence>, CoreError> {
        self.media_by_id(media_id)?;
        let stored = self
            .connection
            .query_row(
                "SELECT columns, rows, frame_count, frame_duration_ms, playback_mode, updated_at
                 FROM sprite_sequences WHERE media_id = ?1",
                [media_id.to_string()],
                |row| {
                    Ok((
                        row.get::<_, i64>(0)?,
                        row.get::<_, i64>(1)?,
                        row.get::<_, i64>(2)?,
                        row.get::<_, i64>(3)?,
                        row.get::<_, String>(4)?,
                        row.get::<_, i64>(5)?,
                    ))
                },
            )
            .optional()?;
        stored
            .map(
                |(columns, rows, frame_count, frame_duration_ms, playback_mode, updated_at)| {
                    Ok(SpriteSequence {
                        media_id,
                        columns: to_u32(columns, "sprite columns")?,
                        rows: to_u32(rows, "sprite rows")?,
                        frame_count: to_u32(frame_count, "sprite frame count")?,
                        frame_duration_ms: to_u32(frame_duration_ms, "sprite frame duration")?,
                        playback_mode: SpritePlaybackMode::parse(&playback_mode)
                            .ok_or_else(|| invalid_data("sprite playback mode"))?,
                        updated_at: timestamp(updated_at)?,
                    })
                },
            )
            .transpose()
    }

    pub(crate) fn set_sprite_sequence(
        &mut self,
        sequence: &SpriteSequenceInput,
    ) -> Result<SpriteSequence, CoreError> {
        let item = self.media_by_id(sequence.media_id)?;
        let duration_ms = validate_sprite_sequence(&item, sequence)?;
        let recipe = self.edit_recipe(sequence.media_id)?;
        let (width, height) = effective_input_dimensions(&item, Some(sequence));
        validate_crop_bounds(recipe.crop, width, height)?;
        let now = Utc::now().timestamp_millis();
        let transaction = self.connection.transaction()?;
        write_sprite_sequence(&transaction, sequence, duration_ms, now)?;
        transaction.commit()?;
        self.sprite_sequence(sequence.media_id)?
            .ok_or_else(|| CoreError::InvalidData("The sprite sequence was not saved.".into()))
    }

    pub(crate) fn clear_sprite_sequence(&mut self, media_id: MediaId) -> Result<(), CoreError> {
        self.media_by_id(media_id)?;
        let transaction = self.connection.transaction()?;
        clear_sprite_sequence(&transaction, media_id, Utc::now().timestamp_millis())?;
        transaction.commit()?;
        Ok(())
    }

    pub(crate) fn media_variants(&self, media_id: MediaId) -> Result<Vec<MediaVariant>, CoreError> {
        self.media_by_id(media_id)?;
        let mut statement = self.connection.prepare(
            "SELECT id, variant_kind, relative_path, width, height, created_at
             FROM media_variants WHERE media_id = ?1 ORDER BY created_at DESC",
        )?;
        let rows = statement.query_map([media_id.to_string()], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, String>(2)?,
                row.get::<_, i64>(3)?,
                row.get::<_, i64>(4)?,
                row.get::<_, i64>(5)?,
            ))
        })?;
        rows.map(|row| {
            let (id, kind, relative_path, width, height, created_at) = row?;
            Ok(MediaVariant {
                id: VariantId(uuid::Uuid::parse_str(&id).map_err(|_| invalid_data("variant ID"))?),
                media_id,
                kind: VariantKind::parse(&kind).ok_or_else(|| invalid_data("variant kind"))?,
                relative_path,
                width: to_u32(width, "variant width")?,
                height: to_u32(height, "variant height")?,
                created_at: timestamp(created_at)?,
            })
        })
        .collect()
    }

    pub(crate) fn upsert_media_variant(
        &mut self,
        media_id: MediaId,
        kind: VariantKind,
        relative_path: &str,
        width: u32,
        height: u32,
    ) -> Result<MediaVariant, CoreError> {
        self.media_by_id(media_id)?;
        validate_variant_path(relative_path)?;
        if width == 0 || height == 0 {
            return Err(CoreError::InvalidInput(
                "Generated media dimensions must be positive.".into(),
            ));
        }

        let existing_id = self
            .connection
            .query_row(
                "SELECT id FROM media_variants WHERE relative_path = ?1",
                [relative_path],
                |row| row.get::<_, String>(0),
            )
            .optional()?;
        let id = match existing_id {
            Some(value) => {
                VariantId(uuid::Uuid::parse_str(&value).map_err(|_| invalid_data("variant ID"))?)
            }
            None => VariantId::new(),
        };
        let created_at = Utc::now();
        self.connection.execute(
            "INSERT INTO media_variants
             (id, media_id, variant_kind, relative_path, width, height, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)
             ON CONFLICT(relative_path) DO UPDATE SET
               media_id = excluded.media_id,
               variant_kind = excluded.variant_kind,
               width = excluded.width,
               height = excluded.height,
               created_at = excluded.created_at",
            params![
                id.to_string(),
                media_id.to_string(),
                kind.as_str(),
                relative_path,
                i64::from(width),
                i64::from(height),
                created_at.timestamp_millis(),
            ],
        )?;
        Ok(MediaVariant {
            id,
            media_id,
            kind,
            relative_path: relative_path.into(),
            width,
            height,
            created_at,
        })
    }

    pub(crate) fn remove_media_variants(
        &mut self,
        media_id: MediaId,
        kind: VariantKind,
    ) -> Result<Vec<String>, CoreError> {
        self.media_by_id(media_id)?;
        let paths = self.variant_paths(
            "SELECT relative_path FROM media_variants
             WHERE media_id = ?1 AND variant_kind = ?2",
            params![media_id.to_string(), kind.as_str()],
        )?;
        self.connection.execute(
            "DELETE FROM media_variants WHERE media_id = ?1 AND variant_kind = ?2",
            params![media_id.to_string(), kind.as_str()],
        )?;
        Ok(paths)
    }

    pub(crate) fn remove_variants_by_prefix(
        &mut self,
        kind: VariantKind,
        relative_prefix: &str,
    ) -> Result<Vec<String>, CoreError> {
        validate_variant_path(relative_prefix.trim_end_matches('/'))?;
        let paths = self.variant_paths(
            "SELECT relative_path FROM media_variants
             WHERE variant_kind = ?1 AND substr(relative_path, 1, length(?2)) = ?2",
            params![kind.as_str(), relative_prefix],
        )?;
        self.connection.execute(
            "DELETE FROM media_variants
             WHERE variant_kind = ?1 AND substr(relative_path, 1, length(?2)) = ?2",
            params![kind.as_str(), relative_prefix],
        )?;
        Ok(paths)
    }

    fn variant_paths<P>(&self, sql: &str, parameters: P) -> Result<Vec<String>, CoreError>
    where
        P: rusqlite::Params,
    {
        let mut statement = self.connection.prepare(sql)?;
        statement
            .query_map(parameters, |row| row.get::<_, String>(0))?
            .collect::<Result<Vec<_>, _>>()
            .map_err(CoreError::from)
    }
}

#[derive(Clone, Copy, Debug)]
struct RecipeValues {
    crop: Option<CropRect>,
    rotation_degrees: i16,
    flip_horizontal: bool,
    background_removal: BackgroundRemovalMode,
    brightness: f64,
    contrast: f64,
    saturation: f64,
}

impl RecipeValues {
    fn merge(current: &EditRecipe, patch: &EditRecipePatch) -> Result<Self, CoreError> {
        let values = Self {
            crop: patch.crop.unwrap_or(current.crop),
            rotation_degrees: patch.rotation_degrees.unwrap_or(current.rotation_degrees),
            flip_horizontal: patch.flip_horizontal.unwrap_or(current.flip_horizontal),
            background_removal: patch
                .background_removal
                .unwrap_or(current.background_removal),
            brightness: patch.brightness.unwrap_or(current.brightness),
            contrast: patch.contrast.unwrap_or(current.contrast),
            saturation: patch.saturation.unwrap_or(current.saturation),
        };
        validate_recipe(
            values.crop,
            values.rotation_degrees,
            values.brightness,
            values.contrast,
            values.saturation,
        )?;
        Ok(values)
    }
}

fn write_edit_recipe(
    connection: &Connection,
    media_id: MediaId,
    values: RecipeValues,
    updated_at: i64,
) -> Result<(), CoreError> {
    let (crop_x, crop_y, crop_width, crop_height) =
        values.crop.map_or((None, None, None, None), |value| {
            (
                Some(value.x),
                Some(value.y),
                Some(value.width),
                Some(value.height),
            )
        });
    connection.execute(
        "UPDATE edit_recipes
         SET crop_x = ?1, crop_y = ?2, crop_width = ?3, crop_height = ?4,
             rotation_degrees = ?5, flip_horizontal = ?6, background_removal = ?7,
             brightness = ?8, contrast = ?9, saturation = ?10, updated_at = ?11
         WHERE media_id = ?12",
        params![
            crop_x,
            crop_y,
            crop_width,
            crop_height,
            values.rotation_degrees,
            values.flip_horizontal,
            values.background_removal.as_str(),
            values.brightness,
            values.contrast,
            values.saturation,
            updated_at,
            media_id.to_string(),
        ],
    )?;
    Ok(())
}

fn validate_sprite_sequence(
    item: &MediaItem,
    sequence: &SpriteSequenceInput,
) -> Result<i64, CoreError> {
    if !matches!(item.asset_kind, AssetKind::Still | AssetKind::SpriteSheet) {
        return Err(CoreError::InvalidInput(
            "Only still images can be configured as sprite sheets.".into(),
        ));
    }
    let capacity = sequence
        .columns
        .checked_mul(sequence.rows)
        .ok_or_else(|| CoreError::InvalidInput("The sprite grid is too large.".into()))?;
    if sequence.columns == 0
        || sequence.rows == 0
        || sequence.frame_count == 0
        || sequence.frame_count > capacity
        || sequence.frame_duration_ms == 0
    {
        return Err(CoreError::InvalidInput(
            "Sprite frames must fit inside a non-empty grid.".into(),
        ));
    }
    if sequence.columns > item.width || sequence.rows > item.height {
        return Err(CoreError::InvalidInput(
            "The sprite grid cannot exceed the source dimensions.".into(),
        ));
    }
    let duration_ms = u64::from(sequence.frame_count)
        .checked_mul(u64::from(sequence.frame_duration_ms))
        .ok_or_else(|| CoreError::InvalidInput("The sprite duration is too long.".into()))?;
    i64::try_from(duration_ms)
        .map_err(|_| CoreError::InvalidInput("The sprite duration is too long.".into()))
}

fn write_sprite_sequence(
    connection: &Connection,
    sequence: &SpriteSequenceInput,
    duration_ms: i64,
    updated_at: i64,
) -> Result<(), CoreError> {
    connection.execute(
        "INSERT INTO sprite_sequences
         (media_id, columns, rows, frame_count, frame_duration_ms, playback_mode, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)
         ON CONFLICT(media_id) DO UPDATE SET
           columns = excluded.columns, rows = excluded.rows, frame_count = excluded.frame_count,
           frame_duration_ms = excluded.frame_duration_ms,
           playback_mode = excluded.playback_mode, updated_at = excluded.updated_at",
        params![
            sequence.media_id.to_string(),
            sequence.columns,
            sequence.rows,
            sequence.frame_count,
            sequence.frame_duration_ms,
            sequence.playback_mode.as_str(),
            updated_at,
        ],
    )?;
    connection.execute(
        "UPDATE media_items SET asset_kind = 'sprite_sheet', frame_count = ?1,
         duration_ms = ?2, updated_at = ?3 WHERE id = ?4",
        params![
            sequence.frame_count,
            duration_ms,
            updated_at,
            sequence.media_id.to_string()
        ],
    )?;
    Ok(())
}

fn clear_sprite_sequence(
    connection: &Connection,
    media_id: MediaId,
    updated_at: i64,
) -> Result<(), CoreError> {
    connection.execute(
        "DELETE FROM sprite_sequences WHERE media_id = ?1",
        [media_id.to_string()],
    )?;
    connection.execute(
        "UPDATE media_items SET asset_kind = 'still', frame_count = 1, duration_ms = 0,
         updated_at = ?1 WHERE id = ?2",
        params![updated_at, media_id.to_string()],
    )?;
    Ok(())
}

fn effective_dimensions(item: &MediaItem, sequence: Option<&SpriteSequence>) -> (u32, u32) {
    sequence.map_or((item.width, item.height), |value| {
        (item.width / value.columns, item.height / value.rows)
    })
}

fn effective_input_dimensions(
    item: &MediaItem,
    sequence: Option<&SpriteSequenceInput>,
) -> (u32, u32) {
    sequence.map_or((item.width, item.height), |value| {
        (item.width / value.columns, item.height / value.rows)
    })
}

fn validate_crop_bounds(
    crop: Option<CropRect>,
    frame_width: u32,
    frame_height: u32,
) -> Result<(), CoreError> {
    if let Some(value) = crop
        && (value.x + value.width > f64::from(frame_width)
            || value.y + value.height > f64::from(frame_height))
    {
        return Err(CoreError::InvalidInput(
            "The crop rectangle must fit inside one animation frame.".into(),
        ));
    }
    Ok(())
}

fn validate_variant_path(value: &str) -> Result<(), CoreError> {
    let path = Path::new(value);
    if value.is_empty()
        || path.is_absolute()
        || path
            .components()
            .any(|component| !matches!(component, Component::Normal(_)))
    {
        return Err(CoreError::InvalidInput(
            "Generated media must use a safe relative path.".into(),
        ));
    }
    Ok(())
}

fn validate_recipe(
    crop: Option<CropRect>,
    rotation_degrees: i16,
    brightness: f64,
    contrast: f64,
    saturation: f64,
) -> Result<(), CoreError> {
    if !matches!(rotation_degrees, 0 | 90 | 180 | 270) {
        return Err(CoreError::InvalidInput(
            "Rotation must be 0, 90, 180, or 270 degrees.".into(),
        ));
    }
    if let Some(value) = crop
        && (!value.x.is_finite()
            || !value.y.is_finite()
            || !value.width.is_finite()
            || !value.height.is_finite()
            || value.x < 0.0
            || value.y < 0.0
            || value.width <= 0.0
            || value.height <= 0.0)
    {
        return Err(CoreError::InvalidInput(
            "The crop rectangle must use finite positive dimensions.".into(),
        ));
    }
    if [brightness, contrast, saturation]
        .into_iter()
        .any(|value| !value.is_finite() || !(0.0..=2.0).contains(&value))
    {
        return Err(CoreError::InvalidInput(
            "Color controls must be between 0 and 2.".into(),
        ));
    }
    Ok(())
}

fn crop_from_columns(
    x: Option<f64>,
    y: Option<f64>,
    width: Option<f64>,
    height: Option<f64>,
) -> Result<Option<CropRect>, CoreError> {
    match (x, y, width, height) {
        (None, None, None, None) => Ok(None),
        (Some(x), Some(y), Some(width), Some(height)) => Ok(Some(CropRect {
            x,
            y,
            width,
            height,
        })),
        _ => Err(invalid_data("crop rectangle")),
    }
}

fn timestamp(value: i64) -> Result<DateTime<Utc>, CoreError> {
    DateTime::from_timestamp_millis(value).ok_or_else(|| invalid_data("timestamp"))
}

fn to_u32(value: i64, field: &str) -> Result<u32, CoreError> {
    u32::try_from(value).map_err(|_| invalid_data(field))
}

fn invalid_data(field: &str) -> CoreError {
    CoreError::InvalidData(format!("The {field} value is invalid."))
}
