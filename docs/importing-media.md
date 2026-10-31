# Importing media

LumaDesk accepts GIF, APNG, animated or static WebP, PNG, and JPEG files. A still image can be configured as a sprite sheet after import. Video and audio are not currently supported.

## Importing files

Select **Import animation**, choose one or more files, or drag files onto the Collection area. Progress is reported for each file so one invalid file does not block the others.

To find new media, enter an optional Collection search term and select **Discover GIFs**. Download a GIF from the page that opens, then import it through the same picker or drag-and-drop flow.

The **Community Gallery** provides a curated catalog inside LumaDesk. Search or filter the catalog, review the author and license, then select **Add to collection**. Catalog files are checked against their declared byte length and SHA-256 before import.

Select **Import URL** in the Gallery to import one supported media file directly from an HTTPS address. A custom name is optional. URL downloads are limited to 100 MiB and pass through the same media validation, managed storage, and duplicate detection used by local files.

## File storage

LumaDesk copies imported media into its application data directory. The source file remains in its original location. The initial display name comes from the filename and can be changed after import.

Every file receives a SHA-256 content hash. If the same content already exists, LumaDesk selects the existing item instead of creating another copy. Corrupt, empty, and unsupported files are rejected before any database record is created.

## Privacy

Importing, metadata inspection, thumbnail generation, and frame decoding happen on the device. LumaDesk does not upload media to an external service.
