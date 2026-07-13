use axum::{
    Json, Router,
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{delete, get, post, put},
};
use serde::Deserialize;
use serde_json::{Value, json};
use sqlx::{Pool, Sqlite, sqlite::SqlitePoolOptions};
use tokio::net::TcpListener;

#[derive(Clone)]
struct AppState {
    db: Pool<Sqlite>,
}

#[derive(Deserialize)]
struct NutritionInput {
    calories: i64,
    protein: f64,
}

#[derive(Deserialize)]
struct CardioInput {
    exercise_name: String,
    duration_in_minutes: i64,
}

#[derive(Deserialize)]
struct SetInput {
    weight_in_pounds: i64,
    repetitions: i64,
}

#[derive(Deserialize)]
struct ExerciseInput {
    name: String,
    sets: Vec<SetInput>,
}

#[derive(Deserialize)]
struct WeightSessionInput {
    name: String,
    exercises: Vec<ExerciseInput>,
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
        .route("/day/{date}", get(get_day))
        .route("/day/{date}/nutrition", put(put_nutrition))
        .route("/day/{date}/cardio-sessions", post(create_cardio_session))
        .route("/day/{date}/weight-sessions", post(create_weight_session))
        .route("/cardio-sessions/{id}", put(update_cardio_session))
        .route("/cardio-sessions/{id}", delete(delete_cardio_session))
        .route("/weight-sessions/{id}", put(update_weight_session))
        .route("/weight-sessions/{id}", delete(delete_weight_session))
        .route("/data", get(get_data))
        .with_state(state);

    println!("Server started successfully at 0.0.0.0:8080");

    let listener = TcpListener::bind("0.0.0.0:8080").await.unwrap();
    axum::serve(listener, app.into_make_service())
        .await
        .unwrap();
}

async fn health_check_handler() -> impl IntoResponse {
    (
        StatusCode::OK,
        Json(json!({
            "status": "ok",
            "message": "API Services"
        })),
    )
}

async fn ensure_daily_log(db: &Pool<Sqlite>, date: &str) -> Result<(), sqlx::Error> {
    sqlx::query!("INSERT OR IGNORE INTO daily_log (date) VALUES (?)", date)
        .execute(db)
        .await?;
    Ok(())
}

async fn fetch_weight_session_json(
    db: &Pool<Sqlite>,
    session_id: i64,
) -> Result<Option<Value>, sqlx::Error> {
    let session = sqlx::query!(
        "SELECT id, name FROM weight_session WHERE id = ?",
        session_id
    )
    .fetch_optional(db)
    .await?;

    let Some(session) = session else {
        return Ok(None);
    };

    let exercises = sqlx::query!(
        "SELECT id, name FROM exercise WHERE weight_session_id = ? ORDER BY id",
        session_id
    )
    .fetch_all(db)
    .await?;

    let mut exercise_json = Vec::new();
    for exercise in exercises {
        let sets = sqlx::query!(
            "SELECT weight_in_pounds, repetitions FROM exercise_sets WHERE exercise_id = ? ORDER BY id",
            exercise.id
        )
        .fetch_all(db)
        .await?;

        exercise_json.push(json!({
            "id": exercise.id,
            "name": exercise.name,
            "sets": sets.iter().map(|set| json!({
                "weight_in_pounds": set.weight_in_pounds,
                "repetitions": set.repetitions,
            })).collect::<Vec<_>>(),
        }));
    }

    Ok(Some(json!({
        "id": session.id,
        "name": session.name,
        "exercises": exercise_json,
    })))
}

async fn build_weight_sessions_for_date(
    db: &Pool<Sqlite>,
    date: &str,
) -> Result<Vec<Value>, sqlx::Error> {
    let sessions = sqlx::query!(
        "SELECT id FROM weight_session WHERE date = ? ORDER BY id",
        date
    )
    .fetch_all(db)
    .await?;

    let mut result = Vec::new();
    for session in sessions {
        if let Some(session_json) = fetch_weight_session_json(db, session.id).await? {
            result.push(session_json);
        }
    }
    Ok(result)
}

async fn insert_weight_session_exercises(
    tx: &mut sqlx::Transaction<'_, Sqlite>,
    session_id: i64,
    exercises: &[ExerciseInput],
) -> Result<(), sqlx::Error> {
    for exercise in exercises {
        let result = sqlx::query!(
            "INSERT INTO exercise (weight_session_id, name) VALUES (?, ?)",
            session_id,
            exercise.name
        )
        .execute(&mut **tx)
        .await?;

        let exercise_id = result.last_insert_rowid();
        for set in &exercise.sets {
            sqlx::query!(
                "INSERT INTO exercise_sets (exercise_id, weight_in_pounds, repetitions) VALUES (?, ?, ?)",
                exercise_id,
                set.weight_in_pounds,
                set.repetitions
            )
            .execute(&mut **tx)
            .await?;
        }
    }
    Ok(())
}

