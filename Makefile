.PHONY: db-init backend-test backend-run frontend-build frontend-serve dev all

BACKEND_DIR = backend
FRONTEND_DIR = frontend
DB_FILE = mini-tracker.db

ifeq ($(OS),Windows_NT)
SHELL := powershell.exe
.SHELLFLAGS := -NoProfile -Command
DB_PATH = $(BACKEND_DIR)\$(DB_FILE)
MIGRATIONS_PATH = $(BACKEND_DIR)\migrations\20260218110000_init_schema.sql
define SET_DB_URL
$$env:DATABASE_URL = "sqlite:///$(CURDIR)\$(DB_PATH)";
endef
define DB_INIT
Get-Content $(MIGRATIONS_PATH) | sqlite3 $(DB_PATH)
endef
else
SHELL := /bin/bash
.SHELLFLAGS := -e -c
DB_PATH = $(BACKEND_DIR)/$(DB_FILE)
MIGRATIONS_PATH = $(BACKEND_DIR)/migrations/20260218110000_init_schema.sql
define SET_DB_URL
DATABASE_URL="sqlite://$(DB_PATH)"
endef
define DB_INIT
sqlite3 $(DB_PATH) < $(MIGRATIONS_PATH)
endef
endif

db-init:
	@$(DB_INIT)

backend-test:
	@cd $(BACKEND_DIR) && $(SET_DB_URL) cargo test

backend-run:
	@cd $(BACKEND_DIR) && $(SET_DB_URL) cargo run

frontend-build:
	@cd $(FRONTEND_DIR) && trunk build

frontend-serve:
	@cd $(FRONTEND_DIR) && trunk serve

dev:
	@$(DB_INIT)
ifeq ($(OS),Windows_NT)
	@cd $(BACKEND_DIR); Start-Process powershell -ArgumentList "-NoProfile -Command $(SET_DB_URL) cargo run"
	@cd $(FRONTEND_DIR); Start-Process powershell -ArgumentList "-NoProfile -Command trunk serve"
else
	@cd $(BACKEND_DIR) && $(SET_DB_URL) cargo run &
	@cd $(FRONTEND_DIR) && trunk serve
endif

all: db-init backend-test frontend-build
