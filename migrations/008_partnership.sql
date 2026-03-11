CREATE TABLE IF NOT EXISTS partner_applications (
    id UUID PRIMARY KEY,
    user_id UUID NOT NULL REFERENCES users(id),
    full_name TEXT NOT NULL,
    email TEXT NOT NULL,
    phone TEXT NOT NULL,
    drivers_license TEXT NOT NULL,
    car_make TEXT NOT NULL,
    car_model TEXT NOT NULL,
    car_year INTEGER NOT NULL,
    car_color TEXT NOT NULL,
    car_plate_number TEXT NOT NULL,
    car_photos TEXT[] NOT NULL DEFAULT '{}',
    car_description TEXT NOT NULL DEFAULT '',
    fuel_type TEXT NOT NULL DEFAULT 'petrol',
    status TEXT NOT NULL DEFAULT 'pending',
    created_at TIMESTAMP NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP NOT NULL DEFAULT NOW()
);
