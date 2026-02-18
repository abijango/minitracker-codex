# Mini Tracker – Phased Execution Plan (Phase 2+)

You are continuing development of an existing Rust fullstack project:

mini-tracker/
  backend/
  frontend/

Backend Phase 1 is complete and functional.

You must now execute development from Phase 2 onward.

---

# GLOBAL RULES (MANDATORY)

1. Do NOT implement future phases early.
2. Complete one full phase at a time.
3. After completing a phase:
   - Print:
     - Summary of what was built
     - Files created/modified
     - How to run
     - Test status
     - Known tradeoffs
   - Then STOP.
   - Ask: "Proceed to next phase?"
4. Wait for confirmation before continuing.
5. Keep code minimal.
6. No unnecessary abstractions.
7. No unused dependencies.
8. No unwrap() in production paths.
9. Keep WASM bundle lean.
10. Prefer explicit SQL over heavy ORM patterns.

---

# DATABASE NOTE (SQLITE)

This project uses SQLite for local persistence.

If switching from Postgres to SQLite in the future, update:
- `sqlx` features: remove `postgres`, add `sqlite`, keep `macros` + `migrate`.
- Connection string: `DATABASE_URL=sqlite://path/to/db.sqlite`.
- Migrations: change `UUID`/`TIMESTAMP` to `TEXT`, use `datetime('now')` instead of `NOW()`.
- Error handling: adapt unique constraint detection to SQLite messages.
- Tests: use in-memory SQLite and enable `PRAGMA foreign_keys = ON`.

---

# PHASE 2 — FRONTEND MVP UI

## Goal

Provide a working UI for:

- Listing user models
- Creating user models
- Updating status inline

No autocomplete.
No offline support.
No IndexedDB.
No hierarchy yet.

---

## Tasks

### 1. Initialize Leptos Frontend (CSR only)

- Ensure frontend project compiles
- Render simple heading: "Mini Tracker"
- Confirm trunk serve works

---

### 2. Fetch and Render Data

Implement:

- Fetch GET /games
- Fetch GET /user-models

Display table:

Columns:
- Model Name
- Game
- Quantity
- Status

Requirements:
- Loading state
- Basic error state
- No UI libraries

---

### 3. Add Model Creation Form

Fields:
- Model name (text input)
- Game dropdown
- Quantity (default 1)
- Status dropdown (default Unassembled)

On submit:
1. Create model definition if needed
2. Create user model
3. Refresh list

Validation:
- Quantity > 0
- Model name not empty

---

### 4. Inline Status Update

- Replace status cell with dropdown
- PATCH /user-models/{id}
- Optimistic UI update
- Revert on failure

---

## Completion Criteria

- Full stack works end-to-end
- Can create and update models
- No console errors
- No clippy warnings

After completion:

STOP.
Print summary.
Ask to proceed.

---

# PHASE 3 — HIERARCHICAL CLASSIFICATION

## Goal

Add:

- Factions
- SubFactions
- Hierarchical filtering in frontend

---

## Backend Changes

1. Add migrations:

factions:
- id UUID PK
- game_id UUID REFERENCES games(id)
- name TEXT NOT NULL

subfactions:
- id UUID PK
- faction_id UUID REFERENCES factions(id)
- name TEXT NOT NULL

2. Update model_definitions:
- Add faction_id
- Add subfaction_id (nullable)

3. Update API:
- Accept hierarchy on create
- Return full joined hierarchy on GET

4. Update integration tests

---

## Frontend Changes

1. Fetch:
- Factions
- SubFactions

2. Update form:
- Game dropdown
- Faction dropdown (filtered by game)
- SubFaction dropdown (filtered by faction)

3. Filtering must be reactive.

---

## Completion Criteria

- Hierarchy persists correctly
- Filtering works
- No reload required
- Tests updated and passing

STOP.
Print summary.
Ask to proceed.

---

# PHASE 4 — LOCAL DB + AUTOCOMPLETE

## Goal

Introduce local IndexedDB layer and fast typeahead search.

---

## Tasks

### 1. Add IndexedDB Wrapper

Use rexie.

Local stores:
- model_definitions
- games
- factions
- subfactions

Expose minimal API:
- init()
- bulk_insert()
- search_by_prefix()

---

### 2. Initial Sync

On app load:
- Fetch backend data
- Populate IndexedDB
- Avoid duplicates

---

### 3. Typeahead Search

Modify model name input:

- On input:
  - Debounce
  - Query IndexedDB
  - Show suggestions
- On selection:
  - Auto-fill hierarchy

Performance requirement:
- <20ms search for 5000 entries

---

### 4. Inline Creation

If no suggestion matches:
- Show "Create new: <typed>"
- Insert locally
- POST to backend

---

## Completion Criteria

- Autocomplete instant
- Local DB works
- No backend required for search
- Clean minimal implementation

STOP.
Print summary.
Ask to proceed.

---

# PHASE 5 — OFFLINE SYNC

## Goal

Enable local edits while offline.

---

## Tasks

### 1. Extend Local Schema

Add:
- is_dirty: bool
- last_modified: timestamp

Mark dirty on changes.

---

### 2. Background Sync Worker

Trigger:
- Every 30 seconds
- On reconnect event

Behavior:
- Push dirty records
- Clear on success
- Retry with backoff

---

### 3. Conflict Resolution

Strategy:
- Last write wins (timestamp comparison)

Add simple conflict test.

---

## Completion Criteria

- Offline edits persist
- Sync works after reconnect
- Dirty flags clear properly

STOP.
Print summary.
Ask to proceed.

---

# PHASE 6 — DASHBOARD

## Goal

Add aggregate analytics.

---

## Backend

Add endpoints:

GET /dashboard/status-summary
GET /dashboard/game-summary

Return:
- Total quantity by status
- Total quantity by game

Add integration tests.

---

## Frontend

Add route:

/dashboard

Display:
- Total models
- Status breakdown
- Game breakdown

Use simple CSS bars.
No chart libraries.

---

## Completion Criteria

- Aggregations correct
- UI minimal and clean
- No heavy libraries

STOP.
Print final project summary:
- Project tree
- LOC estimate
- Dependencies
- Performance notes
- Known limitations
