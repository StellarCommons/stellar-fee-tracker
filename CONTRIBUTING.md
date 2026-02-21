# Contributing to Stellar Fee Tracker

This repo is a monorepo with two packages:

```
stellar-fee-tracker/
├── packages/core/   ← Rust backend (Axum HTTP server + fee polling)
└── packages/ui/     ← Next.js frontend (dashboard)
```

---

## Prerequisites

| Tool | Version | Install |
|------|---------|---------|
| Rust toolchain | stable ≥ 1.75 | https://rustup.rs |
| Node.js | ≥ 20 | https://nodejs.org |
| Git | any recent | — |
| Docker Desktop | ≥ 4.x *(Option A only)* | https://www.docker.com/products/docker-desktop |

---

## Option A — Run with Docker (easiest)

One command starts both services with the correct environment:

```bash
# 1. Clone the repo
git clone https://github.com/your-org/stellar-fee-tracker
cd stellar-fee-tracker

# 2. Start everything
docker compose up --build
```

Open **http://localhost:3000**. The dashboard should show live fee data within ~30 seconds (first poll cycle).

To stop:

```bash
docker compose down
```

---

## Option B — Run manually (faster iteration)

### Terminal 1 — Rust backend

```bash
cd packages/core
cp .env.example .env
# Edit .env if needed (defaults work for testnet)
cargo run
```

Expected output:

```
INFO core: Configuration loaded: ...
INFO core: API server listening on 0.0.0.0:8080
INFO core: Fee polling started (interval: 30s)
INFO core: Polled fee stats — base: 100, min: 100, max: ...
```

### Terminal 2 — Next.js frontend

```bash
cd packages/ui
cp .env.example .env.local
# NEXT_PUBLIC_API_URL=/api  ← leave as-is to use the proxy
npm install
npm run dev
```

Open **http://localhost:3000**.

---

## How the proxy works

The frontend calls `/api/fees/current` (same origin). Next.js rewrites that
server-side to `http://localhost:8080/fees/current` via the `BACKEND_URL`
env var. The browser never makes a cross-origin request, so no CORS
configuration is required during local development.

```
Browser → /api/fees/current
           ↓ (Next.js rewrite — server-side)
         http://localhost:8080/fees/current
```

If you want to call the backend **directly** from the browser (bypassing the
proxy), set `NEXT_PUBLIC_API_URL=http://localhost:8080` in `.env.local` and
ensure the backend has the correct `ALLOWED_ORIGINS` set.

---

## Verify the connection

```bash
# Backend health
curl http://localhost:8080/health
# Expected: ok

# Via Next.js proxy (Option B)
curl http://localhost:3000/api/fees/current
# Expected: JSON with base_fee, min_fee, max_fee, avg_fee

# Via Docker (Option A — same URL)
curl http://localhost:3000/api/fees/current
```

---

## Troubleshooting

| Problem | Cause | Fix |
|---------|-------|-----|
| Dashboard shows no data | Backend not running | Start backend first (`cargo run`) |
| "Failed to load fee data" toast | Wrong `BACKEND_URL` in `.env.local` | Set `BACKEND_URL=http://localhost:8080` |
| CORS errors in browser console | Using direct URL without CORS configured | Use proxy (`NEXT_PUBLIC_API_URL=/api`) |
| `cargo run` exits immediately | Missing `.env` or required env var | `cp packages/core/.env.example packages/core/.env` |
| Port 8080 already in use | Another process | `lsof -ti:8080 \| xargs kill` |
| Port 3000 already in use | Another process | `lsof -ti:3000 \| xargs kill` |
| Docker healthcheck fails | Backend slow to start | Wait 30 s; `docker compose logs backend` |

---

## Running tests

```bash
# Rust unit + property tests
cargo test

# Lint
cargo clippy -- -D warnings

# Next.js lint
cd packages/ui && npm run lint
```

---

## Commit style

Follow [Conventional Commits](https://www.conventionalcommits.org/):

```
feat: add X
fix: correct Y
chore: update Z
```
