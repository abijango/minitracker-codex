use axum::{
    extract::State,
    http::StatusCode,
    response::IntoResponse,
    routing::{get, patch, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use serde_json::json;
use sqlx::SqlitePool;
use uuid::Uuid;

#[derive(Clone)]
pub struct AppState {
    pub pool: SqlitePool,
}

pub fn app(state: AppState) -> Router {
    Router::new()
        .route("/health", get(health))
        .route("/games", post(create_game).get(list_games))
        .route(
            "/model-definitions",
            post(create_model_definition).get(list_model_definitions),
        )
        .route("/user-models", post(create_user_model).get(list_user_models))
        .route("/user-models/:id", patch(update_user_model))
        .with_state(state)
}

async fn health() -> Json<serde_json::Value> {
    Json(json!({ "status": "ok" }))
}

#[derive(Deserialize)]
struct CreateGame {
    name: String,
}

#[derive(Serialize)]
struct Game {
    id: Uuid,
    name: String,
    created_at: String,
}

#[derive(Deserialize)]
struct CreateModelDefinition {
    name: String,
    game_id: Uuid,
}

#[derive(Serialize)]
struct ModelDefinition {
    id: Uuid,
    name: String,
    game: GameSummary,
}

#[derive(Serialize)]
struct GameSummary {
    id: Uuid,
    name: String,
}

#[derive(Deserialize)]
struct CreateUserModel {
    model_definition_id: Uuid,
    quantity: i32,
    status: Status,
}

#[derive(Deserialize)]
struct UpdateUserModel {
    status: Status,
}

#[derive(Serialize)]
struct UserModel {
    id: Uuid,
    model_definition_id: Uuid,
    quantity: i32,
    status: Status,
    created_at: String,
    updated_at: String,
}

#[derive(Serialize)]
struct UserModelListItem {
    id: Uuid,
    model_name: String,
    game_name: String,
    quantity: i32,
    status: Status,
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
enum Status {
    Unassembled,
    Assembled,
    Painted,
}

impl Status {
    fn as_str(self) -> &'static str {
        match self {
            Status::Unassembled => "unassembled",
            Status::Assembled => "assembled",
            Status::Painted => "painted",
        }
    }
}

impl std::str::FromStr for Status {
    type Err = ();

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            "unassembled" => Ok(Status::Unassembled),
            "assembled" => Ok(Status::Assembled),
            "painted" => Ok(Status::Painted),
            _ => Err(()),
        }
    }
}

async fn create_game(
    State(state): State<AppState>,
    Json(payload): Json<CreateGame>,
) -> Result<(StatusCode, Json<Game>), AppError> {
    tracing::info!(name = %payload.name, "creating game");
    let id = Uuid::new_v4();
    let id_value = id.to_string();

    sqlx::query!(
        r#"
        INSERT INTO games (id, name, created_at)
        VALUES ($1, $2, datetime('now'))
        "#,
        id_value,
        payload.name
    )
    .execute(&state.pool)
    .await
    .map_err(map_db_error)?;

    let record = sqlx::query!(
        r#"
        SELECT
            id as "id!: String",
            name,
            created_at as "created_at!: String"
        FROM games
        WHERE id = $1
        "#,
        id_value
    )
    .fetch_one(&state.pool)
    .await
    .map_err(|error| AppError::internal("failed to load game", error))?;

    let game = Game {
        id: parse_uuid(record.id)?,
        name: record.name,
        created_at: record.created_at,
    };

    Ok((StatusCode::CREATED, Json(game)))
}

async fn list_games(State(state): State<AppState>) -> Result<Json<Vec<Game>>, AppError> {
    tracing::info!("listing games");
    let records = sqlx::query!(
        r#"
        SELECT
            id as "id!: String",
            name,
            created_at as "created_at!: String"
        FROM games
        ORDER BY created_at
        "#
    )
    .fetch_all(&state.pool)
    .await
    .map_err(|error| AppError::internal("failed to fetch games", error))?;

    let games = records
        .into_iter()
        .map(|record| {
            Ok(Game {
                id: parse_uuid(record.id)?,
                name: record.name,
                created_at: record.created_at,
            })
        })
        .collect::<Result<Vec<_>, AppError>>()?;

    tracing::info!(count = games.len(), "listed games");
    Ok(Json(games))
}

