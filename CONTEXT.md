# CONTEXT.md — TPP-2M Calculator

このファイルは、実装者が最初に読むためのプロジェクト文脈である。仕様の正本は `AGENTS.md` と `docs/ssot/*.md`、採用判断は `docs/adr/*.md` に置く。

## 現在の目的

TPP-2M式で電子の非弾性平均自由行程（IMFP）を計算するRust workspaceを作る。入口はCLI、TUI、WASMの3つだが、計算ロジックは必ず `crates/core` に集約する。

MVPでは次を重視する。

- CLIから単点計算とスイープを再現可能に実行できる。
- TUIで材料パラメータ、Electron energy、スイープ範囲を編集しながら結果と両対数グラフを確認できる。
- WASMはDOM操作を持たず、ブラウザ向け計算APIだけを公開する。
- 古典学派TDDで、workspace全体のラインカバレッジ80%以上を維持する。

## 実装境界

- `crates/core`: TPP-2M式、単位変換、検証、スイープ、グラフ用 `log10` データ生成。
- `crates/cli`: `clap` による非対話CLI。計算は `core` を呼ぶ。
- `crates/tui`: `ratatui` + `crossterm` によるTUI。状態遷移はreducerでテストする。
- `crates/wasm`: `wasm-bindgen` API境界。計算は `core` を呼ぶ。

UI層にTPP-2M式、係数計算、単位変換、スイープ生成を複製しない。

## 最近の採用判断

- TUIのGraphペインは `plotters-ratatui-backend` + Plottersで描画する。理由は `docs/adr/0005-plotters-ratatui-graph-rendering.md` を参照する。
- Graphペインだけ端末テーマに依存しない白背景にし、軸・tick・ラベルは黒系、プロット線は青系にする。
- X/Y軸は両対数表示で、major tickは `10⁰` のような上付き指数表記にする。minor tickも表示し、tickは内向きに描く。上軸・右軸は下軸・左軸をミラーリングする。
- TUIでは `energy` ではなく `Electron energy` と表示する。
- Electron energyには Al Kα、Mg Kα、Cr Kα、Ga Kα のプリセットを持つ。
- スイープ範囲は `auto` / `manual` を持つ。`auto` は `10..=electron_energy_e_v`、`manual` はユーザー編集値を使う。
- 材料プリセットはMVPでは単元素材料だけにする。化合物、有機化合物、ポリマーの組み込みプリセットは扱わない。
- 単元素材料プリセットの `valence_electrons` はTPP-2M入力用の固定プリセット値であり、実行時に周期表の族番号や電子配置から自動推定しない。
- 外部由来の表ファイルはリポジトリに含めない。必要ならローカル参照に留める。

## 仕様を変える時

振る舞いを変える場合は、実装より先に該当SSoTを更新する。

- プロダクト目的: `docs/ssot/product.md`
- 計算式・単位: `docs/ssot/calculation.md`
- CLI: `docs/ssot/cli.md`
- TUI: `docs/ssot/tui.md`
- WASM: `docs/ssot/wasm.md`
- エラー: `docs/ssot/errors.md`
- 品質・TDD: `docs/ssot/tdd-and-quality.md`
- 用語: `docs/glossary.md`
- 採用判断: `docs/adr/*.md`

## 標準確認

PR前には少なくとも次を通す。

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace --all-features
cargo llvm-cov --workspace --all-features --fail-under-lines 80
wasm-pack build crates/wasm --target web --out-dir ../../pkg
```
