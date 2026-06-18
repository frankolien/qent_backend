-- V2 §7.7, §9.3 — indexes for the §7 filter query.
--
-- The shape of the filter query, from §7.7:
--   WHERE country = $1 AND city = ANY($2) AND status = 'active'
--     AND price_per_day_usdc BETWEEN $3 AND $4
--     AND features @> $5
--     AND NOT EXISTS (SELECT 1 FROM bookings b WHERE car_id = c.id AND
--                     status IN ('paid','active') AND (pickup, return) OVERLAPS ...)
--
-- Each index below covers one of those predicates. All idempotent
-- (`IF NOT EXISTS`) per the V1 book §16 discipline.

CREATE INDEX IF NOT EXISTS cars_country_city_status_idx
    ON cars (country, city, status);

CREATE INDEX IF NOT EXISTS cars_price_usdc_idx
    ON cars (price_per_day_usdc);

CREATE INDEX IF NOT EXISTS cars_features_gin_idx
    ON cars USING GIN (features);

-- Booking overlap check; partial index avoids the full table for
-- the common case where we only care about live/active bookings.
CREATE INDEX IF NOT EXISTS bookings_car_dates_active_idx
    ON bookings (car_id, start_date, end_date)
    WHERE status IN ('paid','active','pending','confirmed','in_progress');

-- Wallet transactions per-user feed.
CREATE INDEX IF NOT EXISTS wallet_transactions_user_idx
    ON wallet_transactions (user_id, created_at DESC);
