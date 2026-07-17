-- V2 §7 — backfill the V2 columns on existing V1 cars so they're
-- visible to /api/v2/cars/search and the country-aware home view.
--
-- All V1 cars were Nigeria-only, so country defaults to 'NG'.
-- City is parsed from the legacy free-text `location` column
-- (e.g. "Lekki, Lagos" → "Lagos"). price_per_day_usdc is a rough
-- conversion at 1500 NGN/USD — close enough for dev demo data; real
-- listings created post-V2 will set this directly.

UPDATE cars
SET country = 'NG'
WHERE country IS NULL OR country = '';

UPDATE cars
SET city = TRIM(
    (string_to_array(location, ','))[
        array_length(string_to_array(location, ','), 1)
    ]
)
WHERE (city IS NULL OR city = '') AND location IS NOT NULL AND location <> '';

UPDATE cars
SET price_per_day_usdc = ROUND((price_per_day / 1500.0)::numeric, 2)
WHERE price_per_day_usdc = 0 AND price_per_day IS NOT NULL AND price_per_day > 0;
