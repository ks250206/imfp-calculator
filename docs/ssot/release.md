# Release SSoT

## バージョニング

SemVerを使う。

- `0.x`: APIとUIが流動的な初期開発。
- `1.0`: CLI引数、WASM JSON API、core主要DTOが安定。

## 互換性ポリシー

### CLI

破壊的変更:

- コマンド名の削除。
- オプション名の変更。
- JSON/CSVキー名または列順の変更。
- 成功/失敗のexit code変更。

非破壊的変更:

- 新オプション追加。
- JSONへの新フィールド追加。
- 警告の追加。ただし既存警告コードの意味変更は破壊的。

### TUI

破壊的変更:

- 数字キーによるフォーカス移動の削除。
- Vim基本操作の削除。
- グラフ軸名または単位の変更。

### WASM

破壊的変更:

- `calculate` / `sweep` の削除。
- JSONキー名変更。
- エラーオブジェクトの必須キー削除。

## リリース前チェック

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace --all-features
cargo llvm-cov --workspace --all-features --fail-under-lines 80
wasm-pack build crates/wasm --target web --out-dir ../../pkg
```

## CHANGELOG

リリース時は `CHANGELOG.md` を更新する。分類は次を使う。

- Added
- Changed
- Deprecated
- Removed
- Fixed
- Security
