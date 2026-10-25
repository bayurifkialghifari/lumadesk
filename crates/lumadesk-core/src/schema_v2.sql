ALTER TABLE overlay_instances
ADD COLUMN remove_background INTEGER NOT NULL DEFAULT 0
CHECK (remove_background IN (0, 1));
