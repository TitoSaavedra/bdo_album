-- Fix presets where image_1 or image_2 are NULL or empty string.
-- Both cases should be 'not_found' so the pending-download query ignores them.
--
-- Preview affected rows before running:
--   SELECT id, image_1, image_2 FROM scrapper_presets
--   WHERE image_1 IS NULL OR image_1 = '' OR image_2 IS NULL OR image_2 = '';

UPDATE scrapper_presets
SET
  image_1    = CASE WHEN COALESCE(image_1, '') = '' THEN 'not_found' ELSE image_1 END,
  image_2    = CASE WHEN COALESCE(image_2, '') = '' THEN 'not_found' ELSE image_2 END,
  updated_at = NOW()
WHERE image_1 IS NULL OR image_1 = ''
   OR image_2 IS NULL OR image_2 = '';
