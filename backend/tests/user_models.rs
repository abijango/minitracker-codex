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

async fn create_model_definition(
    app: axum::Router,
    name: &str,
    game_id: Uuid,
) -> Result<(axum::Router, Uuid), Box<dyn std::error::Error>> {
    let payload = json!({ "name": name, "game_id": game_id }).to_string();
    let request = Request::builder()
        .method("POST")
        .uri("/model-definitions")
        .header("content-type", "application/json")
        .body(Body::from(payload))?;

    let response = app.clone().oneshot(request).await?;
    assert_eq!(response.status(), 201);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX).await?;
    let data: serde_json::Value = serde_json::from_slice(&body)?;
    let id = data["id"]
        .as_str()
        .ok_or("missing model definition id")?
        .parse::<Uuid>()?;

    Ok((app, id))
}

#[tokio::test]
async fn create_user_model_returns_created() -> Result<(), Box<dyn std::error::Error>> {
    let state = setup_state().await?;
    let app = app(state);
    let (app, game_id) = create_game(app, "Conquest").await?;
    let (app, model_definition_id) =
        create_model_definition(app, "Household Guard", game_id).await?;

    let payload = json!({
        "model_definition_id": model_definition_id,
        "quantity": 10,
        "status": "unassembled"
    })
    .to_string();
    let request = Request::builder()
        .method("POST")
        .uri("/user-models")
        .header("content-type", "application/json")
        .body(Body::from(payload))?;

    let response = app.oneshot(request).await?;
    assert_eq!(response.status(), 201);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX).await?;
    let data: serde_json::Value = serde_json::from_slice(&body)?;

    assert_eq!(data["model_definition_id"], model_definition_id.to_string());
    assert_eq!(data["quantity"], 10);
    assert_eq!(data["status"], "unassembled");

    Ok(())
}

#[tokio::test]
async fn update_user_model_status() -> Result<(), Box<dyn std::error::Error>> {
    let state = setup_state().await?;
    let app = app(state);
    let (app, game_id) = create_game(app, "The Old World").await?;
    let (app, model_definition_id) =
        create_model_definition(app, "Empire Knights", game_id).await?;

    let payload = json!({
        "model_definition_id": model_definition_id,
        "quantity": 5,
        "status": "unassembled"
    })
    .to_string();
    let request = Request::builder()
        .method("POST")
        .uri("/user-models")
        .header("content-type", "application/json")
        .body(Body::from(payload))?;
    let response = app.clone().oneshot(request).await?;
    assert_eq!(response.status(), 201);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX).await?;
    let data: serde_json::Value = serde_json::from_slice(&body)?;
    let user_model_id = data["id"]
        .as_str()
        .ok_or("missing user model id")?
        .parse::<Uuid>()?;

    let payload = json!({ "status": "painted" }).to_string();
    let request = Request::builder()
        .method("PATCH")
        .uri(format!("/user-models/{user_model_id}"))
        .header("content-type", "application/json")
        .body(Body::from(payload))?;
    let response = app.oneshot(request).await?;
    assert_eq!(response.status(), 200);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX).await?;
    let data: serde_json::Value = serde_json::from_slice(&body)?;
    assert_eq!(data["status"], "painted");

    Ok(())
}

#[tokio::test]
async fn invalid_status_rejected() -> Result<(), Box<dyn std::error::Error>> {
    let state = setup_state().await?;
    let app = app(state);
    let (app, game_id) = create_game(app, "Malifaux").await?;
    let (app, model_definition_id) =
        create_model_definition(app, "Neverborn", game_id).await?;

    let payload = json!({
        "model_definition_id": model_definition_id,
        "quantity": 3,
        "status": "broken"
    })
    .to_string();
    let request = Request::builder()
        .method("POST")
        .uri("/user-models")
        .header("content-type", "application/json")
        .body(Body::from(payload))?;

    let response = app.oneshot(request).await?;
    assert_eq!(response.status(), 422);

    Ok(())
}
