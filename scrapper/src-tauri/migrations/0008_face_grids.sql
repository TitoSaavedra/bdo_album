CREATE TABLE IF NOT EXISTS album_face_grids (
    id            BIGSERIAL PRIMARY KEY,
    name          TEXT        NOT NULL,
    account_id    TEXT        NOT NULL,
    thumbnail_url TEXT,
    created_at    TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at    TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS album_face_grid_slots (
    id           BIGSERIAL PRIMARY KEY,
    grid_id      BIGINT      NOT NULL REFERENCES album_face_grids(id) ON DELETE CASCADE,
    character_no TEXT        NOT NULL,
    preset_id    BIGINT      REFERENCES scrapper_presets(id) ON DELETE SET NULL,
    slot_order   INT         NOT NULL,
    UNIQUE(grid_id, character_no)
);
