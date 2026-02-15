# Skripta za inicijalizaciju baza podataka

set -e

echo "Kreiram baze podataka..."

psql -v ON_ERROR_STOP=1 --username "$POSTGRES_USER" <<-EOSQL
    CREATE DATABASE auth_db;
    CREATE DATABASE event_db;
    CREATE DATABASE registration_db;
EOSQL

echo "Pokrecem migracije za auth_db..."
psql -v ON_ERROR_STOP=1 --username "$POSTGRES_USER" --dbname auth_db <<-EOSQL
    CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

    CREATE TABLE IF NOT EXISTS users (
        id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
        email VARCHAR(255) UNIQUE NOT NULL,
        password_hash TEXT NOT NULL,
        role VARCHAR(50) NOT NULL DEFAULT 'User',
        created_at TIMESTAMP NOT NULL DEFAULT NOW()
    );
EOSQL

echo "Pokrecem migracije za event_db..."
psql -v ON_ERROR_STOP=1 --username "$POSTGRES_USER" --dbname event_db <<-EOSQL
    CREATE TABLE IF NOT EXISTS events (
        id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
        organizer_id UUID NOT NULL,
        title VARCHAR(255) NOT NULL,
        description TEXT NOT NULL,
        location VARCHAR(255) NOT NULL,
        date_time TIMESTAMP NOT NULL,
        capacity INT NOT NULL DEFAULT 100,
        category VARCHAR(100) NOT NULL DEFAULT 'Ostalo',
        image_url TEXT,
        created_at TIMESTAMP NOT NULL DEFAULT NOW(),
        updated_at TIMESTAMP NOT NULL DEFAULT NOW()
    );
    CREATE INDEX IF NOT EXISTS idx_events_category ON events(category);
EOSQL

echo "Pokrecem migracije za registration_db..."
psql -v ON_ERROR_STOP=1 --username "$POSTGRES_USER" --dbname registration_db <<-EOSQL
    CREATE TABLE IF NOT EXISTS registrations (
        id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
        event_id UUID NOT NULL,
        user_id UUID NOT NULL,
        ticket_code VARCHAR(50) UNIQUE NOT NULL,
        status VARCHAR(20) NOT NULL DEFAULT 'confirmed',
        created_at TIMESTAMP NOT NULL DEFAULT NOW(),
        UNIQUE(event_id, user_id)
    );
    CREATE INDEX IF NOT EXISTS idx_registrations_event ON registrations(event_id);
    CREATE INDEX IF NOT EXISTS idx_registrations_user ON registrations(user_id);
EOSQL

echo "Sve baze su spremne!"
