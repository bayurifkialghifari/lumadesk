use std::collections::HashMap;

use lumadesk_core::{
    DesktopScene, DesktopSceneId, DesktopSceneItem, DesktopSceneSummary, MediaId, MediaItem,
    MonitorDescriptor, NewOverlay, OverlayBatchPatch, OverlayId, OverlayInstance, OverlayPatch,
    OverlayReplacement,
};
use serde::Serialize;
use tauri::{AppHandle, Manager as _, Monitor, State};

use crate::{
    command_error::CommandError,
    overlay_windows,
    state::{AppState, with_library},
};

const DUPLICATE_OFFSET: f64 = 24.0;

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct DesktopOverlay {
    overlay: OverlayInstance,
    media: MediaItem,
}

#[tauri::command]
pub(crate) async fn list_overlays(
    state: State<'_, AppState>,
) -> Result<Vec<OverlayInstance>, CommandError> {
    with_library(state, |library| library.list_overlays()).await
}

#[tauri::command]
pub(crate) async fn list_desktop_overlays(
    state: State<'_, AppState>,
) -> Result<Vec<DesktopOverlay>, CommandError> {
    with_library(state, |library| {
        library
            .list_overlays()?
            .into_iter()
            .map(|overlay| {
                let media = library.media_item(overlay.media_id)?;
                Ok(DesktopOverlay { overlay, media })
            })
            .collect()
    })
    .await
}

#[tauri::command]
pub(crate) async fn create_overlay(
    app: AppHandle,
    state: State<'_, AppState>,
    input: NewOverlay,
) -> Result<OverlayInstance, CommandError> {
    let (overlay, source) = with_library(state, move |library| {
        let source = overlay_windows::OverlayMedia::load(library, input.media_id)?;
        let overlay = library.create_overlay(&input)?;
        Ok((overlay, source))
    })
    .await?;
    if let Err(error) = overlay_windows::sync(&app, &overlay, &source) {
        rollback_created_overlay(&app, overlay.id).await;
        return Err(CommandError::overlay(&error));
    }
    overlay_windows::emit_catalog_changed(&app);
    Ok(overlay)
}

#[tauri::command]
pub(crate) async fn show_media_overlay(
    app: AppHandle,
    state: State<'_, AppState>,
    id: MediaId,
) -> Result<OverlayInstance, CommandError> {
    let (source, width) = with_library(state, move |library| {
        let source = overlay_windows::OverlayMedia::load(library, id)?;
        let settings = library.get_settings()?;
        Ok((source, f64::from(settings.default_overlay_width)))
    })
    .await?;
    let input = overlay_windows::default_input(&app, &source, width)
        .map_err(|error| CommandError::overlay(&error))?;
    let overlay = with_library(app.state(), move |library| library.create_overlay(&input)).await?;
    if let Err(error) = overlay_windows::sync(&app, &overlay, &source) {
        rollback_created_overlay(&app, overlay.id).await;
        return Err(CommandError::overlay(&error));
    }
    overlay_windows::emit_catalog_changed(&app);
    Ok(overlay)
}

#[tauri::command]
pub(crate) async fn update_overlay(
    app: AppHandle,
    state: State<'_, AppState>,
    id: OverlayId,
    patch: OverlayPatch,
) -> Result<OverlayInstance, CommandError> {
    let (previous, overlay, source) = with_library(state, move |library| {
        let previous = library.overlay(id)?;
        let source = overlay_windows::OverlayMedia::load(library, previous.media_id)?;
        let overlay = library.update_overlay(id, &patch)?;
        Ok((previous, overlay, source))
    })
    .await?;
    if let Err(error) = overlay_windows::sync(&app, &overlay, &source) {
        restore_updated_overlay(&app, previous, &source).await;
        return Err(CommandError::overlay(&error));
    }
    overlay_windows::emit_catalog_changed(&app);
    Ok(overlay)
}

