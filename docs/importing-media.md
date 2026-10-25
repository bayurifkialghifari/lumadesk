# Importing media

LumaDesk accepts GIF, APNG, animated or static WebP, PNG, and JPEG files. Video and audio are outside the initial release scope.

## Importing files

Select **Import animation**, choose one or more files, or drag files onto the Collection area. Progress is reported for each file so one invalid file does not block the others.

## File storage

LumaDesk copies imported media into its application data directory. The source file remains in its original location. The initial display name comes from the filename and can be changed after import.

Every file receives a SHA-256 content hash. If the same content already exists, LumaDesk selects the existing item instead of creating another copy. Corrupt, empty, and unsupported files are rejected before any database record is created.

## Privacy

Importing, metadata inspection, thumbnail generation, and frame decoding happen on the device. LumaDesk does not upload media to an external service.
