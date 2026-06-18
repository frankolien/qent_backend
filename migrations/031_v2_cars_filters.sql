-- V2 §7.2, §9.2 — cars gets the filter columns.
--
-- The cars table already has: location, latitude, longitude,
-- features (TEXT[]), price_per_day, plate, photos. V2 adds the
-- columns the §7 filter sheet needs: USDC-denominated price,
-- ISO-2 country, city as a first-class column, transmission, fuel
-- type, seats, instant-book flag, min trip length, delivery flag.
--
-- Existing `location` stays as the free-text label shown in cards;
-- `city` is the structured value the filter query matches on.
-- Existing `latitude`/`longitude` stay; `geo_point` is added
-- separately as a PostGIS-friendly POINT for distance ranking
-- (Section 7.4 inverse distance signal). If PostGIS is not enabled
-- in this DB, distance ranking falls back to plain lat/lng math.

ALTER TABLE cars
    ADD COLUMN IF NOT EXISTS price_per_day_usdc NUMERIC(20,6) NOT NULL DEFAULT 0,
    ADD COLUMN IF NOT EXISTS country            TEXT REFERENCES countries(iso2),
    ADD COLUMN IF NOT EXISTS city               TEXT NOT NULL DEFAULT '',
    ADD COLUMN IF NOT EXISTS transmission       TEXT
        CHECK (transmission IS NULL OR transmission IN ('automatic','manual')),
    ADD COLUMN IF NOT EXISTS fuel_type          TEXT
        CHECK (fuel_type IS NULL OR fuel_type IN ('petrol','diesel','hybrid','electric')),
    ADD COLUMN IF NOT EXISTS seats              INT,
    ADD COLUMN IF NOT EXISTS instant_book       BOOLEAN NOT NULL DEFAULT false,
    ADD COLUMN IF NOT EXISTS min_trip_days      INT NOT NULL DEFAULT 1,
    ADD COLUMN IF NOT EXISTS allows_delivery    BOOLEAN NOT NULL DEFAULT false;

-- geo_point as a plain POINT (PostGIS is optional). If PostGIS is
-- installed later, a separate migration can add a GIST index using
-- the geometry type.
ALTER TABLE cars
    ADD COLUMN IF NOT EXISTS geo_point POINT;
