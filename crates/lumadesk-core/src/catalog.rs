use chrono::{DateTime, Utc};
use rusqlite::{OptionalExtension, Row, params};
use uuid::Uuid;

use crate::{
    AssetKind, CoreError, DeleteReport, LibraryQuery, MediaFormat, MediaId, MediaItem, MediaPage,
    database::Database,
    domain::NewMediaItem,
    validation::{validate_name, validate_tags},
};

#[derive(Debug)]
struct RawMediaItem {
    id: String,
    content_hash: String,
    display_name: String,
    managed_path: String,
    asset_kind: String,
    media_format: String,
    width: i64,
    height: i64,
    frame_count: i64,
    duration_ms: i64,
    is_favorite: bool,
    created_at: i64,
    updated_at: i64,
}

impl Database {
    pub(crate) fn list_media(&self, query: &LibraryQuery) -> Result<MediaPage, CoreError> {
        let search = normalized_optional(query.search.as_deref());
        let tag = normalized_optional(query.tag.as_deref());
        let asset_kind = query.asset_kind.map(AssetKind::as_str);
        let media_format = query.media_format.map(MediaFormat::as_str);
        let favorite = i64::from(query.favorite_only);
        let limit = i64::from(query.limit.unwrap_or(100).clamp(1, 500));
        let offset = i64::from(query.offset);
        let where_clause = "(?1 IS NULL OR instr(lower(m.display_name), lower(?1)) > 0
            OR EXISTS (SELECT 1 FROM media_tags mt JOIN tags t ON t.id = mt.tag_id
                       WHERE mt.media_id = m.id AND instr(lower(t.name), lower(?1)) > 0))
            AND (?2 IS NULL OR m.asset_kind = ?2)
            AND (?3 IS NULL OR m.media_format = ?3)
            AND (?4 = 0 OR m.is_favorite = 1)
            AND (?5 IS NULL OR EXISTS (SELECT 1 FROM media_tags mt JOIN tags t ON t.id = mt.tag_id
                       WHERE mt.media_id = m.id AND t.name = ?5 COLLATE NOCASE))";
        let total_sql = format!("SELECT COUNT(*) FROM media_items m WHERE {where_clause}");
        let total = self.connection.query_row(
            &total_sql,
            params![search, asset_kind, media_format, favorite, tag],
            |row| row.get::<_, i64>(0),
        )?;
        let total = u64::try_from(total).map_err(|_| invalid_data("media count"))?;
        let list_sql = format!(
            "SELECT m.id, m.content_hash, m.display_name, m.managed_path, m.asset_kind,
                    m.media_format,
                    m.width, m.height, m.frame_count, m.duration_ms, m.is_favorite,
                    m.created_at, m.updated_at
             FROM media_items m WHERE {where_clause}
             ORDER BY m.created_at DESC, m.id ASC LIMIT ?6 OFFSET ?7"
        );
        let mut statement = self.connection.prepare(&list_sql)?;
        let rows = statement.query_map(
            params![
                search,
                asset_kind,
                media_format,
                favorite,
                tag,
                limit,
                offset
            ],
            read_raw_media,
        )?;
        let mut items = Vec::new();
        for row in rows {
            items.push(self.hydrate_media(row?)?);
        }
        Ok(MediaPage { items, total })
    }

    pub(crate) fn media_by_id(&self, id: MediaId) -> Result<MediaItem, CoreError> {
        self.find_media("m.id = ?1", id.to_string())?
            .ok_or_else(|| CoreError::NotFound {
                entity: "Animation",
                id: id.to_string(),
            })
    }

    pub(crate) fn media_by_hash(&self, hash: &str) -> Result<Option<MediaItem>, CoreError> {
        self.find_media("m.content_hash = ?1", hash.to_owned())
    }

    pub(crate) fn existing_content_hashes(
        &self,
        hashes: &[String],
    ) -> Result<Vec<String>, CoreError> {
        let mut existing = Vec::new();
        for hash in hashes {
            if self.media_by_hash(hash)?.is_some() {
                existing.push(hash.clone());
            }
        }
        Ok(existing)
    }

    pub(crate) fn insert_media_with_tags(
        &mut self,
        new: &NewMediaItem,
        tags: &[String],
    ) -> Result<MediaItem, CoreError> {
        let display_name = validate_name(&new.display_name)?;
        let tags = validate_tags(tags)?;
        let now = Utc::now().timestamp_millis();
        let duration_ms = i64::try_from(new.duration_ms)
            .map_err(|_| CoreError::InvalidInput("The media duration is too long.".into()))?;
        let transaction = self.connection.transaction()?;
        transaction.execute(
            "INSERT INTO media_items
             (id, content_hash, display_name, managed_path, media_kind, asset_kind, media_format,
              width, height, frame_count, duration_ms, is_favorite, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, 0, ?12, ?12)",
            params![
                new.id.to_string(),
                new.content_hash,
                display_name,
                new.managed_path,
                new.media_format.as_str(),
                new.asset_kind.as_str(),
                new.media_format.as_str(),
                new.width,
                new.height,
                new.frame_count,
                duration_ms,
                now
            ],
        )?;
        transaction.execute(
            "INSERT INTO edit_recipes (media_id, updated_at) VALUES (?1, ?2)",
            params![new.id.to_string(), now],
        )?;
        for tag in tags.values() {
            transaction.execute("INSERT OR IGNORE INTO tags (name) VALUES (?1)", [tag])?;
            transaction.execute(
                "INSERT INTO media_tags (media_id, tag_id)
                 SELECT ?1, id FROM tags WHERE name = ?2 COLLATE NOCASE",
                params![new.id.to_string(), tag],
            )?;
        }
        transaction.commit()?;
        self.media_by_id(new.id)
    }

    pub(crate) fn rename_media(&mut self, id: MediaId, name: &str) -> Result<MediaItem, CoreError> {
        let name = validate_name(name)?;
        let changed = self.connection.execute(
            "UPDATE media_items SET display_name = ?1, updated_at = ?2 WHERE id = ?3",
            params![name, Utc::now().timestamp_millis(), id.to_string()],
        )?;
        ensure_changed(changed, "Animation", id)?;
        self.media_by_id(id)
    }

    pub(crate) fn set_favorite(
        &mut self,
        id: MediaId,
        favorite: bool,
    ) -> Result<MediaItem, CoreError> {
        let changed = self.connection.execute(
            "UPDATE media_items SET is_favorite = ?1, updated_at = ?2 WHERE id = ?3",
            params![favorite, Utc::now().timestamp_millis(), id.to_string()],
        )?;
        ensure_changed(changed, "Animation", id)?;
        self.media_by_id(id)
    }

    pub(crate) fn replace_tags(
        &mut self,
        id: MediaId,
        tags: &[String],
    ) -> Result<MediaItem, CoreError> {
        self.media_by_id(id)?;
        let tags = validate_tags(tags)?;
        let transaction = self.connection.transaction()?;
        transaction.execute(
            "DELETE FROM media_tags WHERE media_id = ?1",
            [id.to_string()],
        )?;
        for tag in tags.values() {
            transaction.execute("INSERT OR IGNORE INTO tags (name) VALUES (?1)", [tag])?;
            transaction.execute(
                "INSERT INTO media_tags (media_id, tag_id)
                 SELECT ?1, id FROM tags WHERE name = ?2 COLLATE NOCASE",
                params![id.to_string(), tag],
            )?;
        }
        transaction.execute(
            "UPDATE media_items SET updated_at = ?1 WHERE id = ?2",
            params![Utc::now().timestamp_millis(), id.to_string()],
        )?;
        transaction.commit()?;
        self.media_by_id(id)
    }

    pub(crate) fn delete_media_record(&mut self, id: MediaId) -> Result<DeleteReport, CoreError> {
        let transaction = self.connection.transaction()?;
        let overlays_removed = transaction.query_row(
            "SELECT COUNT(*) FROM overlay_instances WHERE media_id = ?1",
            [id.to_string()],
            |row| row.get::<_, u32>(0),
        )?;
        let changed =
            transaction.execute("DELETE FROM media_items WHERE id = ?1", [id.to_string()])?;
        ensure_changed(changed, "Animation", id)?;
        transaction.commit()?;
        Ok(DeleteReport {
            media_id: id,
            overlays_removed,
        })
    }

    fn find_media(&self, condition: &str, value: String) -> Result<Option<MediaItem>, CoreError> {
        let sql = format!(
            "SELECT m.id, m.content_hash, m.display_name, m.managed_path, m.asset_kind,
                    m.media_format,
                    m.width, m.height, m.frame_count, m.duration_ms, m.is_favorite,
                    m.created_at, m.updated_at FROM media_items m WHERE {condition}"
        );
        let raw = self
            .connection
            .query_row(&sql, [value], read_raw_media)
            .optional()?;
        raw.map(|item| self.hydrate_media(item)).transpose()
    }

    fn hydrate_media(&self, raw: RawMediaItem) -> Result<MediaItem, CoreError> {
        let id = MediaId(parse_uuid(&raw.id)?);
        let tags = self.tags_for(id)?;
        Ok(MediaItem {
            id,
            content_hash: raw.content_hash,
            display_name: raw.display_name,
            managed_path: raw.managed_path,
            asset_kind: AssetKind::parse(&raw.asset_kind)
                .ok_or_else(|| invalid_data("asset kind"))?,
            media_format: MediaFormat::parse(&raw.media_format)
                .ok_or_else(|| invalid_data("media format"))?,
            width: to_u32(raw.width, "width")?,
            height: to_u32(raw.height, "height")?,
            frame_count: to_u32(raw.frame_count, "frame count")?,
            duration_ms: u64::try_from(raw.duration_ms).map_err(|_| invalid_data("duration"))?,
            is_favorite: raw.is_favorite,
            tags,
            created_at: timestamp(raw.created_at)?,
            updated_at: timestamp(raw.updated_at)?,
        })
    }

    fn tags_for(&self, id: MediaId) -> Result<Vec<String>, CoreError> {
        let mut statement = self.connection.prepare(
            "SELECT t.name FROM tags t JOIN media_tags mt ON mt.tag_id = t.id
             WHERE mt.media_id = ?1 ORDER BY t.name COLLATE NOCASE",
        )?;
        let rows = statement.query_map([id.to_string()], |row| row.get::<_, String>(0))?;
        rows.collect::<Result<Vec<_>, _>>().map_err(Into::into)
    }
}

fn read_raw_media(row: &Row<'_>) -> rusqlite::Result<RawMediaItem> {
    Ok(RawMediaItem {
        id: row.get(0)?,
        content_hash: row.get(1)?,
        display_name: row.get(2)?,
        managed_path: row.get(3)?,
        asset_kind: row.get(4)?,
        media_format: row.get(5)?,
        width: row.get(6)?,
        height: row.get(7)?,
        frame_count: row.get(8)?,
        duration_ms: row.get(9)?,
        is_favorite: row.get(10)?,
        created_at: row.get(11)?,
        updated_at: row.get(12)?,
    })
}

fn normalized_optional(value: Option<&str>) -> Option<String> {
    value
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_owned)
}

fn ensure_changed(changed: usize, entity: &'static str, id: MediaId) -> Result<(), CoreError> {
    if changed == 0 {
        return Err(CoreError::NotFound {
            entity,
            id: id.to_string(),
        });
    }
    Ok(())
}

fn parse_uuid(value: &str) -> Result<Uuid, CoreError> {
    Uuid::parse_str(value).map_err(|_| invalid_data("ID"))
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
