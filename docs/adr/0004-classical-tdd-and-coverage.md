# ADR-0004: Classical TDD and coverage policy

## Status

Accepted

## Context

ユーザーは古典学派スタイルのTDD、モック最小、カバレッジ80%以上を要求している。

## Decision

Detroit/classical style TDDを採用する。`core` は実関数中心の単体テスト、CLIは実バイナリ統合テスト、TUIはreducer/keymapテスト、WASMは公開API契約テストを中心にする。

カバレッジ測定は `cargo llvm-cov` を想定し、workspace全体80%以上をCI fail条件にする。

## Consequences

良い点:

- モック前提の脆い設計を避けられる。
- 計算coreの正確性を高密度に検証できる。
- UI層も状態遷移としてテストしやすい。

悪い点:

- TUI描画そのものの完全自動検証は難しい。
- 実バイナリ統合テストはユニットテストより遅い。

## Alternatives considered

- London/mockist TDD: 外部境界が少ない計算アプリには過剰なため不採用。
- カバレッジを努力目標にする: 品質ゲートが弱くなるため不採用。
