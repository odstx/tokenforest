# TokenForest Makefile
# Quick commands for development and deployment

.PHONY: help dev dev-backend dev-frontend build-backend build-frontend clean install-backend install-frontend

# Default target
help:
	@echo "TokenForest - Available Commands:"
	@echo ""
	@echo "  make dev              - Start both backend and frontend (development)"
	@echo "  make dev-backend      - Start backend only (cargo run)"
	@echo "  make dev-frontend     - Start frontend only (bun run dev)"
	@echo "  make install-backend  - Build backend (cargo build)"
	@echo "  make install-frontend - Install frontend deps (bun install)"
	@echo "  make build-backend    - Build backend for release"
	@echo "  make build-frontend   - Build frontend for production"
	@echo "  make clean            - Remove build artifacts"
	@echo "  make help             - Show this help message"
	@echo ""

# Start both backend and frontend
dev:
	@echo "🚀 Starting TokenForest development servers..."
	@echo "🦀 Backend: http://localhost:3000"
	@echo "⚡ Frontend: http://localhost:5173"
	@echo ""
	@echo "Press Ctrl+C to stop all servers"
	@echo ""
	# Start backend in background
	cargo run --manifest-path backend/Cargo.toml &
	# Wait for backend to start
	sleep 2
	# Start frontend
	cd frontend && bun run dev
	# Cleanup on exit
	@echo ""
	@echo "🛑 Stopping servers..."

# Start backend only
dev-backend:
	@echo "🦀 Starting backend server..."
	cd backend && cargo run

# Start frontend only
dev-frontend:
	@echo "⚡ Starting frontend dev server..."
	cd frontend && bun run dev

# Build backend
install-backend:
	@echo "🦀 Building backend..."
	cd backend && cargo build

# Install frontend dependencies
install-frontend:
	@echo "⚡ Installing frontend dependencies..."
	cd frontend && bun install

# Build backend for release
build-backend:
	@echo "🦀 Building backend for release..."
	cd backend && cargo build --release

# Build frontend for production
build-frontend:
	@echo "⚡ Building frontend for production..."
	cd frontend && bun run build

# Clean build artifacts
clean:
	@echo "🧹 Cleaning build artifacts..."
	cd backend && cargo clean
	cd frontend && rm -rf node_modules .svelte-kit dist
	@echo "✅ Clean complete!"
