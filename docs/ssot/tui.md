# TUI SSoT

## 基本方針

TUIは `ratatui` を使い、Lazygit風の分割ペインUIとして実装する。端末バックエンドは既定で `crossterm` とする。

すべての計算は `crates/core` に委譲する。TUIは状態管理、入力編集、描画、メッセージ表示だけを担当する。

## 起動

```bash
tpp2m
# または
tpp2m tui
```

CLI引数で初期材料パラメータを渡せる。

```bash
tpp2m tui -r 2.3296 -M 28.0855 -v 4 -g 1.12 --energy-min 50 --energy-max 2000
```

## 画面レイアウト

Lazygit風に、左側に操作・入力ペイン、右側に結果・グラフペインを配置する。

```text
┌─ 1 Material/Input ─────────┬─ 3 IMFP log-log graph ─────────────────────┐
│ density                    │ y: IMFP / nm                               │
│ molar mass                 │                                             │
│ valence electrons          │                                             │
│ band gap                   │                                             │
├─ 2 Energy/Sweep ───────────┤                                             │
│ energy                     │                                             │
│ energy min/max             │                                             │
│ points / spacing           │                                             │
├─ 4 Result/Series ──────────┤                                             │
│ current IMFP               │ x: Electron Energy / eV                    │
│ table row                  │                                             │
├─ 5 Help/Log ───────────────┴─────────────────────────────────────────────┤
│ messages, warnings, key hints                                             │
└───────────────────────────────────────────────────────────────────────────┘
```

狭い端末では縦積みレイアウトにフォールバックする。ただしペイン番号とフォーカス移動は維持する。

## ペイン

| 番号 | ペイン | 役割 |
|---:|---|---|
| 1 | Material/Input | ρ, M, Nv, Egの編集。 |
| 2 | Energy/Sweep | 単点E、スイープ範囲、点数、log/linearの編集。 |
| 3 | IMFP log-log graph | 横軸Electron Energy/eV、縦軸IMFP/nmの両対数グラフ。 |
| 4 | Result/Series | 単点結果とスイープ表。 |
| 5 | Help/Log | キーヒント、警告、エラー、操作ログ。 |

## フォーカス移動

数字キーは常にフォーカス移動に使う。入力編集中も、Escで編集モードを抜ければ有効になる。

| キー | 動作 |
|---|---|
| `1` | Material/Inputへフォーカス。 |
| `2` | Energy/Sweepへフォーカス。 |
| `3` | Graphへフォーカス。 |
| `4` | Result/Seriesへフォーカス。 |
| `5` | Help/Logへフォーカス。 |
| `Tab` | 次ペイン。 |
| `Shift-Tab` | 前ペイン。 |
| `h` / `l` | 左右ペインへ移動。 |
| `Ctrl-h` / `Ctrl-l` | 左右ペインへ移動。 |

## Vim風操作

### 共通

| キー | 動作 |
|---|---|
| `j` | 次項目/下へ。 |
| `k` | 前項目/上へ。 |
| `g g` | 先頭へ。 |
| `G` | 末尾へ。 |
| `Ctrl-d` | 半ページ下へ。 |
| `Ctrl-u` | 半ページ上へ。 |
| `/` | ペイン内検索またはフィルタ開始。MVPではResult/Seriesで有効。 |
| `?` | ヘルプ表示トグル。 |
| `q` | 終了確認、またはヘルプを閉じる。 |
| `Esc` | 編集/検索/確認をキャンセル。 |
| `Enter` | 選択または編集確定。 |
| `r` | 再計算。通常は入力変更時に自動再計算するが、手動再計算も可能。 |
| `:` | コマンドラインモード。MVPでは `:q`, `:recalc`, `:json` を扱う。 |

### 入力ペイン

| キー | 動作 |
|---|---|
| `i` | 現在フィールドを編集。 |
| `a` | 現在フィールドを編集し末尾へ。 |
| `x` | 現在フィールドの値をクリア。 |
| `Enter` | 編集開始または確定。 |

### Graphペイン

| キー | 動作 |
|---|---|
| `h` / `l` | カーソルを前後のサンプル点へ移動。 |
| `j` / `k` | Y方向の読み取りカーソルを移動。MVPではサンプル点移動と同義でよい。 |
| `+` / `-` | エネルギー範囲をズーム。 |
| `0` | ズームを既定範囲に戻す。 |

## グラフ仕様

### 軸

- 横軸: `Electron Energy / eV`
- 縦軸: `IMFP / nm`
- 両軸とも対数表示。

### 実装

`ratatui` のChartに渡す座標は `core` が生成した `log10` 座標を使う。

```text
x = log10(Electron Energy / eV)
y = log10(IMFP / nm)
```

軸ラベル、ツールチップ、結果表示では元の単位に戻す。

例:

```text
x_tick: value_log10 = 3.0, label = "1000"
y_tick: value_log10 = 0.0, label = "1"
```

### 既定値

| 項目 | 値 |
|---|---:|
| energy_min_e_v | 50 |
| energy_max_e_v | 2000 |
| points | 200 |
| spacing | log |

### グラフエラー

- 有効点が2点未満の場合、グラフペインにはエラーメッセージを表示する。
- 計算できなかった点はResult/Seriesに警告付きで残す。ただしグラフ線からは除外する。

## 状態モデル

TUIは以下のような状態を持つ。

```rust
struct AppState {
    focused_pane: Pane,
    mode: Mode,
    material: MaterialForm,
    energy: EnergyForm,
    result: Option<Tpp2mOutput>,
    sweep: Option<SweepOutput>,
    graph: Option<LogPlotData>,
    messages: Vec<Message>,
    selected_row: usize,
}
```

キー入力は `Action` に変換してから reducer に渡す。

```rust
fn reduce(state: AppState, action: Action) -> AppState;
```

副作用のある端末I/Oは reducer の外に置く。

## テスト観点

- `1`〜`5` が対応ペインへフォーカスを移す。
- `Tab` と `Shift-Tab` がペイン順序を循環する。
- `h/j/k/l`, `gg`, `G`, `Ctrl-u`, `Ctrl-d` が各ペインで期待通りに働く。
- 入力編集後、`core` に渡る値が更新される。
- 計算エラー時にクラッシュせずHelp/Logにメッセージが残る。
- Graphデータは `log10` 座標であり、軸ラベルは `Electron Energy / eV` と `IMFP / nm` である。
- 狭い端末でもフォーカス番号とキー操作が壊れない。

## 受け入れ条件

- TUIのスクリーンショットまたはsnapshotで、5ペイン構成が確認できる。
- 主要キー操作は端末実機なしに reducer テストで検証できる。
- 目視だけに依存した受け入れをしない。
