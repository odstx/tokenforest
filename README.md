# TokenForest 🌲🪙

A modern full-stack token management application built with Rust and Svelte.

## Features

- 🦀 **Rust Backend** - High-performance API with Axum + SQLx
- ⚡ **Svelte Frontend** - Reactive UI with Bun runtime
- 💾 **SQLite Database** - Zero-configuration data storage
- 🎨 **Modern Design** - Glassmorphism UI with responsive layout
- 🚀 **One-Command Start** - `make dev` runs everything

## Quick Start

```bash
# One-command development setup
make dev

# Or see all available commands
make help
```

## Tech Stack

**Backend**
- Rust + Axum (web framework)
- SQLx + SQLite (database)
- Tokio (async runtime)
- Serde (serialization)

**Frontend**
- SvelteKit + Vite
- Bun runtime (10x faster than npm!)
- TypeScript
- Glassmorphism CSS

## Project Structure

```
tokenforest/
├── backend/          # Rust API server
│   ├── src/
│   │   ├── main.rs
│   │   ├── handlers.rs
│   │   └── db.rs
│   └── Cargo.toml
├── frontend/         # Svelte web app
│   ├── src/
│   │   └── routes/
│   ├── package.json
│   └── bun.lockb
├── database/         # SQL schemas
├── docs/            # Documentation
├── Makefile         # Build commands
└── README.md
```

## Available Commands

| Command | Description |
|---------|-------------|
| `make dev` | Start both backend & frontend |
| `make dev-backend` | Start backend only |
| `make dev-frontend` | Start frontend only |
| `make install-backend` | Build backend |
| `make install-frontend` | Install frontend deps |
| `make build-backend` | Release build (backend) |
| `make build-frontend` | Production build (frontend) |
| `make clean` | Remove build artifacts |
| `make help` | Show all commands |

## License

GPL-3.0 - See [LICENSE](LICENSE) for details.

---

Built with ❤️ using Rust, Svelte, and Bun
