use axum::{
    Json, Router,
    extract::State,
    http::StatusCode,
    response::IntoResponse,
    routing::{delete, get, patch, post}
};
use sqlx::{Pool, Sqlite, sqlite::SqlitePoolOptions};
use tokio::net::TcpListener;

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
        .route("/update", patch(update_data))
        .route("/delete", delete(delete_record))
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

    (StatusCode::OK, Json(json_response))
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
    let result = sqlx::query!("SELECT * FROM cardio_session")
        .fetch_all(&state.db)
        .await;

    match result {
        Ok(data) => (
            StatusCode::OK,
            Json(serde_json::json!(
                data.iter()
                    .map(|record| serde_json::json!({
                        "id": record.id,
                        "date": record.date,
                        "exercise_name": record.exercise_name,
                        "duration_in_minutes": record.duration_in_minutes,
                        "after_weight_session": record.after_weight_session,
                    }))
                    .collect::<Vec<_>>()
            ))
        ),
        Err(err) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": format!("{}", err) }))
        )
    }
}

async fn update_data(State(state): State<AppState>) -> impl IntoResponse {
    let result = sqlx::query!(
        "UPDATE cardio_session
        SET exercise_name = 'Quiditch',
            duration_in_minutes = 120 
        WHERE id = (SELECT MAX(id) FROM cardio_session);"
    )
    .execute(&state.db)
    .await;

    match result {
        Ok(res) if res.rows_affected() == 0 => (
            StatusCode::NOT_FOUND, // 404
            Json(serde_json::json!({ "error": "No records to update" }))
        ),
        Ok(_) => {
            (
                StatusCode::OK, // 200
                Json(serde_json::json!({ "message": "Record updated successfully" }))
            )
        }
        Err(err) => (
            StatusCode::INTERNAL_SERVER_ERROR, // 500
            Json(serde_json::json!({ "error": format!("Internal server error: {}", err) }))
        )
    }
}

async fn delete_record(State(state): State<AppState>) -> impl IntoResponse {
    let result = sqlx::query!(
        "DELETE FROM cardio_session
         WHERE id = (SELECT MAX(id) FROM cardio_session);"
    )
    .execute(&state.db)
    .await;

    match result {
        Ok(res) if res.rows_affected() == 0 => (
            StatusCode::NOT_FOUND, // 404
            Json(serde_json::json!({ "error": "Exercise not found" }))
        ),
        Ok(_) => (StatusCode::NO_CONTENT, Json(serde_json::json!({}))),
        Err(sqlx::Error::Database(db_err)) if db_err.code().as_deref() == Some("23503") => (
            StatusCode::BAD_REQUEST, // 400
            Json(
                serde_json::json!({ "error": "Cannot delete this exercise because it is being used by other records" })
            )
        ),
        Err(err) => (
            StatusCode::INTERNAL_SERVER_ERROR, // 500
            Json(serde_json::json!({ "error": format!("Internal server error: {}", err) }))
        )
    }
}
