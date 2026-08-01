use std::io::{self, Write};
use uuid::Uuid;
use argon2::{
    password_hash::{rand_core::OsRng, PasswordHasher, SaltString},
    Argon2,
};

use crate::state::AppState;

pub async fn create_superuser(state: AppState) -> crate::errors::ApiResult<()> {
    println!("═══════════════════════════════════════════════════════════");
    println!("  Rento Admin — Create Superuser");
    println!("═══════════════════════════════════════════════════════════\n");

    let existing_count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM account_users WHERE is_superuser = true OR role = 'ADMIN'"
    )
        .fetch_one(&state.db.pool)
        .await
        .map_err(|e| crate::errors::ApiError::Internal(e.to_string()))?;

    if existing_count > 0 {
        println!("WARNING: A superuser already exists.");
        print!("Do you want to create another superuser? [y/N]: ");
        io::stdout().flush().unwrap();
        let mut confirm = String::new();
        io::stdin().read_line(&mut confirm).unwrap();
        if confirm.trim().to_lowercase() != "y" {
            println!("Aborted.");
            return Ok(());
        }
    }

    let email = prompt("Email address: ");
    if email.is_empty() || !email.contains('@') {
        println!("Error: Valid email address is required.");
        std::process::exit(1);
    }

    let existing: Option<(String,)> = sqlx::query_as("SELECT email FROM account_users WHERE email = $1")
        .bind(&email)
        .fetch_optional(&state.db.pool)
        .await
        .map_err(|e| crate::errors::ApiError::Internal(e.to_string()))?;

    if existing.is_some() {
        println!("Error: A user with that email already exists.");
        std::process::exit(1);
    }

    let name = prompt("Full name: ");
    if name.is_empty() {
        println!("Error: Name is required.");
        std::process::exit(1);
    }

    let password = prompt_password("Password: ");
    if password.len() < 8 {
        println!("Error: Password must be at least 8 characters.");
        std::process::exit(1);
    }

    let password_confirm = prompt_password("Password (again): ");
    if password != password_confirm {
        println!("Error: Passwords do not match.");
        std::process::exit(1);
    }

    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();
    let password_hash = argon2
        .hash_password(password.as_bytes(), &salt)
        .map_err(|e| crate::errors::ApiError::Internal(e.to_string()))?
        .to_string();

    let user_id = Uuid::new_v4();
    let parts: Vec<&str> = name.splitn(2, ' ').collect();
    let first_name = parts.get(0).unwrap_or(&"").to_string();
    let last_name = parts.get(1).unwrap_or(&"").to_string();

    sqlx::query(
        r#"
        INSERT INTO account_users
            (id, email, username, password_hash, first_name, last_name, role, is_active, is_staff, is_superuser, date_joined)
        VALUES ($1, $2, $3, $4, $5, $6, 'ADMIN', true, true, true, NOW())
        "#
    )
        .bind(user_id)
        .bind(&email)
        .bind(&email)
        .bind(&password_hash)
        .bind(&first_name)
        .bind(&last_name)
        .execute(&state.db.pool)
        .await
        .map_err(|e| crate::errors::ApiError::Internal(e.to_string()))?;

    println!("\n═══════════════════════════════════════════════════════════");
    println!("  Superuser created successfully!");
    println!("  ID:    {}", user_id);
    println!("  Email: {}", email);
    println!("  Name:  {}", name);
    println!("  Role:  ADMIN (superuser)");
    println!("═══════════════════════════════════════════════════════════");

    Ok(())
}

fn prompt(label: &str) -> String {
    print!("{}", label);
    io::stdout().flush().unwrap();
    let mut input = String::new();
    io::stdin().read_line(&mut input).unwrap();
    input.trim().to_string()
}

fn prompt_password(label: &str) -> String {
    print!("{}", label);
    io::stdout().flush().unwrap();
    let mut input = String::new();
    io::stdin().read_line(&mut input).unwrap();
    input.trim().to_string()
}