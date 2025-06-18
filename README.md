# RsChat 🤖💬

A fast, secure, self-hostable chat application built with Rust, TypeScript, and React. Chat with multiple AI providers using your own API keys, with real-time streaming built-in.

**Submission to the [T3 Chat Cloneathon](https://cloneathon.t3.chat/)**

## ✨ Features

### 🚀 Main Features

- **Multiple AI Providers**: Chat with AI models from Anthropic (Claude) and OpenRouter
- **Streaming**: Stream conversations using SSE (Server-Sent Events)
- **Concurrent Streaming**: Stream multiple AI conversations simultaneously
- **Code Highlighting**: Beautiful syntax highlighting for code blocks using [`rehype-highlight`](https://github.com/rehypejs/rehype-highlight)
- **Dark Mode**: Dark/light theme support
- **Fully Type-Safe**: End-to-end type safety with OpenAPI generation and auto-generated client
- **OpenAPI Docs**: API documentation at `/api/docs` for developers to integrate with RsChat
- **Fast and Memory Efficient**: Rust backend using Rocket framework
- **Fast Navigation**: Preloading and optimistic updates with TanStack Router
- **GitHub Authentication**: Secure login with GitHub OAuth and persistent sessions
- **Session Management**: Powered by [`rocket-flex-session`](https://github.com/fa-sharp/rocket-flex-session) for flexible session handling

### ⚡ Convenience Features

- **Smart Titles**: Auto-generation of chat titles
- **Auto-Focus**: Auto-focus on input when opening and switching between chats
- **Smart Scrolling**: Auto-scroll during streaming and when opening previous chats
- **Secure Key Storage**: Your API keys are saved and encrypted

## 🏗️ Architecture

**Backend**: Rust with Rocket framework, PostgreSQL, Redis

**Frontend**: React 19, TypeScript, Vite, TanStack Router, Tailwind CSS, shadcn/ui

**Type Safety**: OpenAPI spec generation with [rocket_okapi](https://github.com/GREsau/okapi), and auto-generated TS client using [`openapi-typescript`](https://openapi-ts.dev/)

## 📁 Project Structure

```
chat-rs/
├── server/               # Rust backend
│   ├── src/              # Backend source code
│   ├── migrations/       # Database migrations
│   └── Cargo.toml        # Rust dependencies
├── web/                  # Vite / React frontend
│   ├── src/
│   │   ├── components/   # React components
│   │   ├── routes/       # TanStack Router routes
│   │   └── lib/          # Utilities and API client
│   ├── public/           # Static assets
│   └── package.json      # Node.js dependencies
├── docker-compose.yml     # Docker Compose file for development
└── Dockerfile             # Dockerfile to build RsChat as a container
```

## 🔑 Setting Up AI Providers

After logging in with GitHub:

1. Click on name in top-left, and go to **API Keys**
2. **Add your provider API keys**:
   - **Anthropic**: Get your key from [Anthropic Console](https://console.anthropic.com/)
   - **OpenRouter**: Get your key from [OpenRouter](https://openrouter.ai/keys)

Your API keys are encrypted in the database.

## 🛠️ Development

### Prerequisites

- **Rust** >= 1.85 ([Install Rust](https://rustup.rs/))
- **Node.js** >= 20 with **pnpm** ([Install pnpm](https://pnpm.io/installation))
- **Docker** and **Docker Compose** (for databases)
### Setup

1. **Clone the repository**
   ```bash
   git clone https://github.com/fa-sharp/rs-chat.git
   cd rs-chat
   ```

2. **Start development databases**
   ```bash
   docker compose up -d
   ```

3. **Set up the backend**
   ```bash
   cd server
   cp .env.example .env  # Edit with your settings
   cargo run             # This will run migrations automatically
   ```

4. **Set up the frontend** (in a new terminal)
   ```bash
   cd web
   pnpm install
   pnpm dev
   ```

5. **Access the application**
   - Frontend: http://localhost:3000
   - Backend API: http://localhost:8000
   - API Docs: http://localhost:8000/api/docs


### API Client Generation

When the backend API changes, regenerate the TypeScript client:

```bash
cd web
pnpm run gen-api
```

### Database Migrations

```bash
cd server
# Create new migration
diesel migration generate migration_name

# Run migrations
cargo run  # Migrations run automatically on startup
```

## 🐳 Self-hosted Deployment

### Using Docker Compose

You'll need an environment with PostgreSQL and Redis (or Redis-compatible database).

```docker-compose.yml
services:
  rschat:
    image: ghcr.io/fa-sharp/rs-chat:latest
    # ports:
    #   - "8080:8080"
    environment:
      RUST_LOG: warn   # 'info' for more logs
      CHAT_RS_SERVER_ADDRESS: https://mydomain.com # where you're hosting the app
      CHAT_RS_DATABASE_URL: postgres://user:pass@mypostgres/mydb # Your PostgreSQL URL
      CHAT_RS_REDIS_URL: redis://myredis:6379 # Your Redis URL
      CHAT_RS_SECRET_KEY: your-secret-key-for-encryption # 64-character hex string
      CHAT_RS_GITHUB_CLIENT_ID: your-github-client-id
      CHAT_RS_GITHUB_CLIENT_SECRET: your-github-client-secret
```

## 🔒 Security & Privacy

- **Encrypted Storage**: API keys are encrypted using AES-GCM
- **Your Keys, Your Control**: You provide and manage your own AI provider API keys
- **Open Source**: Full transparency - audit the code yourself

## 🤝 Contributing

1. Create an issue or discussion to discuss the idea with maintainers
1. Fork the repository
1. Create a feature branch (`git checkout -b feature/amazing-feature`)
1. Commit your changes (`git commit -m 'Add amazing feature'`)
1. Push to the branch (`git push origin feature/amazing-feature`)
1. Open a Pull Request

## 📝 License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## 🙏 Acknowledgments

- [TanStack](https://tanstack.com/) and [Vite](https://vitejs.dev/) for great tooling and libraries
- [Rocket](https://rocket.rs/) for the amazing Rust web framework
- Many, many other open-source maintainers and contributors that make this project possible

❤️ Built with the [Zed editor](https://zed.dev/), with help from [Claude](https://claude.ai/).
