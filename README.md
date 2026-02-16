# Crystal Launcher

A modern, lightweight Minecraft launcher built with Tauri, Rust, and Svelte.

## Features

- **Microsoft Account Integration** - Secure authentication with Microsoft/Xbox Live
- **Version Management** - Install and manage multiple Minecraft versions
- **Instance System** - Create and organize separate game instances
- **Auto-Login** - Remember your account between sessions
- **Real-time Updates** - Progress tracking during installation
- **Cross-Platform** - Works on Windows, macOS, and Linux

## Tech Stack

- **Frontend**: Svelte 5 + TypeScript + Tailwind CSS
- **Backend**: Rust + Tauri v2
- **UI Components**: Custom shadcn-svelte components

## Development

### Prerequisites

- [Node.js](https://nodejs.org/) (v18+)
- [Rust](https://rustup.rs/)
- [Bun](https://bun.sh/) or npm

### Setup

```bash
# Install dependencies
bun install

# Run in development mode
bun run tauri dev

# Build for production
bun run tauri build
```

### Project Structure

```
├── src/                    # Frontend Svelte code
│   ├── lib/               # Shared utilities and components
│   │   ├── components/    # UI components
│   │   └── helpers/       # Helper functions
│   └── pages/             # Page components
├── src-tauri/             # Rust backend code
│   ├── src/
│   │   ├── commands/      # Tauri command handlers
│   │   ├── models/        # Data structures
│   │   └── core/          # Core functionality
│   └── icons/             # Application icons
```

## License

[MIT](LICENSE)
