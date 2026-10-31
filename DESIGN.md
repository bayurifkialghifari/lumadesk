# LumaDesk Design System

**Status:** implementation contract
**Updated:** 31 October 2026

## 0. Design goals

- Product shell: dense desktop-library navigation, media shelves, restrained controls, and clear ownership of the center scroll region.
- README direction: theme-aware banner, one-line promise, compact badges, an early download table, and links to deeper documentation.
- Spatial pattern: fixed 224 px navigation rail, flexible content canvas, and 320 px contextual inspector on wide screens.
- Accessibility review: selected states use tonal surfaces plus icon/text changes; keyboard focus uses an explicit blue focus ring; motion follows `prefers-reduced-motion`.

## 1. Character

LumaDesk feels like a focused media console: dark, compact, calm, and slightly luminous. It uses depth to separate navigation, library, preview, and control surfaces. The interface avoids decorative glass, loud gradients, oversized marketing type, and rounded-card grids that waste desktop space.

## 2. Tokens

### Color

| Token | Value | Use |
| --- | --- | --- |
| `canvas-950` | `#08111c` | application background |
| `canvas-900` | `#0d1826` | navigation and title bar |
| `surface-850` | `#122235` | primary panels |
| `surface-800` | `#172b41` | elevated and hover surfaces |
| `surface-750` | `#1e354e` | selected rows and active controls |
| `line-700` | `#29435f` | neutral borders and dividers |
| `ink-100` | `#f4f8fc` | primary text |
| `ink-300` | `#b8c7d9` | secondary text |
| `ink-500` | `#7890aa` | muted text |
| `brand-400` | `#39a8ff` | primary actions and focus |
| `brand-300` | `#72c5ff` | hover and bright labels |
| `success-400` | `#42d392` | healthy/active state |
| `warning-400` | `#f3b95f` | recoverable warning |
| `danger-400` | `#ff6b75` | destructive action and error |

Gradients are limited to the brand banner, empty-state illustration, and preview ambience. Controls and content surfaces use solid token colors.

When **Follow system** is selected, the dark tokens remain active for a dark operating-system theme. A light operating-system theme maps the same token names to cool gray canvases, white content surfaces, navy text, and a darker blue accent. Overlay windows keep a transparent canvas in both modes.

### Typography

- UI family: `Inter Variable`, locally bundled, with system sans-serif fallback.
- Display family: `Space Grotesk Variable`, locally bundled, for logo and large empty states only.
- Scale: 12, 13, 14, 16, 20, 28, and 40 px. Body defaults to 14 px/1.5.
- Labels use sentence case. Uppercase is reserved for tiny metadata eyebrows with 0.08 em tracking.

### Space, radius, and depth

- Base spacing unit: 4 px. Common steps: 4, 8, 12, 16, 20, 24, 32, and 48 px.
- Control height: 40 px; compact row: 36 px; navigation item: 44 px.
- Radius: 6 px controls, 8 px panels, 12 px media previews. Pills are reserved for tags and status.
- Shadow: `0 12px 36px rgb(0 0 0 / 28%)` only for floating menus, dialogs, and media inspector. Panel depth comes from tonal layers and 1 px neutral borders.

## 3. Application shell

- The top title strip is 40 px and reserves native drag regions.
- Its right edge contains Minimize, Maximize/Restore, and Close controls with familiar desktop hover states.
- The current version sits immediately before the window controls. When a signed update is available, that quiet label becomes a compact **Update vX.Y.Z** action in the same space. Download progress replaces the action label until installation begins. A recoverable failure exposes **Retry update** and **Download manually** actions; a completed install that could not relaunch exposes **Restart to finish update**.
- Closing the main window hides it to the system tray or menu bar by default. The tray menu uses the application icon and exposes **Show LumaDesk** and **Quit LumaDesk**; a supported left click restores and focuses the window.
- The left rail is fixed on desktop and becomes a bottom navigation bar below 760 px.
- The main content column owns vertical scrolling. Navigation and inspector stay fixed within the viewport.
- At widths below 1,100 px, the inspector becomes a modal sheet.
- The library grid uses responsive columns with a minimum card width of 184 px. Media cards use a 4:3 preview stage and show controls only on selection or hover.
- The desktop manager uses one compact row per persisted overlay on wide screens and stacks preview, metadata, actions, and sliders on narrow screens. Hidden overlays remain listed so users can recover them.
- Browser preview omits native-only window controls because no native window exists.

