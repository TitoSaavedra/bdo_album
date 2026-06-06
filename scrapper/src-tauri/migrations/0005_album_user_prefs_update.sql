ALTER TABLE album_user_prefs RENAME COLUMN is_favorite TO is_wanted;
ALTER TABLE album_user_prefs DROP COLUMN IF EXISTS is_popular;
