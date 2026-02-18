# Mini Tracker Backend

## Setup
- Set `DATABASE_URL` in your environment.
- Example (PowerShell):
```powershell
$env:DATABASE_URL = "sqlite://mini-tracker.db"
```
- Initialize the database schema (required for SQLx compile-time checks):
```powershell
sqlite3 mini-tracker.db < migrations/20260218110000_init_schema.sql
```

## Migrations
- The server runs SQLx migrations on startup.

## Run
```powershell
cargo run
```

## Tests
```powershell
cargo test
```
