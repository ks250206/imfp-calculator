use serde::{Deserialize, Serialize};
use serde_json::json;

use crate::error::{ErrorCode, Tpp2mError};
use crate::input::Tpp2mInput;

pub const RECOMMENDED_ENERGY_MIN_EV: f64 = 50.0;
pub const RECOMMENDED_ENERGY_MAX_EV: f64 = 2000.0;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub enum WarningCode {
    EnergyOutsideRecommendedRange,
    SomeSweepPointsFailed,
    GraphPointsOmitted,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Warning {
    pub code: WarningCode,
    pub message: String,
    pub field: Option<String>,
}

impl Warning {
    pub fn energy_outside_recommended_range(value: f64) -> Self {
        Self {
            code: WarningCode::EnergyOutsideRecommendedRange,
            message: format!(
                "electron_energy_e_v {value} is outside the recommended 50..=2000 eV range"
            ),
            field: Some("electron_energy_e_v".to_string()),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct Coefficients {
    pub plasmon_energy_e_v: f64,
    pub beta: f64,
    pub gamma_inverse_e_v: f64,
    pub c: f64,
    pub d: f64,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Tpp2mOutput {
    pub imfp_nm: f64,
    pub imfp_angstrom: f64,
    pub plasmon_energy_e_v: f64,
    pub beta: f64,
    pub gamma_inverse_e_v: f64,
    pub c: f64,
    pub d: f64,
    pub warnings: Vec<Warning>,
}

pub fn calculate(input: Tpp2mInput) -> Result<Tpp2mOutput, Tpp2mError> {
    validate_input(input)?;

    let mut warnings = Vec::new();
    if !is_energy_recommended(input.electron_energy_e_v) {
        if input.allow_extrapolate {
            warnings.push(Warning::energy_outside_recommended_range(
                input.electron_energy_e_v,
            ));
        } else {
            return Err(Tpp2mError::out_of_recommended_range(
                input.electron_energy_e_v,
                RECOMMENDED_ENERGY_MIN_EV,
                RECOMMENDED_ENERGY_MAX_EV,
            ));
        }
    }

    let coefficients = coefficients(input)?;
    let gamma_energy = coefficients.gamma_inverse_e_v * input.electron_energy_e_v;
    if gamma_energy <= 0.0 {
        return Err(Tpp2mError::calculation(
            ErrorCode::NonPositiveLogArgument,
            "ln(gamma * E) argument must be positive",
            json!({ "gamma_energy": gamma_energy }),
        ));
    }

    let denominator_inner = coefficients.beta * gamma_energy.ln()
        - coefficients.c / input.electron_energy_e_v
        + coefficients.d / input.electron_energy_e_v.powi(2);
    let denominator = coefficients.plasmon_energy_e_v.powi(2) * denominator_inner;
    if !denominator.is_finite() {
        return Err(Tpp2mError::calculation(
            ErrorCode::NonFiniteResult,
            "TPP-2M denominator is not finite",
            json!({ "denominator": denominator }),
        ));
    }
    if denominator <= 0.0 {
        return Err(Tpp2mError::calculation(
            ErrorCode::NonPositiveDenominator,
            "TPP-2M denominator must be positive",
            json!({ "denominator": denominator }),
        ));
    }

    let imfp_angstrom = input.electron_energy_e_v / denominator;
    if !imfp_angstrom.is_finite() {
        return Err(Tpp2mError::calculation(
            ErrorCode::NonFiniteResult,
            "IMFP result is not finite",
            json!({ "imfp_angstrom": imfp_angstrom }),
        ));
    }
    if imfp_angstrom <= 0.0 {
        return Err(Tpp2mError::calculation(
            ErrorCode::NonPositiveDenominator,
            "IMFP result must be positive",
            json!({ "imfp_angstrom": imfp_angstrom }),
        ));
    }

    Ok(Tpp2mOutput {
        imfp_nm: imfp_angstrom / 10.0,
        imfp_angstrom,
        plasmon_energy_e_v: coefficients.plasmon_energy_e_v,
        beta: coefficients.beta,
        gamma_inverse_e_v: coefficients.gamma_inverse_e_v,
        c: coefficients.c,
        d: coefficients.d,
        warnings,
    })
}

pub fn coefficients(input: Tpp2mInput) -> Result<Coefficients, Tpp2mError> {
    validate_input(input)?;

    let u = input.valence_electrons * input.density_g_cm3 / input.molar_mass_g_mol;
    let plasmon_energy_e_v = 28.8 * u.sqrt();
    let beta = -0.10
        + 0.944 / (plasmon_energy_e_v.powi(2) + input.band_gap_e_v.powi(2)).sqrt()
        + 0.069 * input.density_g_cm3.powf(0.1);
    let gamma_inverse_e_v = 0.191 * input.density_g_cm3.powf(-0.5);
    let c = 1.97 - 0.91 * u;
    let d = 53.4 - 20.8 * u;

    for (name, value) in [
        ("plasmon_energy_e_v", plasmon_energy_e_v),
        ("beta", beta),
        ("gamma_inverse_e_v", gamma_inverse_e_v),
        ("c", c),
        ("d", d),
    ] {
        if !value.is_finite() {
            return Err(Tpp2mError::calculation(
                ErrorCode::NonFiniteResult,
                format!("{name} is not finite"),
                json!({ name: value }),
            ));
        }
    }

    Ok(Coefficients {
        plasmon_energy_e_v,
        beta,
        gamma_inverse_e_v,
        c,
        d,
    })
}

pub fn validate_input(input: Tpp2mInput) -> Result<(), Tpp2mError> {
    validate_positive("electron_energy_e_v", input.electron_energy_e_v)?;
    validate_positive("density_g_cm3", input.density_g_cm3)?;
    validate_positive("molar_mass_g_mol", input.molar_mass_g_mol)?;
    validate_positive("valence_electrons", input.valence_electrons)?;
    validate_non_negative("band_gap_e_v", input.band_gap_e_v)?;
    Ok(())
}

pub fn is_energy_recommended(energy_e_v: f64) -> bool {
    (RECOMMENDED_ENERGY_MIN_EV..=RECOMMENDED_ENERGY_MAX_EV).contains(&energy_e_v)
}

fn validate_positive(field: &'static str, value: f64) -> Result<(), Tpp2mError> {
    if value.is_finite() && value > 0.0 {
        Ok(())
    } else {
        Err(Tpp2mError::invalid_input(field, value))
    }
}

fn validate_non_negative(field: &'static str, value: f64) -> Result<(), Tpp2mError> {
    if value.is_finite() && value >= 0.0 {
        Ok(())
    } else {
        Err(Tpp2mError::invalid_non_negative(field, value))
    }
}
