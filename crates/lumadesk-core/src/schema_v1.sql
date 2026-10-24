CREATE TABLE media_items (
    id TEXT PRIMARY KEY NOT NULL,
    content_hash TEXT UNIQUE NOT NULL,
    display_name TEXT NOT NULL,
    managed_path TEXT UNIQUE NOT NULL,
    media_kind TEXT NOT NULL,
    width INTEGER NOT NULL CHECK (width > 0),
    height INTEGER NOT NULL CHECK (height > 0),
    frame_count INTEGER NOT NULL CHECK (frame_count > 0),
    duration_ms INTEGER NOT NULL CHECK (duration_ms >= 0),
    is_favorite INTEGER NOT NULL DEFAULT 0 CHECK (is_favorite IN (0, 1)),
    created_at INTEGER NOT NULL,
    updated_at INTEGER NOT NULL
);

CREATE TABLE tags (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT UNIQUE NOT NULL COLLATE NOCASE
);

CREATE TABLE media_tags (
    media_id TEXT NOT NULL REFERENCES media_items(id) ON DELETE CASCADE,
    tag_id INTEGER NOT NULL REFERENCES tags(id) ON DELETE CASCADE,
    PRIMARY KEY (media_id, tag_id)
);

CREATE TABLE overlay_instances (
    id TEXT PRIMARY KEY NOT NULL,
    media_id TEXT NOT NULL REFERENCES media_items(id) ON DELETE CASCADE,
    monitor_id TEXT,
    x REAL NOT NULL,
    y REAL NOT NULL,
    width REAL NOT NULL CHECK (width > 0),
    opacity REAL NOT NULL CHECK (opacity >= 0 AND opacity <= 1),
    speed REAL NOT NULL CHECK (speed >= 0.25 AND speed <= 4),
    flip_horizontal INTEGER NOT NULL DEFAULT 0 CHECK (flip_horizontal IN (0, 1)),
    is_playing INTEGER NOT NULL DEFAULT 1 CHECK (is_playing IN (0, 1)),
    z_order INTEGER NOT NULL DEFAULT 0,
    is_active INTEGER NOT NULL DEFAULT 1 CHECK (is_active IN (0, 1)),
    created_at INTEGER NOT NULL,
    updated_at INTEGER NOT NULL
);

CREATE TABLE settings (
    key TEXT PRIMARY KEY NOT NULL,
    value TEXT NOT NULL
);

CREATE INDEX media_items_created_at_idx ON media_items(created_at DESC);
CREATE INDEX media_tags_tag_id_idx ON media_tags(tag_id);
CREATE INDEX overlay_instances_media_id_idx ON overlay_instances(media_id);
