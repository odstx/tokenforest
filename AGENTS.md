# AGENTS.md - TokenForest Project Guide

## Project Overview

TokenForest is a full-stack token management application with:
- **Backend**: Rust + Axum + SQLx + SQLite
- **Frontend**: SvelteKit + Vite + TypeScript + Tailwind CSS + DaisyUI
- **Runtime**: Bun (frontend), Tokio (backend async)

## Development Commands

### Start Development
```bash
make dev              # Start both backend (port 3000) and frontend (port 5173)
make dev-backend      # Start backend only
make dev-frontend     # Start frontend only
```

### Build
```bash
make install-backend  # Build backend (debug)
make install-frontend # Install frontend dependencies
make build-backend    # Release build (backend)
make build-frontend   # Production build (frontend)
```

### Lint & Typecheck
```bash
# Frontend
cd frontend && bun run check    # Svelte type check

# Backend
cd backend && cargo clippy      # Rust linter
cd backend && cargo test        # Run tests
```

### Clean
```bash
make clean            # Remove all build artifacts
```

## Project Structure

```
tokenforest/
├── backend/           # Rust API server
│   ├── src/
│   │   ├── main.rs    # Entry point
│   │   ├── handlers.rs # HTTP handlers
│   │   └── db.rs      # Database operations
│   └── Cargo.toml
├── frontend/          # SvelteKit web app
│   ├── src/
│   │   ├── routes/    # SvelteKit routes
│   │   └── lib/       # Shared components/utilities
│   └── package.json
├── database/          # SQL schemas
├── docs/              # Documentation
├── Makefile           # Build commands
└── README.md
```

## Code Conventions

### Backend (Rust)
- Use Axum for HTTP handlers
- Use SQLx for database operations with SQLite
- Follow standard Rust formatting (cargo fmt)
- Run clippy before committing: `cargo clippy -- -D warnings`

### Frontend (TypeScript/Svelte)
- SvelteKit file-based routing
- Tailwind CSS + DaisyUI for styling
- TypeScript strict mode
- Run type check: `bun run check`

## Ports

- Backend API: `http://localhost:3000`
- Frontend Dev: `http://localhost:5173`

## Notes

- Uses SQLite for zero-configuration data storage
- Frontend uses Bun runtime (faster than npm)
- One-command start with `make dev`
- Every feature requires corresponding automated tests.
- Automated test pass rate is required to be 100%.
