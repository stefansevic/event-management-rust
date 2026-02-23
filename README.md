# Event Management System

A microservices-based web application for managing events (conferences, workshops, concerts, meetups). Users can register, sign in, browse events, register for events, and manage tickets. Admins create and delete events and control capacity.

## Features

- **Authentication & authorization** — Register, login, JWT-based sessions, roles: User, Admin
- **Events** — Create, update, delete events; optional image upload (stored as base64); category and search filters; past dates rejected
- **Registrations** — Sign up for events, cancel registration; capacity checks; unique ticket codes
- **Tickets & QR codes** — Download ticket info and QR code per registration (Python QR service)
- **Admin** — Seeded admin account; delete events; when an event is deleted, all its registrations are auto-cancelled and shown as “Event removed” in My Registrations

## Architecture

```
                    ┌─────────────┐
                    │   Frontend  │  (port 8080)
                    │   (Nginx)   │
                    └──────┬──────┘
                           │
                    ┌──────▼──────┐
                    │ API Gateway │  (port 3000)
                    └──────┬──────┘
                           │
         ┌─────────────────┼─────────────────┐
         │                 │                 │
    ┌────▼────┐      ┌─────▼────┐      ┌─────▼───────┐
    │  Auth   │      │   Event  │      │ Registration│
    │ (3001)  │      │  (3003)  │      │   (3004)    │
    └────┬────┘      └────┬─────┘      └──────┬──────┘
         │                │                   │
         │                │           ┌───────▼──────┐
         │                │           │  QR Service  │
         │                │           │   (3005)     │
         │                │           └──────────────┘
         │                │
    ┌────▼────────────────▼───────────────────────────┐
    │              PostgreSQL (5432)                  │
    │  auth_db | event_db | registration_db           │
    └─────────────────────────────────────────────────┘
```

- **Auth Service** — Registration, login, JWT issue/validation, roles. Seeds admin `saske@admin.com` / `saske1` if missing.
- **Event Service** — Create/delete events; optional `image_url` (base64); on delete, calls Registration Service to cancel all registrations for that event.
- **Registration Service** — Registrations, capacity checks, ticket codes; calls Event Service for event data and QR Service for QR images.
- **QR Service** — Python/Flask; generates QR code images for ticket codes.
- **API Gateway** — Single entry point; forwards requests to backend services; 5 MB body limit for large payloads (e.g. event images).

Each service (except QR) has its own PostgreSQL database. Inter-service calls use internal Docker hostnames (e.g. `http://event-service:3003`).

## Tech Stack

| Layer        | Technology                          |
|-------------|--------------------------------------|
| Backend     | Rust (Axum), Tokio, SQLx, Serde, JWT |
| QR service  | Python 3, Flask, qrcode              |
| Databases   | PostgreSQL 16                        |
| Frontend    | HTML, CSS, JavaScript (vanilla)      |
| Deployment  | Docker, Docker Compose               |

## Prerequisites

- [Docker](https://docs.docker.com/get-docker/) and Docker Compose
- (Optional) Rust toolchain and PostgreSQL if you run services locally

## Quick Start (Docker)

1. **Clone and enter the project**
   ```bash
   cd ntp-event-management-system
   ```

2. **Start all services**
   ```bash
   docker-compose up --build
   ```

3. **Open the app**
   - Frontend: **http://localhost:8080**
   - API (via gateway): **http://localhost:3000/api**

4. **Default admin** (created on first run if missing)
   - Email: `saske@admin.com`
   - Password: `saske1`

To apply DB schema changes (e.g. new columns), remove volumes and start again:

```bash
docker-compose down -v
docker-compose up --build
```

## Environment Variables

Copy `.env.example` to `.env` and adjust if needed. Main variables:

| Variable | Description |
|----------|-------------|
| `*_DATABASE_URL` | PostgreSQL connection strings per service |
| `JWT_SECRET` | Secret used to sign/verify JWTs (same across services) |
| `EVENT_SERVICE_URL`, `REGISTRATION_SERVICE_URL`, etc. | Used by gateway and inter-service calls |

Docker Compose sets these for the containers; override in `.env` or `docker-compose.yml` for your environment.

## API Overview (via Gateway)

Base URL: `http://localhost:3000/api`

| Method | Path | Description |
|--------|------|-------------|
| POST   | `/auth/register` | Register (email, password) |
| POST   | `/auth/login`    | Login; returns JWT |
| GET    | `/auth/me`       | Current user (requires JWT) |
| GET/POST | `/events`      | List events (query: category, search) / Create event (JWT, Admin) |
| GET/PUT/DELETE | `/events/:id` | Get / Update / Delete event |
| POST   | `/registrations` | Register for event (body: `event_id`) |
| GET    | `/registrations/my` | My registrations |
| DELETE | `/registrations/:id` | Cancel registration |
| GET    | `/registrations/:id/qr` | QR code image |

All protected routes expect header: `Authorization: Bearer <token>`.

## Project Structure

```
ntp-event-management-system/
├── api-gateway/           # Rust; routes and proxy to backend
├── auth-service/          # Rust; register, login, JWT
├── event-service/         # Rust; events + image_url
├── registration-service/  # Rust; registrations, tickets
├── qr-service/            # Python; QR image generation
├── shared/                # Rust lib; JWT helpers, ApiResponse, AppError
├── frontend/              # Static site (HTML/CSS/JS)
├── scripts/
│   └── init-db.sh         # PostgreSQL init (all DBs + tables)
├── docker-compose.yml
├── Dockerfile             # Multi-service Rust build
├── Cargo.toml             # Workspace root
└── .env.example
```