#[tauri::command]
pub(crate) async fn move_overlay(
    app: AppHandle,
    state: State<'_, AppState>,
    id: OverlayId,
    monitor_id: Option<String>,
    x: f64,
    y: f64,
) -> Result<OverlayInstance, CommandError> {
    let patch = OverlayPatch {
        monitor_id: Some(monitor_id),
        x: Some(x),
        y: Some(y),
        ..OverlayPatch::default()
    };
    let (previous, overlay, source) = with_library(state, move |library| {
        let previous = library.overlay(id)?;
        let source = overlay_windows::OverlayMedia::load(library, previous.media_id)?;
        let overlay = library.update_overlay(id, &patch)?;
        Ok((previous, overlay, source))
    })
    .await?;
    if let Err(error) = overlay_windows::emit_state(&app, &overlay) {
        restore_updated_overlay(&app, previous, &source).await;
        return Err(CommandError::overlay(&error));
    }
    overlay_windows::emit_catalog_changed(&app);
    Ok(overlay)
}

#[tauri::command]
pub(crate) async fn move_overlay_to_monitor(
    app: AppHandle,
    state: State<'_, AppState>,
    id: OverlayId,
    monitor_name: String,
) -> Result<OverlayInstance, CommandError> {
    let monitors = monitor_descriptors(&app)?;
    let target = monitors
        .iter()
        .find(|monitor| monitor.name == monitor_name)
        .ok_or_else(|| {
            CommandError::invalid_input("The selected display is no longer available.")
        })?;
    let (previous, source) = with_library(state, move |library| {
        let overlay = library.overlay(id)?;
        let source = overlay_windows::OverlayMedia::load(library, overlay.media_id)?;
        Ok((overlay, source))
    })
    .await?;
    let current = monitor_for_overlay(&monitors, &previous)
        .ok_or_else(|| CommandError::invalid_input("No display is available."))?;
    let normalized_x = (previous.x - current.x) / current.width;
    let normalized_y = (previous.y - current.y) / current.height;
    let (rendered_width, rendered_height) = overlay_windows::rendered_size(previous.width, &source);
    let (x, y) = mapped_position(
        normalized_x,
        normalized_y,
        target,
        rendered_width,
        rendered_height,
    );
    let patch = OverlayPatch {
        monitor_id: Some(Some(target.name.clone())),
        x: Some(x),
        y: Some(y),
        ..OverlayPatch::default()
    };
    let overlay = with_library(app.state(), move |library| {
        library.update_overlay(id, &patch)
    })
    .await?;
    if let Err(error) = overlay_windows::sync(&app, &overlay, &source) {
        restore_updated_overlay(&app, previous, &source).await;
        return Err(CommandError::overlay(&error));
    }
    overlay_windows::emit_catalog_changed(&app);
    Ok(overlay)
}

#[tauri::command]
pub(crate) async fn duplicate_overlay(
    app: AppHandle,
    state: State<'_, AppState>,
    id: OverlayId,
) -> Result<OverlayInstance, CommandError> {
    let (overlay, source) = with_library(state, move |library| {
        let source = library.overlay(id)?;
        let media = overlay_windows::OverlayMedia::load(library, source.media_id)?;
        let overlay = library.duplicate_overlay(
            id,
            source.x + DUPLICATE_OFFSET,
            source.y + DUPLICATE_OFFSET,
        )?;
        Ok((overlay, media))
    })
    .await?;
    if let Err(error) = overlay_windows::sync(&app, &overlay, &source) {
        rollback_created_overlay(&app, overlay.id).await;
        return Err(CommandError::overlay(&error));
    }
    overlay_windows::emit_catalog_changed(&app);
    Ok(overlay)
}

#[tauri::command]
pub(crate) async fn remove_overlay(
    app: AppHandle,
    state: State<'_, AppState>,
    id: OverlayId,
) -> Result<(), CommandError> {
    let (overlay, source) = with_library(state, move |library| {
        let overlay = library.overlay(id)?;
        let source = overlay_windows::OverlayMedia::load(library, overlay.media_id)?;
        Ok((overlay, source))
    })
    .await?;
    overlay_windows::destroy(&app, id).map_err(|error| CommandError::overlay(&error))?;
    if let Err(error) = with_library(app.state(), move |library| library.remove_overlay(id)).await {
        if let Err(restore_error) = overlay_windows::sync(&app, &overlay, &source) {
            eprintln!("failed to restore overlay {id} after remove failed: {restore_error}");
        }
        return Err(error);
    }
    overlay_windows::emit_catalog_changed(&app);
    Ok(())
}

