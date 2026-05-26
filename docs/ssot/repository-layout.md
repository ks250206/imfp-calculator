# Repository Layout SSoT

## 予定ツリー

```text
tpp2m/
├── Cargo.toml
├── AGENTS.md
├── crates/
│   ├── core/
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── input.rs
│   │       ├── formula.rs
│   │       ├── sweep.rs
│   │       ├── error.rs
│   │       └── tests.rs
│   ├── cli/
│   │   ├── Cargo.toml
│   │   └── src/main.rs
│   ├── tui/
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── main.rs
│   │       ├── app.rs
│   │       ├── action.rs
│   │       ├── keymap.rs
│   │       ├── layout.rs
│   │       ├── widgets/
│   │       └── render.rs
│   └── wasm/
│       ├── Cargo.toml
│       └── src/lib.rs
├── docs/
└── tests/
    ├── cli_golden/
    └── wasm_contract/
```

## Workspace方針

- ルート `Cargo.toml` は仮想workspaceとする。
- `resolver = "3"` を使う想定。
- 共通依存は `[workspace.dependencies]` で管理する。
- 共通lintは `[workspace.lints]` で管理する。

## クレート責務

### `crates/core`

責務:

- TPP-2M単点計算。
- エネルギースイープ生成。
- 両対数グラフ用データ生成。
- 入力検証。
- エラー型。
- DTO。ただしUI依存型は置かない。

禁止:

- `clap`, `ratatui`, `crossterm`, `wasm-bindgen` への依存。
- 端末描画。
- JS型への依存。
- ファイルI/O、ネットワークI/O。

推奨API例:

```rust
pub fn calculate(input: Tpp2mInput) -> Result<Tpp2mOutput, Tpp2mError>;
pub fn sweep(input: SweepInput) -> Result<SweepOutput, Tpp2mError>;
pub fn log_plot_points(input: SweepInput) -> Result<LogPlotData, Tpp2mError>;
```

### `crates/cli`

責務:

- コマンドライン引数の解釈。
- JSON/CSV/テキスト出力。
- 終了コード。
- `core` エラーのユーザー向け表示。

禁止:

- TPP-2M式の再実装。
- 独自の入力検証。ただしCLI固有の構文検証は可。

### `crates/tui`

責務:

- `ratatui` による描画。
- `crossterm` 等による端末イベント取得。
- Lazygit風ペインレイアウト。
- 数字フォーカスとVim風キー操作。
- AppState/reducerによるテスト可能な状態遷移。

禁止:

- TPP-2M式の再実装。
- 端末イベントと計算ロジックの密結合。

### `crates/wasm`

責務:

- `wasm-bindgen` でブラウザ向けAPIを公開する。
- JS互換DTOと `core` DTOの変換。
- `wasm-pack` ビルド設定。

禁止:

- TPP-2M式の再実装。
- DOM操作。WASM crateは計算APIに限定する。

## 依存方向

```text
cli  ─┐
tui  ─┼──> core
wasm ─┘
```

`core` は他クレートに依存しない。

## 受け入れ条件

- `cargo test --workspace --all-features` が全クレートを対象にする。
- `core` の公開API変更時はCLI/TUI/WASMの契約テストも更新される。
- 依存方向に逆流がない。
