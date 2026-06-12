-- ── Rename tables ────────────────────────────────────────────
ALTER TABLE scrapper_classes      RENAME TO scraper_classes;
ALTER TABLE scrapper_presets      RENAME TO scraper_presets;
ALTER TABLE scrapper_sessions     RENAME TO scraper_sessions;
ALTER TABLE scrapper_class_stats  RENAME TO scraper_class_stats;
ALTER TABLE scrapper_logs         RENAME TO scraper_logs;
ALTER TABLE scrapper_preset_pabs  RENAME TO scraper_preset_pabs;

-- ── Rename FK constraints (auto-generated names by Postgres) ─
ALTER TABLE scraper_presets
    RENAME CONSTRAINT scrapper_presets_class_id_fkey
    TO scraper_presets_class_id_fkey;

ALTER TABLE scraper_class_stats
    RENAME CONSTRAINT scrapper_class_stats_session_id_fkey
    TO scraper_class_stats_session_id_fkey;

ALTER TABLE scraper_class_stats
    RENAME CONSTRAINT scrapper_class_stats_class_id_fkey
    TO scraper_class_stats_class_id_fkey;

ALTER TABLE scraper_logs
    RENAME CONSTRAINT scrapper_logs_session_id_fkey
    TO scraper_logs_session_id_fkey;

ALTER TABLE scraper_preset_pabs
    RENAME CONSTRAINT scrapper_preset_pabs_preset_id_fkey
    TO scraper_preset_pabs_preset_id_fkey;

-- ── Rename sequences (BIGSERIAL columns) ─────────────────────
ALTER SEQUENCE scrapper_sessions_id_seq     RENAME TO scraper_sessions_id_seq;
ALTER SEQUENCE scrapper_class_stats_id_seq  RENAME TO scraper_class_stats_id_seq;
ALTER SEQUENCE scrapper_logs_id_seq         RENAME TO scraper_logs_id_seq;
ALTER SEQUENCE scrapper_preset_pabs_id_seq  RENAME TO scraper_preset_pabs_id_seq;
