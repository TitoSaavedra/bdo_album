CREATE TABLE IF NOT EXISTS album_class_favorites (
    class_id   INTEGER     PRIMARY KEY REFERENCES scrapper_classes(id) ON DELETE CASCADE,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