async fn create_model_definition(
    State(state): State<AppState>,
    Json(payload): Json<CreateModelDefinition>,
) -> Result<(StatusCode, Json<ModelDefinition>), AppError> {
    tracing::info!(
        name = %payload.name,
        game_id = %payload.game_id,
        "creating model definition"
    );
    let game_id_value = payload.game_id.to_string();
    let game = sqlx::query!(
        r#"
        SELECT
            id as "id!: String",
            name
        FROM games
        WHERE id = $1
        "#,
        game_id_value
    )
    .fetch_optional(&state.pool)
    .await
    .map_err(|error| AppError::internal("failed to validate game", error))?;

    let game = match game {
        Some(game) => game,
        None => return Err(AppError::not_found("game not found")),
    };

    let id = Uuid::new_v4();
    let id_value = id.to_string();
    let game_id_value = payload.game_id.to_string();
    sqlx::query!(
        r#"
        INSERT INTO model_definitions (id, name, game_id, created_at)
        VALUES ($1, $2, $3, datetime('now'))
        "#,
        id_value,
        payload.name,
        game_id_value
    )
    .execute(&state.pool)
    .await
    .map_err(|error| AppError::internal("failed to create model definition", error))?;

    let record = sqlx::query!(
        r#"
        SELECT
            id as "id!: String",
            name,
            game_id as "game_id!: String"
        FROM model_definitions
        WHERE id = $1
        "#,
        id_value
    )
    .fetch_one(&state.pool)
    .await
    .map_err(|error| AppError::internal("failed to load model definition", error))?;

    let model_definition = ModelDefinition {
        id: parse_uuid(record.id)?,
        name: record.name,
        game: GameSummary {
            id: parse_uuid(game.id)?,
            name: game.name,
        },
    };

    Ok((StatusCode::CREATED, Json(model_definition)))
}

async fn list_model_definitions(
    State(state): State<AppState>,
) -> Result<Json<Vec<ModelDefinition>>, AppError> {
    tracing::info!("listing model definitions");
    let records = sqlx::query!(
        r#"
        SELECT
            model_definitions.id as "id!: String",
            model_definitions.name,
            games.id AS "game_id!: String",
            games.name AS game_name
        FROM model_definitions
        INNER JOIN games ON games.id = model_definitions.game_id
        ORDER BY model_definitions.created_at
        "#
    )
    .fetch_all(&state.pool)
    .await
    .map_err(|error| AppError::internal("failed to fetch model definitions", error))?;

    let models = records
        .into_iter()
        .map(|record| {
            Ok(ModelDefinition {
                id: parse_uuid(record.id)?,
                name: record.name,
                game: GameSummary {
                    id: parse_uuid(record.game_id)?,
                    name: record.game_name,
                },
            })
        })
        .collect::<Result<Vec<_>, AppError>>()?;

    tracing::info!(count = models.len(), "listed model definitions");
    Ok(Json(models))
}

async fn create_user_model(
    State(state): State<AppState>,
    Json(payload): Json<CreateUserModel>,
) -> Result<(StatusCode, Json<UserModel>), AppError> {
    tracing::info!(
        model_definition_id = %payload.model_definition_id,
        quantity = payload.quantity,
        status = payload.status.as_str(),
        "creating user model"
    );
    let model_definition_id_value = payload.model_definition_id.to_string();
    let exists = sqlx::query!(
        r#"
        SELECT id
        FROM model_definitions
        WHERE id = $1
        "#,
        model_definition_id_value
    )
    .fetch_optional(&state.pool)
    .await
    .map_err(|error| AppError::internal("failed to validate model definition", error))?;

    if exists.is_none() {
        return Err(AppError::not_found("model definition not found"));
    }

    let id = Uuid::new_v4();
    let id_value = id.to_string();
    let status_value = payload.status.as_str();
    sqlx::query!(
        r#"
        INSERT INTO user_models (id, model_definition_id, quantity, status, created_at, updated_at)
        VALUES ($1, $2, $3, $4, datetime('now'), datetime('now'))
        "#,
        id_value,
        model_definition_id_value,
        payload.quantity,
        status_value
    )
    .execute(&state.pool)
    .await
    .map_err(|error| AppError::internal("failed to create user model", error))?;

    let record = sqlx::query!(
        r#"
        SELECT
            id as "id!: String",
            model_definition_id as "model_definition_id!: String",
            quantity as "quantity!: i64",
            status,
            created_at as "created_at!: String",
            updated_at as "updated_at!: String"
        FROM user_models
        WHERE id = $1
        "#,
        id_value
    )
    .fetch_one(&state.pool)
    .await
    .map_err(|error| AppError::internal("failed to load user model", error))?;

    let status = parse_status(&record.status)?;
    let user_model = UserModel {
        id: parse_uuid(record.id)?,
        model_definition_id: parse_uuid(record.model_definition_id)?,
        quantity: parse_i32(record.quantity)?,
        status,
        created_at: record.created_at,
        updated_at: record.updated_at,
    };

    Ok((StatusCode::CREATED, Json(user_model)))
}

