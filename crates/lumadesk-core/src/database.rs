use std::{fs, path::Path};

use rusqlite::{Connection, OptionalExtension, TransactionBehavior};

use crate::CoreError;

pub const SCHEMA_VERSION: u32 = 3;

#[derive(Debug)]
pub(crate) struct Database {
    pub(crate) connection: Connection,
}

impl Database {
    pub(crate) fn open(path: &Path) -> Result<Self, CoreError> {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        let connection = Connection::open(path)?;
        Self::initialize(connection)
    }

    fn initialize(connection: Connection) -> Result<Self, CoreError> {
        connection.execute_batch(
            "PRAGMA foreign_keys = ON;
             PRAGMA journal_mode = WAL;
             PRAGMA busy_timeout = 5000;",
        )?;
        let mut database = Self { connection };
        database.migrate()?;
        Ok(database)
    }

    fn migrate(&mut self) -> Result<(), CoreError> {
        let transaction = self
            .connection
            .transaction_with_behavior(TransactionBehavior::Immediate)?;
        transaction.execute_batch(
            "CREATE TABLE IF NOT EXISTS schema_version (
                version INTEGER NOT NULL
             );",
        )?;
        let current = transaction
            .query_row("SELECT version FROM schema_version LIMIT 1", [], |row| {
                row.get::<_, u32>(0)
            })
            .optional()?
            .unwrap_or(0);
        if current > SCHEMA_VERSION {
            return Err(CoreError::SchemaTooNew {
                found: current,
                supported: SCHEMA_VERSION,
            });
        }
        if current < 1 {
            migration_v1(&transaction)?;
        }
        if current < 2 {
            migration_v2(&transaction)?;
        }
        if current < 3 {
            migration_v3(&transaction)?;
        }
        if current < SCHEMA_VERSION {
            transaction.execute("DELETE FROM schema_version", [])?;
            transaction.execute(
                "INSERT INTO schema_version (version) VALUES (?1)",
                [SCHEMA_VERSION],
            )?;
        }
        transaction.commit()?;
        Ok(())
    }
}

fn migration_v1(transaction: &rusqlite::Transaction<'_>) -> Result<(), rusqlite::Error> {
    transaction.execute_batch(include_str!("schema_v1.sql"))
}

fn migration_v2(transaction: &rusqlite::Transaction<'_>) -> Result<(), rusqlite::Error> {
    transaction.execute_batch(include_str!("schema_v2.sql"))
}

fn migration_v3(transaction: &rusqlite::Transaction<'_>) -> Result<(), rusqlite::Error> {
    transaction.execute_batch(include_str!("schema_v3.sql"))
}

#[cfg(test)]
mod tests {
    use std::error::Error;

    use rusqlite::Connection;
    use uuid::Uuid;

    use super::{Database, migration_v1};

    #[test]
    fn migration_v2_defaults_existing_overlays_to_original_background() -> Result<(), Box<dyn Error>>
    {
        let mut connection = Connection::open_in_memory()?;
        connection.execute_batch(
            "CREATE TABLE schema_version (version INTEGER NOT NULL);
             INSERT INTO schema_version (version) VALUES (1);",
        )?;
        let transaction = connection.transaction()?;
        migration_v1(&transaction)?;
        transaction.commit()?;

        let media_id = Uuid::new_v4().to_string();
        let overlay_id = Uuid::new_v4().to_string();
        connection.execute(
            "INSERT INTO media_items
             (id, content_hash, display_name, managed_path, media_kind, width, height,
              frame_count, duration_ms, is_favorite, created_at, updated_at)
             VALUES (?1, ?2, 'Fixture', 'fixture.png', 'png', 4, 3, 1, 0, 0, 1, 1)",
            rusqlite::params![media_id, "a".repeat(64)],
        )?;
        connection.execute(
            "INSERT INTO overlay_instances
             (id, media_id, x, y, width, opacity, speed, created_at, updated_at)
             VALUES (?1, ?2, 0, 0, 240, 1, 1, 1, 1)",
            rusqlite::params![overlay_id, media_id],
        )?;

        let database = Database::initialize(connection)?;
        let remove_background = database.connection.query_row(
            "SELECT remove_background FROM overlay_instances WHERE id = ?1",
            [overlay_id],
            |row| row.get::<_, bool>(0),
        )?;
        let version =
            database
                .connection
                .query_row("SELECT version FROM schema_version", [], |row| {
                    row.get::<_, u32>(0)
                })?;

        assert!(!remove_background);
        assert_eq!(version, 3);
        Ok(())
    }

    #[test]
    fn migration_v3_splits_asset_kind_from_media_format() -> Result<(), Box<dyn Error>> {
        let connection = Connection::open_in_memory()?;
        let database = Database::initialize(connection)?;
        let columns = database
            .connection
            .prepare("SELECT name FROM pragma_table_info('media_items') ORDER BY cid")?
            .query_map([], |row| row.get::<_, String>(0))?
            .collect::<Result<Vec<_>, _>>()?;

        assert!(columns.contains(&"asset_kind".to_owned()));
        assert!(columns.contains(&"media_format".to_owned()));
        assert_eq!(
            database
                .connection
                .query_row("SELECT version FROM schema_version", [], |row| row
                    .get::<_, u32>(0))?,
            3
        );
        Ok(())
    }
}
