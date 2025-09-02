use actix_web::{
    App, HttpResponse, HttpServer, Responder, error::ResponseError, http::StatusCode, web,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use thiserror::Error;
use tokio::sync::RwLock;

#[derive(Debug, Error)]
pub enum ApiError {
    #[error("Note not found")]
    NotFound,
    #[error("Internal server error")]
    InternalError,
}

impl ResponseError for ApiError {
    fn status_code(&self) -> StatusCode {
        match self {
            ApiError::NotFound => StatusCode::NOT_FOUND,
            ApiError::InternalError => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Note {
    pub id: u64,
    pub title: String,
    pub body: String,
}

#[derive(Debug, Deserialize)]
pub struct UpdateNote {
    pub title: Option<String>,
    pub body: Option<String>,
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

    HttpServer::new(move || {
        App::new()
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

async fn create_note(data: web::Data<NoteStore>, payload: web::Json<Note>) -> impl Responder {
    let title = &payload.title;
    let body = &payload.body;

    let mut store = data.write().await;

    let id = if let Some(last) = store.last() {
        last.id + 1
    } else {
        1
    };

    store.push(Note {
        id,
        title: title.clone(),
        body: body.clone(),
    });

    HttpResponse::Created().json(serde_json::json!({
        "id": id,
        "title": title
    }))
}

async fn delete_note(data: web::Data<NoteStore>, path: web::Path<u64>) -> impl Responder {
    let id = path.into_inner();

    let mut store = data.write().await;

    // first search the id the delete
    if let Some(pos) = store.iter().position(|n| n.id == id) {
        let removed = store.remove(pos);
        HttpResponse::Ok().json(removed)
    } else {
        HttpResponse::NotFound().body(format!("Note with id {} not found", id))
    }
}

// write a handler that returns a specific note

async fn get_note(data: web::Data<NoteStore>, path: web::Path<u64>) -> impl Responder {
    let id = path.into_inner();
    let store = data.read().await;

    if let Some(note) = store.iter().find(|n| n.id == id) {
        HttpResponse::Ok().json(note)
    } else {
        ApiError::NotFound.error_response()
    }
}

async fn update_note(
    data: web::Data<NoteStore>,
    path: web::Path<u64>,
    payload: web::Json<UpdateNote>,
) -> impl Responder {
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

        HttpResponse::Ok().json(note)
    } else {
        ApiError::NotFound.error_response()
    }
}
