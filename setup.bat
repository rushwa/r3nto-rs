@echo off
REM RentoLink Rust Setup Script for Windows

echo RentoLink Rust Setup
echo ====================

REM Check if psql is available
where psql >nul 2>nul
if %errorlevel% neq 0 (
    echo PostgreSQL client (psql) not found in PATH.
    echo Please ensure PostgreSQL is installed and in your PATH.
    pause
    exit /b 1
)

REM Detect current user
for /f "tokens=*" %%a in ('whoami') do set DB_USER=%%a
set DB_USER=%DB_USER:\=%
echo Detected user: %DB_USER%

REM Create database
echo Creating database 'rento_rs'...
psql -U postgres -c "CREATE DATABASE rento_rs;" 2>nul
if %errorlevel% neq 0 (
    echo Database may already exist or postgres user password required.
)

REM Create user and grant privileges
psql -U postgres -c "CREATE USER %DB_USER%;" 2>nul
psql -U postgres -c "ALTER USER %DB_USER% WITH SUPERUSER;" 2>nul
psql -U postgres -c "GRANT ALL PRIVILEGES ON DATABASE rento_rs TO %DB_USER%;" 2>nul

REM Update .env
echo DATABASE_URL=postgres://%DB_USER%@localhost:5432/rento_rs > .env.tmp
type .env >> .env.tmp
del .env
rename .env.tmp .env

echo.
echo DATABASE_URL set to: postgres://%DB_USER%@localhost:5432/rento_rs
echo.
echo Running migrations...
sqlx migrate run

if %errorlevel% equ 0 (
    echo.
    echo Setup complete! Run: cargo run -p rento-api
) else (
    echo.
    echo Migration failed. Check PostgreSQL is running and credentials are correct.
)

pause
