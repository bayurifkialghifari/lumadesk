use std::error::Error;

use lumadesk_core::{AppSettings, Library, LibraryQuery};
use tempfile::TempDir;

#[test]
fn migrations_create_schema_and_settings_round_trip() -> Result<(), Box<dyn Error>> {
    let temp = TempDir::new()?;
    let mut library = Library::open(temp.path())?;
    assert_eq!(library.list_media(&LibraryQuery::default())?.total, 0);
    assert_eq!(library.get_settings()?, AppSettings::default());

    let settings = AppSettings {
        collection_view: "list".into(),
        default_speed: 1.5,
        ..AppSettings::default()
    };
    assert_eq!(library.update_settings(&settings)?, settings);
    Ok(())
}

#[test]
fn invalid_settings_are_rejected_before_storage() -> Result<(), Box<dyn Error>> {
    let temp = TempDir::new()?;
    let mut library = Library::open(temp.path())?;
    let settings = AppSettings {
        default_speed: 10.0,
        ..AppSettings::default()
    };
    assert!(library.update_settings(&settings).is_err());
    assert_eq!(library.get_settings()?, AppSettings::default());
    Ok(())
}
