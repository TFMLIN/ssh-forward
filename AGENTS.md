# Agent Guidelines for SSH Forward Manager

This is a Tauri application with Vue 3 + TypeScript frontend and Rust backend.

## Project Structure

```
├── src/                    # Vue frontend
│   ├── components/         # Vue SFC components
│   ├── stores/            # Pinia stores
│   ├── types/             # TypeScript type definitions
│   ├── App.vue            # Root component
│   └── main.ts            # Entry point
├── src-tauri/             # Rust backend
│   └── src/
│       ├── commands/      # Tauri command handlers
│       ├── ssh/           # SSH client & forwarder
│       ├── lib.rs         # Main library
│       └── types.rs       # Rust type definitions
├── package.json           # Node dependencies
├── tsconfig.json          # TypeScript config
└── vite.config.ts         # Vite config
```

## Build Commands

### Development
```bash
# Run dev server with hot reload
npm run dev

# Run Tauri dev (frontend + Rust)
npm run tauri dev
```

### Build
```bash
# Build frontend only
npm run build

# Build complete Tauri app
npm run tauri build
```

### Type Checking
```bash
# TypeScript check (no emit)
npx vue-tsc --noEmit
```

### Rust Commands
```bash
# Run Rust tests
cd src-tauri && cargo test

# Run specific test
cd src-tauri && cargo test test_name

# Check Rust code
cd src-tauri && cargo check

# Build Rust only
cd src-tauri && cargo build

# Format Rust code
cd src-tauri && cargo fmt

# Lint Rust code
cd src-tauri && cargo clippy
```

## Code Style Guidelines

### TypeScript / Vue

#### Imports
- Use explicit type imports: `import type { Foo } from './types'`
- Group imports: external libs first, then internal modules
- Use path aliases for internal imports: `@/components/Foo`

#### Naming
- Components: PascalCase (e.g., `ServerList.vue`)
- Composables: camelCase starting with `use` (e.g., `useServerStore`)
- Types/Interfaces: PascalCase (e.g., `SshServer`, `ForwardRule`)
- Variables/functions: camelCase
- Constants: UPPER_SNAKE_CASE for true constants

#### Vue Components
- Use `<script setup lang="ts">` syntax
- Props: define with `defineProps<{ prop: Type }>()`
- Emits: define with `defineEmits<{ (e: 'event', val: Type): void }>()`
- Use scoped styles for component-specific CSS

#### Types
- Prefer interfaces over type aliases for objects
- Use explicit return types on exported functions
- Enable strict mode in tsconfig (already configured)

#### Error Handling
- Use try/catch for async operations
- Show user-friendly messages via Element Plus `ElMessage`
- Log errors to console for debugging

### Rust

#### Naming
- Types/Traits: PascalCase (e.g., `SshServerConfig`, `ForwardSession`)
- Functions/variables: snake_case (e.g., `start_forward`, `session_id`)
- Constants: SCREAMING_SNAKE_CASE
- Modules: snake_case

#### Error Handling
- Use `anyhow::Result` for application errors
- Use `.context()` to add error context
- Convert to String for Tauri command returns: `.map_err(|e| e.to_string())`

#### Async
- Use `tokio` for async runtime
- Prefer `async fn` over manual Future implementations
- Use `tokio::select!` for cancellation

#### Comments
- Use `///` for public API documentation
- Use `//` for implementation comments
- Chinese comments are acceptable in this codebase

### Formatting

#### TypeScript/Vue
- Use default Prettier/Volar formatting
- 2 spaces indentation
- Single quotes for strings
- Semicolons required
- Max line length: 100 (soft limit)

#### Rust
- Use `cargo fmt` for formatting
- 4 spaces indentation
- Max line length: 100

## Key Dependencies

### Frontend
- Vue 3.5 + Composition API
- Element Plus (UI components, Chinese locale)
- Pinia (state management with persistence)
- Tauri API v2

### Backend
- Tauri v2
- russh / russh-keys (SSH client)
- tokio (async runtime)
- anyhow (error handling)
- ssh2-config (SSH config parsing)

## Testing

Currently no test framework is configured. To add tests:

### Frontend (Vitest)
```bash
npm install -D vitest
```

### Backend (built-in)
```bash
cd src-tauri && cargo test
```

Add tests in `src-tauri/src/` files using:
```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_something() {
        assert_eq!(2 + 2, 4);
    }
}
```

## Common Patterns

### Frontend Store Pattern
```typescript
export const useStore = defineStore('name', () => {
  const state = ref<State[]>([])
  
  function addItem(item: Omit<State, 'id'>) {
    state.value.push({ ...item, id: uuidv4() })
  }
  
  return { state, addItem }
}, { persist: { pick: ['state'] } })
```

### Tauri Command Pattern
```rust
#[tauri::command]
pub async fn command_name(
    state: State<'_, Arc<AppState>>,
    param: ParamType,
) -> Result<ReturnType, String> {
    // Implementation
    Ok(result)
}
```

## Notes

- This is a desktop SSH port forwarding manager
- Supports password, private key, and SSH agent authentication
- Supports jump host chains (ProxyJump)
- Data persists to localStorage (frontend) and runs in system tray
- macOS users need to run `xattr -cr` on first launch (unsigned app)
