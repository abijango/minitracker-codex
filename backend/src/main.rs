use sqlx::sqlite::SqlitePoolOptions;
use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() {
    init_tracing();

    let database_url = match std::env::var("DATABASE_URL") {
        Ok(url) => url,
        Err(error) => {
            tracing::error!(%error, "DATABASE_URL is not set");
            return;
        }
    };

    let pool = match SqlitePoolOptions::new()
        .max_connections(5)
        .after_connect(|connection, _| {
            Box::pin(async move {
                sqlx::query("PRAGMA foreign_keys = ON;")
                    .execute(connection)
                    .await
                    .map(|_| ())
            })
        })
        .connect(&database_url)
        .await
    {
        Ok(pool) => pool,
        Err(error) => {
            tracing::error!(%error, "failed to connect to database");
            return;
        }
    };

    if let Err(error) = sqlx::migrate!().run(&pool).await {
        tracing::error!(%error, "failed to run database migrations");
        return;
    }

    let app_state = backend::AppState { pool };
    let app = backend::app(app_state);

    let listener = match tokio::net::TcpListener::bind("0.0.0.0:3000").await {
        Ok(listener) => listener,
        Err(error) => {
            tracing::error!(%error, "failed to bind server address");
            return;
        }
    };

    if let Err(error) = axum::serve(listener, app).await {
        tracing::error!(%error, "server error");
    }
}

fn init_tracing() {
    let env_filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));

    tracing_subscriber::fmt().with_env_filter(env_filter).init();
}
