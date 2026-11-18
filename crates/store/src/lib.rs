//! Postgres storage. Queries are runtime-checked (`sqlx::query`) for now, so
//! building needs no database; migrations are embedded at compile time.

use sqlx::postgres::{PgPool, PgPoolOptions};
use uuid::Uuid;

pub mod password;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("database: {0}")]
    Db(#[from] sqlx::Error),
    #[error("migration: {0}")]
    Migrate(#[from] sqlx::migrate::MigrateError),
    #[error("password hashing: {0}")]
    Password(String),
}

pub type Result<T> = std::result::Result<T, Error>;

/// Handle to the database; cheap to clone.
#[derive(Clone)]
pub struct Store {
    pool: PgPool,
}

impl Store {
    /// Connects to `database_url` (e.g. `postgres://co:co@localhost:5432/co`).
    pub async fn connect(database_url: &str) -> Result<Self> {
        let pool = PgPoolOptions::new()
            .max_connections(16)
            .connect(database_url)
            .await?;
        Ok(Self { pool })
    }

    /// Applies the migrations in `crates/store/migrations`.
    pub async fn migrate(&self) -> Result<()> {
        sqlx::migrate!("./migrations").run(&self.pool).await?;
        Ok(())
    }

    pub fn pool(&self) -> &PgPool {
        &self.pool
    }

    /// Creates a user with an argon2id hash of `password`.
    pub async fn create_user(&self, username: &str, password: &str) -> Result<Uuid> {
        let id = Uuid::now_v7();
        let hash = password::hash(password)?;
        sqlx::query("INSERT INTO users (id, username, password_hash) VALUES ($1, $2, $3)")
            .bind(id)
            .bind(username)
            .bind(hash)
            .execute(&self.pool)
            .await?;
        Ok(id)
    }

    /// The user's id if `password` is right.
    pub async fn verify_user(&self, username: &str, password: &str) -> Result<Option<Uuid>> {
        let row: Option<(Uuid, String)> =
            sqlx::query_as("SELECT id, password_hash FROM users WHERE username = $1")
                .bind(username)
                .fetch_optional(&self.pool)
                .await?;
        Ok(row.and_then(|(id, hash)| password::verify(password, &hash).then_some(id)))
    }
}
