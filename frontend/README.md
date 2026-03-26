# TokenForest Frontend

SvelteKit frontend for TokenForest application.

## Tech Stack
- **Framework**: SvelteKit
- **Build Tool**: Vite
- **Language**: TypeScript
- **Styling**: CSS with glassmorphism design

## Setup

```bash
# Install dependencies
npm install

# Run development server
npm run dev

# Build for production
npm run build

# Preview production build
npm run preview
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
