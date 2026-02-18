use axum::{body::Body, http::Request};
use backend::{app, AppState};
use serde_json::json;
use sqlx::sqlite::SqlitePoolOptions;
use tower::ServiceExt;

async fn setup_state() -> Result<AppState, Box<dyn std::error::Error>> {
    let pool = SqlitePoolOptions::new()
        .max_connections(1)
        .connect("sqlite::memory:")
        .await?;

    sqlx::query("PRAGMA foreign_keys = ON;")
        .execute(&pool)
        .await?;

    sqlx::migrate!().run(&pool).await?;

    sqlx::query("DELETE FROM user_models;")
        .execute(&pool)
        .await?;
    sqlx::query("DELETE FROM model_definitions;")
        .execute(&pool)
        .await?;
    sqlx::query("DELETE FROM games;").execute(&pool).await?;

    Ok(AppState { pool })
}

#[tokio::test]
async fn create_game_returns_created() -> Result<(), Box<dyn std::error::Error>> {
    let state = setup_state().await?;
    let app = app(state);

    let payload = json!({ "name": "Warhammer" }).to_string();
    let request = Request::builder()
        .method("POST")
        .uri("/games")
        .header("content-type", "application/json")
        .body(Body::from(payload))?;

    let response = app.oneshot(request).await?;
    assert_eq!(response.status(), 201);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX).await?;
    let data: serde_json::Value = serde_json::from_slice(&body)?;

    assert_eq!(data["name"], "Warhammer");
    assert!(data["id"].is_string());
    assert!(data["created_at"].is_string());

    Ok(())
}

#[tokio::test]
async fn duplicate_game_returns_conflict() -> Result<(), Box<dyn std::error::Error>> {
    let state = setup_state().await?;
    let app = app(state);

    let payload = json!({ "name": "Infinity" }).to_string();
    let request = Request::builder()
        .method("POST")
        .uri("/games")
        .header("content-type", "application/json")
        .body(Body::from(payload.clone()))?;

    let response = app.clone().oneshot(request).await?;
    assert_eq!(response.status(), 201);

    let request = Request::builder()
        .method("POST")
        .uri("/games")
        .header("content-type", "application/json")
        .body(Body::from(payload))?;

    let response = app.oneshot(request).await?;
    assert_eq!(response.status(), 409);

    Ok(())
}

#[tokio::test]
async fn list_games_returns_ordered_data() -> Result<(), Box<dyn std::error::Error>> {
    let state = setup_state().await?;
    let app = app(state);

    let payload = json!({ "name": "Kill Team" }).to_string();
    let request = Request::builder()
        .method("POST")
        .uri("/games")
        .header("content-type", "application/json")
        .body(Body::from(payload))?;
    let response = app.clone().oneshot(request).await?;
    assert_eq!(response.status(), 201);

    tokio::time::sleep(std::time::Duration::from_millis(5)).await;

    let payload = json!({ "name": "Legion" }).to_string();
    let request = Request::builder()
        .method("POST")
        .uri("/games")
        .header("content-type", "application/json")
        .body(Body::from(payload))?;
    let response = app.clone().oneshot(request).await?;
    assert_eq!(response.status(), 201);

    let request = Request::builder()
        .method("GET")
        .uri("/games")
        .body(Body::empty())?;
    let response = app.oneshot(request).await?;
    assert_eq!(response.status(), 200);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX).await?;
    let data: serde_json::Value = serde_json::from_slice(&body)?;
    let list = data.as_array().ok_or("expected array response")?;

    assert_eq!(list.len(), 2);
    assert_eq!(list[0]["name"], "Kill Team");
    assert_eq!(list[1]["name"], "Legion");

    Ok(())
}
