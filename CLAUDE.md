# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Git Commits

Never add `Co-Authored-By` trailers to commit messages.

## Project Overview

A desktop SSH port forwarding manager built with Tauri v2. The frontend is Vue 3 + TypeScript + DaisyUI/Tailwind CSS 4, and the backend is Rust. Supports password, private key, and SSH agent authentication, as well as jump host chains (ProxyJump).

## Commands

### Development
```bash
npm run dev              # Frontend dev server (hot reload, port 5173)
npm run tauri dev        # Full Tauri app with Rust backend
```

### Build
```bash
npm run build            # Frontend only (runs vue-tsc check + vite build)
npm run tauri build      # Complete desktop app (dmg/deb/appimage/nsis)
```

### Type Checking & Linting
```bash
npx vue-tsc --noEmit     # TypeScript check

cd src-tauri
cargo check              # Rust compile check
cargo clippy             # Rust lint
cargo fmt                # Rust format
```

### Testing
```bash
cd src-tauri && cargo test              # All Rust tests
cd src-tauri && cargo test test_name    # Specific test
cd src-tauri && cargo test -- --nocapture  # With stdout
```

No frontend test framework is currently configured.

## Architecture

### Frontend–Backend Communication
The Vue frontend communicates with the Rust backend exclusively through **Tauri commands** (invoked via `@tauri-apps/api`). There is no HTTP API. Key commands in `src-tauri/src/commands/mod_commands.rs`:
- `test_connection` / `start_forward_cmd` / `stop_forward_cmd` — lifecycle management
- `get_forward_status` / `get_all_statuses` — runtime session polling
- `import_ssh_config` — parse `~/.ssh/config` with ProxyJump support

### State Management
A single Pinia store (`src/stores/servers.ts`) manages all app state:
- **Persisted** (to file via `saveToFile()`): `servers` and `rules` arrays
- **Runtime only**: `sessions` (forwarding session status, not persisted)
- `buildJumpChain()` resolves UI jump entries (server reference or inline config) into the flat `JumpHost[]` format the Rust backend expects

### Jump Host Chain Resolution
The frontend has two representations for jump hosts:
- `JumpEntry` (UI): can reference another server by ID or hold inline config
- `JumpHost` (wire format / Rust): fully resolved inline config

Before sending a server config to Rust commands, call `buildJumpChain()` to convert `JumpEntry[]` → `JumpHost[]`.

### Rust SSH Layer
- `src-tauri/src/ssh/client.rs`: Establishes SSH connections. `connect_via_jumps()` chains through proxies iteratively (avoids async recursion issues). Authentication is handled separately for direct targets vs. jump hosts.
- `src-tauri/src/ssh/forwarder.rs`: Binds local port, accepts TCP connections, and does bidirectional forwarding over the SSH channel.
- `src-tauri/src/lib.rs`: Tauri app setup — creates system tray icon, intercepts window close to hide instead of quit, registers all commands.

### Config Persistence
Config is stored in the OS app data directory (from `get_config_dir` command). On startup, `main.ts` calls `loadFromFile()` in the store. Import/export dialogs in `App.vue` support JSON merge or replace modes.

## Code Style

### TypeScript/Vue
- `<script setup lang="ts">` syntax for all components
- Explicit type imports: `import type { Foo } from './types'`
- `defineProps<{ prop: Type }>()` and `defineEmits<...>()` for component interfaces
- 2-space indent, single quotes, semicolons required
- Chinese comments are acceptable in this codebase

### Rust
- Tauri commands return `Result<T, String>` — use `.map_err(|e| e.to_string())`
- Use `anyhow::Result` internally with `.context()` for error chaining
- Async with `tokio`; use `tokio::select!` for cancellation
- `cargo fmt` enforced (4-space indent, 100-char line limit)

## macOS Note
The distributed app is unsigned. First-time users must run:
```bash
xattr -cr "/Applications/SSH Port Forward Manager.app"
```
