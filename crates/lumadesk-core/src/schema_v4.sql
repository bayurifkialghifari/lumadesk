ALTER TABLE overlay_instances
ADD COLUMN click_through INTEGER NOT NULL DEFAULT 0
CHECK (click_through IN (0, 1));

CREATE TABLE desktop_scenes (
    id TEXT PRIMARY KEY NOT NULL,
    name TEXT NOT NULL COLLATE NOCASE UNIQUE,
    created_at INTEGER NOT NULL,
    updated_at INTEGER NOT NULL
);

CREATE TABLE desktop_scene_items (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    scene_id TEXT NOT NULL REFERENCES desktop_scenes(id) ON DELETE CASCADE,
    media_id TEXT NOT NULL REFERENCES media_items(id) ON DELETE CASCADE,
    monitor_name TEXT,
    monitor_width REAL NOT NULL CHECK (monitor_width > 0),
    monitor_height REAL NOT NULL CHECK (monitor_height > 0),
    normalized_x REAL NOT NULL,
    normalized_y REAL NOT NULL,
    width REAL NOT NULL CHECK (width >= 32 AND width <= 4096),
    opacity REAL NOT NULL CHECK (opacity >= 0.1 AND opacity <= 1),
    speed REAL NOT NULL CHECK (speed >= 0.25 AND speed <= 4),
    flip_horizontal INTEGER NOT NULL DEFAULT 0 CHECK (flip_horizontal IN (0, 1)),
    remove_background INTEGER NOT NULL DEFAULT 0 CHECK (remove_background IN (0, 1)),
    is_playing INTEGER NOT NULL DEFAULT 1 CHECK (is_playing IN (0, 1)),
    click_through INTEGER NOT NULL DEFAULT 0 CHECK (click_through IN (0, 1)),
    z_order INTEGER NOT NULL DEFAULT 0
);

CREATE INDEX desktop_scene_items_scene_id_idx ON desktop_scene_items(scene_id);
CREATE INDEX desktop_scene_items_media_id_idx ON desktop_scene_items(media_id);
