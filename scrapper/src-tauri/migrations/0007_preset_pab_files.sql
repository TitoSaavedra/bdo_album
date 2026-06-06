CREATE TABLE IF NOT EXISTS scrapper_preset_pabs (
    id        BIGSERIAL   PRIMARY KEY,
    preset_id BIGINT      NOT NULL REFERENCES scrapper_presets(id) ON DELETE CASCADE,
    url       TEXT        NOT NULL,
    synced_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_preset_pabs_preset ON scrapper_preset_pabs(preset_id);

ALTER TABLE scrapper_presets DROP COLUMN IF EXISTS pab_url;
