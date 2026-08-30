ALTER TABLE album_user_prefs
    ADD COLUMN IF NOT EXISTS auto_download_requested_at TIMESTAMPTZ,
    ADD COLUMN IF NOT EXISTS auto_download_error TEXT;

CREATE INDEX IF NOT EXISTS idx_album_user_prefs_auto_download
    ON album_user_prefs (auto_download_requested_at)
    WHERE auto_download_requested_at IS NOT NULL;
