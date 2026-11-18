# co development tasks. Run inside `nix develop` (or direnv).

set dotenv-load

default:
    @just --list

# Install JS dependencies
install:
    pnpm install

# Start the dev Postgres (docker compose) and wait until it is ready
db:
    docker compose up -d --wait postgres

# Wipe and recreate the dev database
db-reset:
    docker compose down -v
    just db

# Open psql on the dev database
psql:
    psql "$DATABASE_URL"

# Add a migration: just migration add_sessions
migration name:
    sqlx migrate add --source crates/store/migrations {{name}}

# Run the server (applies migrations on start)
serve: db
    cargo run -p co-cli -- serve

# Run a tool host for this machine
tool-host:
    cargo run -p co-cli -- tool-host

# Run the TUI
tui:
    cargo run -p co-cli

# Run the web frontend (Vite dev server on :5173)
web:
    pnpm --filter @co/web dev

# Server + web together
dev: db
    pnpm exec concurrently -k -n server,web -c blue,green "cargo run -p co-cli -- serve" "pnpm --filter @co/web dev"

# Run the desktop app (Tauri; starts the web dev server itself)
desktop:
    pnpm --filter @co/desktop dev

# Build the desktop bundles for this platform
desktop-build:
    pnpm --filter @co/desktop build

# Regenerate the TypeScript protocol types from crates/protocol
codegen:
    rm -rf packages/protocol/src/generated
    cargo test -p co-protocol export_bindings

# Build everything except the desktop app
build:
    cargo build
    pnpm -r build

# Format everything
fmt:
    cargo fmt --all
    pnpm fmt

# Lint, typecheck and check formatting (what CI runs)
check:
    cargo fmt --all --check
    cargo clippy --all-targets -- -D warnings
    pnpm lint
    pnpm typecheck

# Run all tests
test:
    cargo test
    pnpm test
