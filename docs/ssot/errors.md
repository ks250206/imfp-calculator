# Errors SSoT

## 基本方針

エラーはユーザーが修正できる形で返す。panicで制御フローを作らない。

## エラー型

`core` のエラーは概念的に次を持つ。

```rust
pub struct Tpp2mError {
    pub code: ErrorCode,
    pub message: String,
    pub field: Option<&'static str>,
    pub details: ErrorDetails,
}
```

## ErrorCode

| code | 意味 | CLI exit |
|---|---|---:|
| `InvalidInput` | 正であるべき値が0以下、NaN、無限大など。 | 1 |
| `OutOfRecommendedRange` | 推奨範囲外で `allow_extrapolate=false`。 | 1 |
| `InvalidSweepRange` | スイープ範囲または点数が不正。 | 1 |
| `NonPositiveLogArgument` | `ln(γE)` の引数が正でない。 | 2 |
| `NonPositiveDenominator` | TPP-2M式の分母が0以下。 | 2 |
| `NonFiniteResult` | NaNまたは無限大。 | 2 |
| `OutputFormatConflict` | `--json` と `--csv` の同時指定など。 | 3 |
| `TerminalInitialization` | TUI初期化失敗。 | 4 |
| `Internal` | 予期しない内部エラー。 | 70 |

## 警告

警告は計算成功時にも返る。

| warning | 条件 |
|---|---|
| `EnergyOutsideRecommendedRange` | `allow_extrapolate=true` で範囲外Eを計算した。 |
| `SomeSweepPointsFailed` | スイープ内の一部点が失敗した。 |
| `GraphPointsOmitted` | グラフに描けない点を除外した。 |

## CLI表示

stderr:

```text
error[OutOfRecommendedRange]: electron_energy_e_v must be within 50..=2000 eV unless --allow-extrapolate is set
  field: electron_energy_e_v
  value: 5000
```

JSONエラー出力:

```json
{
  "error": {
    "code": "OutOfRecommendedRange",
    "message": "electron_energy_e_v must be within 50..=2000 eV unless allow_extrapolate is true",
    "field": "electron_energy_e_v",
    "details": { "value": 5000.0, "min": 50.0, "max": 2000.0 }
  }
}
```

## TUI表示

- Help/Logペインにエラーを積む。
- 入力フィールドに紐づくエラーは該当行を強調する。
- 計算失敗時もTUIは終了しない。

## WASM表示

JS側には次の形で返す。

```js
{
  code: "InvalidInput",
  message: "density_g_cm3 must be positive and finite",
  field: "density_g_cm3",
  details: { value: -1 }
}
```

## テスト観点

- すべてのErrorCodeにCLI exit code対応がある。
- CLI/TUI/WASMのメッセージは同じ `core` error codeを起点にする。
- panicを期待するテストを書かない。panicはバグとして扱う。