#[tauri::command]
pub(crate) fn list_monitors(app: AppHandle) -> Result<Vec<MonitorDescriptor>, CommandError> {
    monitor_descriptors(&app)
}

#[tauri::command]
pub(crate) async fn list_desktop_scenes(
    state: State<'_, AppState>,
) -> Result<Vec<DesktopSceneSummary>, CommandError> {
    with_library(state, |library| library.list_desktop_scenes()).await
}

#[tauri::command]
pub(crate) async fn create_desktop_scene(
    app: AppHandle,
    state: State<'_, AppState>,
    name: String,
) -> Result<DesktopScene, CommandError> {
    let items = capture_scene_items(&app, state).await?;
    with_library(app.state(), move |library| {
        library.create_desktop_scene(&name, &items)
    })
    .await
}

#[tauri::command]
pub(crate) async fn update_desktop_scene(
    app: AppHandle,
    state: State<'_, AppState>,
    id: DesktopSceneId,
) -> Result<DesktopScene, CommandError> {
    let items = capture_scene_items(&app, state).await?;
    with_library(app.state(), move |library| {
        library.update_desktop_scene(id, &items)
    })
    .await
}

#[tauri::command]
pub(crate) async fn rename_desktop_scene(
    state: State<'_, AppState>,
    id: DesktopSceneId,
    name: String,
) -> Result<DesktopScene, CommandError> {
    with_library(state, move |library| {
        library.rename_desktop_scene(id, &name)
    })
    .await
}

#[tauri::command]
pub(crate) async fn duplicate_desktop_scene(
    state: State<'_, AppState>,
    id: DesktopSceneId,
) -> Result<DesktopScene, CommandError> {
    with_library(state, move |library| library.duplicate_desktop_scene(id)).await
}

#[tauri::command]
pub(crate) async fn delete_desktop_scene(
    state: State<'_, AppState>,
    id: DesktopSceneId,
) -> Result<(), CommandError> {
    with_library(state, move |library| library.delete_desktop_scene(id)).await
}

#[tauri::command]
pub(crate) async fn apply_desktop_scene(
    app: AppHandle,
    state: State<'_, AppState>,
    id: DesktopSceneId,
) -> Result<Vec<OverlayInstance>, CommandError> {
    let monitors = monitor_descriptors(&app)?;
    let (scene, previous, previous_sources, scene_sources) = with_library(state, move |library| {
        let scene = library.desktop_scene(id)?;
        let previous = library.list_overlays()?;
        let previous_sources = load_sources(library, &previous)?;
        let mut scene_sources = HashMap::new();
        for item in &scene.items {
            scene_sources
                .entry(item.media_id)
                .or_insert(overlay_windows::OverlayMedia::load(library, item.media_id)?);
        }
        Ok((scene, previous, previous_sources, scene_sources))
    })
    .await?;
    if !scene.items.is_empty() && monitors.is_empty() {
        return Err(CommandError::invalid_input("No display is available."));
    }
    let replacements = scene
        .items
        .iter()
        .map(|item| {
            let target = monitor_for_scene_item(&monitors, item)
                .ok_or_else(|| CommandError::invalid_input("No display is available."))?;
            let source = scene_sources
                .get(&item.media_id)
                .ok_or_else(|| CommandError::invalid_data("Scene media could not be prepared."))?;
            let (rendered_width, rendered_height) =
                overlay_windows::rendered_size(item.width, source);
            let (x, y) = mapped_position(
                item.normalized_x,
                item.normalized_y,
                target,
                rendered_width,
                rendered_height,
            );
            Ok(OverlayReplacement {
                media_id: item.media_id,
                monitor_id: Some(target.name.clone()),
                x,
                y,
                width: item.width,
                opacity: item.opacity,
                speed: item.speed,
                flip_horizontal: item.flip_horizontal,
                remove_background: item.remove_background,
                is_playing: item.is_playing,
                click_through: item.click_through,
                z_order: item.z_order,
                is_active: item.is_active,
            })
        })
        .collect::<Result<Vec<_>, CommandError>>()?;
    let applied = with_library(app.state(), move |library| {
        library.replace_overlays(&replacements)
    })
    .await?;
    if let Err(error) = replace_native_layout(&app, &previous, &applied, &scene_sources) {
        rollback_layout(&app, &previous, &previous_sources, &applied).await;
        return Err(CommandError::overlay(&error));
    }
    overlay_windows::emit_catalog_changed(&app);
    Ok(applied)
}

