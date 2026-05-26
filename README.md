# imfp-calculator

Rust workspace for calculating electron inelastic mean free paths (IMFP) with the TPP-2M formula.

## Features

- Shared calculation core for CLI, TUI, and WASM.
- Single-point IMFP calculation in nm.
- Energy sweep output as CSV or JSON.
- Lazygit-style TUI with editable material parameters and a log-log IMFP plot.
- Browser-facing WASM API via `wasm-pack`.

## Workspace

```text
crates/core   TPP-2M calculation, validation, sweep, graph data
crates/cli    command-line interface
crates/tui    ratatui/crossterm terminal UI
crates/wasm   wasm-bindgen API boundary
docs/         SSoT and ADRs
```

## CLI

```bash
cargo run -p tpp2m-cli -- calc \
  --energy 1000 \
  --density 2.3296 \
  --molar-mass 28.0855 \
  --valence-electrons 4 \
  --band-gap 1.12
```

```bash
cargo run -p tpp2m-cli -- sweep \
  --energy-min 50 \
  --energy-max 2000 \
  --points 200 \
  --spacing log \
  --density 2.3296 \
  --molar-mass 28.0855 \
  --valence-electrons 4 \
  --band-gap 1.12 \
  --csv
```

Start the TUI:

```bash
cargo run -p tpp2m-cli -- tui
```

## Quality Gates

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace --all-features
cargo llvm-cov --workspace --all-features --fail-under-lines 80
wasm-pack build crates/wasm --target web --out-dir ../../pkg
```

## Documentation

Project source-of-truth documents are in `docs/`. Start with `docs/README.md`.

## Citation

The TPP-2M formula is based on:

S. Tanuma, C. J. Powell, D. R. Penn, Surf. Interf. Anal., Vol. 21, 165 (1994).

## License

MIT. See `LICENSE`.
