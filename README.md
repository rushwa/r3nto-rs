# RentoLink - Rust + Dioxus Migration

This is the Rust reimplementation of the RentoLink Django project, using:
- **Axum** for the REST API backend
- **Dioxus** for the web frontend
- **sqlx** for database access with compile-time checked SQL
- **PostgreSQL** for the database
- **Redis** for sessions and task queues

## Project Structure

```
rento-rs/
├── Cargo.toml              # Workspace root
├── crates/
│   ├── core/               # Shared models, auth, errors, DB types
│   ├── api/                # Axum REST API server (replaces Django)
│   ├── web/                # Dioxus frontend (replaces React templates)
│   └── admin/              # Admin dashboard (replaces Django Admin)
├── migrations/             # SQLx database migrations
├── docker-compose.yml      # Development environment
└── .env.example            # Environment configuration template
```

## Migration Mapping

| Django Component | Rust Equivalent |
|-----------------|-----------------|
| Django Models | `sqlx` structs in `crates/core/src/models.rs` |
| Django ORM | `sqlx` with compile-time checked SQL |
| DRF ViewSets | Axum handlers in `crates/api/src/handlers/` |
| DRF Serializers | `serde` + `validator` on structs |
| Django Admin | Dioxus admin dashboard |
| Django Templates | Dioxus RSX components |
| Celery Tasks | `tokio` async tasks / `apalis` |
| Django Auth | Custom JWT + Argon2 |
| Django Signals | Database triggers + application events |
| django-filter | Custom query building in handlers |
| drf-yasg | `utoipa` for OpenAPI docs |

## Quick Start

### Prerequisites
- Rust 1.75+ (install via [rustup](https://rustup.rs))
- PostgreSQL 16+
- Redis 7+
- Docker & Docker Compose (optional)

### 1. Clone and setup
```bash
git clone <repo-url>
cd rento-rs
cp .env.example .env
# Edit .env with your configuration
```

### 2. Database setup
```bash
# Start PostgreSQL and Redis
docker-compose up -d db redis mailhog

# Run migrations
cargo sqlx migrate run
```

### 3. Run the API server
```bash
cargo run -p rento-api
# Server starts at http://localhost:8000
# API docs at http://localhost:8000/docs
```

### 4. Run the web frontend
```bash
cargo run -p rento-web
# Or for development with hot reload:
dioxus serve --package rento-web
```

### 5. Run everything with Docker
```bash
docker-compose up --build
```

## API Endpoints

### Auth
- `POST /auth/register` - Register new user
- `POST /auth/login` - Login
- `POST /auth/refresh` - Refresh token
- `POST /auth/verify-email` - Request email OTP
- `POST /auth/verify-phone` - Request phone OTP
- `POST /auth/activate` - Activate account
- `POST /auth/password-reset` - Request password reset
- `POST /auth/password-reset/confirm` - Confirm password reset

### Users
- `GET /users/me` - Get current user
- `PATCH /users/me` - Update current user
- `POST /users/me/complete-profile` - Complete profile
- `GET /users` - List users (admin/staff)
- `GET /users/:id` - Get user by ID
- `POST /users/:id/convert-role` - Convert user role
- `DELETE /users/:id` - Delete user
- `GET /users/stats` - User statistics

### Agents
- `GET /agents` - List agents
- `POST /agents` - Create agent (admin)
- `GET /agents/me` - Get my agent profile
- `GET /agents/me/commissions` - Get my commissions
- `GET /agents/me/property-owners` - Get my property owners
- `GET /agents/:id/commissions` - Get agent commissions (admin)
- `GET /agents/:id/property-owners` - Get agent property owners
- `GET /agents/stats` - Agent statistics
- `POST /agents/register-property-owner` - Register property owner

### Properties
- `GET /properties` - List properties
- `POST /properties` - Create property
- `GET /properties/my-properties` - Get my properties
- `GET /properties/:id` - Get property
- `PATCH /properties/:id` - Update property
- `DELETE /properties/:id` - Delete property
- `POST /properties/:id/activate` - Activate property
- `POST /properties/:id/deactivate` - Deactivate property
- `POST /properties/:id/add-unit` - Add unit
- `PATCH /properties/:id/images` - Add images

### Units
- `GET /property-units` - List units
- `GET /property-units/:id` - Get unit
- `PATCH /property-units/:id` - Update unit
- `DELETE /property-units/:id` - Delete unit
- `POST /property-units/:id/activate` - Activate unit
- `POST /property-units/:id/deactivate` - Deactivate unit

### Subscriptions
- `GET /subscription-plans` - List plans
- `GET /subscriptions` - List subscriptions
- `POST /subscriptions` - Create subscription
- `POST /subscriptions/:id/activate` - Activate subscription
- `POST /subscriptions/:id/cancel` - Cancel subscription
- `POST /subscriptions/:id/renew` - Renew subscription
- `POST /subscriptions/activate-free-trial` - Activate free trial
- `POST /subscriptions/upgrade` - Upgrade subscription
- `POST /subscriptions/downgrade` - Downgrade subscription

## Development Guide

### Adding a new endpoint

1. Define the model in `crates/core/src/models.rs`
2. Add the migration in `migrations/`
3. Create the handler in `crates/api/src/handlers/`
4. Register the route in `crates/api/src/main.rs`
5. Add the frontend page in `crates/web/src/pages/`
6. Add the route in `crates/web/src/lib.rs`

### Database changes

```bash
# After modifying migrations, prepare sqlx queries
cargo sqlx prepare
```

## Performance Comparison

Expected improvements over Django:
- **10x faster** request handling (async Rust)
- **5x lower** memory usage
- **Compile-time** SQL verification (no runtime errors)
- **Type-safe** API contracts between frontend and backend

## License

MIT
