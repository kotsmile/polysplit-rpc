use anyhow::{Context, Result};
use sqlx::{postgres::PgPoolOptions, Pool, Postgres};

use crate::models::user::User;

pub struct StorageRepo {
    pool: Pool<Postgres>,
}

impl StorageRepo {
    pub async fn new(database_url: String, max_connections: u32) -> Result<Self> {
        let pool = PgPoolOptions::new()
            .max_connections(max_connections)
            .connect(&database_url)
            .await
            .context("failed to initiate connection")?;

        sqlx::migrate!()
            .run(&pool)
            .await
            .context("failed to run migrations")?;

        Ok(Self { pool })
    }

    pub async fn create_user(&self, new_user: &User) -> Result<Option<User>> {
        sqlx::query_as!(
            User,
            r#"
                insert into users (id, email) 
                values ($1, $2) 
                returning *
            "#,
            new_user.id,
            new_user.email,
        )
        .fetch_optional(&self.pool)
        .await
        .context("failed to insert user")
    }

    pub async fn get_user_by_email(&self, email: &str) -> Result<Option<User>> {
        sqlx::query_as!(
            User,
            r#"
                select * 
                from users 
                where email = $1
            "#,
            email
        )
        .fetch_optional(&self.pool)
        .await
        .context("failed to find user by email")
    }
}
