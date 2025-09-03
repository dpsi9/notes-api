use actix_web::{App, HttpResponse, HttpServer, Responder, web};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::RwLock;
use validator::Validate;

mod error;
mod middlewares;
mod tracing;
mod types;

use crate::{
    error::ApiError,
    types::{CreateNote, UpdateNote},
};
use actix_web::middleware;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Note {
    pub id: u64,
    pub title: String,
    pub body: String,
}

pub type NoteStore = Arc<RwLock<Vec<Note>>>;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let notes: Vec<Note> = vec![
        Note {
            id: 1,
            title: "First Note".into(),
            body: "This is the body".into(),
        },
        Note {
            id: 2,
            title: String::from("Second Note"),
            body: "This is the second body".into(),
        },
    ];
    let store: NoteStore = Arc::new(RwLock::new(notes));

    tracing::init_tracing();

    HttpServer::new(move || {
        App::new()
            .wrap(middlewares::RequestId)
            .wrap(middleware::Compress::default())
            .wrap(middleware::NormalizePath::trim())
            .app_data(web::Data::new(store.clone()))
            .route("/notes", web::get().to(list_notes))
            .route("/notes", web::post().to(create_note))
            .route("/notes/{id}", web::delete().to(delete_note))
            .route("/notes/{id}", web::get().to(get_note))
            .route("/notes/{id}", web::put().to(update_note))
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}

async fn list_notes(data: web::Data<NoteStore>) -> impl Responder {
    let store = data.read().await;

    HttpResponse::Ok().json(&*store)
}

async fn create_note(
    data: web::Data<NoteStore>,
    payload: web::Json<CreateNote>,
) -> Result<impl Responder, ApiError> {
    payload.validate()?;

    let mut store = data.write().await;

    let id = if let Some(last) = store.last() {
        last.id + 1
    } else {
        1
    };

    let note = Note {
        id,
        title: payload.title.clone(),
        body: payload.body.clone(),
    };

    store.push(note.clone());

    Ok(HttpResponse::Created().json(note))
}

async fn delete_note(
    data: web::Data<NoteStore>,
    path: web::Path<u64>,
) -> Result<impl Responder, ApiError> {
    let id = path.into_inner();

    let mut store = data.write().await;

    // first search the id the delete
    if let Some(pos) = store.iter().position(|n| n.id == id) {
        let removed = store.remove(pos);
        Ok(HttpResponse::Ok().json(removed))
    } else {
        Err(ApiError::BadRequest(format!(
            "Note with id {} not found",
            id
        )))
    }
}

// write a handler that returns a specific note

async fn get_note(
    data: web::Data<NoteStore>,
    path: web::Path<u64>,
) -> Result<impl Responder, ApiError> {
    let id = path.into_inner();
    let store = data.read().await;

    if let Some(note) = store.iter().find(|n| n.id == id) {
        Ok(HttpResponse::Ok().json(note))
    } else {
        Err(ApiError::NotFound)
    }
}

async fn update_note(
    data: web::Data<NoteStore>,
    path: web::Path<u64>,
    payload: web::Json<UpdateNote>,
) -> Result<impl Responder, ApiError> {
    payload.validate()?;

    let id = path.into_inner();
    let mut store = data.write().await;

    // find the note with the id, then match if it has title and body update accordingly
    if let Some(note) = store.iter_mut().find(|n| n.id == id) {
        if let Some(title) = &payload.title {
            note.title = title.clone();
        }
        if let Some(body) = &payload.body {
            note.body = body.clone();
        }

        Ok(HttpResponse::Ok().json(note))
    } else {
        Err(ApiError::NotFound)
    }
}
