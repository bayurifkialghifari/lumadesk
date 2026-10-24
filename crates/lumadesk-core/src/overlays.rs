use chrono::{DateTime, Utc};
use rusqlite::{OptionalExtension as _, Row, params};
use uuid::Uuid;

use crate::{
    CoreError, MediaId, NewOverlay, OverlayId, OverlayInstance, OverlayPatch, database::Database,
};

#[derive(Debug)]
struct RawOverlay {
    id: String,
    media_id: String,
    monitor_id: Option<String>,
    x: f64,
    y: f64,
    width: f64,
    opacity: f64,
    speed: f64,
    flip_horizontal: bool,
    is_playing: bool,
    z_order: i32,
    is_active: bool,
    created_at: i64,
    updated_at: i64,
}

impl Database {
    pub(crate) fn list_overlays(&self) -> Result<Vec<OverlayInstance>, CoreError> {
        let mut statement = self.connection.prepare(
            "SELECT id, media_id, monitor_id, x, y, width, opacity, speed, flip_horizontal,
                    is_playing, z_order, is_active, created_at, updated_at
             FROM overlay_instances ORDER BY z_order ASC, created_at ASC",
        )?;
        let rows = statement.query_map([], read_raw_overlay)?;
        rows.map(|row| hydrate_overlay(row?)).collect()
    }

    pub(crate) fn create_overlay(
        &mut self,
        input: &NewOverlay,
    ) -> Result<OverlayInstance, CoreError> {
        self.media_by_id(input.media_id)?;
        validate_position(input.x, input.y)?;
        let settings = self.get_settings()?;
        let id = OverlayId::new();
        let now = Utc::now().timestamp_millis();
        self.connection.execute(
            "INSERT INTO overlay_instances
             (id, media_id, monitor_id, x, y, width, opacity, speed, flip_horizontal,
              is_playing, z_order, is_active, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, 0, 1, 0, 1, ?9, ?9)",
            params![
                id.to_string(),
                input.media_id.to_string(),
                input.monitor_id,
                input.x,
                input.y,
                settings.default_overlay_width,
                settings.default_opacity,
                settings.default_speed,
                now
            ],
        )?;
        self.overlay_by_id(id)
    }

    pub(crate) fn update_overlay(
        &mut self,
        id: OverlayId,
        patch: &OverlayPatch,
    ) -> Result<OverlayInstance, CoreError> {
        let current = self.overlay_by_id(id)?;
        let monitor_id = patch.monitor_id.clone().unwrap_or(current.monitor_id);
        let x = patch.x.unwrap_or(current.x);
        let y = patch.y.unwrap_or(current.y);
        let width = patch.width.unwrap_or(current.width);
        let opacity = patch.opacity.unwrap_or(current.opacity);
        let speed = patch.speed.unwrap_or(current.speed);
        validate_overlay_values(x, y, width, opacity, speed)?;
        self.connection.execute(
            "UPDATE overlay_instances SET monitor_id = ?1, x = ?2, y = ?3, width = ?4,
             opacity = ?5, speed = ?6, flip_horizontal = ?7, is_playing = ?8, z_order = ?9,
             is_active = ?10, updated_at = ?11 WHERE id = ?12",
            params![
                monitor_id,
                x,
                y,
                width,
                opacity,
                speed,
                patch.flip_horizontal.unwrap_or(current.flip_horizontal),
                patch.is_playing.unwrap_or(current.is_playing),
                patch.z_order.unwrap_or(current.z_order),
                patch.is_active.unwrap_or(current.is_active),
                Utc::now().timestamp_millis(),
                id.to_string()
            ],
        )?;
        self.overlay_by_id(id)
    }

    pub(crate) fn duplicate_overlay(
        &mut self,
        id: OverlayId,
        x: f64,
        y: f64,
    ) -> Result<OverlayInstance, CoreError> {
        validate_position(x, y)?;
        let source = self.overlay_by_id(id)?;
        let duplicate_id = OverlayId::new();
        let now = Utc::now().timestamp_millis();
        self.connection.execute(
            "INSERT INTO overlay_instances
             (id, media_id, monitor_id, x, y, width, opacity, speed, flip_horizontal,
              is_playing, z_order, is_active, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, 1, ?12, ?12)",
            params![
                duplicate_id.to_string(),
                source.media_id.to_string(),
                source.monitor_id,
                x,
                y,
                source.width,
                source.opacity,
                source.speed,
                source.flip_horizontal,
                source.is_playing,
                source.z_order.saturating_add(1),
                now
            ],
        )?;
        self.overlay_by_id(duplicate_id)
    }

