-- The 0012 rename migration renamed the table but left this CHECK constraint
-- under its old auto-generated name, and it never allowed 'interrupted' even
-- though session_repo::recover_interrupted() has always written that value —
-- every startup recovery UPDATE has been silently failing.
ALTER TABLE scraper_sessions
    DROP CONSTRAINT scrapper_sessions_status_check;

ALTER TABLE scraper_sessions
    ADD CONSTRAINT scraper_sessions_status_check
    CHECK (status IN ('running', 'done', 'error', 'cancelled', 'interrupted'));
