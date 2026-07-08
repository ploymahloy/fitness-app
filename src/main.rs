use axum::{
    Json, Router,
    extract::State,
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post}
};
use serde::Serialize;
use sqlx::{Pool, Sqlite, sqlite::SqlitePoolOptions};
use tokio::net::TcpListener;

#[derive(Serialize, sqlx::FromRow)]
struct Log {
    date: String
}

#[derive(Clone)]
struct AppState {
    db: Pool<Sqlite>
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
        .route("/data", get(get_data))
        .route("/upload/cardio", post(upload_cardio))
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

    let _ = Json(json_response);
}

async fn upload_cardio(State(state): State<AppState>) -> impl IntoResponse {
    let result = sqlx::query!(
        "INSERT INTO cardio_session (date, exercise_name, duration_in_minutes, after_weight_session)
        VALUES ('2026-07-01', 'Treadmill HIIT', 45, 1);",
        )
    .execute(&state.db)
    .await;

    match result {
        Ok(_) => (
            StatusCode::CREATED, // 201
            Json(serde_json::json!({ "message": "Exercise created successfully" }))
        ),

        Err(sqlx::Error::Database(db_err)) => {
            if db_err.code().as_deref() == Some("23505") {
                (
                    StatusCode::CONFLICT, // 409
                    Json(
                        serde_json::json!({ "error": "An exercise with this name already exists" })
                    )
                )
            } else if db_err.code().as_deref() == Some("42501") {
                (
                    StatusCode::FORBIDDEN, // 403
                    Json(
                        serde_json::json!({ "error": "You do not have permission to modify this resource" })
                    )
                )
            } else {
                (
                    StatusCode::INTERNAL_SERVER_ERROR, // 500
                    Json(serde_json::json!({ "error": "Database constraint violation" }))
                )
            }
        }

        Err(err) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": format!("Internal server error: {}", err) }))
        )
    }
}

async fn get_data(State(state): State<AppState>) -> impl IntoResponse {
    let result = sqlx::query_as::<_, Log>("SELECT date FROM daily_log")
        .fetch_all(&state.db)
        .await;

    match result {
        Ok(logs) => (
            StatusCode::OK,
            Json(serde_json::to_value(logs).expect("failed to serialize logs"))
        ),
        Err(err) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": format!("{}", err) }))
        )
    }
}
