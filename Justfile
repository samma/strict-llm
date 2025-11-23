set shell := ["pwsh", "-NoProfile", "-Command"]

# Formatting
fmt:
    cargo fmt --all

# Clippy lint pass with full feature set
lint:
    cargo clippy --all-targets --all-features -D warnings

# High-throughput test runner
test:
    cargo nextest run --workspace --all-targets

# Native binary build
build-native:
    cargo build --all-targets

# WebAssembly build (assumes wasm32 target installed)
build-wasm:
    pwsh -NoProfile -ExecutionPolicy Bypass -File scripts/build_wasm.ps1

# Asset validation placeholder
asset-validate:
    pwsh -NoProfile -ExecutionPolicy Bypass -File scripts/validate_assets.ps1

# Run fmt -> lint -> test sequence
verify-core:
    just fmt
    just lint
    just test

# Build both native and wasm plus assets
build-all:
    just build-native
    just build-wasm
    just asset-validate

