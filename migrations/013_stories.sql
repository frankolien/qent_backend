-- Stories for QENT partners (hosts) to showcase their cars
CREATE TABLE stories (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    host_id UUID NOT NULL REFERENCES users(id),
    car_id UUID REFERENCES cars(id),
    image_url TEXT NOT NULL,
    caption TEXT DEFAULT '',
    created_at TIMESTAMP NOT NULL DEFAULT NOW(),
    expires_at TIMESTAMP NOT NULL DEFAULT (NOW() + INTERVAL '24 hours')
);

CREATE INDEX idx_stories_host_id ON stories(host_id);
CREATE INDEX idx_stories_expires_at ON stories(expires_at);
