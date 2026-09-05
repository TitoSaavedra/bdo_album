-- Finished, R2-backed modifications the user has personally uploaded for a
-- preset — written by the scraper's preset_modifications worker, read by
-- album. One preset can have N modifications.
CREATE TABLE IF NOT EXISTS album_preset_modifications (
    id          BIGSERIAL   PRIMARY KEY,
    preset_id   BIGINT      NOT NULL REFERENCES scraper_presets(id) ON DELETE CASCADE,
    image_1_url TEXT        NOT NULL,
    image_2_url TEXT,
    created_at  TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_preset_modifications_preset
    ON album_preset_modifications(preset_id);

-- Staging queue: album writes the raw pasted image bytes here (it never talks
-- to R2 directly); the scraper's preset_modifications worker drains this
-- table every 60s, uploads to R2, inserts the finished row above, then
-- purges this one. A row that fails upload gets `error` set and is excluded
-- from future polling instead of retrying forever (same dead-letter pattern
-- as album_user_prefs.auto_download_error).
CREATE TABLE IF NOT EXISTS album_pending_preset_modifications (
    id           BIGSERIAL   PRIMARY KEY,
    preset_id    BIGINT      NOT NULL REFERENCES scraper_presets(id) ON DELETE CASCADE,
    image_1_data BYTEA       NOT NULL,
    image_2_data BYTEA,
    requested_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    error        TEXT
);

CREATE INDEX IF NOT EXISTS idx_pending_preset_modifications_pending
    ON album_pending_preset_modifications(requested_at) WHERE error IS NULL;
