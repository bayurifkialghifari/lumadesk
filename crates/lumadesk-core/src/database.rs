use std::{fs, path::Path};

use rusqlite::{Connection, OptionalExtension, TransactionBehavior};

use crate::CoreError;

pub const SCHEMA_VERSION: u32 = 1;

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
