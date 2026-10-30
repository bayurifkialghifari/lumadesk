use chrono::{DateTime, Utc};
use rusqlite::{OptionalExtension, params};

use crate::{
    AssetKind, BackgroundRemovalMode, CoreError, CropRect, EditRecipe, EditRecipePatch, MediaId,
    MediaVariant, SpritePlaybackMode, SpriteSequence, SpriteSequenceInput, VariantId, VariantKind,
    database::Database,
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
        let crop = patch.crop.unwrap_or(current.crop);
        let rotation_degrees = patch.rotation_degrees.unwrap_or(current.rotation_degrees);
        let flip_horizontal = patch.flip_horizontal.unwrap_or(current.flip_horizontal);
        let background_removal = patch
            .background_removal
            .unwrap_or(current.background_removal);
        let brightness = patch.brightness.unwrap_or(current.brightness);
        let contrast = patch.contrast.unwrap_or(current.contrast);
        let saturation = patch.saturation.unwrap_or(current.saturation);
        validate_recipe(crop, rotation_degrees, brightness, contrast, saturation)?;
        let (crop_x, crop_y, crop_width, crop_height) =
            crop.map_or((None, None, None, None), |value| {
                (
                    Some(value.x),
                    Some(value.y),
                    Some(value.width),
                    Some(value.height),
                )
            });
        self.connection.execute(
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
                rotation_degrees,
                flip_horizontal,
                background_removal.as_str(),
                brightness,
                contrast,
                saturation,
                Utc::now().timestamp_millis(),
                media_id.to_string(),
            ],
        )?;
        self.edit_recipe(media_id)
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
        let duration_ms = u64::from(sequence.frame_count)
            .checked_mul(u64::from(sequence.frame_duration_ms))
            .ok_or_else(|| CoreError::InvalidInput("The sprite duration is too long.".into()))?;
        let duration_ms = i64::try_from(duration_ms)
            .map_err(|_| CoreError::InvalidInput("The sprite duration is too long.".into()))?;
        let now = Utc::now().timestamp_millis();
        let transaction = self.connection.transaction()?;
        transaction.execute(
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
                now,
            ],
        )?;
        transaction.execute(
            "UPDATE media_items SET asset_kind = 'sprite_sheet', frame_count = ?1,
             duration_ms = ?2, updated_at = ?3 WHERE id = ?4",
            params![
                sequence.frame_count,
                duration_ms,
                now,
                sequence.media_id.to_string()
            ],
        )?;
        transaction.commit()?;
        self.sprite_sequence(sequence.media_id)?
            .ok_or_else(|| CoreError::InvalidData("The sprite sequence was not saved.".into()))
    }

    pub(crate) fn clear_sprite_sequence(&mut self, media_id: MediaId) -> Result<(), CoreError> {
        self.media_by_id(media_id)?;
        let transaction = self.connection.transaction()?;
        transaction.execute(
            "DELETE FROM sprite_sequences WHERE media_id = ?1",
            [media_id.to_string()],
        )?;
        transaction.execute(
            "UPDATE media_items SET asset_kind = 'still', frame_count = 1, duration_ms = 0,
             updated_at = ?1 WHERE id = ?2",
            params![Utc::now().timestamp_millis(), media_id.to_string()],
        )?;
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