#[tauri::command]
pub(crate) async fn patch_all_overlays(
    app: AppHandle,
    patch: OverlayBatchPatch,
) -> Result<Vec<OverlayInstance>, CommandError> {
    patch_all_overlays_impl(&app, patch).await
}

pub(crate) async fn patch_all_overlays_impl(
    app: &AppHandle,
    patch: OverlayBatchPatch,
) -> Result<Vec<OverlayInstance>, CommandError> {
    let (previous, sources, updated) = with_library(app.state(), move |library| {
        let previous = library.list_overlays()?;
        let sources = load_sources(library, &previous)?;
        let updated = library.patch_all_overlays(&patch)?;
        Ok((previous, sources, updated))
    })
    .await?;
    if let Err(error) = sync_layout(app, &updated, &sources) {
        rollback_layout(app, &previous, &sources, &updated).await;
        return Err(CommandError::overlay(&error));
    }
    overlay_windows::emit_catalog_changed(app);
    Ok(updated)
}

fn monitor_descriptors(app: &AppHandle) -> Result<Vec<MonitorDescriptor>, CommandError> {
    let monitors = app
        .available_monitors()
        .map_err(|error| CommandError::native("Displays could not be listed", error))?;
    let primary = app
        .primary_monitor()
        .map_err(|error| CommandError::native("The primary display could not be read", error))?;
    Ok(monitors
        .iter()
        .enumerate()
        .map(|(index, monitor)| descriptor(monitor, index, primary.as_ref()))
        .collect())
}

fn descriptor(monitor: &Monitor, index: usize, primary: Option<&Monitor>) -> MonitorDescriptor {
    let scale_factor = monitor.scale_factor();
    let position = monitor.position().to_logical::<f64>(scale_factor);
    let size = monitor.size().to_logical::<f64>(scale_factor);
    MonitorDescriptor {
        name: monitor
            .name()
            .cloned()
            .unwrap_or_else(|| format!("Display {}", index + 1)),
        x: position.x,
        y: position.y,
        width: size.width,
        height: size.height,
        scale_factor,
        is_primary: primary.is_some_and(|candidate| same_monitor(monitor, candidate)),
    }
}

fn same_monitor(left: &Monitor, right: &Monitor) -> bool {
    left.name() == right.name()
        && left.position() == right.position()
        && left.size() == right.size()
}

async fn capture_scene_items(
    app: &AppHandle,
    state: State<'_, AppState>,
) -> Result<Vec<DesktopSceneItem>, CommandError> {
    let monitors = monitor_descriptors(app)?;
    let overlays = with_library(state, |library| library.list_overlays()).await?;
    overlays
        .into_iter()
        .map(|overlay| {
            let monitor = monitor_for_overlay(&monitors, &overlay)
                .ok_or_else(|| CommandError::invalid_input("No display is available."))?;
            Ok(DesktopSceneItem {
                media_id: overlay.media_id,
                monitor_name: Some(monitor.name.clone()),
                monitor_width: monitor.width,
                monitor_height: monitor.height,
                normalized_x: (overlay.x - monitor.x) / monitor.width,
                normalized_y: (overlay.y - monitor.y) / monitor.height,
                width: overlay.width,
                opacity: overlay.opacity,
                speed: overlay.speed,
                flip_horizontal: overlay.flip_horizontal,
                remove_background: overlay.remove_background,
                is_playing: overlay.is_playing,
                click_through: overlay.click_through,
                z_order: overlay.z_order,
                is_active: overlay.is_active,
            })
        })
        .collect()
}

