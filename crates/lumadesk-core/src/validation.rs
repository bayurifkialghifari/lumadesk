use std::collections::BTreeMap;

use crate::CoreError;

pub(crate) fn validate_name(name: &str) -> Result<&str, CoreError> {
    let name = name.trim();
    if name.is_empty() || name.chars().count() > 120 {
        return Err(CoreError::InvalidInput(
            "The name must contain 1–120 characters.".into(),
        ));
    }
    Ok(name)
}

pub(crate) fn validate_tags(tags: &[String]) -> Result<BTreeMap<String, String>, CoreError> {
    if tags.len() > 20 {
        return Err(CoreError::InvalidInput(
            "Each animation can have at most 20 tags.".into(),
        ));
    }
    let mut unique = BTreeMap::new();
    for tag in tags {
        let tag = tag.trim();
        if tag.is_empty() || tag.chars().count() > 32 {
            return Err(CoreError::InvalidInput(
                "Each tag must contain 1–32 characters.".into(),
            ));
        }
        unique
            .entry(tag.to_lowercase())
            .or_insert_with(|| tag.to_owned());
    }
    Ok(unique)
}
