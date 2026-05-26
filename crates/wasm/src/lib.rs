use wasm_bindgen::prelude::*;

#[derive(serde::Deserialize, serde::Serialize)]
struct CalculateResponse {
    input: tpp2m_core::Tpp2mInput,
    output: tpp2m_core::Tpp2mOutput,
    warnings: Vec<tpp2m_core::Warning>,
}

#[wasm_bindgen]
pub fn calculate(input: JsValue) -> Result<JsValue, JsValue> {
    let input: tpp2m_core::Tpp2mInput = serde_wasm_bindgen::from_value(input)
        .map_err(|error| JsValue::from_str(&format!("invalid calculate input: {error}")))?;
    let output = tpp2m_core::calculate(input).map_err(error_to_js)?;
    let response = CalculateResponse {
        input,
        warnings: output.warnings.clone(),
        output,
    };
    serde_wasm_bindgen::to_value(&response)
        .map_err(|error| JsValue::from_str(&format!("failed to serialize output: {error}")))
}

#[wasm_bindgen]
pub fn sweep(input: JsValue) -> Result<JsValue, JsValue> {
    let input: tpp2m_core::SweepInput = serde_wasm_bindgen::from_value(input)
        .map_err(|error| JsValue::from_str(&format!("invalid sweep input: {error}")))?;
    let output = tpp2m_core::sweep(input).map_err(error_to_js)?;
    serde_wasm_bindgen::to_value(&output)
        .map_err(|error| JsValue::from_str(&format!("failed to serialize output: {error}")))
}

fn error_to_js(error: tpp2m_core::Tpp2mError) -> JsValue {
    serde_wasm_bindgen::to_value(&error)
        .unwrap_or_else(|_| JsValue::from_str("failed to serialize Tpp2mError"))
}

#[cfg(all(test, target_arch = "wasm32"))]
mod tests {
    use wasm_bindgen_test::*;

    wasm_bindgen_test_configure!(run_in_browser);

    #[wasm_bindgen_test]
    fn calculate_returns_reference_value() {
        let input = serde_wasm_bindgen::to_value(&tpp2m_core::Tpp2mInput {
            electron_energy_e_v: 1000.0,
            density_g_cm3: 2.3296,
            molar_mass_g_mol: 28.0855,
            valence_electrons: 4.0,
            band_gap_e_v: 1.12,
            allow_extrapolate: false,
        })
        .unwrap();

        let output = crate::calculate(input).unwrap();
        let output: CalculateResponse = serde_wasm_bindgen::from_value(output).unwrap();

        assert_eq!(output.input.electron_energy_e_v, 1000.0);
        assert!((output.output.imfp_nm - 2.3864329956020653).abs() < 1e-12);
    }

    #[wasm_bindgen_test]
    fn calculate_returns_structured_error() {
        let input = serde_wasm_bindgen::to_value(&tpp2m_core::Tpp2mInput {
            electron_energy_e_v: 5000.0,
            density_g_cm3: 2.3296,
            molar_mass_g_mol: 28.0855,
            valence_electrons: 4.0,
            band_gap_e_v: 1.12,
            allow_extrapolate: false,
        })
        .unwrap();

        let error = crate::calculate(input).unwrap_err();
        let error: tpp2m_core::Tpp2mError = serde_wasm_bindgen::from_value(error).unwrap();

        assert_eq!(error.code, tpp2m_core::ErrorCode::OutOfRecommendedRange);
        assert_eq!(error.field.as_deref(), Some("electron_energy_e_v"));
    }
}