## 4. Core primitives

- **Button:** primary blue fill, neutral secondary fill, quiet text action, and danger variant. Every button includes an icon only when the icon clarifies the action.
- **IconButton:** 40 px target, tooltip, accessible label, neutral surface states.
- **NavItem:** icon, label, optional count; selected state uses `surface-750` and `ink-100`.
- **MediaCard:** preview stage, name, format metadata, favorite action, active-instance count. Selection uses a tonal wash and check glyph.
- **SearchField:** leading search icon, clear action, keyboard shortcut hint.
- **External discovery action:** a neutral secondary button beside import opens GIF search in the system browser and reuses the current collection query.
- **Tag:** compact neutral pill; removable tags include a labeled close action.
- **InspectorSection:** heading, grouped controls, neutral separators.
- **Dialog:** focused title, concise copy, ordered actions, Escape and focus trapping.
- **Toast:** status icon and one sentence; errors may add a single recovery action.
- **EmptyState:** original LumaDesk mark, direct explanation, one primary action.
- **GlobalActionBar:** horizontally scrollable desktop-wide controls grouped by visibility, playback, interaction, and destructive actions.
- **UpdateAction:** a compact title-bar control with available, downloading, installing, failed, and restart-required states. It uses the brand tonal surface, a download or progress glyph, explicit text, and an accessible status label.
- **SceneManager:** compact save form, selectable scene list, selected-scene actions, and inline confirmation for deletion.
- **StateBadge:** short text plus status color for visible, hidden, playing, paused, and locked states.
- **GalleryCard:** animated preview, title, author, format, size, tags, license/source links, and one collection action. Installed state uses a check icon and explicit text.

## 5. Interaction and motion

- Hover and press transitions run for 120–160 ms. Sheets and dialogs run for 180–220 ms.
- Overlay edit chrome reveals over 140 ms and hides after 3,000 ms without interaction. Pointer presence or keyboard focus inside the toolbar suspends the idle timer.
- Animate only `transform`, `opacity`, and `filter`.
- Reduced-motion mode reveals overlay edit chrome with opacity only.
- Dragging an overlay has no easing; releasing it uses a subtle 120 ms settle only if clamped to a visible boundary.
- Import progress is determinate when byte progress exists and indeterminate otherwise.
- Update checks run without interrupting the workspace. Installation starts only after the user presses the title-bar update action; known download size produces integer progress, and an unknown size uses a short activity label. Failed operations return focus to a retry action and keep a direct-release fallback in the same control.
- Reduced-motion mode removes translation and scale while keeping short opacity changes.
- Gallery previews stay paused when reduced motion is requested. Each card exposes an explicit play action and restores the paused state when the system preference changes.
- Destructive operations always require explicit confirmation and return focus to a logical element afterward.

## 6. Overlay controls

- Every overlay accepts pointer input and can be dragged directly from the animation surface or toolbar grip.
- The toolbar and subtle boundary start hidden. Pressing the animation reveals both, interaction restarts the idle timer, and they hide after 3,000 ms of inactivity.
- The toolbar remains visible while its pointer or keyboard focus is inside the control surface.
- The toolbar exposes drag, play/pause, flip, background removal, duplicate, and close. Detailed size, opacity, and speed controls live in the main-window inspector.
- Click-through is available in both the overlay toolbar and the main-window row. Enabling it removes pointer focus from the native overlay; the manager and tray remain recovery surfaces.
- The **Remove background** effect uses a standard checkbox in the main-window inspector. Its supporting copy stays concise and explains that solid or near-solid backgrounds produce the best result.
- Effect processing never modifies the managed source file. A checked state is persisted per overlay and remains distinguishable through text and control state without relying on color.
- The **Active on Desktop** page shows total, visible, playing, and locked counts. Its global bar provides show, hide, play, pause, lock, unlock, and confirmed close actions.
- Each overlay row includes a connected-display selector. Changing it preserves the overlay's relative position and keeps the complete overlay inside the destination display.
- Hidden rows use a dashed neutral boundary and reduced preview saturation. Paused and click-through states add explicit text badges.

