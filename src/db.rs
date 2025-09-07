use anyhow::{Context, Result};
use sqlx::PgPool;
use sqlx::postgres::PgPoolOptions;

pub async fn setup_db(db_url: &str) -> Result<PgPool> {
    let pool = PgPoolOptions::new()
        .max_connections(10)
        .connect(&db_url)
        .await
        .context("failed to connect to Postgres")?;

    sqlx::migrate!("./migrations")
        .run(&pool)
        .await
        .context("failed to run DB migrations")?;

    let notes = vec![
        ("First note", "This is the body"),
        ("Second note", "This is the second body"),
    ];

    for (title, body) in notes {
        sqlx::query!(
            r#"
            INSERT INTO notes (title, body)
            VALUES ($1, $2)
            ON CONFLICT DO NOTHING
            "#,
            title,
            body
        )
        .execute(&pool)
        .await?;
    }

    Ok(pool)
}
