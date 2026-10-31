# Background removal

LumaDesk offers three non-destructive background-removal modes. Choose a mode in Creator Studio and save the recipe. Active previews and desktop overlays then use the same result.

## Edge removal

**Edge** is built in and starts immediately. It estimates the dominant color along a frame's edges, follows similar pixels connected to those edges, and feathers their opacity. Existing transparent pixels stay transparent. It works best for a solid or near-solid background that reaches the image boundary.

The compact eraser control on an active overlay remains available as a per-instance shortcut for edge removal. This instance setting can be toggled without replacing the Creator recipe.

## Local AI models

**AI fast** uses U²-NetP for smaller downloads and responsive processing. **AI quality** uses the larger U²-Net model for more detail. Install only the model you want from **Settings → Local AI models**.

Model downloads are optional and user initiated. LumaDesk verifies the exact byte length and SHA-256 digest before loading a model with the local ONNX runtime. A failed or incomplete download is never reported as installed. Removing a model also removes its generated masks.

## Masks and animated media

LumaDesk generates a grayscale mask for each requested frame batch and composites it with the frame in the renderer. GIF, APNG, animated WebP, and sprite sequences are processed frame by frame; still images need one mask. Generated masks are cached on disk and registered in SQLite so they can be removed when the media, model, cache, or sprite configuration changes.

Mask generation and compositing do not rewrite the imported file or its managed copy. Switching to **Original** returns to the untouched frame.

## Choosing a mode

- Use **Original** when the source already has useful transparency.
- Use **Edge** for flat colors, pixel art, and clean backdrops.
- Use **AI fast** for quick subject isolation with a small model.
- Use **AI quality** when edge detail matters more than processing time and download size.
