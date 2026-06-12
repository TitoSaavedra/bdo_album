-- Truncate face grid tables
-- Run with: psql -U bdo -d bdo_album -f truncate_grids.sql

TRUNCATE TABLE album_face_grid_slots CASCADE;
TRUNCATE TABLE album_face_grids CASCADE;
