-- The user can also attach the .pab file currently sitting in their local
-- BDO "Customization" export folder (see album's export_to_bdo) alongside
-- the 1-2 images — same staging/upload pipeline, just one more optional
-- payload. Nullable everywhere: a modification with only images is still
-- valid (the folder may be empty if nothing was ever exported).
ALTER TABLE album_pending_preset_modifications
    ADD COLUMN IF NOT EXISTS pab_data BYTEA;

ALTER TABLE album_preset_modifications
    ADD COLUMN IF NOT EXISTS pab_url TEXT;

-- Both tables are brand new (shipped this same release, 0017) — anything in
-- them predates the pab pipeline and the modificacion_N naming scheme, so
-- it's pre-release test data, not real user data. Clearing rather than
-- migrating it in place (R2 objects from that testing are cleaned up
-- separately, out of band — a Postgres migration has no way to reach R2).
DELETE FROM album_pending_preset_modifications;
DELETE FROM album_preset_modifications;
