use rusqlite::OptionalExtension;

use crate::{AppSettings, CoreError, database::Database};

const SETTINGS_KEY: &str = "app_settings";

impl Database {
    pub(crate) fn get_settings(&self) -> Result<AppSettings, CoreError> {
        let stored = self
            .connection
            .query_row(
                "SELECT value FROM settings WHERE key = ?1",
                [SETTINGS_KEY],
                |row| row.get::<_, String>(0),
            )
            .optional()?;
        stored.map_or_else(
            || Ok(AppSettings::default()),
            |value| {
                serde_json::from_str(&value).map_err(|error| {
                    CoreError::InvalidData(format!("Pengaturan tidak dapat dibaca: {error}"))
                })
            },
        )
    }

    pub(crate) fn update_settings(
        &mut self,
        settings: &AppSettings,
    ) -> Result<AppSettings, CoreError> {
        validate_settings(settings)?;
        let value = serde_json::to_string(settings).map_err(|error| {
            CoreError::InvalidData(format!("Pengaturan tidak dapat disimpan: {error}"))
        })?;
        self.connection.execute(
            "INSERT INTO settings (key, value) VALUES (?1, ?2)
             ON CONFLICT(key) DO UPDATE SET value = excluded.value",
            [SETTINGS_KEY, &value],
        )?;
        self.get_settings()
    }
}

fn validate_settings(settings: &AppSettings) -> Result<(), CoreError> {
    if !matches!(settings.collection_view.as_str(), "grid" | "list") {
        return Err(CoreError::InvalidInput(
            "Tampilan koleksi harus grid atau list.".into(),
        ));
    }
    if !(64..=2_048).contains(&settings.default_overlay_width) {
        return Err(CoreError::InvalidInput(
            "Lebar overlay bawaan harus 64–2048 piksel.".into(),
        ));
    }
    if !(0.1..=1.0).contains(&settings.default_opacity) {
        return Err(CoreError::InvalidInput(
            "Opasitas harus berada di antara 0,1 dan 1.".into(),
        ));
    }
    if !(0.25..=4.0).contains(&settings.default_speed) {
        return Err(CoreError::InvalidInput(
            "Kecepatan harus berada di antara 0,25× dan 4×.".into(),
        ));
    }
    if !(64..=16_384).contains(&settings.cache_limit_mb) {
        return Err(CoreError::InvalidInput(
            "Batas cache harus 64–16384 MB.".into(),
        ));
    }
    Ok(())
}
