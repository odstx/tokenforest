# TokenForest

[English](README.md) | [中文](README_CN.md)

[![License: GPL v3](https://img.shields.io/badge/License-GPLv3-blue.svg)](https://www.gnu.org/licenses/gpl-3.0)

A modern full-stack application for managing AI API keys and token pools.

## Features

- User authentication with JWT
- API key management with CIDR restrictions
- Token pool management for multiple AI models
- Dashboard with usage statistics
- Swagger UI API documentation
- Responsive web interface

## Tech Stack

| Layer | Technologies |
|-------|-------------|
| Backend | Rust, Axum, SQLx, SQLite, Tokio |
| Frontend | SvelteKit, TypeScript, Tailwind CSS, DaisyUI |
| Runtime | Bun (frontend), Tokio (backend) |

## Quick Start

```bash
make dev
```

| Service | URL |
|---------|-----|
| Frontend | http://localhost:5173 |
| Backend API | http://localhost:3000 |
| Swagger UI | http://localhost:3000/swagger-ui |

## API Reference

### Auth
| Method | Endpoint | Description |
|--------|----------|-------------|
| POST | `/api/auth/register` | Register user |
| POST | `/api/auth/login` | Login |

### API Keys
| Method | Endpoint | Description |
|--------|----------|-------------|
| GET | `/api/api-keys` | List keys |
| POST | `/api/api-keys` | Create key |
| DELETE | `/api/api-keys/:id` | Delete key |
| PUT | `/api/api-keys/:id/toggle` | Toggle key |

### Token Pools
| Method | Endpoint | Description |
|--------|----------|-------------|
| GET | `/api/token-pools` | List pools |
| POST | `/api/token-pools` | Create pool |
| PUT | `/api/token-pools/:id` | Update pool |
| DELETE | `/api/token-pools/:id` | Delete pool |
| PUT | `/api/token-pools/:id/toggle` | Toggle pool |
| POST | `/api/token-pools/:id/test` | Test connection |

## Development

| Command | Description |
|---------|-------------|
| `make dev` | Start backend + frontend |
| `make build-backend` | Release build |
| `make build-frontend` | Production build |
| `make clean` | Remove artifacts |

```bash
# Lint & test
cd frontend && bun run check
cd backend && cargo clippy && cargo test
```

## Configuration

Backend `.dev.env`:
```
DATABASE_URL=sqlite:./database/tokenforest.db?mode=rwc
JWT_SECRET=your-secret-key
HOST=0.0.0.0
PORT=3000
```

## License

[GPL-3.0](LICENSE)
