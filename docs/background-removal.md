# Background removal

Each active overlay has an independent **Remove background** checkbox on the **Active on Desktop** page. The setting is stored locally with that overlay, copied when the overlay is duplicated, and restored when LumaDesk starts again.

## How it works

LumaDesk processes the rendered frame locally. It estimates the dominant color along the frame edges, follows similar colors that remain connected to those edges, and feathers their opacity. Existing transparent pixels stay transparent. Every frame of a GIF, APNG, or animated WebP is processed when needed, while PNG and JPEG files are processed once.

The imported file and LumaDesk's managed copy are never rewritten. Disabling the checkbox immediately returns to the original frame. Processed frames stay only in the overlay's bounded in-memory playback buffer.

## Best results

- Use media with a solid or near-solid background that reaches the image edges.
- Leave clear color separation between the subject and background.
- Prefer source media with built-in alpha transparency when it is available.

Complex photographs, shadows close to the background color, subjects touching every edge, and detailed hair may need a dedicated image editor. AI subject segmentation is outside the current local effect.
