# WASM SSoT

## 基本方針

WASMはWebブラウザから `crates/core` のTPP-2M計算を呼ぶための薄い境界である。ビルドには `wasm-pack` を使う。

## クレート

`crates/wasm` は以下を満たす。

- `crate-type = ["cdylib", "rlib"]` を設定する。
- `wasm-bindgen` を使って公開APIを定義する。
- `serde` / `serde-wasm-bindgen` によりJS値とRust DTOを変換する。
- 計算は `tpp2m-core` に委譲する。

## 公開API

MVPの公開APIはJSON互換入力を受け取る。

```rust
#[wasm_bindgen]
pub fn calculate(input: JsValue) -> Result<JsValue, JsValue>;

#[wasm_bindgen]
pub fn sweep(input: JsValue) -> Result<JsValue, JsValue>;
```

将来的に型付きAPIを追加してよいが、上記JSON互換APIは後方互換性を維持する。

## 入力例

```js
const result = calculate({
  electron_energy_e_v: 1000,
  density_g_cm3: 2.3296,
  molar_mass_g_mol: 28.0855,
  valence_electrons: 4,
  band_gap_e_v: 1.12,
  allow_extrapolate: false,
});
```

## 出力例

```js
{
  input: { /* normalized input */ },
  output: {
    imfp_nm: 2.3864329956020653,
    imfp_angstrom: 23.864329956020653,
    plasmon_energy_e_v: 16.589071625484447,
    beta: 0.03186483916389788,
    gamma_inverse_e_v: 0.12513900238367897,
    c: 1.63891,
    d: 45.8354
  },
  warnings: []
}
```

## ビルド

WASM配布物はrelease buildを標準とする。workspaceの `[profile.release]` はサイズ優先に設定し、`crates/wasm` の `package.metadata.wasm-pack.profile.release` で `wasm-opt -Oz` を明示する。Rustが出すbulk-memory等の命令を `wasm-opt` が検証できるよう、必要なWebAssembly feature flagも併せて指定する。

ブラウザ向けES module生成:

```bash
wasm-pack build crates/wasm --release --target web --out-dir ../../pkg
```

バンドラ向け:

```bash
wasm-pack build crates/wasm --release --target bundler --out-dir ../../pkg-bundler
```

Nodeテスト用:

```bash
wasm-pack build crates/wasm --release --target nodejs --out-dir ../../pkg-node
```

## エラー

- JS側には説明可能なオブジェクトを返す。
- Rust panicをJS API境界へ露出させない。
- `Tpp2mError` は `code`, `message`, `field`, `details` を持つJS値に変換する。

例:

```js
{
  code: "OutOfRecommendedRange",
  message: "electron_energy_e_v must be within 50..=2000 eV unless allow_extrapolate is true",
  field: "electron_energy_e_v",
  details: { value: 5000, min: 50, max: 2000 }
}
```

## テスト

- `wasm-bindgen-test` で `calculate` と `sweep` を検証する。
- `calculation.md` のテストベクトルをWASM公開API越しに確認する。
- JSONキー名の互換性テストを追加する。

## 禁止事項

- WASM crate内に式をコピーしない。
- DOM操作をしない。
- UI表示用丸め値をAPI結果に混ぜない。
- API名やJSONキーをSSoTなしに変更しない。
