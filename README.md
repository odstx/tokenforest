# TokenForest

A modern full-stack token management application for managing AI API keys and token pools.

## Features

- **User Authentication** - Register, login with JWT-based sessions
- **API Key Management** - Create, list, toggle, delete API keys
- **Token Pool Management** - Manage token pools with different AI models
- **Dashboard Stats** - Overview of API keys and token pools counts
- **Swagger UI** - Interactive API documentation at `/swagger-ui`
- **Responsive Design** - Works on desktop and mobile

## Tech Stack

**Backend**
- Rust + Axum (web framework)
- SQLx + SQLite (database)
- Tokio (async runtime)
- utoipa + swagger-ui (OpenAPI docs)
- JWT for authentication

**Frontend**
- SvelteKit + Vite
- Bun runtime
- TypeScript
- Tailwind CSS + DaisyUI

## Quick Start

```bash
# Start both backend and frontend
make dev

# Or see all available commands
make help
```

This starts:
- Backend API at http://localhost:3000
- Frontend at http://localhost:5173
- Swagger UI at http://localhost:3000/swagger-ui

## API Endpoints

### Authentication
| Method | Endpoint | Description |
|--------|----------|-------------|
| POST | `/api/auth/register` | Register new user |
| POST | `/api/auth/login` | Login and get JWT token |

### API Keys (requires auth)
| Method | Endpoint | Description |
|--------|----------|-------------|
| GET | `/api/api-keys` | List all API keys |
| POST | `/api/api-keys` | Create new API key |
| DELETE | `/api/api-keys/:id` | Delete API key |
| PUT | `/api/api-keys/:id/toggle` | Enable/disable API key |

### Token Pools (requires auth)
| Method | Endpoint | Description |
|--------|----------|-------------|
| GET | `/api/token-pools` | List all token pools |
| POST | `/api/token-pools` | Create new token pool |
| PUT | `/api/token-pools/:id` | Update token pool |
| DELETE | `/api/token-pools/:id` | Delete token pool |
| PUT | `/api/token-pools/:id/toggle` | Enable/disable token pool |
| POST | `/api/token-pools/:id/test` | Test token pool connection |

### Stats (requires auth)
| Method | Endpoint | Description |
|--------|----------|-------------|
| GET | `/api/stats` | Get API keys and token pools counts |

## Project Structure

```
tokenforest/
├── backend/
│   ├── src/
│   │   ├── main.rs        # Entry point, routes, OpenAPI spec
│   │   ├── handlers.rs    # HTTP request handlers
│   │   ├── models.rs      # Data models
│   │   ├── db.rs          # Database migrations
│   │   ├── auth.rs        # JWT authentication middleware
│   │   └── crypto.rs      # Password hashing
│   └── Cargo.toml
├── frontend/
│   ├── src/
│   │   ├── routes/
│   │   │   ├── +page.svelte       # Homepage with stats dashboard
│   │   │   ├── +layout.svelte     # Layout with navigation
│   │   │   ├── login/             # Login page
│   │   │   ├── register/          # Registration page
│   │   │   ├── api-keys/          # API keys management
│   │   │   └── token-pools/       # Token pools management
│   │   └── lib/                   # Shared components
│   └── package.json
├── database/
│   └── tokenforest.db     # SQLite database (auto-created)
├── docs/
├── Makefile
└── README.md
```

## Development Commands

| Command | Description |
|---------|-------------|
| `make dev` | Start backend (port 3000) & frontend (port 5173) |
| `make dev-backend` | Start backend only |
| `make dev-frontend` | Start frontend only |
| `make install-backend` | Build backend (debug) |
| `make install-frontend` | Install frontend dependencies |
| `make build-backend` | Release build (backend) |
| `make build-frontend` | Production build (frontend) |
| `make clean` | Remove build artifacts |

### Lint & Typecheck

```bash
# Frontend
cd frontend && bun run check

# Backend
cd backend && cargo clippy
cd backend && cargo test
```

## Environment Variables

Backend (`.dev.env`):
```
DATABASE_URL=sqlite:./database/tokenforest.db?mode=rwc
JWT_SECRET=your-secret-key
HOST=0.0.0.0
PORT=3000
```

## License

GPL-3.0 - See [LICENSE](LICENSE) for details.
