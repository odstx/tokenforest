# TokenForest Frontend

SvelteKit frontend for TokenForest application.

## Tech Stack
- **Runtime**: Bun 🚀
- **Framework**: SvelteKit
- **Build Tool**: Vite
- **Language**: TypeScript
- **Styling**: CSS with glassmorphism design

## Setup

```bash
# Install dependencies with Bun
bun install

# Run development server
bun run dev

# Build for production
bun run build

# Preview production build
bun run preview
```

## Configuration

The frontend expects the backend API to be available at `/api/` endpoints.
For local development, you may need to set up a proxy in `vite.config.ts`:

```ts
export default defineConfig({
  plugins: [sveltekit()],
  server: {
    proxy: {
      '/api': 'http://localhost:3000'
    }
  }
});
```

## Features

- 🌱 Create new tokens
- 📊 View all tokens in a beautiful grid
- 🎨 Modern glassmorphism UI design
- 📱 Responsive layout
