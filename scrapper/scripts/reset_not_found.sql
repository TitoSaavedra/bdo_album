-- Reset presets marked as not_found so the pending loop retries their image download.
-- Run with: psql $DATABASE_URL -f scripts/reset_not_found.sql

-- Preview how many will be affected
SELECT
    COUNT(*) FILTER (WHERE image_1 = 'not_found') AS not_found_image_1,
    COUNT(*) FILTER (WHERE image_2 = 'not_found') AS not_found_image_2,
    COUNT(*) FILTER (WHERE image_1 = 'not_found' OR image_2 = 'not_found') AS total_affected
FROM scrapper_presets;

-- Reset: clear not_found flags so pending loop picks them up again
UPDATE scrapper_presets
SET image_1 = NULL
WHERE image_1 = 'not_found';

UPDATE scrapper_presets
SET image_2 = NULL
WHERE image_2 = 'not_found';

-- Confirm result
SELECT
    COUNT(*) FILTER (WHERE image_1 IS NULL AND image_1_url IS NULL) AS pending_image_1,
    COUNT(*) FILTER (WHERE image_2 IS NULL AND image_2_url IS NULL) AS pending_image_2
FROM scrapper_presets;
