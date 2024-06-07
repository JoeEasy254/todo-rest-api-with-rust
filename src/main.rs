// this line indicates that there's a module named models which is assumed to define the Todo struct based on its role
mod models;

// These use statements import necessary components from various crates:
// axum for the web framework.
// models::Todo for the Todo struct.
// serde_json::json for creating JSON responses.
// Standard library modules for network addresses (SocketAddr) and concurrency (Arc, Mutex).
// tokio::main for asynchronous main function.
// uuid for generating unique IDs.
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{delete, get, post, put},
    Json, Router,
};
use models::Todo;
use serde_json::json;
use std::{
    net::SocketAddr,
    sync::{Arc, Mutex},
};
use tokio::main;
use uuid::Uuid;





#[derive(Clone)]
struct AppState {
    // ARC-> atomic reference counting
    todos: Arc<Mutex<Vec<Todo>>>,
}

#[main]
async fn main() {
    // Initialize shared state
    let state = AppState {
        todos: Arc::new(Mutex::new(Vec::new())),
    };

    // Build our application
    let app = Router::new()
        .route("/todos", get(list_todos).post(create_todo))
        .route(
            "/todos/:id",
            get(get_todo).put(update_todo).delete(delete_todo),
        )
        .with_state(state);

    // Run our application
    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    println!("Listening on {}", addr);

    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}

async fn list_todos(State(state): State<AppState>) -> impl IntoResponse {
    let todos = state.todos.lock().unwrap();
    Json(todos.clone())
}

async fn create_todo(
    State(state): State<AppState>,
    Json(payload): Json<Todo>,
) -> impl IntoResponse {
    let mut todos = state.todos.lock().unwrap();
    let new_todo = Todo {
        id: Uuid::new_v4(),
        title: payload.title,
        completed: payload.completed,
    };
    todos.push(new_todo.clone());
    (StatusCode::CREATED, Json(new_todo))
}

async fn get_todo(State(state): State<AppState>, Path(id): Path<Uuid>) -> impl IntoResponse {
    let todos = state.todos.lock().unwrap();
    if let Some(todo) = todos.iter().find(|&todo| todo.id == id) {
        Json(todo.clone()).into_response()
    } else {
        (
            StatusCode::NOT_FOUND,
            Json(json!({ "error": "Todo not found" })),
        )
            .into_response()
    }
}

async fn update_todo(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(payload): Json<Todo>,
) -> impl IntoResponse {
    let mut todos = state.todos.lock().unwrap();
    if let Some(todo) = todos.iter_mut().find(|todo| todo.id == id) {
        todo.title = payload.title;
        todo.completed = payload.completed;
        Json(todo.clone()).into_response()
    } else {
        (
            StatusCode::NOT_FOUND,
            Json(json!({ "error": "Todo not found" })),
        )
            .into_response()
    }
}

async fn delete_todo(State(state): State<AppState>, Path(id): Path<Uuid>) -> impl IntoResponse {
    let mut todos = state.todos.lock().unwrap();
    if let Some(pos) = todos.iter().position(|todo| todo.id == id) {
        todos.remove(pos);
        StatusCode::NO_CONTENT
    } else {
        StatusCode::NOT_FOUND
    }
}
