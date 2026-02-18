CREATE TABLE IF NOT EXISTS games (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL UNIQUE,
    created_at TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS model_definitions (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    game_id TEXT NOT NULL REFERENCES games(id) ON DELETE CASCADE,
    created_at TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS user_models (
    id TEXT PRIMARY KEY,
    model_definition_id TEXT NOT NULL REFERENCES model_definitions(id) ON DELETE CASCADE,
    quantity INTEGER NOT NULL,
    status TEXT NOT NULL,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);
