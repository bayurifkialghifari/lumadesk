# LumaDesk Design System

**Status:** implementation contract
**Updated:** 25 October 2026

## 0. Research log

- Product shell: dense desktop-library navigation, media shelves, restrained controls, and clear ownership of the center scroll region.
- README direction: original theme-aware banner, one-line promise, compact badges, authentic product capture, early download table, and links to deeper documentation.
- Spatial pattern: fixed 224 px navigation rail, flexible content canvas, and 320 px contextual inspector on wide screens.
- Accessibility review: selected states use tonal surfaces plus icon/text changes; keyboard focus uses an explicit blue focus ring; motion follows `prefers-reduced-motion`.

External product names and assets are research material only. LumaDesk ships its own identity, copy, icons, and layouts.

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
- The left rail is fixed on desktop and becomes a bottom navigation bar below 760 px.
- The main content column owns vertical scrolling. Navigation and inspector stay fixed within the viewport.
- At widths below 1,100 px, the inspector becomes a modal sheet.
- The library grid uses responsive columns with a minimum card width of 184 px. Media cards use a 4:3 preview stage and show controls only on selection or hover.
- The desktop manager uses one compact row per active overlay on wide screens and stacks preview, metadata, actions, and sliders on narrow screens.
- Browser preview keeps native-only controls visible but disabled and explains why they are unavailable.

## 4. Core primitives

- **Button:** primary blue fill, neutral secondary fill, quiet text action, and danger variant. Every button includes an icon only when the icon clarifies the action.
- **IconButton:** 40 px target, tooltip, accessible label, neutral surface states.
- **NavItem:** icon, label, optional count; selected state uses `surface-750` and `ink-100`.
- **MediaCard:** preview stage, name, format metadata, favorite action, active-instance count. Selection uses a tonal wash and check glyph.
- **SearchField:** leading search icon, clear action, keyboard shortcut hint.
- **Tag:** compact neutral pill; removable tags include a labeled close action.
- **InspectorSection:** heading, grouped controls, neutral separators.
- **Dialog:** focused title, concise copy, ordered actions, Escape and focus trapping.
- **Toast:** status icon and one sentence; errors may add a single recovery action.
- **EmptyState:** original LumaDesk mark, direct explanation, one primary action.

## 5. Interaction and motion

- Hover and press transitions run for 120–160 ms. Sheets and dialogs run for 180–220 ms.
- Animate only `transform`, `opacity`, and `filter`.
- Dragging an overlay has no easing; releasing it uses a subtle 120 ms settle only if clamped to a visible boundary.
- Import progress is determinate when byte progress exists and indeterminate otherwise.
- Reduced-motion mode removes translation and scale while keeping short opacity changes.
- Destructive operations always require explicit confirmation and return focus to a logical element afterward.

## 6. Overlay controls

- Display mode is the default and ignores pointer events.
- Edit mode adds a compact neutral toolbar near the selected overlay and a subtle bounding box.
- The toolbar exposes drag, play/pause, flip, duplicate, and close. Detailed size, opacity, and speed controls live in the main-window inspector.
- The **Active on Desktop** page owns global edit mode, live active and playing counts, pause-all, and confirmed close-all actions.
- Overlay windows never display product chrome during display mode.

## 7. Content language

- The current interface language is Indonesian; public project documentation is English.
- Prefer direct verbs. Current interface examples include `Impor animasi` (Import animation), `Tampilkan di desktop` (Show on desktop), `Ganti nama` (Rename), and `Hapus dari koleksi` (Remove from collection).
- Explain file ownership precisely: deleting from LumaDesk removes its managed copy and never deletes the original source.
- Error messages state what failed and one useful next step.

## 8. Accessibility constraints and accepted debt

- Text contrast targets WCAG AA; focus indicators remain visible against every surface.
- All controls are keyboard reachable and expose names to accessibility APIs.
- Color never carries status alone.
- Transparent overlay interaction is verified per operating system because accessibility and input behavior differ across window managers.
- v0.1 supports an Indonesian UI only. Localization infrastructure is deferred, but copy stays centralized so additional languages can be added without redesign.
