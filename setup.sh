#!/bin/bash
# RentoLink Rust Setup Script

echo "🦀 RentoLink Rust Setup"
echo "========================"

# Check if PostgreSQL is running
if ! pg_isready -q; then
    echo "❌ PostgreSQL is not running. Please start it first:"
    echo "   sudo systemctl start postgresql"
    echo "   OR"
    echo "   docker run -d -p 5432:5432 -e POSTGRES_PASSWORD=postgres postgres:16"
    exit 1
fi

echo "✅ PostgreSQL is running"

# Detect current user
DB_USER=$(whoami)
echo "📋 Detected database user: $DB_USER"

# Create database if it doesn't exist
echo "📦 Creating database 'rento_rs' (if it doesn't exist)..."
psql -d postgres -c "CREATE DATABASE rento_rs;" 2>/dev/null || echo "   Database already exists or permission denied"

# Check if we can connect
echo "🔍 Testing database connection..."
if psql -d rento_rs -c "SELECT 1;" >/dev/null 2>&1; then
    echo "✅ Can connect to rento_rs as $DB_USER"

    # Set DATABASE_URL in .env
    sed -i "s|DATABASE_URL=.*|DATABASE_URL=postgres://$DB_USER@localhost:5432/rento_rs|" .env
    echo "📝 Updated .env with: postgres://$DB_USER@localhost:5432/rento_rs"

elif psql -U postgres -d postgres -c "SELECT 1;" >/dev/null 2>&1; then
    echo "✅ Can connect as postgres user"

    # Create user if needed
    psql -U postgres -d postgres -c "CREATE USER $DB_USER WITH PASSWORD '$DB_USER';" 2>/dev/null || true
    psql -U postgres -d postgres -c "ALTER USER $DB_USER WITH SUPERUSER;" 2>/dev/null || true
    psql -U postgres -d postgres -c "GRANT ALL PRIVILEGES ON DATABASE rento_rs TO $DB_USER;" 2>/dev/null || true
    psql -U postgres -d rento_rs -c "GRANT ALL ON SCHEMA public TO $DB_USER;" 2>/dev/null || true

    # Set DATABASE_URL in .env
    sed -i "s|DATABASE_URL=.*|DATABASE_URL=postgres://postgres:postgres@localhost:5432/rento_rs|" .env
    echo "📝 Updated .env with: postgres://postgres:postgres@localhost:5432/rento_rs"

else
    echo "❌ Cannot connect to PostgreSQL. Common fixes:"
    echo ""
    echo "1. If you have a password, set it in .env:"
    echo "   DATABASE_URL=postgres://aviet:YOUR_PASSWORD@localhost:5432/rento_rs"
    echo ""
    echo "2. If using default postgres user:"
    echo "   sudo -u postgres psql -c "CREATE DATABASE rento_rs;""
    echo "   sudo -u postgres psql -c "CREATE USER aviet WITH PASSWORD 'your_password';""
    echo "   sudo -u postgres psql -c "GRANT ALL PRIVILEGES ON DATABASE rento_rs TO aviet;""
    echo ""
    echo "3. Or use Docker:"
    echo "   docker run -d --name rento-postgres -p 5432:5432 -e POSTGRES_PASSWORD=postgres postgres:16"
    echo "   DATABASE_URL=postgres://postgres:postgres@localhost:5432/rento_rs"
    exit 1
fi

# Export for current session
export DATABASE_URL=$(grep DATABASE_URL .env | cut -d'=' -f2-)
echo ""
echo "🔧 DATABASE_URL set to: $DATABASE_URL"

# Run migrations
echo ""
echo "🚀 Running sqlx migrations..."
sqlx migrate run

if [ $? -eq 0 ]; then
    echo ""
    echo "✅ Setup complete! You can now run:"
    echo "   cargo run -p rento-api"
else
    echo ""
    echo "❌ Migration failed. Check your DATABASE_URL in .env"
fi
