use axum::{
    Router, 
    Json,
    routing::get, 
    response::IntoResponse
};
use tokio::net::TcpListener;

#[tokio::main]
async fn main() {
    let app = Router::new()
        .route("/health", get(health_check_handler))
        .route("/upload", get(upload_workout));

    println!("Server started successfully at 0.0.0.0:8080");

    let listener = TcpListener::bind("0.0.0.0:8080").await.unwrap();
    axum::serve(listener, app.into_make_service())
        .await
        .unwrap();
}

async fn health_check_handler() -> impl IntoResponse {
    const MESSAGE: &str = "API Services";

    let json_response = serde_json::json!({
        "status": "ok",
        "message": MESSAGE
    });

    Json(json_response)
}

async fn upload_workout() -> impl IntoResponse {
    let json_response = serde_json::json!({
        "message": "Workout uploaded successfully."
    });

    Json(json_response)
}