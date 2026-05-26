# ADR-0003: wasm-pack browser API

## Status

Accepted

## Context

core機能をWebブラウザから呼び出したい。RustからWASMを生成し、JavaScriptと相互運用する必要がある。

## Decision

`crates/wasm` を作り、`wasm-bindgen` と `wasm-pack` を使う。公開APIはMVPではJSON互換の `calculate(input)` と `sweep(input)` にする。

## Consequences

良い点:

- ブラウザから同一coreを呼び出せる。
- `wasm-pack build --target web` でES module向け成果物を生成できる。
- JSON互換APIによりWebフォームとの接続が簡単になる。

悪い点:

- JS境界の型変換とエラー変換が必要。
- WASMバンドルサイズへの配慮が必要。

## Alternatives considered

- 手動 `wasm-bindgen` 実行: ユーザー要求が `wasm-pack` であるため不採用。
- REST APIサーバー: ブラウザ内で完結しないため不採用。
