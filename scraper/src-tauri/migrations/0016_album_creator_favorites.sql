CREATE TABLE IF NOT EXISTS album_creator_favorites (
    creator_nickname TEXT        PRIMARY KEY,
    updated_at        TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
