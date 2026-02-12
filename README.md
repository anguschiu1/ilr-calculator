# ILR Calculator

A Rust-based calculator for computing rolling absence periods using a 365-day window. The project supports both CLI usage and WebAssembly compilation for browser integration.

## Project Structure

```
ilr-calculator/
├── src/                # Rust source code
│   ├── lib.rs          # Core calculation logic (WASM-compatible)
│   ├── main.rs         # CLI application
│   └── wasm.rs         # WebAssembly bindings
├── components/         # Vue components
│   └── ui/             # shadcn/ui components
├── pages/              # Nuxt.js pages
├── composables/        # Vue composables
├── types/              # TypeScript types
├── assets/             # Static assets
│   └── css/            # Global styles
├── Cargo.toml          # Rust project configuration
├── package.json         # Node.js dependencies
├── nuxt.config.ts       # Nuxt.js configuration
├── tailwind.config.js   # Tailwind CSS configuration
├── build-wasm.sh        # WASM build script
└── README.md            # This file
```

## Features

- **365-day rolling window calculation**: For each absence period, calculates total absence days within a 365-day window ending on the absence end date
- **Overlap merging**: Automatically merges overlapping and adjacent absence periods to prevent double-counting
- **Multiple input methods**: Supports JSON file input and interactive CLI input
- **WebAssembly support**: Can be compiled to WASM for browser use
- **Comprehensive test suite**: Extensive unit tests covering edge cases

## Core Components

### Library (`src/lib.rs`)

The core library contains pure calculation logic that is WASM-compatible:

- `AbsencePeriod`: Struct representing a single absence period
- `CalculationResult`: Struct containing calculation results
- `validate_absence_periods()`: Validates absence periods
- `merge_absence_periods()`: Merges overlapping/adjacent periods
- `calculate_overlap_days()`: Calculates overlap between period and window
- `calculate_rolling_absences()`: Main calculation function
- `calculate_from_json()`: Convenience function for JSON input

### CLI (`src/main.rs`)

Command-line interface for interactive or file-based input:

```bash
# Interactive mode
cargo run

# File input mode
cargo run absences.json
```

### WebAssembly (`src/wasm.rs`)

JavaScript-compatible functions for browser integration:

- `calculate_rolling_absences(json_input: &str) -> Result<String, JsValue>`
- `validate_absence_periods(json_input: &str) -> Result<String, JsValue>`

## Frontend Setup

The project includes a Nuxt.js frontend with shadcn/ui components and dark mode styling.

### Prerequisites

- Node.js 18+ and pnpm
- Rust toolchain (for WASM builds)

### Installation

```bash
# Install frontend dependencies
pnpm install

# Build WASM module (required for calculations)
chmod +x build-wasm.sh
./build-wasm.sh
```

### Development

```bash
# Start development server
pnpm dev

# The app will be available at http://localhost:3000
```

If you ever see `Failed to resolve import "#app-manifest"`, the project disables the experimental app manifest in `nuxt.config.ts` to avoid this known Nuxt 3.15+ issue. After a production build, you can run `pnpm dev:clean` (or `pnpm clean && pnpm dev`) to clear build artifacts before dev.

### Building for Production

```bash
# Build WASM module
./build-wasm.sh

# Build frontend
pnpm build

# Preview production build
pnpm preview
```
<｜tool▁calls▁begin｜><｜tool▁call▁begin｜>
read_file

### Frontend Features

- **Mobile-first design**: Optimized for mobile devices with touch-friendly UI
- **Dark mode**: Beautiful dark theme matching shadcn/ui design system
- **Responsive layout**: Adapts seamlessly from mobile to desktop
- **Type-safe**: Full TypeScript support
- **Component library**: shadcn/ui components for consistent UI

## Building

### Standard Rust Build

```bash
# Build the project
cargo build

# Run tests
cargo test

# Run the CLI
cargo run
```

### WebAssembly Build

```bash
# Make the build script executable
chmod +x build-wasm.sh

# Build WASM binary
./build-wasm.sh
```

Or manually:

```bash
wasm-pack build --target web --out-dir pkg
```

The WASM output will be in the `pkg/` directory.

### Updating wasm-pack

To get the latest wasm-pack (e.g. 0.14.x) and avoid version warnings:

**Option A – Official installer (recommended; no Rust version requirement):**

```bash
./scripts/update-wasm-pack.sh
# or: curl -sSf https://rustwasm.github.io/wasm-pack/installer/init.sh | sh -s -- -f
```

**Option B – Cargo (requires Rust 1.86+ for wasm-pack 0.14+):**

```bash
rustup update stable
cargo install wasm-pack --force
```

### Testing the WASM build

**Option 1 – Node (after building for Node):**

```bash
# Build for Node.js
pnpm run wasm:build:node

# Run Node test script
pnpm run wasm:test
```

**Option 2 – Rust WASM tests in headless browser:**

```bash
# Requires Chrome/Chromium
pnpm run wasm:test:rust
# or: wasm-pack test --headless --chrome
```

The Node script (`scripts/wasm-test.mjs`) checks `calculate_rolling_absences` and `validate_absence_periods` with valid and invalid input.

## Usage Examples

### JSON Input Format

```json
[
  { "start_date": "2023-01-01", "end_date": "2023-01-10" },
  { "start_date": "2023-05-15", "end_date": "2023-05-20" }
]
```

### JavaScript/WASM Usage

```javascript
import init, { calculate_rolling_absences } from './pkg/ilr_calculator.js';

await init();

const input = JSON.stringify([
  { start_date: '2023-01-01', end_date: '2023-01-10' },
]);

const result = calculate_rolling_absences(input);
const results = JSON.parse(result);

console.log(results[0].total_days_in_window); // 10
```

## Testing

The project includes comprehensive test coverage:

```bash
# Run all tests
cargo test

# Run only library tests
cargo test --lib

# Run with output
cargo test -- --nocapture
```

### Test Coverage

- JSON parsing and validation
- Period merging (overlapping, adjacent, separate)
- Overlap calculations
- Rolling window calculations
- Edge cases (leap years, year boundaries, empty inputs)
- WASM bindings

## Architecture Decisions

1. **Separation of concerns**: Core logic is separated from CLI and WASM bindings
2. **WASM compatibility**: Core functions avoid `std::io` and other non-WASM-compatible APIs
3. **Error handling**: Uses `Result` types for proper error propagation
4. **Testability**: Pure functions make testing straightforward
5. **JSON-first**: WASM interface uses JSON strings for easy JavaScript integration

## Dependencies

### Rust Dependencies

- `chrono`: Date handling
- `serde` / `serde_json`: JSON serialization/deserialization
- `wasm-bindgen`: WebAssembly bindings (for WASM builds)

### Frontend Dependencies

- `nuxt`: Vue.js framework with SSR support
- `@nuxtjs/tailwindcss`: Tailwind CSS integration
- `clsx` / `tailwind-merge`: Utility for conditional classes
- `vue`: Progressive JavaScript framework

## License

[Add your license here]
