# TokenForest Backend

Rust backend for TokenForest web application.

## Tech Stack
- **Framework**: Axum (web framework)
- **Database**: SQLite with SQLx
- **Runtime**: Tokio

## Setup

```bash
# Install dependencies
cargo build

# Run the server
cargo run
```

## Environment Variables

Create a `.env` file in the backend directory:

```env
DATABASE_URL=./database/tokenforest.db
RUST_LOG=debug
```

## API Endpoints

- `GET /` - Welcome message
- `GET /api/tokens` - List all tokens
- `POST /api/tokens` - Create a new token

## Development

```bash
# Run with auto-reload (requires cargo-watch)
cargo watch -x run

# Run tests
cargo test
```
