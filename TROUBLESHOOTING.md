# PostgreSQL Connection Troubleshooting

## Error: "password authentication failed for user 'aviet'"

This means sqlx is trying to connect with your system username but PostgreSQL
requires a password or doesn't recognize that user.

## Quick Fixes

### Option 1: No Password (Peer/Trust Auth)
If PostgreSQL is configured for peer/trust authentication:

```bash
# Check your pg_hba.conf
cat /etc/postgresql/16/main/pg_hba.conf | grep -v "^#" | grep -v "^$"

# If you see "peer" or "trust", try:
export DATABASE_URL="postgres://aviet@localhost:5432/rento_rs"
sqlx migrate run
```

### Option 2: With Password
```bash
# Set your actual PostgreSQL password
export DATABASE_URL="postgres://aviet:YOUR_PASSWORD@localhost:5432/rento_rs"
sqlx migrate run
```

### Option 3: Use Postgres Superuser
```bash
# Connect as postgres superuser
export DATABASE_URL="postgres://postgres:postgres@localhost:5432/rento_rs"
sqlx migrate run
```

### Option 4: Create Database & User First
```bash
# Switch to postgres user
sudo -u postgres psql

# In psql:
CREATE DATABASE rento_rs;
CREATE USER aviet WITH PASSWORD 'your_password';
GRANT ALL PRIVILEGES ON DATABASE rento_rs TO aviet;
ALTER USER aviet WITH SUPERUSER;
\q

# Then:
export DATABASE_URL="postgres://aviet:your_password@localhost:5432/rento_rs"
sqlx migrate run
```

### Option 5: Docker (Easiest)
```bash
# Start PostgreSQL in Docker
docker run -d --name rento-postgres \
  -e POSTGRES_PASSWORD=postgres \
  -e POSTGRES_DB=rento_rs \
  -p 5432:5432 \
  postgres:16

# Wait a few seconds, then:
export DATABASE_URL="postgres://postgres:postgres@localhost:5432/rento_rs"
sqlx migrate run
```

## Verify Connection

```bash
# Test with psql
psql "postgres://aviet@localhost:5432/rento_rs" -c "SELECT 1;"

# Or with pg_isready
pg_isready -d "postgres://aviet@localhost:5432/rento_rs"
```

## Permanent Fix

Add to your shell profile (~/.bashrc, ~/.zshrc, etc.):

```bash
export DATABASE_URL="postgres://aviet:your_password@localhost:5432/rento_rs"
```

Then reload:
```bash
source ~/.bashrc  # or ~/.zshrc
```

## sqlx prepare (for offline builds)

If you can't connect to a database during builds:

```bash
# With database connection:
export DATABASE_URL="postgres://..."
sqlx migrate run
cargo sqlx prepare --workspace

# Now builds work without DB connection:
SQLX_OFFLINE=true cargo build
```
