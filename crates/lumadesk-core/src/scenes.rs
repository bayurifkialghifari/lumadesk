use chrono::{DateTime, Utc};
use rusqlite::{OptionalExtension as _, Row, Transaction, params};
use uuid::Uuid;

use crate::{
    CoreError, DesktopScene, DesktopSceneId, DesktopSceneItem, DesktopSceneSummary,
    database::Database, overlays::validate_overlay_values,
};

impl Database {
    pub(crate) fn list_desktop_scenes(&self) -> Result<Vec<DesktopSceneSummary>, CoreError> {
        let mut statement = self.connection.prepare(
            "SELECT scenes.id, scenes.name, COUNT(items.id), scenes.created_at, scenes.updated_at
             FROM desktop_scenes scenes
             LEFT JOIN desktop_scene_items items ON items.scene_id = scenes.id
             GROUP BY scenes.id
             ORDER BY scenes.name COLLATE NOCASE ASC",
        )?;
        let rows = statement.query_map([], |row| {
            Ok(RawSceneSummary {
                id: row.get(0)?,
                name: row.get(1)?,
                item_count: row.get(2)?,
                created_at: row.get(3)?,
                updated_at: row.get(4)?,
            })
        })?;
        rows.map(|row| hydrate_summary(row?)).collect()
    }

    pub(crate) fn desktop_scene(&self, id: DesktopSceneId) -> Result<DesktopScene, CoreError> {
        let raw = self
            .connection
            .query_row(
                "SELECT id, name, created_at, updated_at FROM desktop_scenes WHERE id = ?1",
                [id.to_string()],
                |row| {
                    Ok(RawScene {
                        id: row.get(0)?,
                        name: row.get(1)?,
                        created_at: row.get(2)?,
                        updated_at: row.get(3)?,
                    })
                },
            )
            .optional()?
            .ok_or_else(|| scene_not_found(id))?;
        let items = self.scene_items(id)?;
        hydrate_scene(raw, items)
    }

    pub(crate) fn create_desktop_scene(
        &mut self,
        name: &str,
        items: &[DesktopSceneItem],
    ) -> Result<DesktopScene, CoreError> {
        let name = validate_name(name)?;
        validate_items(items)?;
        ensure_unique_name(&self.connection, &name, None)?;
        let id = DesktopSceneId::new();
        let now = Utc::now().timestamp_millis();
        let transaction = self.connection.transaction()?;
        transaction.execute(
            "INSERT INTO desktop_scenes (id, name, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?3)",
            params![id.to_string(), name, now],
        )?;
        insert_items(&transaction, id, items)?;
        transaction.commit()?;
        self.desktop_scene(id)
    }

    pub(crate) fn update_desktop_scene(
        &mut self,
        id: DesktopSceneId,
        items: &[DesktopSceneItem],
    ) -> Result<DesktopScene, CoreError> {
        validate_items(items)?;
        self.desktop_scene(id)?;
        let transaction = self.connection.transaction()?;
        transaction.execute(
            "DELETE FROM desktop_scene_items WHERE scene_id = ?1",
            [id.to_string()],
        )?;
        insert_items(&transaction, id, items)?;
        transaction.execute(
            "UPDATE desktop_scenes SET updated_at = ?1 WHERE id = ?2",
            params![Utc::now().timestamp_millis(), id.to_string()],
        )?;
        transaction.commit()?;
        self.desktop_scene(id)
    }

    pub(crate) fn rename_desktop_scene(
        &mut self,
        id: DesktopSceneId,
        name: &str,
    ) -> Result<DesktopScene, CoreError> {
        self.desktop_scene(id)?;
        let name = validate_name(name)?;
        ensure_unique_name(&self.connection, &name, Some(id))?;
        self.connection.execute(
            "UPDATE desktop_scenes SET name = ?1, updated_at = ?2 WHERE id = ?3",
            params![name, Utc::now().timestamp_millis(), id.to_string()],
        )?;
        self.desktop_scene(id)
    }

