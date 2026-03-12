use sqlx::SqlitePool;
use chrono::NaiveDateTime;

use shared::models::user::{User, UserRole};
use shared::dto::user_dto::{CreateUserRequest, UpdateUserRequest};

pub async fn create_user(pool: &SqlitePool, req: &CreateUserRequest) -> Result<User, sqlx::Error> {
    let id = uuid::Uuid::new_v4().to_string();
    let role = req.role.as_deref().unwrap_or("user");
    let now = chrono::Utc::now().naive_utc();

    sqlx::query(
        "INSERT INTO users (id, username, email, role, created_at, updated_at) VALUES (?, ?, ?, ?, ?, ?)"
    )
    .bind(&id)
    .bind(&req.username)
    .bind(&req.email)
    .bind(role)
    .bind(now)
    .bind(now)
    .execute(pool)
    .await?;

    Ok(User {
        id,
        username: req.username.clone(),
        email: req.email.clone(),
        role: role.parse().unwrap_or(UserRole::User),
        created_at: now,
        updated_at: now,
    })
}

pub async fn get_user_by_id(pool: &SqlitePool, id: &str) -> Result<Option<User>, sqlx::Error> {
    let row = sqlx::query_as::<_, (String, String, String, String, NaiveDateTime, NaiveDateTime)>(
        "SELECT id, username, email, role, created_at, updated_at FROM users WHERE id = ?"
    )
    .bind(id)
    .fetch_optional(pool)
    .await?;

    Ok(row.map(|(id, username, email, role, created_at, updated_at)| User {
        id,
        username,
        email,
        role: role.parse().unwrap_or(UserRole::User),
        created_at,
        updated_at,
    }))
}

pub async fn get_all_users(pool: &SqlitePool) -> Result<Vec<User>, sqlx::Error> {
    let rows = sqlx::query_as::<_, (String, String, String, String, NaiveDateTime, NaiveDateTime)>(
        "SELECT id, username, email, role, created_at, updated_at FROM users ORDER BY created_at DESC"
    )
    .fetch_all(pool)
    .await?;

    Ok(rows.into_iter().map(|(id, username, email, role, created_at, updated_at)| User {
        id,
        username,
        email,
        role: role.parse().unwrap_or(UserRole::User),
        created_at,
        updated_at,
    }).collect())
}

pub async fn update_user(pool: &SqlitePool, id: &str, req: &UpdateUserRequest) -> Result<bool, sqlx::Error> {
    let now = chrono::Utc::now().naive_utc();

    let current = get_user_by_id(pool, id).await?;
    let Some(current) = current else { return Ok(false) };

    let username = req.username.as_deref().unwrap_or(&current.username);
    let email = req.email.as_deref().unwrap_or(&current.email);
    let current_role = current.role.to_string();
    let role = req.role.as_deref().unwrap_or(&current_role);

    let result = sqlx::query(
        "UPDATE users SET username = ?, email = ?, role = ?, updated_at = ? WHERE id = ?"
    )
    .bind(username)
    .bind(email)
    .bind(role)
    .bind(now)
    .bind(id)
    .execute(pool)
    .await?;

    Ok(result.rows_affected() > 0)
}

pub async fn delete_user(pool: &SqlitePool, id: &str) -> Result<bool, sqlx::Error> {
    let result = sqlx::query("DELETE FROM users WHERE id = ?")
        .bind(id)
        .execute(pool)
        .await?;
    Ok(result.rows_affected() > 0)
}

pub async fn get_user_by_username(pool: &SqlitePool, username: &str) -> Result<Option<(User, String)>, sqlx::Error> {
    let row = sqlx::query_as::<_, (String, String, String, String, String, NaiveDateTime, NaiveDateTime)>(
        "SELECT id, username, email, password_hash, role, created_at, updated_at FROM users WHERE username = ?"
    )
    .bind(username)
    .fetch_optional(pool)
    .await?;

    Ok(row.map(|(id, username, email, password_hash, role, created_at, updated_at)| {
        (User {
            id,
            username,
            email,
            role: role.parse().unwrap_or(UserRole::User),
            created_at,
            updated_at,
        }, password_hash)
    }))
}
