# CLI SSoT

## バイナリ名

`tp2m` ではなく `tpp2m` を正式名とする。

## 基本方針

- CLIは非対話処理の入口である。
- TUI起動も同じバイナリから行う。
- すべての計算は `crates/core` に委譲する。
- 出力形式はスクリプトで扱いやすいJSON/CSVを優先する。

## コマンド構成

```text
tpp2m [GLOBAL OPTIONS] <COMMAND>

Commands:
  calc    単点IMFP計算
  sweep   エネルギースイープ計算
  tui     Lazygit風TUIを起動
  help    ヘルプ
```

`COMMAND` なしの場合はTUIを起動する。ただし `--help` はヘルプを表示する。

## グローバルオプション

| オプション | 意味 | 既定 |
|---|---|---|
| `--quiet` | 警告以外を抑制 | false |
| `--verbose` | 診断情報を増やす | false |
| `--no-color` | ANSI色を無効化 | false |

## `calc`

### 例

```bash
tpp2m calc \
  --energy 1000 \
  --density 2.3296 \
  --molar-mass 28.0855 \
  --valence-electrons 4 \
  --band-gap 1.12
```

JSON:

```bash
tpp2m calc -E 1000 -r 2.3296 -M 28.0855 -v 4 -g 1.12 --json
```

### オプション

| 長い形式 | 短い形式 | 必須 | 値 | 備考 |
|---|---:|---:|---|---|
| `--energy` | `-E` | yes | eV | 単点電子エネルギー。 |
| `--density` | `-r` | yes | g/cm³ | `rho` のr。 |
| `--molar-mass` | `-M` | yes | g/mol | 大文字M。 |
| `--valence-electrons` | `-v` | yes | count | `Nv`。 |
| `--band-gap` | `-g` | no | eV | 既定0。 |
| `--allow-extrapolate` | - | no | flag | 推奨範囲外の計算を許可。 |
| `--json` | - | no | flag | JSON出力。 |
| `--precision` | - | no | integer | テキスト表示の有効数字。既定6。 |

### テキスト出力

既定のテキスト出力:

```text
IMFP: 2.38643 nm
```

`--verbose` 時:

```text
IMFP: 2.38643 nm (23.8643 Å)
Ep: 16.5891 eV
beta: 0.0318648
gamma: 0.125139 1/eV
C: 1.63891
D: 45.8354
```

### JSON出力スキーマ

```json
{
  "input": {
    "electron_energy_e_v": 1000.0,
    "density_g_cm3": 2.3296,
    "molar_mass_g_mol": 28.0855,
    "valence_electrons": 4.0,
    "band_gap_e_v": 1.12,
    "allow_extrapolate": false
  },
  "output": {
    "imfp_nm": 2.3864329956020653,
    "imfp_angstrom": 23.864329956020653,
    "plasmon_energy_e_v": 16.589071625484447,
    "beta": 0.03186483916389788,
    "gamma_inverse_e_v": 0.12513900238367897,
    "c": 1.63891,
    "d": 45.8354
  },
  "warnings": []
}
```

## `sweep`

### 例

```bash
tpp2m sweep \
  --energy-min 50 \
  --energy-max 2000 \
  --points 200 \
  --spacing log \
  --density 2.3296 \
  --molar-mass 28.0855 \
  --valence-electrons 4 \
  --band-gap 1.12 \
  --csv
```

### オプション

`calc` の材料パラメータに加えて以下を受け付ける。

| オプション | 必須 | 値 | 既定 |
|---|---:|---|---:|
| `--energy-min` | no | eV | 50 |
| `--energy-max` | no | eV | 2000 |
| `--points` | no | count | 200 |
| `--spacing` | no | `log` / `linear` | `log` |
| `--json` | no | flag | false |
| `--csv` | no | flag | true |

`--json` と `--csv` が両方指定された場合はエラーにする。

### CSV列

列順は固定する。

```csv
electron_energy_e_v,imfp_nm,imfp_angstrom,warning
```

## `tui`

```bash
tpp2m tui --density 2.3296 --molar-mass 28.0855 --valence-electrons 4 --band-gap 1.12
```

CLI引数はTUIの初期状態として使う。TUIの詳細は `docs/ssot/tui.md` を正とする。

## 終了コード

| コード | 意味 |
|---:|---|
| 0 | 成功。 |
| 1 | 入力検証エラー。 |
| 2 | 計算エラー。 |
| 3 | 出力形式エラー。 |
| 4 | TUI初期化エラー。 |
| 70 | 予期しない内部エラー。 |

## テスト観点

- `calc` のJSON出力が `calculation.md` のテストベクトルと一致する。
- `sweep` のCSV列順が固定される。
- 範囲外入力は `--allow-extrapolate` なしでエラーになる。
- `--allow-extrapolate` ありでは警告を出して成功する。
- `COMMAND` なしはTUI起動パスに入る。ただし自動テストではTUI runnerを直接起動せず、起動判定関数をテストする。
