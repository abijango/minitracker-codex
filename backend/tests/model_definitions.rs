use axum::{body::Body, http::Request};
use backend::{app, AppState};
use serde_json::json;
use sqlx::sqlite::SqlitePoolOptions;
use tower::ServiceExt;
use uuid::Uuid;

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

async fn create_game(
    app: axum::Router,
    name: &str,
) -> Result<(axum::Router, Uuid), Box<dyn std::error::Error>> {
    let payload = json!({ "name": name }).to_string();
    let request = Request::builder()
        .method("POST")
        .uri("/games")
        .header("content-type", "application/json")
        .body(Body::from(payload))?;

    let response = app.clone().oneshot(request).await?;
    assert_eq!(response.status(), 201);
    let body = axum::body::to_bytes(response.into_body(), usize::MAX).await?;
    let data: serde_json::Value = serde_json::from_slice(&body)?;
    let id = data["id"]
        .as_str()
        .ok_or("missing game id")?
        .parse::<Uuid>()?;

    Ok((app, id))
}

#[tokio::test]
async fn invalid_game_id_returns_not_found() -> Result<(), Box<dyn std::error::Error>> {
    let state = setup_state().await?;
    let app = app(state);

    let payload = json!({ "name": "Stormcast", "game_id": Uuid::new_v4() }).to_string();
    let request = Request::builder()
        .method("POST")
        .uri("/model-definitions")
        .header("content-type", "application/json")
        .body(Body::from(payload))?;

    let response = app.oneshot(request).await?;
    assert_eq!(response.status(), 404);

    Ok(())
}

#[tokio::test]
async fn create_model_definition_returns_created() -> Result<(), Box<dyn std::error::Error>> {
    let state = setup_state().await?;
    let app = app(state);
    let (app, game_id) = create_game(app, "Age of Sigmar").await?;

    let payload = json!({ "name": "Stormcast", "game_id": game_id }).to_string();
    let request = Request::builder()
        .method("POST")
        .uri("/model-definitions")
        .header("content-type", "application/json")
        .body(Body::from(payload))?;

    let response = app.oneshot(request).await?;
    assert_eq!(response.status(), 201);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX).await?;
    let data: serde_json::Value = serde_json::from_slice(&body)?;

    assert_eq!(data["name"], "Stormcast");
    assert_eq!(data["game"]["id"], game_id.to_string());
    assert_eq!(data["game"]["name"], "Age of Sigmar");

    Ok(())
}

#[tokio::test]
async fn list_model_definitions_includes_game_info() -> Result<(), Box<dyn std::error::Error>> {
    let state = setup_state().await?;
    let app = app(state);
    let (app, game_id) = create_game(app, "Star Wars Legion").await?;

    let payload = json!({ "name": "Clone Troopers", "game_id": game_id }).to_string();
    let request = Request::builder()
        .method("POST")
        .uri("/model-definitions")
        .header("content-type", "application/json")
        .body(Body::from(payload))?;
    let response = app.clone().oneshot(request).await?;
    assert_eq!(response.status(), 201);

    let request = Request::builder()
        .method("GET")
        .uri("/model-definitions")
        .body(Body::empty())?;
    let response = app.oneshot(request).await?;
    assert_eq!(response.status(), 200);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX).await?;
    let data: serde_json::Value = serde_json::from_slice(&body)?;
    let list = data.as_array().ok_or("expected array response")?;

    assert_eq!(list.len(), 1);
    assert_eq!(list[0]["name"], "Clone Troopers");
    assert_eq!(list[0]["game"]["id"], game_id.to_string());
    assert_eq!(list[0]["game"]["name"], "Star Wars Legion");

    Ok(())
}
