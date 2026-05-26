# ADR-0002: Ratatui Lazygit-style TUI

## Status

Accepted

## Context

ユーザーはTUI、Lazygit風UI、数字キーでフォーカス移動、Vim操作、両対数グラフを要求している。

## Decision

TUIは `ratatui` を使い、端末バックエンドは `crossterm` を既定とする。UIは5ペイン構成とし、`1`〜`5` の数字キーをペインフォーカスに固定する。Vim風操作を標準キーマップとして採用する。

グラフは `core` が生成する `log10(E_eV), log10(IMFP_nm)` の座標列を `ratatui` のChartに描画する。軸ラベルは元単位で表示する。

## Consequences

良い点:

- Lazygit風の高速なキーボード操作が実現できる。
- reducerを切り出せば端末なしにテストできる。
- グラフ座標変換をcoreに置くことでTUI依存の数値バグを減らせる。

悪い点:

- Chart自体は物理的な対数軸を直接理解しないため、軸ラベル変換を丁寧に実装する必要がある。
- Vim風操作と入力編集モードの衝突を状態機械で管理する必要がある。

## Alternatives considered

- `tui-rs`: メンテナンス継続性の観点で `ratatui` を優先。
- 独自ANSI描画: グラフとレイアウトの保守が重いため不採用。