async fn get_day(State(state): State<AppState>, Path(date): Path<String>) -> impl IntoResponse {
    let nutrition = sqlx::query!(
        "SELECT COALESCE(SUM(calories), 0) AS calories, COALESCE(SUM(protein), 0.0) AS protein
         FROM nutrition WHERE date = ?",
        date
    )
    .fetch_one(&state.db)
    .await;

    let cardio = sqlx::query!(
        "SELECT id, exercise_name, duration_in_minutes FROM cardio_session WHERE date = ? ORDER BY id",
        date
    )
    .fetch_all(&state.db)
    .await;

    let weight_sessions = build_weight_sessions_for_date(&state.db, &date).await;

    match (nutrition, cardio, weight_sessions) {
        (Ok(nutrition), Ok(cardio), Ok(weight_sessions)) => (
            StatusCode::OK,
            Json(json!({
                "date": date,
                "nutrition": {
                    "calories": nutrition.calories,
                    "protein": nutrition.protein,
                },
                "cardio": cardio.iter().map(|record| json!({
                    "id": record.id,
                    "exercise_name": record.exercise_name,
                    "duration_in_minutes": record.duration_in_minutes,
                })).collect::<Vec<_>>(),
                "weight_sessions": weight_sessions,
            })),
        ),
        _ => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "error": "Failed to fetch day data" })),
        ),
    }
}

async fn put_nutrition(
    State(state): State<AppState>,
    Path(date): Path<String>,
    Json(input): Json<NutritionInput>,
) -> impl IntoResponse {
    let mut tx = match state.db.begin().await {
        Ok(tx) => tx,
        Err(err) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": format!("Internal server error: {}", err) })),
            );
        }
    };

    if let Err(err) = ensure_daily_log(&state.db, &date).await {
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "error": format!("Internal server error: {}", err) })),
        );
    }

    if let Err(err) = sqlx::query!("DELETE FROM nutrition WHERE date = ?", date)
        .execute(&mut *tx)
        .await
    {
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "error": format!("Internal server error: {}", err) })),
        );
    }

    if input.calories > 0 || input.protein > 0.0 {
        if let Err(err) = sqlx::query!(
            "INSERT INTO nutrition (date, calories, protein) VALUES (?, ?, ?)",
            date,
            input.calories,
            input.protein
        )
        .execute(&mut *tx)
        .await
        {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": format!("Internal server error: {}", err) })),
            );
        }
    }

    if let Err(err) = tx.commit().await {
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "error": format!("Internal server error: {}", err) })),
        );
    }

    (
        StatusCode::OK,
        Json(json!({
            "calories": input.calories,
            "protein": input.protein,
        })),
    )
}

async fn create_cardio_session(
    State(state): State<AppState>,
    Path(date): Path<String>,
    Json(input): Json<CardioInput>,
) -> impl IntoResponse {
    if let Err(err) = ensure_daily_log(&state.db, &date).await {
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "error": format!("Internal server error: {}", err) })),
        );
    }

    let result = sqlx::query!(
        "INSERT INTO cardio_session (date, exercise_name, duration_in_minutes, after_weight_session)
         VALUES (?, ?, ?, NULL)",
        date,
        input.exercise_name,
        input.duration_in_minutes
    )
    .execute(&state.db)
    .await;

    match result {
        Ok(result) => (
            StatusCode::CREATED,
            Json(json!({
                "id": result.last_insert_rowid(),
                "exercise_name": input.exercise_name,
                "duration_in_minutes": input.duration_in_minutes,
            })),
        ),
        Err(err) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "error": format!("Internal server error: {}", err) })),
        ),
    }
}

async fn update_cardio_session(
    State(state): State<AppState>,
    Path(id): Path<i64>,
    Json(input): Json<CardioInput>,
) -> impl IntoResponse {
    let result = sqlx::query!(
        "UPDATE cardio_session
         SET exercise_name = ?, duration_in_minutes = ?
         WHERE id = ?",
        input.exercise_name,
        input.duration_in_minutes,
        id
    )
    .execute(&state.db)
    .await;

    match result {
        Ok(res) if res.rows_affected() == 0 => (
            StatusCode::NOT_FOUND,
            Json(json!({ "error": "Cardio session not found" })),
        ),
        Ok(_) => (
            StatusCode::OK,
            Json(json!({
                "id": id,
                "exercise_name": input.exercise_name,
                "duration_in_minutes": input.duration_in_minutes,
            })),
        ),
        Err(err) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "error": format!("Internal server error: {}", err) })),
        ),
    }
}

