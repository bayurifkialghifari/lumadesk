ALTER TABLE media_items
ADD COLUMN asset_kind TEXT NOT NULL DEFAULT 'animation'
CHECK (asset_kind IN ('still', 'animation', 'sprite_sheet', 'video', 'audio'));

ALTER TABLE media_items
ADD COLUMN media_format TEXT NOT NULL DEFAULT 'png'
CHECK (media_format IN ('gif', 'apng', 'webp', 'png', 'jpeg', 'mp4', 'webm', 'mp3', 'wav', 'ogg'));

UPDATE media_items
SET media_format = media_kind,
    asset_kind = CASE WHEN frame_count > 1 THEN 'animation' ELSE 'still' END;

CREATE INDEX media_items_asset_kind_idx ON media_items(asset_kind);
CREATE INDEX media_items_media_format_idx ON media_items(media_format);

CREATE TABLE edit_recipes (
    media_id TEXT PRIMARY KEY NOT NULL REFERENCES media_items(id) ON DELETE CASCADE,
    crop_x REAL,
    crop_y REAL,
    crop_width REAL,
    crop_height REAL,
    rotation_degrees INTEGER NOT NULL DEFAULT 0 CHECK (rotation_degrees IN (0, 90, 180, 270)),
    flip_horizontal INTEGER NOT NULL DEFAULT 0 CHECK (flip_horizontal IN (0, 1)),
    background_removal TEXT NOT NULL DEFAULT 'none'
        CHECK (background_removal IN ('none', 'edge', 'ai_fast', 'ai_quality')),
    brightness REAL NOT NULL DEFAULT 1 CHECK (brightness >= 0 AND brightness <= 2),
    contrast REAL NOT NULL DEFAULT 1 CHECK (contrast >= 0 AND contrast <= 2),
    saturation REAL NOT NULL DEFAULT 1 CHECK (saturation >= 0 AND saturation <= 2),
    updated_at INTEGER NOT NULL,
    CHECK ((crop_x IS NULL AND crop_y IS NULL AND crop_width IS NULL AND crop_height IS NULL) OR
           (crop_x IS NOT NULL AND crop_y IS NOT NULL AND crop_width > 0 AND crop_height > 0))
);

INSERT INTO edit_recipes (media_id, updated_at)
SELECT id, updated_at FROM media_items;

CREATE TABLE sprite_sequences (
    media_id TEXT PRIMARY KEY NOT NULL REFERENCES media_items(id) ON DELETE CASCADE,
    columns INTEGER NOT NULL CHECK (columns > 0),
    rows INTEGER NOT NULL CHECK (rows > 0),
    frame_count INTEGER NOT NULL CHECK (frame_count > 0),
    frame_duration_ms INTEGER NOT NULL CHECK (frame_duration_ms > 0),
    playback_mode TEXT NOT NULL DEFAULT 'loop'
        CHECK (playback_mode IN ('loop', 'once', 'ping_pong')),
    updated_at INTEGER NOT NULL
);

CREATE TABLE media_variants (
    id TEXT PRIMARY KEY NOT NULL,
    media_id TEXT NOT NULL REFERENCES media_items(id) ON DELETE CASCADE,
    variant_kind TEXT NOT NULL
        CHECK (variant_kind IN ('preview', 'export', 'proxy', 'segmentation_mask')),
    relative_path TEXT UNIQUE NOT NULL,
    width INTEGER NOT NULL CHECK (width > 0),
    height INTEGER NOT NULL CHECK (height > 0),
    created_at INTEGER NOT NULL
);

CREATE INDEX media_variants_media_id_idx ON media_variants(media_id);
