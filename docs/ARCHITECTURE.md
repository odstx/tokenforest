# TokenForest Documentation

## Architecture Overview

TokenForest is a full-stack web application consisting of:

1. **Rust Backend** - RESTful API using Axum framework
2. **Svelte Frontend** - Modern reactive UI
3. **SQLite Database** - Lightweight, file-based storage

## Backend Architecture

### Components

- **main.rs**: Application entry point, server configuration
- **handlers.rs**: HTTP request handlers for API endpoints
- **models.rs**: Data structures (Token entity)
- **db.rs**: Database connection and migrations

### API Design

The backend follows RESTful conventions:
- GET endpoints for retrieving data
- POST endpoints for creating resources
- JSON request/response format

### Database Schema

```sql
tokens (
  id: INTEGER PRIMARY KEY,
  name: TEXT,
  symbol: TEXT UNIQUE,
  supply: INTEGER,
  created_at: TEXT
)
```

## Frontend Architecture

### Components

- **+page.svelte**: Main page with token list and creation form
- **Reactive State**: Svelte stores for managing application state
- **API Integration**: Fetch API for backend communication

### Styling

- Glassmorphism design with backdrop blur
- Responsive grid layout
- Gradient backgrounds
- Smooth animations and transitions

## Deployment

### Backend

```bash
cd backend
cargo build --release
./target/release/tokenforest_backend
```

### Frontend

```bash
cd frontend
npm run build
# Deploy 'build' directory to static hosting
```

### Database

SQLite database file is created automatically on first run.
Location: `backend/database/tokenforest.db`

## Security Considerations

- Input validation on both frontend and backend
- SQL injection prevention via parameterized queries
- CORS configuration for cross-origin requests

## Future Enhancements

- [ ] User authentication
- [ ] Token transfer functionality
- [ ] Real-time updates with WebSockets
- [ ] Advanced analytics dashboard
- [ ] Multi-database support (PostgreSQL, MySQL)
