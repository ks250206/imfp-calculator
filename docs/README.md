# docs — TPP-2M Calculator SSoT

このディレクトリはTPP-2M CalculatorのSSoT（Single Source of Truth）を保持する。仕様がコードと矛盾した場合は、原則としてSSoTを正とし、コードまたはSSoTのどちらかをPR内で必ず修正する。

## 読む順番

1. `../CONTEXT.md` — 現在の実装文脈と最近の採用判断。
2. `ssot/product.md` — 何を作るか。
3. `ssot/repository-layout.md` — どこに実装するか。
4. `ssot/calculation.md` — 何をどう計算するか。
5. `ssot/cli.md` / `ssot/tui.md` / `ssot/wasm.md` — 入口別の仕様。
6. `ssot/tdd-and-quality.md` — どうテストするか。
7. `ssot/errors.md` — どう失敗を扱うか。
8. `adr/*.md` — なぜその選択をしたか。

## SSoT更新ルール

- 振る舞いを変えるPRは、必ず該当SSoTを更新する。
- SSoTの変更は、受け入れ条件とテスト観点まで書く。
- まだ決まっていない仕様は「未決」と明記し、実装側で勝手に固定しない。
- 計算式、単位、丸め、エラー、キー割り当て、WASM APIはSSoTなしに変更しない。

## 用語

用語は `glossary.md` に集約する。
