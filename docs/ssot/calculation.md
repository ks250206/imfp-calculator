# Calculation SSoT — TPP-2M

この文書はTPP-2M計算の唯一の仕様である。CLI、TUI、WASMはこの仕様を直接実装せず、`crates/core` のAPIを呼ぶ。

## 入力

| フィールド | 記号 | 単位 | 制約 | 備考 |
|---|---:|---:|---|---|
| electron_energy_e_v | E | eV | `E > 0` | 単点計算の電子運動エネルギー。 |
| density_g_cm3 | ρ | g/cm³ | `ρ > 0` | 密度。 |
| molar_mass_g_mol | M | g/mol | `M > 0` | 原子量または分子量。 |
| valence_electrons | Nv | count | `Nv > 0` | 原子または分子あたりの価電子数。 |
| band_gap_e_v | Eg | eV | `Eg >= 0` | 金属では0。 |
| allow_extrapolate | - | bool | 既定 `false` | 推奨範囲外を許可するか。 |

## 出力

| フィールド | 単位 | 備考 |
|---|---:|---|
| imfp_nm | nm | 既定のユーザー向け結果。 |
| imfp_angstrom | Å | 式の直接結果。 |
| plasmon_energy_e_v | eV | `Ep`。 |
| beta | dimensionless | TPP-2M係数。 |
| gamma_inverse_e_v | 1/eV | 実装上は `γE` が無次元になるよう扱う。 |
| c | eV | TPP-2M係数。 |
| d | eV² | TPP-2M係数。 |
| warnings | list | 推奨範囲外など。 |

## 式

TPP-2Mの単点計算は次で定義する。

```text
Ep = 28.8 * sqrt(Nv * ρ / M)
U  = Nv * ρ / M = Ep^2 / 829.44
β  = -0.10 + 0.944 / sqrt(Ep^2 + Eg^2) + 0.069 * ρ^0.1
γ  = 0.191 * ρ^-0.5
C  = 1.97 - 0.91 * U
D  = 53.4 - 20.8 * U
λÅ = E / (Ep^2 * (β * ln(γ * E) - C / E + D / E^2))
λnm = λÅ / 10
```

実装では浮動小数点は `f64` を使う。

### 注意

- `ln(γE)` の引数は正でなければならない。
- 分母が0以下、NaN、無限大の場合は計算エラーにする。
- `λÅ <= 0` は計算エラーにする。
- `Ep`, `β`, `γ`, `C`, `D` は診断のため出力可能にする。

## 推奨エネルギー範囲

MVPの既定では `50 eV <= E <= 2000 eV` を推奨範囲とする。

- `allow_extrapolate = false` の場合、範囲外は `OutOfRecommendedRange` エラーにする。
- `allow_extrapolate = true` の場合、計算は行い `warnings` に範囲外警告を入れる。
- `allow_extrapolate = true` の結果はTPP-2M式の機械的な外挿であり、`50 eV` 未満の低エネルギー領域で一般に知られるIMFP増大傾向を保証しない。

この範囲はプロジェクト既定であり、将来の論文・検証に基づきADR付きで変更できる。

## スイープ

### 入力

| フィールド | 単位 | 制約 | 既定 |
|---|---:|---|---:|
| energy_min_e_v | eV | `> 0` | 50 |
| energy_max_e_v | eV | `> energy_min_e_v` | 2000 |
| points | count | `2 <= points <= 10000` | 200 |
| spacing | enum | `log` または `linear` | `log` |

### 対数間隔

`points = n` のとき、`i = 0..n-1` について次を使う。

```text
t = i / (n - 1)
E_i = 10^(log10(E_min) + t * (log10(E_max) - log10(E_min)))
```

### 線形間隔

```text
t = i / (n - 1)
E_i = E_min + t * (E_max - E_min)
```

### 出力順

エネルギー昇順で返す。

## TUIグラフ用データ

