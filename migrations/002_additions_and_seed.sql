-- Add missing fields for Flutter app compatibility

ALTER TABLE users ADD COLUMN IF NOT EXISTS country VARCHAR(100) NOT NULL DEFAULT 'Nigeria';

ALTER TABLE cars ADD COLUMN IF NOT EXISTS seats INTEGER NOT NULL DEFAULT 5;

-- Favorites table
CREATE TABLE IF NOT EXISTS favorites (
    id UUID PRIMARY KEY,
    user_id UUID NOT NULL REFERENCES users(id),
    car_id UUID NOT NULL REFERENCES cars(id),
    created_at TIMESTAMP NOT NULL DEFAULT NOW(),
    UNIQUE(user_id, car_id)
);

CREATE INDEX IF NOT EXISTS idx_favorites_user ON favorites(user_id);

-- Notifications table
CREATE TABLE IF NOT EXISTS notifications (
    id UUID PRIMARY KEY,
    user_id UUID NOT NULL REFERENCES users(id),
    title VARCHAR(255) NOT NULL,
    message TEXT NOT NULL,
    notification_type VARCHAR(50) NOT NULL DEFAULT 'general',
    is_read BOOLEAN NOT NULL DEFAULT false,
    image_url TEXT,
    data JSONB,
    created_at TIMESTAMP NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_notifications_user ON notifications(user_id);

-- Email verification codes table
CREATE TABLE IF NOT EXISTS verification_codes (
    id UUID PRIMARY KEY,
    email VARCHAR(255) NOT NULL,
    code VARCHAR(6) NOT NULL,
    expires_at TIMESTAMP NOT NULL,
    used BOOLEAN NOT NULL DEFAULT false,
    created_at TIMESTAMP NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_verification_codes_email ON verification_codes(email);

-- ============================================================
-- MOCK / SEED DATA
-- ============================================================

-- Mock users (password is "password123" hashed with bcrypt)
INSERT INTO users (id, email, phone, password_hash, full_name, role, verification_status, wallet_balance, is_active, country, created_at, updated_at) VALUES
    ('a1111111-1111-1111-1111-111111111111', 'admin@cruise.ng', '+2348012345670', '$2b$12$LJ3m4ys3Lk0TSwHjlS5P0u8bVGQk7RRj6b6bCwFbNqGqQ3kZ1xFa6', 'Cruise Admin', 'admin', 'verified', 0.0, true, 'Nigeria', NOW(), NOW()),
    ('b2222222-2222-2222-2222-222222222222', 'emeka.obi@gmail.com', '+2348023456781', '$2b$12$LJ3m4ys3Lk0TSwHjlS5P0u8bVGQk7RRj6b6bCwFbNqGqQ3kZ1xFa6', 'Emeka Obi', 'host', 'verified', 45000.0, true, 'Nigeria', NOW(), NOW()),
    ('c3333333-3333-3333-3333-333333333333', 'fatima.yusuf@gmail.com', '+2348034567892', '$2b$12$LJ3m4ys3Lk0TSwHjlS5P0u8bVGQk7RRj6b6bCwFbNqGqQ3kZ1xFa6', 'Fatima Yusuf', 'host', 'verified', 120000.0, true, 'Nigeria', NOW(), NOW()),
    ('d4444444-4444-4444-4444-444444444444', 'chidi.nwosu@gmail.com', '+2348045678903', '$2b$12$LJ3m4ys3Lk0TSwHjlS5P0u8bVGQk7RRj6b6bCwFbNqGqQ3kZ1xFa6', 'Chidi Nwosu', 'renter', 'verified', 5000.0, true, 'Nigeria', NOW(), NOW()),
    ('e5555555-5555-5555-5555-555555555555', 'aisha.bello@gmail.com', '+2348056789014', '$2b$12$LJ3m4ys3Lk0TSwHjlS5P0u8bVGQk7RRj6b6bCwFbNqGqQ3kZ1xFa6', 'Aisha Bello', 'renter', 'verified', 0.0, true, 'Nigeria', NOW(), NOW()),
    ('f6666666-6666-6666-6666-666666666666', 'tunde.adeola@gmail.com', '+2348067890125', '$2b$12$LJ3m4ys3Lk0TSwHjlS5P0u8bVGQk7RRj6b6bCwFbNqGqQ3kZ1xFa6', 'Tunde Adeola', 'host', 'verified', 85000.0, true, 'Nigeria', NOW(), NOW())
ON CONFLICT (id) DO NOTHING;

-- Mock cars (all in Lagos)
INSERT INTO cars (id, host_id, make, model, year, color, plate_number, description, price_per_day, location, latitude, longitude, photos, features, status, seats, created_at, updated_at) VALUES
    ('11111111-aaaa-1111-aaaa-111111111111', 'b2222222-2222-2222-2222-222222222222', 'Toyota', 'Camry', 2022, 'Silver', 'LAG-234-XY', 'Clean and well-maintained Toyota Camry. Perfect for city drives and trips. AC works perfectly, smooth ride guaranteed.', 25000.0, 'Lekki, Lagos', 6.4698, 3.5852, ARRAY['https://images.unsplash.com/photo-1621007947382-bb3c3994e3fb?w=800&q=80'], ARRAY['AC', 'Bluetooth', 'Backup Camera', 'USB Charging'], 'active', 5, NOW(), NOW()),
    ('22222222-bbbb-2222-bbbb-222222222222', 'b2222222-2222-2222-2222-222222222222', 'Honda', 'Accord', 2021, 'Black', 'LAG-567-AB', 'Sleek Honda Accord in excellent condition. Leather interior, powerful engine, very fuel efficient.', 28000.0, 'Victoria Island, Lagos', 6.4281, 3.4219, ARRAY['https://images.unsplash.com/photo-1619682817481-e994891cd1f5?w=800&q=80'], ARRAY['AC', 'Leather Seats', 'Sunroof', 'Bluetooth'], 'active', 5, NOW(), NOW()),
    ('33333333-cccc-3333-cccc-333333333333', 'c3333333-3333-3333-3333-333333333333', 'Mercedes-Benz', 'GLE 350', 2023, 'White', 'LAG-890-CD', 'Luxury Mercedes-Benz GLE SUV. Premium sound system, panoramic roof, perfect for business or leisure.', 65000.0, 'Ikoyi, Lagos', 6.4474, 3.4345, ARRAY['https://images.unsplash.com/photo-1618843479313-40f8afb4b4d8?w=800&q=80'], ARRAY['AC', 'Premium Sound', 'Panoramic Roof', 'Leather Seats', 'Navigation'], 'active', 5, NOW(), NOW()),
    ('44444444-dddd-4444-dddd-444444444444', 'c3333333-3333-3333-3333-333333333333', 'Toyota', 'Highlander', 2022, 'Blue', 'LAG-123-EF', 'Spacious Toyota Highlander, great for family trips. Third row seating, excellent fuel economy for an SUV.', 35000.0, 'Ajah, Lagos', 6.4698, 3.5710, ARRAY['https://images.unsplash.com/photo-1533473359331-0135ef1b58bf?w=800&q=80'], ARRAY['AC', '3rd Row Seating', 'Bluetooth', 'Backup Camera'], 'active', 7, NOW(), NOW()),
    ('55555555-eeee-5555-eeee-555555555555', 'f6666666-6666-6666-6666-666666666666', 'Lexus', 'RX 350', 2023, 'Grey', 'LAG-456-GH', 'Premium Lexus RX 350 in pristine condition. Luxury comfort meets reliability. Mark Levinson sound system.', 55000.0, 'Ikeja, Lagos', 6.6018, 3.3515, ARRAY['https://images.unsplash.com/photo-1606611013016-969c19ba27bb?w=800&q=80'], ARRAY['AC', 'Premium Sound', 'Leather Seats', 'Heated Seats', 'Navigation'], 'active', 5, NOW(), NOW()),
    ('66666666-ffff-6666-ffff-666666666666', 'f6666666-6666-6666-6666-666666666666', 'Toyota', 'Corolla', 2021, 'White', 'LAG-789-IJ', 'Reliable Toyota Corolla. Budget-friendly, fuel efficient, perfect for daily commuting around Lagos.', 18000.0, 'Surulere, Lagos', 6.5059, 3.3509, ARRAY['https://images.unsplash.com/photo-1590362891991-f776e747a588?w=800&q=80'], ARRAY['AC', 'Bluetooth', 'USB Charging'], 'active', 5, NOW(), NOW()),
    ('77777777-aaaa-7777-aaaa-777777777777', 'b2222222-2222-2222-2222-222222222222', 'Range Rover', 'Sport', 2023, 'Black', 'LAG-321-KL', 'Head-turning Range Rover Sport. Commanding road presence, luxury interior, perfect for making a statement.', 85000.0, 'Banana Island, Lagos', 6.4560, 3.4290, ARRAY['https://images.unsplash.com/photo-1606016159991-dfe4f2746ad5?w=800&q=80'], ARRAY['AC', 'Premium Sound', 'Panoramic Roof', 'Leather Seats', 'Off-road Mode', '360 Camera'], 'active', 5, NOW(), NOW()),
    ('88888888-bbbb-8888-bbbb-888888888888', 'c3333333-3333-3333-3333-333333333333', 'Honda', 'CR-V', 2022, 'Red', 'LAG-654-MN', 'Practical Honda CR-V. Great cargo space, comfortable ride, perfect for weekend getaways.', 30000.0, 'Yaba, Lagos', 6.5095, 3.3711, ARRAY['https://images.unsplash.com/photo-1609521263047-f8f205293f24?w=800&q=80'], ARRAY['AC', 'Bluetooth', 'Backup Camera', 'Roof Rails'], 'active', 5, NOW(), NOW())
ON CONFLICT (id) DO NOTHING;

-- Mock bookings
INSERT INTO bookings (id, car_id, renter_id, host_id, start_date, end_date, total_days, price_per_day, subtotal, protection_fee, service_fee, total_amount, status, created_at, updated_at) VALUES
    ('aaaa1111-0000-0000-0000-000000000001', '11111111-aaaa-1111-aaaa-111111111111', 'd4444444-4444-4444-4444-444444444444', 'b2222222-2222-2222-2222-222222222222', '2026-03-15', '2026-03-18', 3, 25000.0, 75000.0, 4500.0, 7500.0, 87000.0, 'completed', NOW(), NOW()),
    ('aaaa1111-0000-0000-0000-000000000002', '33333333-cccc-3333-cccc-333333333333', 'e5555555-5555-5555-5555-555555555555', 'c3333333-3333-3333-3333-333333333333', '2026-03-20', '2026-03-22', 2, 65000.0, 130000.0, 7000.0, 13000.0, 150000.0, 'confirmed', NOW(), NOW()),
    ('aaaa1111-0000-0000-0000-000000000003', '66666666-ffff-6666-ffff-666666666666', 'd4444444-4444-4444-4444-444444444444', 'f6666666-6666-6666-6666-666666666666', '2026-03-25', '2026-03-30', 5, 18000.0, 90000.0, 7500.0, 9000.0, 106500.0, 'pending', NOW(), NOW())
ON CONFLICT (id) DO NOTHING;

-- Mock reviews
INSERT INTO reviews (id, booking_id, reviewer_id, reviewee_id, rating, comment, created_at) VALUES
    (gen_random_uuid(), 'aaaa1111-0000-0000-0000-000000000001', 'd4444444-4444-4444-4444-444444444444', 'b2222222-2222-2222-2222-222222222222', 5, 'Excellent car! Very clean and well maintained. Emeka was very responsive and friendly. Will definitely rent again.', NOW()),
    (gen_random_uuid(), 'aaaa1111-0000-0000-0000-000000000001', 'b2222222-2222-2222-2222-222222222222', 'd4444444-4444-4444-4444-444444444444', 4, 'Great renter, returned the car on time and in good condition.', NOW())
ON CONFLICT DO NOTHING;

-- Mock notifications
INSERT INTO notifications (id, user_id, title, message, notification_type, is_read, created_at) VALUES
    (gen_random_uuid(), 'd4444444-4444-4444-4444-444444444444', 'Booking Confirmed', 'Your booking for Toyota Camry has been confirmed. Enjoy your trip!', 'bookingSuccess', true, NOW() - INTERVAL '2 days'),
    (gen_random_uuid(), 'd4444444-4444-4444-4444-444444444444', 'Payment Successful', 'Payment of N87,000 was successful for your Toyota Camry booking.', 'payment', true, NOW() - INTERVAL '2 days'),
    (gen_random_uuid(), 'e5555555-5555-5555-5555-555555555555', 'Booking Approved', 'Your booking for Mercedes-Benz GLE 350 has been approved. Please complete payment.', 'bookingSuccess', false, NOW() - INTERVAL '1 day'),
    (gen_random_uuid(), 'b2222222-2222-2222-2222-222222222222', 'New Booking Request', 'You have a new booking request for your Toyota Camry from Chidi Nwosu.', 'bookingSuccess', true, NOW() - INTERVAL '3 days'),
    (gen_random_uuid(), 'd4444444-4444-4444-4444-444444444444', 'New Booking Pending', 'Your booking request for Toyota Corolla is pending host approval.', 'bookingSuccess', false, NOW())
ON CONFLICT DO NOTHING;

-- Mock favorites
INSERT INTO favorites (id, user_id, car_id, created_at) VALUES
    (gen_random_uuid(), 'd4444444-4444-4444-4444-444444444444', '33333333-cccc-3333-cccc-333333333333', NOW()),
    (gen_random_uuid(), 'd4444444-4444-4444-4444-444444444444', '77777777-aaaa-7777-aaaa-777777777777', NOW()),
    (gen_random_uuid(), 'e5555555-5555-5555-5555-555555555555', '11111111-aaaa-1111-aaaa-111111111111', NOW()),
    (gen_random_uuid(), 'e5555555-5555-5555-5555-555555555555', '55555555-eeee-5555-eeee-555555555555', NOW())
ON CONFLICT DO NOTHING;
