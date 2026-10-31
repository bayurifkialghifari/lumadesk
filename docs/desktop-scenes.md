# Desktop scenes

Desktop scenes save a complete overlay workspace under a reusable name. Open **Active on Desktop** to create and manage scenes.

## What a scene stores

Every persisted overlay is included, including hidden instances. A scene records:

- the Collection item used by each overlay;
- the saved display and relative position;
- size, opacity, speed, and horizontal flip;
- immediate background removal;
- playing or paused state;
- interactive or click-through state;
- visible or hidden state and z-order.

## Managing scenes

- **Save current** creates a scene from the current persisted layout. Scene names are required and must be unique without regard to capitalization.
- **Apply scene** replaces the current overlay layout with the selected snapshot.
- **Update snapshot** replaces the selected scene's items with the current layout while keeping its name.
- **Rename** changes the selected scene name.
- **Duplicate** creates a copy with a generated unique name.
- **Delete** removes only the saved scene after confirmation. It does not remove Collection media or the current overlays.

## Applying a scene

LumaDesk maps every scene item to its saved display. If that display is unavailable, the item moves to the primary display. Positions are stored as monitor-relative values and clamped so the complete overlay remains inside the destination display.

A scene is applied as one coordinated layout change. LumaDesk restores the previous persisted layout and native windows if any native window cannot be synchronized.

Deleting a Collection item removes that item from saved scenes. The scene remains available and its item count updates.

See [Multi-monitor layouts](multi-monitor-layouts.md), [Click-through overlays](click-through.md), and [Desktop overlays](desktop-overlays.md).