fn monitor_for_overlay<'a>(
    monitors: &'a [MonitorDescriptor],
    overlay: &OverlayInstance,
) -> Option<&'a MonitorDescriptor> {
    overlay
        .monitor_id
        .as_ref()
        .and_then(|name| monitors.iter().find(|monitor| &monitor.name == name))
        .or_else(|| {
            monitors.iter().find(|monitor| {
                overlay.x >= monitor.x
                    && overlay.x < monitor.x + monitor.width
                    && overlay.y >= monitor.y
                    && overlay.y < monitor.y + monitor.height
            })
        })
        .or_else(|| primary_monitor(monitors))
}

fn monitor_for_scene_item<'a>(
    monitors: &'a [MonitorDescriptor],
    item: &DesktopSceneItem,
) -> Option<&'a MonitorDescriptor> {
    item.monitor_name
        .as_ref()
        .and_then(|name| monitors.iter().find(|monitor| &monitor.name == name))
        .or_else(|| primary_monitor(monitors))
}

fn primary_monitor(monitors: &[MonitorDescriptor]) -> Option<&MonitorDescriptor> {
    monitors
        .iter()
        .find(|monitor| monitor.is_primary)
        .or_else(|| monitors.first())
}

fn mapped_position(
    normalized_x: f64,
    normalized_y: f64,
    target: &MonitorDescriptor,
    overlay_width: f64,
    overlay_height: f64,
) -> (f64, f64) {
    let x = target.x + normalized_x * target.width;
    let y = target.y + normalized_y * target.height;
    (
        clamp_axis(x, target.x, target.width, overlay_width),
        clamp_axis(y, target.y, target.height, overlay_height),
    )
}

fn clamp_axis(value: f64, origin: f64, available: f64, content: f64) -> f64 {
    value.clamp(origin, origin + (available - content).max(0.0))
}

fn load_sources(
    library: &lumadesk_core::Library,
    overlays: &[OverlayInstance],
) -> Result<HashMap<MediaId, overlay_windows::OverlayMedia>, lumadesk_core::CoreError> {
    let mut sources = HashMap::new();
    for overlay in overlays {
        sources
            .entry(overlay.media_id)
            .or_insert(overlay_windows::OverlayMedia::load(
                library,
                overlay.media_id,
            )?);
    }
    Ok(sources)
}

fn sync_layout(
    app: &AppHandle,
    overlays: &[OverlayInstance],
    sources: &HashMap<MediaId, overlay_windows::OverlayMedia>,
) -> tauri::Result<()> {
    for overlay in overlays {
        let source = sources.get(&overlay.media_id).ok_or_else(|| {
            tauri::Error::AssetNotFound(format!("media source {}", overlay.media_id))
        })?;
        overlay_windows::sync(app, overlay, source)?;
    }
    Ok(())
}

fn replace_native_layout(
    app: &AppHandle,
    previous: &[OverlayInstance],
    applied: &[OverlayInstance],
    sources: &HashMap<MediaId, overlay_windows::OverlayMedia>,
) -> tauri::Result<()> {
    for overlay in previous {
        overlay_windows::destroy(app, overlay.id)?;
    }
    sync_layout(app, applied, sources)
}

async fn rollback_layout(
    app: &AppHandle,
    previous: &[OverlayInstance],
    previous_sources: &HashMap<MediaId, overlay_windows::OverlayMedia>,
    current: &[OverlayInstance],
) {
    for overlay in current {
        if let Err(error) = overlay_windows::destroy(app, overlay.id) {
            eprintln!(
                "failed to destroy overlay {} during rollback: {error}",
                overlay.id
            );
        }
    }
    let previous = previous.to_vec();
    let restored = with_library(app.state(), move |library| {
        library.restore_overlays(&previous)
    })
    .await;
    let restored = match restored {
        Ok(restored) => restored,
        Err(error) => {
            eprintln!("failed to restore the previous desktop layout: {error:?}");
            return;
        }
    };
    if let Err(error) = sync_layout(app, &restored, previous_sources) {
        eprintln!("failed to restore native overlay windows: {error}");
    }
}

