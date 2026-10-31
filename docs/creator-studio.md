# Creator Studio

Creator Studio prepares a Collection item for previews and desktop overlays without modifying the imported file. Select an asset from the left rail, adjust its recipe, inspect the live preview, and choose **Save changes**.

## Transform and color

- Crop with explicit X, Y, width, and height values.
- Rotate left or right in 90-degree steps.
- Flip the recipe horizontally.
- Adjust brightness, contrast, and saturation from 0% to 200%.
- Reset provisional recipe edits to their defaults before saving.

For sprite sheets, crop coordinates are measured inside one decoded frame cell. Changing the grid clamps an existing crop to the new frame dimensions.

## Background modes

- **Original:** keep the frame unchanged.
- **Edge:** remove a connected solid or near-solid edge color immediately.
- **AI fast:** use the optional smaller local model.
- **AI quality:** use the optional larger local model.

Install and remove AI models from Settings. Selecting an unavailable AI mode keeps the recipe choice, but processing requires the matching model.

## Saving and validation

Recipe and sprite changes are validated together. **Save changes** writes both in one SQLite transaction, so an invalid frame grid, duration, count, or crop cannot save only half of the edit. A successful save refreshes the Collection and active overlays.

The recipe stores instructions rather than rendered output. Source media remains available for reset, future edits, and different per-instance overlay settings.
