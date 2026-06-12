-- Character face custom uploads
CREATE TABLE IF NOT EXISTS album_character_faces (
    character_no TEXT        PRIMARY KEY,
    image_url    TEXT        NOT NULL,
    updated_at   TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_character_faces_updated
    ON album_character_faces(updated_at DESC);
