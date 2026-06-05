-- ── Classes ──────────────────────────────────────────────────

CREATE TABLE IF NOT EXISTS scrapper_classes (
    id         INTEGER     PRIMARY KEY,
    id_pa      INTEGER     NOT NULL,
    display    TEXT        NOT NULL UNIQUE,
    icon_svg   TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- ── Presets ──────────────────────────────────────────────────

CREATE TABLE IF NOT EXISTS scrapper_presets (
    id             BIGINT      PRIMARY KEY,
    class_id       INTEGER     NOT NULL REFERENCES scrapper_classes(id),
    title          TEXT,
    user_nickname  TEXT,
    character_name TEXT,
    downloads      BIGINT      NOT NULL DEFAULT 0,
    views          BIGINT      NOT NULL DEFAULT 0,
    likes          BIGINT      NOT NULL DEFAULT 0,
    image_1        TEXT,
    image_2        TEXT,
    image_1_url    TEXT,
    image_2_url    TEXT,
    pab_url        TEXT,
    creation_at    BIGINT,
    customizing_id BIGINT,
    region         TEXT,
    score          BIGINT,
    synced_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at     TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_presets_class
    ON scrapper_presets(class_id, downloads DESC);

CREATE INDEX IF NOT EXISTS idx_presets_region
    ON scrapper_presets(region, class_id);

CREATE INDEX IF NOT EXISTS idx_presets_synced
    ON scrapper_presets(synced_at DESC);

-- ── Sessions ─────────────────────────────────────────────────

CREATE TABLE IF NOT EXISTS scrapper_sessions (
    id             BIGSERIAL   PRIMARY KEY,
    started_at     TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    finished_at    TIMESTAMPTZ,
    status         TEXT        NOT NULL DEFAULT 'running'
                   CHECK (status IN ('running', 'done', 'error', 'cancelled')),
    total_fetched  INTEGER     NOT NULL DEFAULT 0,
    total_images   INTEGER     NOT NULL DEFAULT 0,
    total_uploaded INTEGER     NOT NULL DEFAULT 0,
    errors         INTEGER     NOT NULL DEFAULT 0,
    elapsed_secs   INTEGER,
    cf_used        BOOLEAN     NOT NULL DEFAULT FALSE
);

CREATE INDEX IF NOT EXISTS idx_sessions_started
    ON scrapper_sessions(started_at DESC);

-- ── Class stats per session ───────────────────────────────────

CREATE TABLE IF NOT EXISTS scrapper_class_stats (
    id          BIGSERIAL   PRIMARY KEY,
    session_id  BIGINT      NOT NULL REFERENCES scrapper_sessions(id) ON DELETE CASCADE,
    class_id    INTEGER     NOT NULL REFERENCES scrapper_classes(id),
    fetched     INTEGER     NOT NULL DEFAULT 0,
    images_ok   INTEGER     NOT NULL DEFAULT 0,
    errors      INTEGER     NOT NULL DEFAULT 0,
    snapshot_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE (session_id, class_id)
);

CREATE INDEX IF NOT EXISTS idx_class_stats_session
    ON scrapper_class_stats(session_id, class_id);

-- ── Logs ─────────────────────────────────────────────────────

CREATE TABLE IF NOT EXISTS scrapper_logs (
    id         BIGSERIAL   PRIMARY KEY,
    session_id BIGINT      REFERENCES scrapper_sessions(id) ON DELETE SET NULL,
    ts         TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    tag        TEXT        NOT NULL,
    source     TEXT        NOT NULL,
    msg        TEXT        NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_logs_ts
    ON scrapper_logs(ts DESC);

CREATE INDEX IF NOT EXISTS idx_logs_session
    ON scrapper_logs(session_id, ts DESC);

-- ── Album user prefs ─────────────────────────────────────────

CREATE TABLE IF NOT EXISTS album_user_prefs (
    id           BIGSERIAL   PRIMARY KEY,
    preset_id    BIGINT      NOT NULL UNIQUE,
    is_popular   BOOLEAN     NOT NULL DEFAULT TRUE,
    is_favorite  BOOLEAN     NOT NULL DEFAULT FALSE,
    is_discarded BOOLEAN     NOT NULL DEFAULT FALSE,
    updated_at   TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
