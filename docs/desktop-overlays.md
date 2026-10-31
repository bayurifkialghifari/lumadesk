# Desktop overlays

Select an item in Collection and choose **Show on desktop**. One library item can create multiple native overlay windows with independent positions and instance settings.

## Available behavior

- New overlays open near the center of the primary monitor using the configured default width, opacity, and speed.
- Transparent windows stay above regular windows, remain hidden from the taskbar, and accept pointer input for direct dragging.
- GIF, APNG, animated WebP, and sprite sheets use their saved frame timing and playback mode. Static PNG and JPEG files render once.
- Saved Creator recipes apply crop, rotation, recipe flip, color adjustments, and background mode before per-instance controls.
- Each instance keeps its own display, size, opacity, speed, horizontal flip, visibility, play state, click-through state, and edge-removal shortcut.
- Pressing the animation reveals a subtle boundary and toolbar. The controls hide after three seconds without interaction and remain visible while the toolbar is in use.
- The latest dragged position is stored in logical coordinates.
- **Active on Desktop** lists every persisted instance, including hidden overlays, and provides detailed controls plus global show, hide, play, pause, lock, unlock, and confirmed close actions.
- Active instances return at launch when **Restore overlays** is enabled.
- Removing media closes its overlay windows before deleting managed data. Captured windows are restored if persistence fails.

## Overlay controls

- **Position:** drag anywhere on the animation or use the toolbar grip.
- **Size:** change width while preserving the edited frame's aspect ratio.
- **Opacity:** 10% to 100%.
- **Speed:** 0.25× to 4×.
- **Horizontal flip:** mirror one instance in addition to the saved Creator recipe.
- **Remove background:** toggle immediate edge removal for the selected instance.
- **Display:** move the instance to any connected monitor while retaining its relative placement.
- **Show/Hide:** keep the instance configured while controlling whether its native window is visible.
- **Click-through:** pass pointer input to windows below the overlay. Unlock it from the manager or tray.
- **Play/Pause:** control one animation.
- **Duplicate:** create another instance with matching settings and a small position offset.
- **Close:** remove the desktop instance without deleting its Collection item.

## Sprite playback

A configured sprite sheet behaves like other animated media. Loop repeats continuously, Once stops on the final frame, and Ping-pong reverses direction at each end. Editing the sprite grid invalidates old decoded frames and refreshes active overlays.

## Multiple monitors

LumaDesk stores logical coordinates with the monitor identity from the latest drag. New overlays open on the primary monitor. Moving an overlay between displays preserves its relative position and clamps the complete rendered bounds inside the destination display. Applying a scene uses the primary display when a saved monitor is unavailable.

See [Desktop scenes](desktop-scenes.md), [Multi-monitor layouts](multi-monitor-layouts.md), [Click-through overlays](click-through.md), [Creator Studio](creator-studio.md), [Sprite sheets](sprite-sheets.md), and [Background removal](background-removal.md).
