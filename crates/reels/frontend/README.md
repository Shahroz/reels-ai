# Reels Frontend

React + TypeScript frontend for Reels AI with React Router.

## Setup

1. Install dependencies:
```bash
npm install
```

2. Create a `.env` file (optional):
```bash
VITE_API_BASE_URL=http://localhost:8080
```

3. Start the development server:
```bash
npm run dev
```

The app will be available at `http://localhost:3000`

## Build

To build for production:
```bash
npm run build
```

The built files will be in the `dist/` directory.

## Project Structure

```
src/
├── components/          # Reusable React components
│   ├── ReelGeneratorForm.tsx
│   ├── StatusSection.tsx
│   ├── VideoSection.tsx
│   └── LogSection.tsx
├── pages/               # Page components
│   ├── Home.tsx
│   └── ReelGenerator.tsx
├── hooks/               # Custom React hooks
│   └── useReelGeneration.ts
├── api/                 # Generated API client
├── App.tsx              # Main app component with routing
└── main.tsx             # Entry point
```

## Features

- **React Router**: Client-side routing
- **TypeScript**: Type-safe development
- **Reel Generator Form**: Component for generating reels
- **Real-time Updates**: WebSocket integration for live status
- **Video Playback**: Stream generated videos with range request support
