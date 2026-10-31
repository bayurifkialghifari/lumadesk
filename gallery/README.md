# LumaDesk Gallery catalog

The Community Gallery is a versioned static catalog. It does not require an application server: LumaDesk reads catalog.v1.json, verifies every asset by byte size and SHA-256, and imports a managed local copy.

## Add an item

1. Add an original or redistributable GIF, APNG, animated WebP, PNG, or JPEG under src/assets/gallery.
2. Add one entry to catalog.v1.json with a stable lowercase ID, attribution, tags, HTTPS URLs, exact byte size, and SHA-256.
3. Increase the catalog revision.
4. Run pnpm validate:gallery.

Preview and download files may be the same asset. Keep previews compact because the app stores verified copies in its local gallery cache. Catalog changes are reviewed like source code and ship through the normal CI pipeline.