async fn rollback_created_overlay(app: &AppHandle, id: OverlayId) {
    let result = with_library(app.state(), move |library| library.remove_overlay(id)).await;
    if let Err(error) = result {
        eprintln!("failed to roll back overlay {id}: {error:?}");
    }
}

async fn restore_updated_overlay(
    app: &AppHandle,
    previous: OverlayInstance,
    source: &overlay_windows::OverlayMedia,
) {
    let id = previous.id;
    let patch = OverlayPatch {
        monitor_id: Some(previous.monitor_id.clone()),
        x: Some(previous.x),
        y: Some(previous.y),
        width: Some(previous.width),
        opacity: Some(previous.opacity),
        speed: Some(previous.speed),
        flip_horizontal: Some(previous.flip_horizontal),
        remove_background: Some(previous.remove_background),
        click_through: Some(previous.click_through),
        is_playing: Some(previous.is_playing),
        z_order: Some(previous.z_order),
        is_active: Some(previous.is_active),
    };
    let restored = with_library(app.state(), move |library| {
        library.update_overlay(id, &patch)
    })
    .await;
    let restored = match restored {
        Ok(restored) => restored,
        Err(error) => {
            eprintln!("failed to roll back overlay {id}: {error:?}");
            return;
        }
    };
    if let Err(error) = overlay_windows::sync(app, &restored, source) {
        eprintln!("failed to restore overlay window {id}: {error}");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn monitor(name: &str, x: f64, y: f64, width: f64, height: f64) -> MonitorDescriptor {
        MonitorDescriptor {
            name: name.into(),
            x,
            y,
            width,
            height,
            scale_factor: 1.0,
            is_primary: name == "Primary",
        }
    }

    #[test]
    fn mapping_preserves_relative_position_across_resolution_changes() {
        let target = monitor("Primary", 0.0, 0.0, 2_560.0, 1_440.0);
        let result = mapped_position(0.25, 0.5, &target, 320.0, 180.0);

        assert_eq!(result, (640.0, 720.0));
    }

    #[test]
    fn mapping_handles_negative_origins_and_clamps_the_complete_overlay() {
        let target = monitor("Left", -1_920.0, -200.0, 1_920.0, 1_080.0);
        let result = mapped_position(0.95, 0.95, &target, 400.0, 300.0);

        assert_eq!(result, (-400.0, 580.0));
    }

    #[test]
    fn oversized_overlays_are_anchored_to_the_monitor_origin() {
        let target = monitor("Primary", 100.0, 50.0, 800.0, 600.0);
        let result = mapped_position(0.5, 0.5, &target, 1_000.0, 900.0);

        assert_eq!(result, (100.0, 50.0));
    }

    #[test]
    fn missing_saved_monitor_falls_back_to_primary() {
        let monitors = [
            monitor("Primary", 0.0, 0.0, 1_920.0, 1_080.0),
            monitor("Right", 1_920.0, 0.0, 1_920.0, 1_080.0),
        ];
        let item = DesktopSceneItem {
            media_id: MediaId::new(),
            monitor_name: Some("Missing".into()),
            monitor_width: 1_920.0,
            monitor_height: 1_080.0,
            normalized_x: 0.0,
            normalized_y: 0.0,
            width: 320.0,
            opacity: 1.0,
            speed: 1.0,
            flip_horizontal: false,
            remove_background: false,
            is_playing: true,
            click_through: false,
            z_order: 0,
            is_active: true,
        };

        assert_eq!(
            monitor_for_scene_item(&monitors, &item).map(|value| value.name.as_str()),
            Some("Primary")
        );
    }

    #[test]
    fn logical_mapping_is_independent_of_scale_factor() {
        let mut target = monitor("Primary", 0.0, 0.0, 1_440.0, 900.0);
        target.scale_factor = 2.0;

        assert_eq!(
            mapped_position(0.5, 0.5, &target, 300.0, 200.0),
            (720.0, 450.0)
        );
    }
}
