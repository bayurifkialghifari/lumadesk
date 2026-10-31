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
                    CoreError::InvalidData(format!("Settings could not be read: {error}"))
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
            CoreError::InvalidData(format!("Settings could not be saved: {error}"))
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
    if !matches!(settings.theme.as_str(), "dark" | "system") {
        return Err(CoreError::InvalidInput(
            "The theme must be dark or system.".into(),
        ));
    }
    if !matches!(settings.collection_view.as_str(), "grid" | "list") {
        return Err(CoreError::InvalidInput(
            "The collection view must be grid or list.".into(),
        ));
    }
    if !(64..=2_048).contains(&settings.default_overlay_width) {
        return Err(CoreError::InvalidInput(
            "The default overlay width must be 64–2048 pixels.".into(),
        ));
    }
    if !(0.1..=1.0).contains(&settings.default_opacity) {
        return Err(CoreError::InvalidInput(
            "Opacity must be between 0.1 and 1.".into(),
        ));
    }
    if !(0.25..=4.0).contains(&settings.default_speed) {
        return Err(CoreError::InvalidInput(
            "Speed must be between 0.25× and 4×.".into(),
        ));
    }
    Ok(())
}