async fn list_user_models(
    State(state): State<AppState>,
) -> Result<Json<Vec<UserModelListItem>>, AppError> {
    tracing::info!("listing user models");
    let records = sqlx::query!(
        r#"
        SELECT
            user_models.id as "id!: String",
            model_definitions.name AS model_name,
            games.name AS game_name,
            user_models.quantity as "quantity!: i64",
            user_models.status
        FROM user_models
        INNER JOIN model_definitions ON model_definitions.id = user_models.model_definition_id
        INNER JOIN games ON games.id = model_definitions.game_id
        ORDER BY user_models.created_at
        "#
    )
    .fetch_all(&state.pool)
    .await
    .map_err(|error| AppError::internal("failed to fetch user models", error))?;

    let models = records
        .into_iter()
        .map(|record| {
            let status = parse_status(&record.status)?;
            Ok(UserModelListItem {
                id: parse_uuid(record.id)?,
                model_name: record.model_name,
                game_name: record.game_name,
                quantity: parse_i32(record.quantity)?,
                status,
            })
        })
        .collect::<Result<Vec<_>, AppError>>()?;

    tracing::info!(count = models.len(), "listed user models");
    Ok(Json(models))
}

async fn update_user_model(
    State(state): State<AppState>,
    axum::extract::Path(id): axum::extract::Path<Uuid>,
    Json(payload): Json<UpdateUserModel>,
) -> Result<Json<UserModel>, AppError> {
    tracing::info!(
        user_model_id = %id,
        status = payload.status.as_str(),
        "updating user model"
    );
    let status_value = payload.status.as_str();
    let id_value = id.to_string();
    sqlx::query!(
        r#"
        UPDATE user_models
        SET status = $1,
            updated_at = datetime('now')
        WHERE id = $2
        "#,
        status_value,
        id_value
    )
    .execute(&state.pool)
    .await
    .map_err(|error| AppError::internal("failed to update user model", error))?;

    let record = sqlx::query!(
        r#"
        SELECT
            id as "id!: String",
            model_definition_id as "model_definition_id!: String",
            quantity as "quantity!: i64",
            status,
            created_at as "created_at!: String",
            updated_at as "updated_at!: String"
        FROM user_models
        WHERE id = $1
        "#,
        id_value
    )
    .fetch_optional(&state.pool)
    .await
    .map_err(|error| AppError::internal("failed to load user model", error))?;

    let record = match record {
        Some(record) => record,
        None => return Err(AppError::not_found("user model not found")),
    };

    let status = parse_status(&record.status)?;
    let user_model = UserModel {
        id: parse_uuid(record.id)?,
        model_definition_id: parse_uuid(record.model_definition_id)?,
        quantity: parse_i32(record.quantity)?,
        status,
        created_at: record.created_at,
        updated_at: record.updated_at,
    };

    Ok(Json(user_model))
}

struct AppError {
    status: StatusCode,
    message: &'static str,
}

impl AppError {
    fn conflict(message: &'static str) -> Self {
        Self {
            status: StatusCode::CONFLICT,
            message,
        }
    }

    fn internal(message: &'static str, error: sqlx::Error) -> Self {
        tracing::error!(%error, "{message}");
        Self {
            status: StatusCode::INTERNAL_SERVER_ERROR,
            message,
        }
    }

    fn internal_message(message: &'static str) -> Self {
        tracing::error!("{message}");
        Self {
            status: StatusCode::INTERNAL_SERVER_ERROR,
            message,
        }
    }

    fn not_found(message: &'static str) -> Self {
        Self {
            status: StatusCode::NOT_FOUND,
            message,
        }
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> axum::response::Response {
        let body = Json(json!({ "error": self.message }));
        (self.status, body).into_response()
    }
}

fn map_db_error(error: sqlx::Error) -> AppError {
    if let sqlx::Error::Database(db_error) = &error {
        let message = db_error.message();
        if message.contains("UNIQUE constraint failed: games.name") {
            return AppError::conflict("game name already exists");
        }
    }

    AppError::internal("failed to create game", error)
}

fn parse_status(value: &str) -> Result<Status, AppError> {
    value
        .parse::<Status>()
        .map_err(|_| AppError::internal_message("invalid status stored in database"))
}

fn parse_uuid(value: String) -> Result<Uuid, AppError> {
    Uuid::parse_str(&value).map_err(|_| AppError::internal_message("invalid id stored in database"))
}

fn parse_i32(value: i64) -> Result<i32, AppError> {
    i32::try_from(value)
        .map_err(|_| AppError::internal_message("invalid quantity stored in database"))
}