async fn delete_cardio_session(
    State(state): State<AppState>,
    Path(id): Path<i64>,
) -> impl IntoResponse {
    let result = sqlx::query!("DELETE FROM cardio_session WHERE id = ?", id)
        .execute(&state.db)
        .await;

    match result {
        Ok(res) if res.rows_affected() == 0 => (
            StatusCode::NOT_FOUND,
            Json(json!({ "error": "Cardio session not found" })),
        )
            .into_response(),
        Ok(_) => StatusCode::NO_CONTENT.into_response(),
        Err(err) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "error": format!("Internal server error: {}", err) })),
        )
            .into_response(),
    }
}

async fn create_weight_session(
    State(state): State<AppState>,
    Path(date): Path<String>,
    Json(input): Json<WeightSessionInput>,
) -> impl IntoResponse {
    if let Err(err) = ensure_daily_log(&state.db, &date).await {
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "error": format!("Internal server error: {}", err) })),
        );
    }

    let mut tx = match state.db.begin().await {
        Ok(tx) => tx,
        Err(err) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": format!("Internal server error: {}", err) })),
            );
        }
    };

    let result = match sqlx::query!(
        "INSERT INTO weight_session (date, name) VALUES (?, ?)",
        date,
        input.name
    )
    .execute(&mut *tx)
    .await
    {
        Ok(result) => result,
        Err(err) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": format!("Internal server error: {}", err) })),
            );
        }
    };

    let session_id = result.last_insert_rowid();

    if let Err(err) = insert_weight_session_exercises(&mut tx, session_id, &input.exercises).await {
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "error": format!("Internal server error: {}", err) })),
        );
    }

    if let Err(err) = tx.commit().await {
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "error": format!("Internal server error: {}", err) })),
        );
    }

    match fetch_weight_session_json(&state.db, session_id).await {
        Ok(Some(session)) => (StatusCode::CREATED, Json(session)),
        Ok(None) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "error": "Failed to fetch created session" })),
        ),
        Err(err) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "error": format!("Internal server error: {}", err) })),
        ),
    }
}

async fn update_weight_session(
    State(state): State<AppState>,
    Path(id): Path<i64>,
    Json(input): Json<WeightSessionInput>,
) -> impl IntoResponse {
    let mut tx = match state.db.begin().await {
        Ok(tx) => tx,
        Err(err) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": format!("Internal server error: {}", err) })),
            );
        }
    };

    let update_result = sqlx::query!(
        "UPDATE weight_session SET name = ? WHERE id = ?",
        input.name,
        id
    )
    .execute(&mut *tx)
    .await;

    match update_result {
        Ok(res) if res.rows_affected() == 0 => {
            return (
                StatusCode::NOT_FOUND,
                Json(json!({ "error": "Weight session not found" })),
            );
        }
        Err(err) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": format!("Internal server error: {}", err) })),
            );
        }
        Ok(_) => {}
    }

    if let Err(err) = sqlx::query!("DELETE FROM exercise WHERE weight_session_id = ?", id)
        .execute(&mut *tx)
        .await
    {
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "error": format!("Internal server error: {}", err) })),
        );
    }

    if let Err(err) = insert_weight_session_exercises(&mut tx, id, &input.exercises).await {
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "error": format!("Internal server error: {}", err) })),
        );
    }

    if let Err(err) = tx.commit().await {
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "error": format!("Internal server error: {}", err) })),
        );
    }

    match fetch_weight_session_json(&state.db, id).await {
        Ok(Some(session)) => (StatusCode::OK, Json(session)),
        Ok(None) => (
            StatusCode::NOT_FOUND,
            Json(json!({ "error": "Weight session not found" })),
        ),
        Err(err) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "error": format!("Internal server error: {}", err) })),
        ),
    }
}

async fn delete_weight_session(
    State(state): State<AppState>,
    Path(id): Path<i64>,
) -> impl IntoResponse {
    let result = sqlx::query!("DELETE FROM weight_session WHERE id = ?", id)
        .execute(&state.db)
        .await;

    match result {
        Ok(res) if res.rows_affected() == 0 => (
            StatusCode::NOT_FOUND,
            Json(json!({ "error": "Weight session not found" })),
        )
            .into_response(),
        Ok(_) => StatusCode::NO_CONTENT.into_response(),
        Err(err) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "error": format!("Internal server error: {}", err) })),
        )
            .into_response(),
    }
}

async fn get_data(State(state): State<AppState>) -> impl IntoResponse {
    let result = sqlx::query!("SELECT * FROM cardio_session")
        .fetch_all(&state.db)
        .await;

    match result {
        Ok(data) => (
            StatusCode::OK,
            Json(json!(
                data.iter()
                    .map(|record| json!({
                        "id": record.id,
                        "date": record.date,
                        "exercise_name": record.exercise_name,
                        "duration_in_minutes": record.duration_in_minutes,
                        "after_weight_session": record.after_weight_session,
                    }))
                    .collect::<Vec<_>>()
            )),
        ),
        Err(err) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "error": format!("{}", err) })),
        ),
    }
}
