# Click-through overlays

Click-through lets pointer input pass through an overlay to the application or desktop below it. It is useful after an overlay has been positioned and no longer needs direct interaction.

## Enabling click-through

Use either control:

- the lock action in the overlay's temporary toolbar;
- the **Click-through** checkbox or lock action in **Active on Desktop**.

At the native-window layer, LumaDesk makes the overlay non-focusable and ignores cursor events. The setting persists with the overlay and is included in desktop scenes.

## Recovering interaction

A click-through overlay cannot receive a click to reveal its own toolbar. Unlock it from a surface that remains interactive:

- clear **Click-through** in its **Active on Desktop** row;
- choose **Unlock all** in the global action bar;
- choose **Unlock all overlays** from the tray or menu-bar menu;
- apply a scene in which the overlay is interactive.

Hidden click-through overlays remain listed in the manager, so visibility and interaction can be restored independently.

## Batch controls

**Lock all** makes every persisted overlay click-through. **Unlock all** restores pointer interaction for every overlay. These actions update SQLite and native windows together; if native synchronization fails, LumaDesk restores the previous state.

See [Desktop overlays](desktop-overlays.md), [Desktop scenes](desktop-scenes.md), and [Window and application lifecycle](window-lifecycle.md).
