# TokenForest Makefile
# Quick commands for development and deployment

.PHONY: help dev dev-backend dev-core dev-frontend build build-backend build-frontend clean install-backend install-frontend test test-backend test-frontend test-ui kill-all

CARGO := $(HOME)/.cargo/bin/cargo
JWT_SECRET := dev-secret-key-change-in-production-32chars
ENCRYPTION_KEY := dev-encryption-key-change-in-production

# Kill process on a specific port
kill-port = @lsof -ti:$(1) | xargs kill -9 2>/dev/null || true

# Kill all development servers
kill-all:
	@echo "🔪 Killing processes on ports 3000, 8000, 5173..."
	$(call kill-port,3000)
	$(call kill-port,8000)
	$(call kill-port,5173)

# Default target
help:
	@echo "TokenForest - Available Commands:"
	@echo ""
	@echo "  make dev              - Start backend, core, and frontend (development)"
	@echo "  make dev-backend      - Start backend only (cargo run)"
	@echo "  make dev-core         - Start core server only (port 8000)"
	@echo "  make dev-frontend     - Start frontend only (bun run dev)"
	@echo "  make install-backend  - Build backend (cargo build)"
	@echo "  make install-frontend - Install frontend deps (bun install)"
	@echo "  make build-backend    - Build backend for release"
	@echo "  make build-frontend   - Build frontend for production"
	@echo "  make test             - Run Playwright tests"
	@echo "  make test-backend     - Run backend unit tests"
	@echo "  make test-frontend    - Run frontend type check"
	@echo "  make test-ui          - Run Playwright tests with UI"
	@echo "  make clean            - Remove build artifacts"
	@echo "  make kill-all         - Kill all dev servers (ports 3000, 8000, 5173)"
	@echo "  make help             - Show this help message"
	@echo ""

# Start backend, core, and frontend
dev:
	@echo "🔪 Killing existing processes on ports 3000, 8000, 5173..."
	@lsof -ti:3000 | xargs kill -9 2>/dev/null || true
	@lsof -ti:8000 | xargs kill -9 2>/dev/null || true
	@lsof -ti:5173 | xargs kill -9 2>/dev/null || true
	@echo "🚀 Starting TokenForest development servers..."
	@echo "🦀 Backend: http://localhost:3000"
	@echo "🔧 Core: http://localhost:8000"
	@echo "⚡ Frontend: http://localhost:5173"
	@echo ""
	@echo "Press Ctrl+C to stop all servers"
	@echo ""
	# Start backend in background
	cd backend && RUN_MODE=dev JWT_SECRET=$(JWT_SECRET) ENCRYPTION_KEY=$(ENCRYPTION_KEY) $(CARGO) run &
	# Wait for backend to start
	sleep 2
	# Start core in background
	cd backend && RUN_MODE=dev JWT_SECRET=$(JWT_SECRET) ENCRYPTION_KEY=$(ENCRYPTION_KEY) $(CARGO) run --bin tokenforest_core &
	# Wait for core to start
	sleep 1
	# Start frontend
	cd frontend && bun run dev
	# Cleanup on exit
	@echo ""
	@echo "🛑 Stopping servers..."

# Start backend only
dev-backend:
	@echo "🔪 Killing existing process on port 3000..."
	@lsof -ti:3000 | xargs kill -9 2>/dev/null || true
	@echo "🦀 Starting backend server..."
	cd backend && RUN_MODE=dev JWT_SECRET=$(JWT_SECRET) ENCRYPTION_KEY=$(ENCRYPTION_KEY) $(CARGO) run

# Start core server only
dev-core:
	@echo "🔪 Killing existing process on port 8000..."
	@lsof -ti:8000 | xargs kill -9 2>/dev/null || true
	@echo "🔧 Starting core server..."
	cd backend && RUN_MODE=dev JWT_SECRET=$(JWT_SECRET) ENCRYPTION_KEY=$(ENCRYPTION_KEY) $(CARGO) run --bin tokenforest_core

# Start frontend only
dev-frontend:
	@echo "🔪 Killing existing process on port 5173..."
	@lsof -ti:5173 | xargs kill -9 2>/dev/null || true
	@echo "⚡ Starting frontend dev server..."
	cd frontend && bun run dev

# Build backend
install-backend:
	@echo "🦀 Building backend..."
	cd backend && $(CARGO) build

# Install frontend dependencies
install-frontend:
	@echo "⚡ Installing frontend dependencies..."
	cd frontend && bun install

# Build backend for release
build-backend:
	@echo "🦀 Building backend for release..."
	cd backend && $(CARGO) build --release

# Build frontend for production
build-frontend:
	@echo "⚡ Building frontend for production..."
	cd frontend && bun run build

# Build everything for production
build: build-frontend build-backend
	@echo "✅ Build complete!"

# Clean build artifacts
clean:
	@echo "🧹 Cleaning build artifacts..."
	cd backend && $(CARGO) clean
	cd frontend && rm -rf node_modules .svelte-kit dist
	@echo "✅ Clean complete!"

# Run Playwright tests
test:
	@echo "🧪 Running Playwright tests..."
	@echo "🔪 Killing existing processes on ports 3000, 8000, 5173..."
	@lsof -ti:3000 | xargs kill -9 2>/dev/null || true
	@lsof -ti:8000 | xargs kill -9 2>/dev/null || true
	@lsof -ti:5173 | xargs kill -9 2>/dev/null || true
	@echo "🦀 Starting backend server..."
	cd backend && RUN_MODE=test JWT_SECRET=$(JWT_SECRET) ENCRYPTION_KEY=$(ENCRYPTION_KEY) $(CARGO) run &
	@echo "⏳ Waiting for backend to start..."
	@sleep 5
	cd frontend && RUN_MODE=test JWT_SECRET=$(JWT_SECRET) ENCRYPTION_KEY=$(ENCRYPTION_KEY) bunx playwright test; \
	RET=$$?; \
	echo "🛑 Stopping backend..."; \
	lsof -ti:3000 | xargs kill -9 2>/dev/null || true; \
	exit $$RET

# Run backend unit tests
test-backend:
	@echo "🧪 Running backend tests..."
	cd backend && $(CARGO) test --lib

# Run frontend unit tests
test-frontend:
	@echo "🧪 Running frontend tests..."
	cd frontend && bun run check

# Run Playwright tests with UI
test-ui:
	@echo "🧪 Running Playwright tests with UI..."
	@echo "🔪 Killing existing processes on ports 3000, 8000, 5173..."
	@lsof -ti:3000 | xargs kill -9 2>/dev/null || true
	@lsof -ti:8000 | xargs kill -9 2>/dev/null || true
	@lsof -ti:5173 | xargs kill -9 2>/dev/null || true
	@echo "🦀 Starting backend server..."
	cd backend && RUN_MODE=test JWT_SECRET=$(JWT_SECRET) ENCRYPTION_KEY=$(ENCRYPTION_KEY) $(CARGO) run &
	@echo "⏳ Waiting for backend to start..."
	@sleep 5
	cd frontend && RUN_MODE=test JWT_SECRET=$(JWT_SECRET) ENCRYPTION_KEY=$(ENCRYPTION_KEY) bunx playwright test --ui; \
	RET=$$?; \
	echo "🛑 Stopping backend..."; \
	lsof -ti:3000 | xargs kill -9 2>/dev/null || true; \
	exit $$RET
