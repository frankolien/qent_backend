-- Add views tracking to cars
ALTER TABLE cars ADD COLUMN IF NOT EXISTS views_count INTEGER NOT NULL DEFAULT 0;

-- Host stats: track total earnings from completed bookings
-- (bookings table already has total_price, we just query it)

-- Index for faster host stats queries
CREATE INDEX IF NOT EXISTS idx_bookings_car_id ON bookings(car_id);
CREATE INDEX IF NOT EXISTS idx_cars_host_id ON cars(host_id);
