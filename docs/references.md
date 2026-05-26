# References

実装前に一次情報または公式資料を確認する。仕様としての正本は `docs/ssot/*.md` であり、外部リンクは根拠確認用である。

## TPP-2M

- NIST SRD 71: Electron Inelastic-Mean-Free-Path Database — https://www.nist.gov/srd/nist-standard-reference-database-71
- Tanuma, Powell, Penn TPP-2M関連論文・NIST資料。計算式は `docs/ssot/calculation.md` を正とする。

## Rust / Workspace

- The Cargo Book: Workspaces — https://doc.rust-lang.org/cargo/reference/workspaces.html
- The Rust Book: Cargo Workspaces — https://doc.rust-lang.org/book/ch14-03-cargo-workspaces.html

## Ratatui

- Ratatui API documentation — https://docs.rs/ratatui/latest/ratatui/
- Ratatui website — https://ratatui.rs/

## Element presets

- periodic-table-on-an-enum crate — https://docs.rs/periodic-table-on-an-enum
- PubChem Periodic Table data, as surfaced by the periodic-table-on-an-enum crate. Preset `valence_electrons` values are local TPP-2M input defaults and are not inferred from periodic-table group numbers at runtime.

## WASM

- wasm-pack book: build command — https://rustwasm.github.io/docs/wasm-pack/commands/build.html
- MDN: Compiling from Rust to WebAssembly — https://developer.mozilla.org/en-US/docs/WebAssembly/Guides/Rust_to_Wasm

## Coverage

- cargo-llvm-cov — https://github.com/taiki-e/cargo-llvm-cov
- rustc book: instrumentation-based code coverage — https://doc.rust-lang.org/rustc/instrument-coverage.html
