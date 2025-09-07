use anyhow::{Context, Result};
use sqlx::postgres::PgPoolOptions;
use std::env;

pub async fn setup_db() -> Result<sqlx::Pool<sqlx::Postgres>> {
    let db_url = env::var("DATABASE_URL").context("DATABASE_URL is not set")?;

    let pool = PgPoolOptions::new()
        .max_connections(10)
        .connect(&db_url)
        .await
        .context("failed to connect to Postgres")?;

    sqlx::migrate!("./migrations")
        .run(&pool)
        .await
        .context("failed to run DB migrations")?;

    Ok(pool)
}
