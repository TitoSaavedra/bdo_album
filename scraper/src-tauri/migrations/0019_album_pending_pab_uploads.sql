-- Staging queue for .pab files the user drags into album's "Presets to
-- download" modal: album only writes the raw filename + bytes here (no
-- validation, it never talks to R2) and fires `NOTIFY pab_upload_queued`.
-- The scraper's pab_uploads worker does all the validation (preset id from
-- `ID<digits>` in the filename, preset exists, no PAB yet), uploads via
-- `service::upload_pab` (which inserts scraper_preset_pabs and notifies
-- `preset_uploaded`), then purges the row. A row that fails validation or
-- upload gets `error` set and is excluded from future polling (same
-- dead-letter pattern as album_pending_preset_modifications).
CREATE TABLE IF NOT EXISTS album_pending_pab_uploads (
    id           BIGSERIAL   PRIMARY KEY,
    filename     TEXT        NOT NULL,
    pab_data     BYTEA       NOT NULL,
    requested_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    error        TEXT
);

CREATE INDEX IF NOT EXISTS idx_pending_pab_uploads_pending
    ON album_pending_pab_uploads(requested_at) WHERE error IS NULL;
