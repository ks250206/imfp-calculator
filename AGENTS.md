# AGENTS.md — TPP-2M Calculator

このリポジトリは、TPP-2M式で電子の非弾性平均自由行程（IMFP）を計算する Rust アプリケーションである。すべての実装エージェント、人間開発者、レビュー担当者は、このファイル、`CONTEXT.md`、`docs/` 配下のSSoTを優先する。

## 絶対ルール

1. **計算ロジックは `crates/core` に集約する。** CLI、TUI、WASMは `core` を呼ぶだけにし、TPP-2M式、単位変換、入力検証、スイープ生成、グラフ用サンプル生成を複製してはならない。
2. **SSoTを先に更新する。** 仕様、入出力、キー操作、エラー、式、テスト基準を変えるPRでは、実装より先に該当する `docs/ssot/*.md` を更新する。
3. **Rust workspace構成を守る。** 予定構成は `docs/ssot/repository-layout.md` に従う。
4. **古典学派（Detroit/classical）TDDで実装する。** 失敗するテストを書く → 最小実装 → リファクタリング、の順番を守る。モック主導設計は禁止しないが、外部境界以外では極力使わない。
5. **カバレッジ目標は workspace 全体で80%以上。** `core` は95%以上を目標にする。UIの描画詳細より、状態遷移・入力解釈・計算連携を厚くテストする。
6. **TUIは `ratatui` を使う。** Lazygit風の分割ペイン、数字キーによるフォーカス移動、Vim風操作、両対数IMFPグラフを備える。
7. **WASMは `wasm-pack` でビルドする。** ブラウザ向けAPIは `crates/wasm` に閉じ込め、計算は必ず `crates/core` を呼ぶ。
8. **安全で移植可能なRustを優先する。** `unsafe` は原則禁止。必要な場合はADRを追加し、代替案と安全性の根拠を書く。
9. **物理単位を曖昧にしない。** 内部式はÅを経由してよいが、ユーザー向け既定出力は nm とする。入力単位は eV、g/cm³、g/mol、個数、eV に固定する。
10. **TPP-2Mの妥当範囲外は明示する。** 範囲外入力を黙って通常値として扱わない。必要なら `--allow-extrapolate` 相当の明示フラグで警告を伴って許可する。

## SSoTマップ

| 領域 | SSoT |
|---|---|
| プロダクト目的・MVP | `docs/ssot/product.md` |
| リポジトリ構成・クレート責務 | `docs/ssot/repository-layout.md` |
| TPP-2M計算式・単位・テストベクトル | `docs/ssot/calculation.md` |
| CLI仕様 | `docs/ssot/cli.md` |
| TUI仕様 | `docs/ssot/tui.md` |
| WASM仕様 | `docs/ssot/wasm.md` |
| TDD・品質・カバレッジ | `docs/ssot/tdd-and-quality.md` |
| エラー・診断 | `docs/ssot/errors.md` |
| リリース・互換性 | `docs/ssot/release.md` |
| 用語 | `docs/glossary.md` |
| 採用判断 | `docs/adr/*.md` |
| 実装文脈・現在の判断の入口 | `CONTEXT.md` |

## 予定ワークスペース構成

```text
imfp-calculator/
├── AGENTS.md
├── Cargo.toml
├── crates/
│   ├── core/   # 純粋なTPP-2M計算、検証、スイープ、DTO
│   ├── cli/    # clap等による非対話CLI
│   ├── tui/    # ratatui/crosstermによるLazygit風TUI
│   └── wasm/   # wasm-bindgen + wasm-pack用ブラウザAPI
└── docs/
```

## 標準コマンド

実装後、PR前に以下を通す。

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace --all-features
cargo llvm-cov --workspace --all-features --fail-under-lines 80
wasm-pack build crates/wasm --release --target web --out-dir ../../pkg
```

必要に応じてHTMLカバレッジを確認する。

```bash
cargo llvm-cov --workspace --all-features --html
```

## TDD運用

1. 変更対象のSSoTを読む。
2. 期待振る舞いをテスト名で表す。
3. まず失敗するテストを書く。
4. 最小実装で通す。
5. 重複や命名をリファクタリングする。
6. `core` に寄せられるロジックがUIに混ざっていないか確認する。
7. カバレッジ低下時は、テスト追加または責務分割で解消する。

## モック方針

- `core` ではモックを使わない。
- CLIは実バイナリを `assert_cmd` 等で呼ぶ統合テストを優先する。
- TUIは端末をモックするのではなく、`AppState` と `Action` / `Event` の純粋な reducer をテストする。
- WASMは `wasm-bindgen-test` またはNode/ブラウザ相当の実行系で、公開APIの契約を検証する。
- ファイルシステム、環境変数、時計、乱数、ネットワークなど外部境界だけ、薄いアダプタまたはテスト用fixtureを許可する。

## TUI実装ガードレール

- `ratatui` + `crossterm` を既定とする。
- Lazygit風に複数ペインを常時表示する。
- 数字キーは `docs/ssot/tui.md` のフォーカス可能ペインに割り当てる。Help/Logのような非操作ペインはフォーカス対象にしない。
- Vim風操作を提供する。最低限 `h/j/k/l`, `gg`, `G`, `Ctrl-u`, `Ctrl-d`, `/`, `?`, `q` を扱う。
- グラフは横軸 `Electron Energy / eV`、縦軸 `IMFP / nm` の両対数表示とする。
- グラフ描画は `core` が返す数値列を `log10` 変換して `ratatui` の描画座標に渡す。軸ラベルは元の物理単位で表示する。

## レビュー観点

- UIごとに計算式が重複していないか。
- `nm` と `Å`、`eV` とその他単位が混ざっていないか。
- エラーがユーザーに説明可能か。
- TUIのキーマップがSSoTと一致しているか。
- WASM APIがRust内部型に過度に依存していないか。
- テストが実装詳細ではなく振る舞いを固定しているか。
- カバレッジが80%を下回っていないか。

## 変更時のチェックリスト

- [ ] 関連SSoTを更新した。
- [ ] 失敗するテストから始めた。
- [ ] `core` に置くべきロジックをUI層に置いていない。
- [ ] CLI/TUI/WASMすべてが同じ `core` APIを使う。
- [ ] TUIキー操作のテストを追加または更新した。
- [ ] WASM公開APIの後方互換性を確認した。
- [ ] `cargo fmt`, `clippy`, `test`, `llvm-cov`, `wasm-pack build` を通した。
