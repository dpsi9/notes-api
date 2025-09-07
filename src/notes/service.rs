use sqlx::PgPool;

use crate::notes::model::Note;

pub async fn list_notes(pool: &PgPool) -> Result<Vec<Note>, sqlx::Error> {
    let notes = sqlx::query_as!(
        Note,
        r#"
        SELECT id, title, body, created_at as "created_at!" FROM notes ORDER BY id DESC
        "#
    )
    .fetch_all(pool)
    .await?;

    Ok(notes)
}

pub async fn create_note(pool: &PgPool, title: &str, body: &str) -> Result<Note, sqlx::Error> {
    let note = sqlx::query_as!(
        Note, 
        r#"INSERT INTO notes (title, body) VALUES($1, $2) RETURNING id, title, body, created_at as "created_at!""#,
        title,
        body
    ).fetch_one(pool).await?;

    Ok(note)
}

pub async fn get_note(pool: &PgPool, id: i64) -> Result<Option<Note>, sqlx::Error> {
    let note = sqlx::query_as!(
        Note,
        r#"SELECT id, title, body, created_at as "created_at!" FROM notes WHERE id = $1"#,
        id
    ).fetch_optional(pool).await?;

    Ok(note)
}

pub async fn delete_note(pool: &PgPool, id: i64) -> Result<bool, sqlx::Error> {
    let res = sqlx::query!(
        r#"DELETE FROM notes WHERE id = $1"#, id
    ).execute(pool).await?;

    Ok(res.rows_affected() > 0)
}

pub async fn update_note(pool: &PgPool, id: i64, title: Option<&str>, body: Option<&str>) -> Result<Option<Note>, sqlx::Error> {
    let mut tx = pool.begin().await?;

    if let Some( note) = sqlx::query_as!(
        Note,
        r#"SELECT id, title, body, created_at as "created_at!" FROM notes WHERE id = $1"#, id
    ).fetch_optional(&mut *tx).await? {
        let new_title = title.unwrap_or(&note.title);
        let new_body = body.unwrap_or(&note.body);

        let updated = sqlx::query_as!(
            Note,
            r#"UPDATE notes SET title = $1, body = $2 WHERE id = $3 RETURNING id, title, body, created_at as "created_at!""#,
            new_title,
            new_body,
            id
        ).fetch_one(&mut *tx).await?;

        tx.commit().await?;
        Ok(Some(updated))
    } else {
        tx.rollback().await?;
        Ok(None)
    }
}