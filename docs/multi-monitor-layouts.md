# Multi-monitor layouts

LumaDesk reads connected displays from the operating system and works in logical coordinates so layouts remain stable across display scale factors.

## Moving one overlay

Open **Active on Desktop** and use the **Display** selector in an overlay row. LumaDesk calculates the overlay's relative position on its current display, maps that position to the selected display, and clamps the rendered width and height inside the destination bounds.

The selector marks the primary display. If an overlay still references a disconnected display, the unavailable name remains visible until the overlay is moved or a scene is applied.

## Saved scenes

Scene snapshots store the display name, display dimensions, and normalized position for each overlay. Applying a scene follows this order:

1. Use the saved display when it is connected.
2. Otherwise use the primary display.
3. If the platform reports no primary display, use the first available display.
4. Clamp the complete overlay inside the chosen display.

Applying a non-empty scene requires at least one available display. The previous layout is restored if native synchronization fails.

## Platform notes

Windows and macOS use the displays reported by the native window system. Linux uses X11 directly and XWayland in Wayland sessions. Display names come from the operating system and can change after driver, dock, or hardware changes; the primary-display fallback keeps saved scenes usable in that case.

See [Desktop scenes](desktop-scenes.md) and [Desktop overlays](desktop-overlays.md).
