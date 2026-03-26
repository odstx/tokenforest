# TokenForest 🌲

A modern web application for managing tokens, built with Rust, Svelte, and SQLite.

## Project Structure

```
tokenforest/
├── backend/          # Rust backend (Axum + SQLx)
│   ├── src/
│   │   ├── main.rs   # Application entry point
│   │   ├── handlers.rs  # API route handlers
│   │   ├── models.rs    # Data models
│   │   └── db.rs        # Database migrations
│   ├── Cargo.toml
│   └── README.md
├── frontend/         # SvelteKit frontend
│   ├── src/
│   │   └── routes/
│   │       └── +page.svelte  # Main page
│   ├── package.json
│   └── README.md
├── database/         # Database schemas and scripts
│   └── schema.sql
└── docs/            # Documentation
```

## Tech Stack

### Backend
- **Language**: Rust 🦀
- **Web Framework**: Axum
- **Database**: SQLite with SQLx
- **Async Runtime**: Tokio

### Frontend
- **Framework**: SvelteKit
- **Build Tool**: Vite
- **Language**: TypeScript
- **Styling**: Custom CSS with glassmorphism

## Quick Start

### Prerequisites
- Rust (1.70+)
- Node.js (18+)
- npm or pnpm

### Backend Setup

```bash
cd backend

# Build and run
cargo build
cargo run

# Or with auto-reload
cargo watch -x run
```

The backend will start on `http://localhost:3000`

### Frontend Setup

```bash
cd frontend

# Install dependencies
npm install

# Run development server
npm run dev
```

The frontend will start on `http://localhost:5173`

## API Endpoints

| Method | Endpoint | Description |
|--------|----------|-------------|
| GET | `/` | Welcome message |
| GET | `/api/tokens` | List all tokens |
| POST | `/api/tokens` | Create a new token |

## Environment Variables

Create a `.env` file in the backend directory:

```env
DATABASE_URL=./database/tokenforest.db
RUST_LOG=debug
```

## Development

### Running Tests

```bash
# Backend tests
cd backend && cargo test

# Frontend tests
cd frontend && npm test
```

### Database Migrations

Migrations are automatically run when the backend starts.
Manual migration:

```bash
sqlite3 database/tokenforest.db < database/schema.sql
```

## License

MIT

---

Built with ❤️ using Rust and Svelte
