-- Update face grid slots: replace preset_id with image_url
ALTER TABLE album_face_grid_slots DROP COLUMN preset_id;
ALTER TABLE album_face_grid_slots ADD COLUMN image_url TEXT NOT NULL DEFAULT '';
