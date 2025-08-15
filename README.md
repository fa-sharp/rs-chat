# RsChat 🤖💬

A fast, secure, self-hostable chat application built with Rust, TypeScript, and React. Chat with multiple AI providers using your own API keys, with real-time streaming built-in.

!! **Submission to the [T3 Chat Cloneathon](https://cloneathon.t3.chat/)** !!

Demo link: https://rschat.fasharp.io (⚠️ This is a demo - don't expect your account/chats to be there when you come back. It may intermittently delete data. Please also don't enter any sensitive information or confidential data)

## ✨ Features

### 🚀 Main Features

- **Multiple AI Providers**: Chat with AI models from OpenAI, Anthropic, and OpenRouter
- **Streaming**: Streams responses using SSE (Server-Sent Events)
- **Concurrent Streaming**: Seamlessly switch between multiple AI conversations streamed at the same time
- **Resumable Conversations**: Resume the conversation if your connection is lost or the page is refreshed
- **Code Highlighting**: Beautiful syntax highlighting for code blocks using [`rehype-highlight`](https://github.com/rehypejs/rehype-highlight)
- **Dark Mode**: Dark/light theme support
- **Responsive Design**: Mobile-friendly layout
- **Search Chats**: Full-text search of chat session titles and messages
- **Fast and Memory Efficient**: Rust backend using the [Rocket framework](https://rocket.rs/)
- **Users & Authentication**: Login via OAuth providers (Google, GitHub, etc.), custom OIDC, and SSO header authentication
- **API Key Access and OpenAPI Docs**: API key access and documentation at `/api/docs` for developers to integrate with RsChat
- **Fully Type-Safe**: End-to-end type safety with auto-generated client from OpenAPI spec

### ⚡ Convenience Features

- **Smart Titles**: Auto-generation of chat titles
- **Smart Scrolling**: Auto-scroll during streaming and when opening previous chats
- **Secure Key Storage**: Your API keys are saved and encrypted

## 🏗️ Architecture

**Backend**: Rust with Rocket framework, PostgreSQL, Redis

**Frontend**: React 19, TypeScript, Vite, TanStack Router, Tailwind CSS, shadcn/ui

**Type Safety**: OpenAPI spec generation with [rocket_okapi](https://github.com/GREsau/okapi), and auto-generated TS client using [`openapi-typescript`](https://openapi-ts.dev/)

## 📁 Project Structure

```
rs-chat/
├── server/                 # Rust backend
│   ├── src/
│   │   ├── api/           # API route handlers
│   │   ├── auth/          # Authentication services
│   │   ├── db/            # Database models and services
│   │   ├── provider/      # AI provider integrations
│   │   ├── utils/         # Utility functions
│   │   ├── config.rs      # Reading configuration / env variables
│   │   ├── lib.rs         # Server setup
│   │   ├── main.rs        # Server entry point
│   │   └── ...            # Other modules
│   ├── migrations/         # Database migrations
│   └── Cargo.toml          # Rust dependencies
├── web/                    # Vite / React frontend
│   ├── src/
│   │   ├── components/   # React components
│   │   ├── routes/       # TanStack Router routes
│   │   └── lib/          # Utilities and API client
│   ├── public/            # Static assets
│   └── package.json       # Node.js dependencies
├── docker-compose.yml      # Docker Compose file for development
└── Dockerfile              # Dockerfile to build RsChat as a container
```

## 🔑 Setting Up AI Providers

After logging in:

1. Click on name in top-left, and go to **API Keys**
2. Add your provider API keys:
  - **OpenAI**: Get your key from [OpenAI](https://platform.openai.com/api-keys)
  - **Anthropic**: Get your key from [Anthropic Console](https://console.anthropic.com/)
  - **OpenRouter**: Get your key from [OpenRouter](https://openrouter.ai/settings/keys)

Your API keys are encrypted and stored in the database.

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
   docker compose up -d db redis
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
      RUST_LOG: warn   # 'info' or 'debug' for more logs
      RS_CHAT_SERVER_ADDRESS: https://mydomain.com # where you're hosting the app
      RS_CHAT_DATABASE_URL: postgres://user:pass@mypostgres/mydb # Your PostgreSQL URL
      RS_CHAT_REDIS_URL: redis://myredis:6379 # Your Redis URL
      RS_CHAT_SECRET_KEY: your-secret-key-for-encryption # 64-character hex string
      ## For GitHub login: callback URL should be {your_server_address}/api/auth/login/github/callback
      # RS_CHAT_GITHUB_CLIENT_ID: your-github-client-id
      # RS_CHAT_GITHUB_CLIENT_SECRET: your-github-client-secret
      ## Similar config for other OAuth providers - see server/src/auth/oauth/ folder
      # RS_CHAT_DISCORD_CLIENT_ID: your-discord-client-id
      # ...
      ## For SSO header auth - see server/src/auth/sso_header.rs for all config options
      # RS_CHAT_SSO_HEADER_ENABLED: true
      # RS_CHAT_SSO_USERNAME_HEADER: X-Remote-User
      # ...
      ## For running code on a remote Docker host
      # DOCKER_HOST: tcp://remote-docker-host:port
      # DOCKER_TLS_VERIFY: 1
      # DOCKER_CERT_PATH: /certs
    volumes:
      ## For running code on local Docker host
      # - /var/run/docker.sock:/var/run/docker.sock:ro
      ## Certificates for remote Docker host
      # - ./path/to/certs:/certs
```

## 🔒 Security & Privacy

- **Your Keys, Your Control**: You provide and manage your own AI provider API keys
- **Encrypted Storage**: API keys are encrypted using AES-GCM
- **Open Source**: Full transparency - audit the code yourself

## 🤝 Contributing

1. Create an issue or discussion to discuss the idea with maintainers
1. Fork the repository
1. Create a feature branch (`git checkout -b feature/amazing-feature`)
1. Commit your changes (`git commit -m 'Add amazing feature'`)
1. Push to the branch (`git push origin feature/amazing-feature`)
1. Open a Pull Request

## 📝 License

This project is licensed under the MIT License - see the [LICENSE.md](LICENSE.md) file for details.

## 🙏 Acknowledgments

- [TanStack](https://tanstack.com/) and [Vite](https://vitejs.dev/) for great JS tooling and libraries
- [Rocket](https://rocket.rs/) for the amazing Rust web framework
- [shadcn](https://ui.shadcn.com/) and [shadcn-chat](https://github.com/jakobhoeg/shadcn-chat) for the UI
- Many, many other open-source maintainers and contributors that make this project possible

❤️ Built with the [Zed editor](https://zed.dev/), with help from [Claude](https://claude.ai/).
