# ADR-0001: Rust workspace and core boundary

## Status

Accepted

## Context

TPP-2M計算をCLI、TUI、WASMから呼び出す必要がある。各入口で式を再実装すると、単位・丸め・範囲外処理が分岐し、検証不能になる。

## Decision

Rust workspaceを使い、`crates/core`, `crates/cli`, `crates/tui`, `crates/wasm` に分ける。TPP-2M式、入力検証、スイープ、グラフ用log変換は `core` に集約する。

## Consequences

良い点:

- すべての入口で同じ計算結果になる。
- `core` を高カバレッジでテストできる。
- WASM化が容易になる。

悪い点:

- crate間DTO設計が必要になる。
- 初期セットアップが単一バイナリより少し重い。

## Alternatives considered

- 単一crateにCLI/TUI/WASMをまとめる: 境界が曖昧になるため不採用。
- TUIを主アプリにしてCLIをサブ機能にする: スクリプト利用が弱くなるため不採用。
