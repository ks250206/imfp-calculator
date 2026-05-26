# TDD and Quality SSoT

## TDDスタイル

本プロジェクトは古典学派（Detroit/classical）TDDで進める。

特徴:

- 実オブジェクトを組み合わせたテストを優先する。
- モックは外部境界に限定する。
- テストは実装詳細ではなく振る舞いを記述する。
- 小さなサイクルで「失敗 → 成功 → リファクタリング」を回す。

## カバレッジ基準

| 対象 | 目標 |
|---|---:|
| workspace全体 line coverage | 80%以上 |
| `crates/core` line coverage | 95%以上 |
| `crates/cli` line coverage | 80%以上 |
| `crates/tui` reducer/keymap | 90%以上 |
| `crates/wasm` public API wrapper | 80%以上 |

CIでは最低でもworkspace全体80%をfail条件にする。

```bash
cargo llvm-cov --workspace --all-features --fail-under-lines 80
```

## テスト分類

### Core unit tests

対象:

- 式の中間値。
- IMFP計算。
- 入力検証。
- 範囲外処理。
- スイープ生成。
- `log10` グラフデータ生成。

基準:

- `calculation.md` のテストベクトルに一致する。
- 浮動小数点許容誤差を明示する。

### Core property tests

候補:

- 対数スイープのエネルギー列は単調増加する。
- `points = n` なら出力点数はnになる。
- `energy_min < energy_max` が保たれる限り、端点が期待値に一致する。
- 正常入力で有限の正のIMFPが返る。ただし物理的単調性は全材料で仮定しない。

### CLI integration tests

対象:

- `tpp2m calc --json`。
- `tpp2m calc` テキスト出力。
- `tpp2m sweep --csv`。
- エラー時の終了コード。

方針:

- 実バイナリを起動する。
- stdout/stderr/exit codeを検証する。
- fixtureを使い、モックしない。

### TUI reducer tests

対象:

- keymap。
- focus移動。
- edit mode遷移。
- Graphペイン操作。
- エラーメッセージ蓄積。

方針:

- 端末描画そのものより、`AppState` と `Action` をテストする。
- 必要に応じて小さなsnapshotでレイアウト名とペイン番号を固定する。

### WASM contract tests

対象:

- `calculate` 公開API。
- `sweep` 公開API。
- エラーオブジェクト。
- JSONキー互換性。

## モック制限

許可:

- 時計。
- ファイルI/O。
- 環境変数。
- 端末サイズ取得。
- 将来の外部DB/API。

非推奨:

- `core` の計算関数をモックする。
- TUIで計算結果だけをモックして状態遷移を隠す。
- CLIパーサーをモックしてコマンド実行を避ける。

## テスト名規約

英語の振る舞い名を推奨する。

例:

```rust
#[test]
fn calculates_silicon_imfp_at_1000_ev() {}

#[test]
fn rejects_energy_outside_recommended_range_without_extrapolation() {}

#[test]
fn number_keys_move_focus_to_matching_pane() {}
```

## PR受け入れ基準

- 失敗から始まったテストが含まれる。
- 新しい仕様にはSSoT更新がある。
- カバレッジが低下して80%未満にならない。
- `core` の式に触れたら、全入口の契約テストが通る。
- UIの挙動は目視だけでなくreducer/keymapテストがある。
