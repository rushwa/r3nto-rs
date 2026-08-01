# 1. Set up database
export DATABASE_URL="postgres://user:pass@localhost/rento"

# 2. Create superuser (mandatory first step)
cargo run -- manage.py createsuperuser

# 3. Start server (fails if step 2 skipped)
cargo run

# 4. Login via admin panel → get JWT
# 5. Superuser can grant admin privileges to other users via UI