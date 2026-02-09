

CREATE TABLE IF NOT EXISTS events (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organizer_id UUID NOT NULL,
    title VARCHAR(255) NOT NULL,
    description TEXT NOT NULL,
    location VARCHAR(255) NOT NULL,
    date_time TIMESTAMP NOT NULL,
    capacity INT NOT NULL DEFAULT 100,
    category VARCHAR(100) NOT NULL DEFAULT 'Ostalo',
    created_at TIMESTAMP NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP NOT NULL DEFAULT NOW()
);

-- index for search by category
CREATE INDEX IF NOT EXISTS idx_events_category ON events(category);