## 7. Desktop scenes

- The scene manager sits between global actions and overlay instances so layout-level work precedes per-instance tuning.
- Saving requires a visible label and a non-empty unique name. Supporting copy states that hidden and locked states are included.
- The selected scene exposes one blue **Apply scene** action, a neutral **Update snapshot** action, and secondary rename, duplicate, and delete actions.
- Applying a scene replaces the current layout. Scene selection alone never changes native windows.
- Rename stays inline. Delete requires a compact confirmation beside the initiating action.
- On narrow screens, save controls stack, the scene list moves above scene actions, and the manager keeps a single scroll owner in the workspace.

## 8. Community Gallery

- The Gallery header identifies the active catalog source, item count, refresh action, and direct URL import.
- Search spans title, description, author, and tags. Format and tag filters remain visible beside search on desktop and wrap below it on narrow screens.
- The responsive grid uses a 280 px minimum card width. Every card gives the animated preview the largest area and keeps attribution beside the collection action.
- Compact Gallery layouts hide the visual scrollbar while preserving touch, wheel, and keyboard scrolling so the content keeps the full viewport width.
- **Add to collection** becomes an inline busy state while downloading and a persistent **In collection** state after exact hash detection.
- Source and license actions open in the system browser. Attribution remains visible before import.
- Browser preview uses the bundled starter catalog and disables native collection actions with explanatory copy.
- Loading, catalog error, no-results, preview-unavailable, importing, duplicate, and imported states use direct English labels.
- Direct URL import uses a focused dialog with an HTTPS URL, optional display name, 100 MiB limit note, and one primary import action.

## 9. Creator Studio

- The Creator workspace uses a three-column desktop layout: a compact asset rail, a large live preview, and grouped editing controls. At narrower widths, controls move below the preview and the asset rail becomes a short horizontal region.
- Transform controls stay close to the preview toolbar. Numeric crop controls use the dimensions of one sprite cell when sprite playback is enabled.
- Brightness, contrast, and saturation use live range controls with visible numeric output. Rotation, flip, and reset have labeled tooltips and accessible names.
- Background choices use a four-option segmented control: **Original**, **Edge**, **AI fast**, and **AI quality**. Model requirements appear only when an AI mode is selected.
- Sprite controls show grid dimensions, frame count, frame duration, and playback mode. Animated source formats disable sprite setup because they already contain a frame sequence.
- Changes remain provisional until **Save changes** is pressed. A successful save updates the preview and active overlays; a failed save leaves the persisted recipe unchanged.

## 10. Settings

- Settings uses compact two-column sections for appearance, desktop behavior, overlay defaults, and storage. The local-model section spans the full content width.
- Every setting uses a native form control with a visible label and current value. Settings are written only through **Save settings**.
- Model cards distinguish built-in edge removal from downloadable Fast and Quality models. Download, installed, removing, and error states use text and icons in addition to color.
- Destructive model removal uses a direct labeled action. Generated masks for that model are removed with it.

## 11. Content language

- The interface and public project documentation use English.
- Prefer direct verbs. Interface examples include `Import animation`, `Show on desktop`, `Rename`, and `Remove from collection`.
- Explain file ownership precisely: deleting from LumaDesk removes its managed copy and never deletes the original source.
- Error messages state what failed and one useful next step.

## 12. Accessibility constraints and accepted debt

- Text contrast targets WCAG AA; focus indicators remain visible against every surface.
- All controls are keyboard reachable and expose names to accessibility APIs.
- Color never carries status alone.
- Transparent overlay interaction is verified per operating system because accessibility and input behavior differ across window managers.
- v0.1 supports an English UI. Localization infrastructure is deferred, but copy stays centralized so additional languages can be added without redesign.
