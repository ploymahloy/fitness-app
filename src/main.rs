use axum::{
    Json, Router,
    extract::State,
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
};
use serde::Serialize;
use sqlx::{Pool, Sqlite, sqlite::SqlitePoolOptions};
use tokio::net::TcpListener;

#[derive(Serialize, sqlx::FromRow)]
struct Log {
    date: String,
}

#[derive(Clone)]
struct AppState {
    db: Pool<Sqlite>,
}

#[tokio::main]
async fn main() {
    let pool = SqlitePoolOptions::new()
        .max_connections(3)
        .connect("sqlite://fitness_app.db")
        .await
        .expect("Failed to connect to the database pool");

    let state = AppState { db: pool };

    let app = Router::new()
        .route("/health", get(health_check_handler))
        .route("/upload", post(upload_workout))
        .route("/data", get(get_data))
        .with_state(state);

    println!("Server started successfully at 0.0.0.0:8080");

    let listener = TcpListener::bind("0.0.0.0:8080").await.unwrap();
    axum::serve(listener, app.into_make_service())
        .await
        .unwrap();
}

async fn health_check_handler() -> impl IntoResponse {
    let json_response = serde_json::json!({
        "status": "ok",
        "message": "API Services"
    });

    Json(json_response)
}

async fn upload_workout() -> impl IntoResponse {
    let json_response = serde_json::json!({
        "message": "Workout uploaded successfully."
    });

    Json(json_response)
}

async fn get_data(State(state): State<AppState>) -> impl IntoResponse {
    let result = sqlx::query_as::<_, Log>("SELECT date FROM daily_log")
        .fetch_all(&state.db)
        .await;

    match result {
        Ok(logs) => (
            StatusCode::OK,
            Json(serde_json::to_value(logs).expect("failed to serialize logs")),
        ),
        Err(err) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": format!("{}", err) })),
        ),
    }
}
