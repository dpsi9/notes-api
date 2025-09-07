use actix_web::{HttpResponse, Responder, web};
use sqlx::PgPool;
use validator::Validate;

use crate::{
    error::{ApiError, map_sqlx_error},
    notes::service,
    types::{CreateNote, UpdateNote},
};

pub fn init_routes(cfg: &mut web::ServiceConfig) {
    cfg.route("/notes", web::get().to(list_notes_handler))
        .route("/notes", web::post().to(create_note_handler))
        .route("/notes/{id}", web::delete().to(delete_note_handler))
        .route("/notes/{id}", web::get().to(get_note_handler))
        .route("/notes/{id}", web::put().to(update_note_handler));
}

pub async fn list_notes_handler(pool: web::Data<PgPool>) -> Result<impl Responder, ApiError> {
    let notes = service::list_notes(pool.get_ref())
        .await
        .map_err(map_sqlx_error)?;
    Ok(HttpResponse::Ok().json(notes))
}

pub async fn create_note_handler(
    pool: web::Data<PgPool>,
    payload: web::Json<CreateNote>,
) -> Result<impl Responder, ApiError> {
    payload.validate()?;
    let note = service::create_note(pool.get_ref(), &payload.title, &payload.body)
        .await
        .map_err(map_sqlx_error)?;

    Ok(HttpResponse::Ok().json(note))
}

pub async fn get_note_handler(
    pool: web::Data<PgPool>,
    path: web::Path<i64>,
) -> Result<impl Responder, ApiError> {
    let id = path.into_inner();
    match service::get_note(pool.get_ref(), id)
        .await
        .map_err(map_sqlx_error)?
    {
        Some(note) => Ok(HttpResponse::Ok().json(note)),
        None => Err(ApiError::NotFound),
    }
}

pub async fn delete_note_handler(
    pool: web::Data<PgPool>,
    path: web::Path<i64>,
) -> Result<impl Responder, ApiError> {
    let id = path.into_inner();
    let deleted = service::delete_note(pool.get_ref(), id)
        .await
        .map_err(map_sqlx_error)?;

    if deleted {
        Ok(HttpResponse::NoContent().finish())
    } else {
        Err(ApiError::NotFound)
    }
}

pub async fn update_note_handler(
    pool: web::Data<PgPool>,
    path: web::Path<i64>,
    payload: web::Json<UpdateNote>,
) -> Result<impl Responder, ApiError> {
    let id = path.into_inner();
    payload.validate()?;

    let updated = service::update_note(
        pool.get_ref(),
        id,
        payload.title.as_deref(),
        payload.body.as_deref(),
    )
    .await
    .map_err(map_sqlx_error)?;

    match updated {
        Some(note) => Ok(HttpResponse::Ok().json(note)),
        None => Err(ApiError::NotFound),
    }
}
