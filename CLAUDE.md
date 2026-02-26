# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

**MyloAir** is a secure, cross-platform password manager desktop application currently being migrated from Electron to **Tauri 2.0**. The original Electron codebase is preserved in `bak/` for reference.

Tech stack: Tauri 2.0 + React 18 + TypeScript + Webpack 5 + Ant Design v5 + SQLite (via Rust/rusqlite).

## Commands

```bash
# Development
npm run tauri:dev         # Full Tauri dev mode (sets env + launches Tauri + webpack)
npm run dev:renderer      # Frontend only (webpack dev server on port 3000)

# Build
npm run tauri:build       # Production build (outputs .app only, see macOS DMG note below)
npm run build:renderer    # Frontend only → dist/renderer/

# Lint & Format
npm run lint              # ESLint on TypeScript/TSX
npm run lint:fix          # ESLint with auto-fix
npm run format            # Prettier

# Rust
cargo check --manifest-path src-tauri/Cargo.toml   # Check Rust compilation
cargo test --manifest-path src-tauri/Cargo.toml    # Run Rust unit tests
```

## Architecture

The app follows Tauri's two-process model:

**Frontend** (`src/renderer/`) — React SPA served at `localhost:3000` in dev mode, bundled to `dist/renderer/` for production.
- `api/tauriAPI.ts` — Single IPC bridge layer; all `invoke()` calls to the Rust backend go through here.
- `components/` — React UI components (Ant Design).

**Backend** (`src-tauri/src/`) — Rust process that manages data and security.
- `commands/security.rs` — Tauri command handlers exposed to the frontend (encryption, password ops).
- `services/database.rs` — SQLite business logic layer using `rusqlite` (bundled).
- `models/` — Rust structs for serialization.

**IPC flow**: Frontend calls `tauriAPI.ts` → `invoke()` → Tauri command in `commands/` → `services/database.rs` → SQLite (`myloair.db`).

**Tauri plugins in use**: `tauri-plugin-fs`, `tauri-plugin-dialog`, `tauri-plugin-shell`, `tauri-plugin-store`, `tauri-plugin-sql` (preloads `sqlite:myloair.db`), `tauri-plugin-log`.

**Encryption**: AES-256-CBC + SHA-2 on the Rust side (`aes`, `cbc`, `sha2` crates); `crypto-js` on the frontend.

## Key Configuration

- `src-tauri/tauri.conf.json` — App window config, bundle targets, CSP (currently disabled), SQLite DB name.
- `webpack.renderer.config.js` — Frontend bundler config.
- `scripts/set-tauri-env.js` — Switches environment variables between dev/prod modes.
- App identifier is still a placeholder: `com.yourcompany.myloair.dev`.

## macOS DMG 打包

由于 Tauri 内置的 `create-dmg` 不支持 macOS 26+，`tauri.conf.json` 中 `targets` 已设为 `["app"]`，需手动执行第二步打包 DMG：

```bash
# 第一步：构建 .app
npm run tauri:build

# 第二步：打包 DMG（使用 macOS 内置 hdiutil）
bash scripts/create-dmg.sh
```

输出：`src-tauri/target/release/bundle/dmg/MyloAir_1.0.0_aarch64.dmg`

待 Tauri 官方修复对 macOS 26 的兼容后，可将 `targets` 改回 `"all"` 并删除 `scripts/create-dmg.sh`。

## Migration Status

The project is mid-migration from Electron to Tauri 2.0. Refer to `docs/electron-to-tauri-analysis.md` for migration background and `bak/` for the original Electron source.
