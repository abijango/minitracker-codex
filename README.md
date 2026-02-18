# Mini Tracker

Rust fullstack app for tracking miniature models. Current stack:
- Backend: Axum + SQLx (SQLite)
- Frontend: Leptos CSR (Trunk)

## Current Progress
- Backend API: games, model definitions, user models (create/list/update status)
- SQLite migrations and in-memory SQLite integration tests
- Frontend MVP: list models, create models, inline status update
- Makefile for common tasks (db init, tests, frontend build/serve)

## Requirements
- Rust toolchain
- SQLite (`sqlite3` CLI)
- Trunk (for frontend)

Install Trunk + wasm target:
```powershell
cargo install trunk
rustup target add wasm32-unknown-unknown
```

## Makefile Usage
From the project root (`mini-tracker/`):

- Initialize database schema:
```powershell
make db-init
```
- Run backend tests:
```powershell
make backend-test
```
- Run backend server:
```powershell
make backend-run
```
- Build frontend:
```powershell
make frontend-build
```
- Serve frontend:
```powershell
make frontend-serve
```
- Run full flow (db init + backend tests + frontend build):
```powershell
make all
```

The Makefile auto-detects Windows (PowerShell) vs Bash/macOS and uses the correct commands.

## Backend
- Location: `backend/`
- Database: SQLite file (`mini-tracker.db`) in `backend/`
- `DATABASE_URL` should be set to:
  - Windows example:
  ```powershell
  $env:DATABASE_URL = "sqlite://mini-tracker.db"
  ```

### Run
```powershell
cd backend
sqlite3 mini-tracker.db ".read migrations/20260218110000_init_schema.sql"
cargo run
```

### Test
```powershell
cd backend
$env:DATABASE_URL = "sqlite://mini-tracker.db"
cargo test
```

## Frontend
- Location: `frontend/`
- Uses Trunk and Leptos CSR
- API proxy is configured in `frontend/Trunk.toml` to `/api`

### Run
```powershell
cd frontend
trunk serve
```

## Notes
- Backend runs on `http://localhost:3000`
- Frontend dev server runs on `http://localhost:8080`
- No Postgres dependency: all storage is SQLite