    pub(crate) fn remove_overlay(&mut self, id: OverlayId) -> Result<(), CoreError> {
        let changed = self.connection.execute(
            "DELETE FROM overlay_instances WHERE id = ?1",
            [id.to_string()],
        )?;
        if changed == 0 {
            return Err(CoreError::NotFound {
                entity: "Overlay",
                id: id.to_string(),
            });
        }
        Ok(())
    }

    pub(crate) fn overlay_by_id(&self, id: OverlayId) -> Result<OverlayInstance, CoreError> {
        let raw = self
            .connection
            .query_row(
                "SELECT id, media_id, monitor_id, x, y, width, opacity, speed, flip_horizontal,
                    is_playing, z_order, is_active, created_at, updated_at
             FROM overlay_instances WHERE id = ?1",
                [id.to_string()],
                read_raw_overlay,
            )
            .optional()?
            .ok_or_else(|| CoreError::NotFound {
                entity: "Overlay",
                id: id.to_string(),
            })?;
        hydrate_overlay(raw)
    }
}

fn read_raw_overlay(row: &Row<'_>) -> rusqlite::Result<RawOverlay> {
    Ok(RawOverlay {
        id: row.get(0)?,
        media_id: row.get(1)?,
        monitor_id: row.get(2)?,
        x: row.get(3)?,
        y: row.get(4)?,
        width: row.get(5)?,
        opacity: row.get(6)?,
        speed: row.get(7)?,
        flip_horizontal: row.get(8)?,
        is_playing: row.get(9)?,
        z_order: row.get(10)?,
        is_active: row.get(11)?,
        created_at: row.get(12)?,
        updated_at: row.get(13)?,
    })
}

fn hydrate_overlay(raw: RawOverlay) -> Result<OverlayInstance, CoreError> {
    Ok(OverlayInstance {
        id: OverlayId(parse_uuid(&raw.id)?),
        media_id: MediaId(parse_uuid(&raw.media_id)?),
        monitor_id: raw.monitor_id,
        x: raw.x,
        y: raw.y,
        width: raw.width,
        opacity: raw.opacity,
        speed: raw.speed,
        flip_horizontal: raw.flip_horizontal,
        is_playing: raw.is_playing,
        z_order: raw.z_order,
        is_active: raw.is_active,
        created_at: timestamp(raw.created_at)?,
        updated_at: timestamp(raw.updated_at)?,
    })
}

fn validate_overlay_values(
    x: f64,
    y: f64,
    width: f64,
    opacity: f64,
    speed: f64,
) -> Result<(), CoreError> {
    validate_position(x, y)?;
    if !width.is_finite() || !(32.0..=4_096.0).contains(&width) {
        return Err(CoreError::InvalidInput(
            "Lebar overlay harus 32–4096 piksel.".into(),
        ));
    }
    if !opacity.is_finite() || !(0.1..=1.0).contains(&opacity) {
        return Err(CoreError::InvalidInput(
            "Opasitas overlay harus 0,1–1.".into(),
        ));
    }
    if !speed.is_finite() || !(0.25..=4.0).contains(&speed) {
        return Err(CoreError::InvalidInput(
            "Kecepatan overlay harus 0,25×–4×.".into(),
        ));
    }
    Ok(())
}

fn validate_position(x: f64, y: f64) -> Result<(), CoreError> {
    if !x.is_finite() || !y.is_finite() {
        return Err(CoreError::InvalidInput(
            "Posisi overlay tidak valid.".into(),
        ));
    }
    Ok(())
}

fn parse_uuid(value: &str) -> Result<Uuid, CoreError> {
    Uuid::parse_str(value).map_err(|_| CoreError::InvalidData("ID overlay tidak valid.".into()))
}

fn timestamp(value: i64) -> Result<DateTime<Utc>, CoreError> {
    DateTime::from_timestamp_millis(value)
        .ok_or_else(|| CoreError::InvalidData("Waktu overlay tidak valid.".into()))
}
