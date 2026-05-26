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
| `j` / `k` | フィールドを上下に移動。 |
| `Tab` | 次フィールドへ移動。入力ペイン以外では次ペインへ移動。 |
| `Shift-Tab` | 前フィールドへ移動。入力ペイン以外では前ペインへ移動。 |
| `i` | 現在フィールドを編集。 |
| `a` | 現在フィールドを編集し末尾へ。 |
| `x` | 現在フィールドの値をクリア。 |
| `Enter` | 編集開始または確定。 |
| `h` / `l` | プリセット項目では前後のプリセットへ切り替える。 |

フォーム編集はフィールド選択とinline編集を基本とする。編集確定時に自動再計算する。

inline編集中は、端末カーソルを現在の入力位置に表示する。MVPでは入力位置は編集バッファ末尾でよい。

材料プリセット、Electron energyプリセット、Sweep範囲モードのような選択式フィールドでは、`Enter` で候補リストを開くか、`h` / `l` で前後の候補へ切り替える。

`energy_min_e_v` または `energy_max_e_v` を手動編集した場合、Sweep範囲モードは `manual` に切り替わる。

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
- Graphペインのプロット領域は端末テーマに依存せず白背景にする。
- 軸、目盛、軸ラベルは黒系、プロット線は赤系で描く。
- 白背景はGraphペインに限定し、他ペインは通常のTUI表示を維持する。

### 実装

Graphペインの描画は、`ratatui` のbufferへ直接描画する。標準の `ratatui::widgets::Chart` やCanvas系バックエンドでは、solid line、minor tick、内向きtick、上軸・右軸ミラーリング、白背景プロット領域の制御が不足しやすいためである。

描画座標は `core` が生成した `log10` 座標を使う。

```text
x = log10(Electron Energy / eV)
y = log10(IMFP / nm)
```

軸ラベル、ツールチップ、結果表示では元の単位に戻す。

例:

```text
x_tick: value_log10 = 3.0, label = "10³"
y_tick: value_log10 = 0.0, label = "10⁰"
```

### 目盛

X軸、Y軸とも、major tickは10の整数冪で表示する。ラベルは `10⁰`, `10¹`, `10²` のように、指数を上付き文字で表示し、`^` 記号は使わない。

minor tickは各decade内に表示する。major tickとminor tickはいずれもプロット領域の内向きに描く。枠線、tick、プロット線は点線ではなく実線で描く。

右軸と上軸は、それぞれ左軸と下軸をミラーリングする。右軸と上軸にもtickとminor tickを表示するが、軸ラベルとtickラベルは表示しない。

### 既定値

| 項目 | 値 |
|---|---:|
| energy_min_e_v | 50 |
| energy_max_e_v | 2000 |
| points | 200 |
| spacing | log |

## Electron energyプリセット

TUIでは `energy` ではなく `Electron energy` と表示する。

Electron energyには、任意のeV入力に加えて次のX線源プリセットを用意する。

| プリセット | electron_energy_e_v |
|---|---:|
| Al Kα | 1486.6 |
| Mg Kα | 1253.6 |
| Cr Kα | 5414.8 |
| Ga Kα | 9252.13 |

X線源プリセットはElectron energy入力補助であり、TPP-2M式そのものは変更しない。

## 材料プリセット

TUIには単元素材料プリセットを用意する。化合物、有機化合物、ポリマーの組み込みプリセットはMVPでは扱わない。

単元素材料プリセットは、TPP-2M入力用の代表値セットとして次を持つ。

| フィールド | 意味 |
|---|---|
| material_name | 表示名。例: `Si`, `Au`, `C`。 |
| density_g_cm3 | 密度。 |
| molar_mass_g_mol | 原子量。 |
| valence_electrons | TPP-2M入力として使う価電子数。 |
| band_gap_e_v | バンドギャップ。既定は0 eV。 |

元素プリセットデータは、再利用条件が明確な出典から再構成し、出典を `docs/references.md` またはプリセットデータ近傍に記録する。外部由来の元データファイルはリポジトリに含めない。

`valence_electrons` はTPP-2M入力用のプリセット値として固定し、周期表の族番号や電子配置から自動推定しない。ユーザーはTUI上で常に編集できる。

プリセット適用後にユーザーが値を編集した場合、材料名はユーザー編集値であることが分かる表示にする。例: `Custom from Si`。

## Sweep範囲モード

TUIのスイープ範囲は `auto` または `manual` で選択する。

| モード | 動作 |
|---|---|
| `auto` | `energy_min_e_v = 10`、`energy_max_e_v = electron_energy_e_v` とする。X線源プリセット選択時は線源エネルギーが上限になる。 |
| `manual` | ユーザーが `energy_min_e_v` と `energy_max_e_v` を直接編集する。 |

`manual` のフォームは常にTUI上に残す。`auto` の間は現在の自動範囲を表示し、手動編集を始めた時点で `manual` に切り替える。

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
- Graphペインのsnapshotまたはスクリーンショットで、白背景、黒系の軸・目盛・軸ラベル、赤系の実線プロット、上軸・右軸ミラーリング、major/minor tickが確認できる。
- 主要キー操作は端末実機なしに reducer テストで検証できる。
- 目視だけに依存した受け入れをしない。