    pub(crate) fn duplicate_desktop_scene(
        &mut self,
        id: DesktopSceneId,
    ) -> Result<DesktopScene, CoreError> {
        let source = self.desktop_scene(id)?;
        let name = self.next_duplicate_name(&source.name)?;
        self.create_desktop_scene(&name, &source.items)
    }

    pub(crate) fn delete_desktop_scene(&mut self, id: DesktopSceneId) -> Result<(), CoreError> {
        let changed = self
            .connection
            .execute("DELETE FROM desktop_scenes WHERE id = ?1", [id.to_string()])?;
        if changed == 0 {
            return Err(scene_not_found(id));
        }
        Ok(())
    }

    fn scene_items(&self, id: DesktopSceneId) -> Result<Vec<DesktopSceneItem>, CoreError> {
        let mut statement = self.connection.prepare(
            "SELECT media_id, monitor_name, monitor_width, monitor_height, normalized_x,
                    normalized_y, width, opacity, speed, flip_horizontal, remove_background,
                    is_playing, click_through, z_order, is_active
             FROM desktop_scene_items WHERE scene_id = ?1 ORDER BY z_order ASC, id ASC",
        )?;
        let rows = statement.query_map([id.to_string()], read_item)?;
        rows.map(|row| hydrate_item(row?)).collect()
    }

    fn next_duplicate_name(&self, source: &str) -> Result<String, CoreError> {
        for suffix in 1..=10_000_u32 {
            let ending = if suffix == 1 {
                " copy".to_owned()
            } else {
                format!(" copy {suffix}")
            };
            let prefix_limit = 80_usize.saturating_sub(ending.chars().count());
            let prefix: String = source.chars().take(prefix_limit).collect();
            let candidate = format!("{prefix}{ending}");
            if !scene_name_exists(&self.connection, &candidate, None)? {
                return Ok(candidate);
            }
        }
        Err(CoreError::InvalidInput(
            "A unique scene name could not be created.".into(),
        ))
    }
}

#[derive(Debug)]
struct RawScene {
    id: String,
    name: String,
    created_at: i64,
    updated_at: i64,
}

#[derive(Debug)]
struct RawSceneSummary {
    id: String,
    name: String,
    item_count: u32,
    created_at: i64,
    updated_at: i64,
}

#[derive(Debug)]
#[expect(
    clippy::struct_excessive_bools,
    reason = "the database row mirrors independent scene controls"
)]
struct RawSceneItem {
    media_id: String,
    monitor_name: Option<String>,
    monitor_width: f64,
    monitor_height: f64,
    normalized_x: f64,
    normalized_y: f64,
    width: f64,
    opacity: f64,
    speed: f64,
    flip_horizontal: bool,
    remove_background: bool,
    is_playing: bool,
    click_through: bool,
    z_order: i32,
    is_active: bool,
}

fn insert_items(
    transaction: &Transaction<'_>,
    scene_id: DesktopSceneId,
    items: &[DesktopSceneItem],
) -> Result<(), rusqlite::Error> {
    for item in items {
        transaction.execute(
            "INSERT INTO desktop_scene_items
             (scene_id, media_id, monitor_name, monitor_width, monitor_height, normalized_x,
              normalized_y, width, opacity, speed, flip_horizontal, remove_background,
              is_playing, click_through, z_order, is_active)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16)",
            params![
                scene_id.to_string(),
                item.media_id.to_string(),
                item.monitor_name,
                item.monitor_width,
                item.monitor_height,
                item.normalized_x,
                item.normalized_y,
                item.width,
                item.opacity,
                item.speed,
                item.flip_horizontal,
                item.remove_background,
                item.is_playing,
                item.click_through,
                item.z_order,
                item.is_active
            ],
        )?;
    }
    Ok(())
}

fn validate_name(name: &str) -> Result<String, CoreError> {
    let name = name.trim();
    let length = name.chars().count();
    if !(1..=80).contains(&length) {
        return Err(CoreError::InvalidInput(
            "Scene names must contain 1–80 characters.".into(),
        ));
    }
    Ok(name.to_owned())
}

