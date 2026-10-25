# Desktop overlays

Select an item in the Collection and choose **Tampilkan di desktop** (Show on desktop). One library item can have multiple instances with independent positions and settings.

## Available behavior

- New overlays open in the center of the primary monitor using the configured default size.
- Transparent windows stay above regular windows, remain hidden from the taskbar, and pass pointer input through to applications below them.
- GIF, APNG, and animated WebP files use their original frame timing. Static PNG and JPEG files render once without an animation scheduler.
- The renderer applies persisted size, opacity, speed, horizontal flip, and play or pause state for each instance.
- The **Aktif di Desktop** (Active on Desktop) page lists every active instance with its media, monitor, size, status, and controls.
- **Mode edit** (Edit mode) enables pointer input for every overlay. Each overlay displays a subtle boundary and a toolbar for moving, playing or pausing, flipping, duplicating, and closing it.
- The latest dragged position is stored locally in logical coordinates.
- Bulk actions can pause every playing overlay or close all overlays after confirmation.
- Active overlays are restored when LumaDesk opens if **Pulihkan overlay** (Restore overlays) is enabled.
- Removing media from the Collection closes its overlay windows before deleting local data. Closed windows are restored if deletion fails.
- Instance changes return to their previous value if the native window cannot be updated. Removing an instance also closes its window before deleting the local record.

## Instance controls

- **Position:** drag the overlay while Edit mode is active.
- **Size:** change the width while preserving the media aspect ratio.
- **Opacity:** 10% to 100%.
- **Speed:** 0.25× to 4×.
- **Horizontal flip:** mirror the visual without modifying the original media.
- **Play/Pause:** control one instance.
- **Duplicate:** create another instance with matching settings and a slightly offset position.
- **Close:** remove the desktop instance without deleting the Collection item.

Detailed controls are available on the **Active on Desktop** page. The toolbar attached to an overlay stays compact so it does not cover the animation. When the interface runs in a browser during development, the page remains available for preview while native window controls are disabled.

## Multiple monitors

LumaDesk stores positions in logical coordinates together with the monitor identity from the latest drag operation. New overlays open on the primary monitor. Automatically moving an overlay into a visible area when its saved monitor is unavailable remains planned work.
