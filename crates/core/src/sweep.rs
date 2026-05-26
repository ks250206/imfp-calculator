use serde::{Deserialize, Serialize};

use crate::error::{ErrorCode, Tpp2mError};
use crate::formula::{Warning, WarningCode, calculate};
use crate::input::{Spacing, SweepInput};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SweepPoint {
    pub electron_energy_e_v: f64,
    pub imfp_nm: f64,
    pub imfp_angstrom: f64,
    pub warning: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SweepOutput {
    pub input: SweepInput,
    pub points: Vec<SweepPoint>,
    pub warnings: Vec<Warning>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Tick {
    pub value_log10: f64,
    pub label: String,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct LogPlotData {
    pub points_log10: Vec<(f64, f64)>,
    pub x_ticks: Vec<Tick>,
    pub y_ticks: Vec<Tick>,
    pub raw_points: Vec<SweepPoint>,
    pub x_axis_label: String,
    pub y_axis_label: String,
    pub warnings: Vec<Warning>,
}

pub fn sweep(input: SweepInput) -> Result<SweepOutput, Tpp2mError> {
    validate_sweep_input(input)?;

    let mut points = Vec::with_capacity(input.points);
    let mut warnings = Vec::new();

    for index in 0..input.points {
        let energy = energy_at(input, index);
        let point_input = crate::input::Tpp2mInput {
            electron_energy_e_v: energy,
            ..input.material
        };
        let result = calculate(point_input);
        match result {
            Ok(output) => {
                let warning = output
                    .warnings
                    .first()
                    .map(|warning| warning.message.clone());
                warnings.extend(output.warnings);
                points.push(SweepPoint {
                    electron_energy_e_v: energy,
                    imfp_nm: output.imfp_nm,
                    imfp_angstrom: output.imfp_angstrom,
                    warning,
                });
            }
            Err(error) => {
                return Err(error);
            }
        }
    }

    Ok(SweepOutput {
        input,
        points,
        warnings,
    })
}

pub fn log_plot_points(input: SweepInput) -> Result<LogPlotData, Tpp2mError> {
    let output = sweep(input)?;
    let mut omitted = 0usize;
    let points_log10: Vec<(f64, f64)> = output
        .points
        .iter()
        .filter_map(|point| {
            let x = point.electron_energy_e_v.log10();
            let y = point.imfp_nm.log10();
            if x.is_finite() && y.is_finite() {
                Some((x, y))
            } else {
                omitted += 1;
                None
            }
        })
        .collect();

    if points_log10.len() < 2 {
        return Err(Tpp2mError::calculation(
            ErrorCode::NonFiniteResult,
            "at least two finite graph points are required",
            serde_json::json!({ "finite_points": points_log10.len() }),
        ));
    }

    let mut warnings = output.warnings.clone();
    if omitted > 0 {
        warnings.push(Warning {
            code: WarningCode::GraphPointsOmitted,
            message: format!("{omitted} graph points were omitted"),
            field: None,
        });
    }

    Ok(LogPlotData {
        x_ticks: ticks_for(
            output.input.energy_min_e_v.log10(),
            output.input.energy_max_e_v.log10(),
        ),
        y_ticks: ticks_for_points(points_log10.iter().map(|(_, y)| *y)),
        points_log10,
        raw_points: output.points,
        x_axis_label: "Electron Energy / eV".to_string(),
        y_axis_label: "IMFP / nm".to_string(),
        warnings,
    })
}

fn validate_sweep_input(input: SweepInput) -> Result<(), Tpp2mError> {
    if !(input.energy_min_e_v.is_finite() && input.energy_min_e_v > 0.0) {
        return Err(Tpp2mError::invalid_sweep_range(
            "energy_min_e_v must be positive and finite",
        ));
    }
    if !(input.energy_max_e_v.is_finite() && input.energy_max_e_v > input.energy_min_e_v) {
        return Err(Tpp2mError::invalid_sweep_range(
            "energy_max_e_v must be finite and greater than energy_min_e_v",
        ));
    }
    if !(2..=10_000).contains(&input.points) {
        return Err(Tpp2mError::invalid_sweep_range(
            "points must be within 2..=10000",
        ));
    }
    crate::formula::validate_input(input.material)
}

fn energy_at(input: SweepInput, index: usize) -> f64 {
    if index == 0 {
        return input.energy_min_e_v;
    }
    if index == input.points - 1 {
        return input.energy_max_e_v;
    }

    let t = index as f64 / (input.points - 1) as f64;
    match input.spacing {
        Spacing::Log => {
            let min = input.energy_min_e_v.log10();
            let max = input.energy_max_e_v.log10();
            10_f64.powf(min + t * (max - min))
        }
        Spacing::Linear => input.energy_min_e_v + t * (input.energy_max_e_v - input.energy_min_e_v),
    }
}

fn ticks_for_points(values: impl Iterator<Item = f64>) -> Vec<Tick> {
    let finite_values: Vec<f64> = values.filter(|value| value.is_finite()).collect();
    let min = finite_values.iter().copied().fold(f64::INFINITY, f64::min);
    let max = finite_values
        .iter()
        .copied()
        .fold(f64::NEG_INFINITY, f64::max);
    ticks_for(min, max)
}

fn ticks_for(min_log10: f64, max_log10: f64) -> Vec<Tick> {
    if !(min_log10.is_finite() && max_log10.is_finite()) {
        return Vec::new();
    }

    let start = min_log10.floor() as i32;
    let end = max_log10.ceil() as i32;
    (start..=end)
        .map(|power| Tick {
            value_log10: f64::from(power),
            label: format_tick_label(power),
        })
        .collect()
}

fn format_tick_label(power: i32) -> String {
    let value = 10_f64.powi(power);
    if value >= 1.0 {
        format!("{value:.0}")
    } else {
        format!("{value:.3}")
            .trim_end_matches('0')
            .trim_end_matches('.')
            .to_string()
    }
}
