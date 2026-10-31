# Sprite sheets

Creator Studio can turn a still image into an animated sequence. Enable **Sprite playback** for a PNG, JPEG, static WebP, or previously configured sprite sheet. Animated source files already contain frames and cannot be converted through this control.

## Grid and frame order

Set the number of columns and rows in the source image. LumaDesk divides the source into equal cells, then reads frames from left to right and top to bottom. **Frames** may be lower than the grid capacity when unused cells remain at the end.

Columns and rows are limited by the source dimensions and a maximum of 64 per axis. Frame count must fit the grid. Frame duration accepts 10 to 60,000 milliseconds. Invalid values are rejected before any Creator data is committed.

## Playback modes

- **Loop:** return to the first frame after the last frame.
- **Once:** stop on the final frame.
- **Ping-pong:** play forward, then backward without repeating an endpoint twice.

The configured sequence is used by Collection previews, Creator Studio, and desktop overlays. Overlay speed multiplies the saved frame duration.

## Editing and cache behavior

Crop values apply to one sprite cell. When grid dimensions change, LumaDesk regenerates decoded frame data and discards segmentation masks tied to the previous sequence. The complete source sheet remains unchanged.
