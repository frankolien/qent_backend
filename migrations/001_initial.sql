-- Cruise P2P Car Rental Platform - Initial Schema

-- Custom enum types
CREATE TYPE user_role AS ENUM ('renter', 'host', 'admin');
CREATE TYPE verification_status AS ENUM ('pending', 'verified', 'rejected');
CREATE TYPE car_status AS ENUM ('active', 'inactive', 'pendingapproval', 'rejected');
CREATE TYPE booking_status AS ENUM ('pending', 'approved', 'rejected', 'confirmed', 'active', 'completed', 'cancelled');
CREATE TYPE payment_status AS ENUM ('pending', 'success', 'failed', 'refunded');
CREATE TYPE transaction_type AS ENUM ('payment', 'payout', 'refund');
CREATE TYPE plan_tier AS ENUM ('basic', 'standard', 'premium');

-- Users table
CREATE TABLE users (
    id UUID PRIMARY KEY,
    email VARCHAR(255) UNIQUE NOT NULL,
    phone VARCHAR(20),
    password_hash TEXT NOT NULL,
    full_name VARCHAR(255) NOT NULL,
    role user_role NOT NULL DEFAULT 'renter',
    profile_photo_url TEXT,
    drivers_license_url TEXT,
    id_card_url TEXT,
    verification_status verification_status NOT NULL DEFAULT 'pending',
    wallet_balance DOUBLE PRECISION NOT NULL DEFAULT 0.0,
    is_active BOOLEAN NOT NULL DEFAULT true,
    created_at TIMESTAMP NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_users_email ON users(email);

-- Cars table
CREATE TABLE cars (
    id UUID PRIMARY KEY,
    host_id UUID NOT NULL REFERENCES users(id),
    make VARCHAR(100) NOT NULL,
    model VARCHAR(100) NOT NULL,
    year INTEGER NOT NULL,
    color VARCHAR(50) NOT NULL,
    plate_number VARCHAR(20) NOT NULL,
    description TEXT NOT NULL,
    price_per_day DOUBLE PRECISION NOT NULL,
    location VARCHAR(255) NOT NULL,
    latitude DOUBLE PRECISION,
    longitude DOUBLE PRECISION,
    photos TEXT[] NOT NULL DEFAULT '{}',
    features TEXT[] NOT NULL DEFAULT '{}',
    status car_status NOT NULL DEFAULT 'pendingapproval',
    available_from DATE,
    available_to DATE,
    created_at TIMESTAMP NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_cars_host ON cars(host_id);
CREATE INDEX idx_cars_status ON cars(status);
CREATE INDEX idx_cars_location ON cars(location);

-- Protection plans table
CREATE TABLE protection_plans (
    id UUID PRIMARY KEY,
    name VARCHAR(100) NOT NULL,
    tier plan_tier NOT NULL,
    description TEXT NOT NULL,
    daily_rate DOUBLE PRECISION NOT NULL,
    coverage_amount DOUBLE PRECISION NOT NULL,
    is_active BOOLEAN NOT NULL DEFAULT true,
    created_at TIMESTAMP NOT NULL DEFAULT NOW()
);

-- Seed protection plans
INSERT INTO protection_plans (id, name, tier, description, daily_rate, coverage_amount) VALUES
    (gen_random_uuid(), 'Basic Protection', 'basic', 'Covers minor scratches and dents up to N50,000. Renter pays N5,000 excess.', 1500.0, 50000.0),
    (gen_random_uuid(), 'Standard Protection', 'standard', 'Covers damage up to N200,000. Renter pays N15,000 excess. Includes roadside assistance.', 3500.0, 200000.0),
    (gen_random_uuid(), 'Premium Protection', 'premium', 'Full coverage up to N1,000,000. Zero excess. Includes roadside assistance and replacement vehicle.', 6000.0, 1000000.0);

-- Bookings table
CREATE TABLE bookings (
    id UUID PRIMARY KEY,
    car_id UUID NOT NULL REFERENCES cars(id),
    renter_id UUID NOT NULL REFERENCES users(id),
    host_id UUID NOT NULL REFERENCES users(id),
    start_date DATE NOT NULL,
    end_date DATE NOT NULL,
    total_days INTEGER NOT NULL,
    price_per_day DOUBLE PRECISION NOT NULL,
    subtotal DOUBLE PRECISION NOT NULL,
    protection_plan_id UUID REFERENCES protection_plans(id),
    protection_fee DOUBLE PRECISION NOT NULL DEFAULT 0.0,
    service_fee DOUBLE PRECISION NOT NULL DEFAULT 0.0,
    total_amount DOUBLE PRECISION NOT NULL,
    status booking_status NOT NULL DEFAULT 'pending',
    cancellation_reason TEXT,
    created_at TIMESTAMP NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_bookings_car ON bookings(car_id);
CREATE INDEX idx_bookings_renter ON bookings(renter_id);
CREATE INDEX idx_bookings_host ON bookings(host_id);
CREATE INDEX idx_bookings_status ON bookings(status);

-- Payments table
CREATE TABLE payments (
    id UUID PRIMARY KEY,
    booking_id UUID NOT NULL REFERENCES bookings(id),
    payer_id UUID NOT NULL REFERENCES users(id),
    amount DOUBLE PRECISION NOT NULL,
    currency VARCHAR(3) NOT NULL DEFAULT 'NGN',
    provider VARCHAR(50) NOT NULL,
    provider_reference TEXT,
    status payment_status NOT NULL DEFAULT 'pending',
    transaction_type transaction_type NOT NULL,
    created_at TIMESTAMP NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_payments_booking ON payments(booking_id);
CREATE INDEX idx_payments_reference ON payments(provider_reference);

-- Wallet transactions table
CREATE TABLE wallet_transactions (
    id UUID PRIMARY KEY,
    user_id UUID NOT NULL REFERENCES users(id),
    amount DOUBLE PRECISION NOT NULL,
    balance_after DOUBLE PRECISION NOT NULL,
    description TEXT NOT NULL,
    reference_id UUID,
    created_at TIMESTAMP NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_wallet_tx_user ON wallet_transactions(user_id);

-- Reviews table
CREATE TABLE reviews (
    id UUID PRIMARY KEY,
    booking_id UUID NOT NULL REFERENCES bookings(id),
    reviewer_id UUID NOT NULL REFERENCES users(id),
    reviewee_id UUID NOT NULL REFERENCES users(id),
    rating INTEGER NOT NULL CHECK (rating >= 1 AND rating <= 5),
    comment TEXT,
    created_at TIMESTAMP NOT NULL DEFAULT NOW(),
    UNIQUE(booking_id, reviewer_id)
);

CREATE INDEX idx_reviews_reviewee ON reviews(reviewee_id);