fn validate_items(items: &[DesktopSceneItem]) -> Result<(), CoreError> {
    for item in items {
        if !item.monitor_width.is_finite()
            || item.monitor_width <= 0.0
            || !item.monitor_height.is_finite()
            || item.monitor_height <= 0.0
            || !item.normalized_x.is_finite()
            || !item.normalized_y.is_finite()
        {
            return Err(CoreError::InvalidInput(
                "Scene monitor placement is invalid.".into(),
            ));
        }
        validate_overlay_values(0.0, 0.0, item.width, item.opacity, item.speed)?;
    }
    Ok(())
}

fn ensure_unique_name(
    connection: &rusqlite::Connection,
    name: &str,
    excluded_id: Option<DesktopSceneId>,
) -> Result<(), CoreError> {
    if scene_name_exists(connection, name, excluded_id)? {
        return Err(CoreError::InvalidInput(
            "A scene with that name already exists.".into(),
        ));
    }
    Ok(())
}

fn scene_name_exists(
    connection: &rusqlite::Connection,
    name: &str,
    excluded_id: Option<DesktopSceneId>,
) -> Result<bool, rusqlite::Error> {
    connection.query_row(
        "SELECT EXISTS(
             SELECT 1 FROM desktop_scenes
             WHERE name = ?1 COLLATE NOCASE AND (?2 IS NULL OR id != ?2)
         )",
        params![name, excluded_id.map(|id| id.to_string())],
        |row| row.get(0),
    )
}

fn read_item(row: &Row<'_>) -> rusqlite::Result<RawSceneItem> {
    Ok(RawSceneItem {
        media_id: row.get(0)?,
        monitor_name: row.get(1)?,
        monitor_width: row.get(2)?,
        monitor_height: row.get(3)?,
        normalized_x: row.get(4)?,
        normalized_y: row.get(5)?,
        width: row.get(6)?,
        opacity: row.get(7)?,
        speed: row.get(8)?,
        flip_horizontal: row.get(9)?,
        remove_background: row.get(10)?,
        is_playing: row.get(11)?,
        click_through: row.get(12)?,
        z_order: row.get(13)?,
        is_active: row.get(14)?,
    })
}

fn hydrate_scene(raw: RawScene, items: Vec<DesktopSceneItem>) -> Result<DesktopScene, CoreError> {
    Ok(DesktopScene {
        id: DesktopSceneId(parse_uuid(&raw.id)?),
        name: raw.name,
        items,
        created_at: timestamp(raw.created_at)?,
        updated_at: timestamp(raw.updated_at)?,
    })
}

fn hydrate_summary(raw: RawSceneSummary) -> Result<DesktopSceneSummary, CoreError> {
    Ok(DesktopSceneSummary {
        id: DesktopSceneId(parse_uuid(&raw.id)?),
        name: raw.name,
        item_count: raw.item_count,
        created_at: timestamp(raw.created_at)?,
        updated_at: timestamp(raw.updated_at)?,
    })
}

fn hydrate_item(raw: RawSceneItem) -> Result<DesktopSceneItem, CoreError> {
    Ok(DesktopSceneItem {
        media_id: crate::MediaId(parse_uuid(&raw.media_id)?),
        monitor_name: raw.monitor_name,
        monitor_width: raw.monitor_width,
        monitor_height: raw.monitor_height,
        normalized_x: raw.normalized_x,
        normalized_y: raw.normalized_y,
        width: raw.width,
        opacity: raw.opacity,
        speed: raw.speed,
        flip_horizontal: raw.flip_horizontal,
        remove_background: raw.remove_background,
        is_playing: raw.is_playing,
        click_through: raw.click_through,
        z_order: raw.z_order,
        is_active: raw.is_active,
    })
}

fn scene_not_found(id: DesktopSceneId) -> CoreError {
    CoreError::NotFound {
        entity: "Desktop scene",
        id: id.to_string(),
    }
}

fn parse_uuid(value: &str) -> Result<Uuid, CoreError> {
    Uuid::parse_str(value).map_err(|_| CoreError::InvalidData("The scene ID is invalid.".into()))
}

fn timestamp(value: i64) -> Result<DateTime<Utc>, CoreError> {
    DateTime::from_timestamp_millis(value)
        .ok_or_else(|| CoreError::InvalidData("The scene timestamp is invalid.".into()))
}