TUIでは `ratatui` の描画座標に渡す前に、`core` 側で次のデータを返す。

```text
x = log10(E_eV)
y = log10(IMFP_nm)
```

ただしUI表示の軸ラベルは元の物理単位に戻し、次の文言を使う。

- X軸: `Electron Energy / eV`
- Y軸: `IMFP / nm`

グラフデータには次を含める。

- `points_log10: Vec<(f64, f64)>`
- `x_ticks: Vec<Tick { value_log10, label }>`
- `y_ticks: Vec<Tick { value_log10, label }>`
- `raw_points: Vec<SweepPoint>`

## 丸めと表示

- 内部計算は丸めない。
- CLIテキスト表示の既定は有効数字6桁。
- JSON出力は `f64` をそのままシリアライズする。
- CSV出力は列ごとに十分な桁を保つ。既定は `%.12g` 相当。
- TUI表示はスペース制約に応じて有効数字4〜6桁に丸めるが、詳細ペインでより長い値を表示してよい。

## 契約テストベクトル

以下はこのプロジェクトの実装固定用テストベクトルである。外部測定値ではなく、上記式を `f64` で評価した期待値である。

許容誤差:

- `Ep`, `β`, `γ`, `C`, `D`: 絶対誤差 `1e-10`。
- `imfp_nm`: 相対誤差 `1e-9`。

### Silicon-like input

入力:

```json
{
  "density_g_cm3": 2.3296,
  "molar_mass_g_mol": 28.0855,
  "valence_electrons": 4.0,
  "band_gap_e_v": 1.12,
  "allow_extrapolate": false
}
```

中間値:

| 値 | 期待値 |
|---|---:|
| Ep | 16.589071625484447 |
| beta | 0.03186483916389788 |
| gamma | 0.12513900238367897 |

単点:

| E / eV | IMFP / nm |
|---:|---:|
| 50 | 0.41606265357756095 |
| 100 | 0.530580020540264 |
| 200 | 0.7615824795629295 |
| 500 | 1.4122460878393244 |
| 1000 | 2.3864329956020653 |
| 2000 | 4.149226034982695 |

### Gold-like input

入力:

```json
{
  "density_g_cm3": 19.32,
  "molar_mass_g_mol": 196.96657,
  "valence_electrons": 11.0,
  "band_gap_e_v": 0.0,
  "allow_extrapolate": false
}
```

中間値:

| 値 | 期待値 |
|---|---:|
| Ep | 29.9154906590236 |
| beta | 0.02433458269617972 |
| gamma | 0.0434540046209592 |

単点:

| E / eV | IMFP / nm |
|---:|---:|
| 50 | 0.4856942674203609 |
| 100 | 0.38577904714110545 |
| 200 | 0.46124807574715654 |
| 500 | 0.7646817632068598 |
| 1000 | 1.2302667858782024 |
| 2000 | 2.066118649972704 |

### Silicon dioxide-like input

入力:

```json
{
  "density_g_cm3": 2.2,
  "molar_mass_g_mol": 60.0843,
  "valence_electrons": 16.0,
  "band_gap_e_v": 9.0,
  "allow_extrapolate": false
}
```

中間値:

| 値 | 期待値 |
|---|---:|
| Ep | 22.04364034088592 |
| beta | 0.014307577867941158 |
| gamma | 0.12877217373047924 |

単点:

| E / eV | IMFP / nm |
|---:|---:|
| 50 | 0.7148954594936587 |
| 100 | 0.7820400704300251 |
| 200 | 1.0206622261925593 |
| 500 | 1.8089998055924568 |
| 1000 | 3.0214466258686064 |
| 2000 | 5.228781492465633 |

## 実装上の禁止事項

- `core` 以外で式を再実装しない。
- `nm` 出力のために係数を改変しない。必ず `λÅ / 10` とする。
- 画面表示用の丸め値を再計算入力に使わない。
- 範囲外値を警告なしで通常値として扱わない。
