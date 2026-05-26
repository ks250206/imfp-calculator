use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use thiserror::Error;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub enum ErrorCode {
    InvalidInput,
    OutOfRecommendedRange,
    InvalidSweepRange,
    NonPositiveLogArgument,
    NonPositiveDenominator,
    NonFiniteResult,
    OutputFormatConflict,
    TerminalInitialization,
    Internal,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, Error)]
#[error("{message}")]
pub struct Tpp2mError {
    pub code: ErrorCode,
    pub message: String,
    pub field: Option<String>,
    pub details: Value,
}

impl Tpp2mError {
    pub fn invalid_input(field: &'static str, value: f64) -> Self {
        Self {
            code: ErrorCode::InvalidInput,
            message: format!("{field} must be positive and finite"),
            field: Some(field.to_string()),
            details: json!({ "value": value }),
        }
    }

    pub fn invalid_non_negative(field: &'static str, value: f64) -> Self {
        Self {
            code: ErrorCode::InvalidInput,
            message: format!("{field} must be non-negative and finite"),
            field: Some(field.to_string()),
            details: json!({ "value": value }),
        }
    }

    pub fn out_of_recommended_range(value: f64, min: f64, max: f64) -> Self {
        Self {
            code: ErrorCode::OutOfRecommendedRange,
            message:
                "electron_energy_e_v must be within 50..=2000 eV unless allow_extrapolate is true"
                    .to_string(),
            field: Some("electron_energy_e_v".to_string()),
            details: json!({ "value": value, "min": min, "max": max }),
        }
    }

    pub fn invalid_sweep_range(message: impl Into<String>) -> Self {
        Self {
            code: ErrorCode::InvalidSweepRange,
            message: message.into(),
            field: None,
            details: json!({}),
        }
    }

    pub fn calculation(code: ErrorCode, message: impl Into<String>, details: Value) -> Self {
        Self {
            code,
            message: message.into(),
            field: None,
            details,
        }
    }

    pub fn exit_code(&self) -> i32 {
        match self.code {
            ErrorCode::InvalidInput
            | ErrorCode::OutOfRecommendedRange
            | ErrorCode::InvalidSweepRange => 1,
            ErrorCode::NonPositiveLogArgument
            | ErrorCode::NonPositiveDenominator
            | ErrorCode::NonFiniteResult => 2,
            ErrorCode::OutputFormatConflict => 3,
            ErrorCode::TerminalInitialization => 4,
            ErrorCode::Internal => 70,
        }
    }
}
