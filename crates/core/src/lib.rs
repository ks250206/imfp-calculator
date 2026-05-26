mod error;
mod formula;
mod input;
mod sweep;

pub use error::{ErrorCode, Tpp2mError};
pub use input::{Spacing, SweepInput, Tpp2mInput};
pub use sweep::{LogPlotData, SweepOutput, SweepPoint, Tick};

pub use formula::{Coefficients, Tpp2mOutput, Warning, WarningCode};

pub fn calculate(input: Tpp2mInput) -> Result<Tpp2mOutput, Tpp2mError> {
    formula::calculate(input)
}

pub fn sweep(input: SweepInput) -> Result<SweepOutput, Tpp2mError> {
    sweep::sweep(input)
}

pub fn log_plot_points(input: SweepInput) -> Result<LogPlotData, Tpp2mError> {
    sweep::log_plot_points(input)
}

#[cfg(test)]
mod tests {
    use super::*;

    const EPS_ABS: f64 = 1e-10;
    const EPS_REL: f64 = 1e-9;

    fn assert_close(actual: f64, expected: f64, tolerance: f64) {
        let diff = (actual - expected).abs();
        assert!(
            diff <= tolerance,
            "actual={actual}, expected={expected}, diff={diff}, tolerance={tolerance}"
        );
    }

    fn assert_relative_close(actual: f64, expected: f64) {
        let diff = (actual - expected).abs();
        let limit = expected.abs() * EPS_REL;
        assert!(
            diff <= limit,
            "actual={actual}, expected={expected}, diff={diff}, limit={limit}"
        );
    }

    fn silicon_base(energy: f64) -> Tpp2mInput {
        Tpp2mInput {
            electron_energy_e_v: energy,
            density_g_cm3: 2.3296,
            molar_mass_g_mol: 28.0855,
            valence_electrons: 4.0,
            band_gap_e_v: 1.12,
            allow_extrapolate: false,
        }
    }

    #[test]
    fn sweep_input_defaults_match_ssot() {
        let input = SweepInput::with_defaults(silicon_base(1000.0));

        assert_eq!(input.energy_min_e_v, 50.0);
        assert_eq!(input.energy_max_e_v, 2000.0);
        assert_eq!(input.points, 200);
        assert_eq!(input.spacing, Spacing::Log);
    }

    #[test]
    fn calculates_silicon_coefficients_and_imfp_at_1000_ev() {
        let result = calculate(silicon_base(1000.0)).unwrap();

        assert_close(result.plasmon_energy_e_v, 16.589071625484447, EPS_ABS);
        assert_close(result.beta, 0.03186483916389788, EPS_ABS);
        assert_close(result.gamma_inverse_e_v, 0.12513900238367897, EPS_ABS);
        assert_relative_close(result.imfp_nm, 2.3864329956020653);
        assert!(result.warnings.is_empty());
    }

    #[test]
    fn calculates_reference_imfp_vectors() {
        let cases = [
            (
                Tpp2mInput {
                    electron_energy_e_v: 50.0,
                    density_g_cm3: 2.3296,
                    molar_mass_g_mol: 28.0855,
                    valence_electrons: 4.0,
                    band_gap_e_v: 1.12,
                    allow_extrapolate: false,
                },
                0.41606265357756095,
            ),
            (
                Tpp2mInput {
                    electron_energy_e_v: 2000.0,
                    density_g_cm3: 19.32,
                    molar_mass_g_mol: 196.96657,
                    valence_electrons: 11.0,
                    band_gap_e_v: 0.0,
                    allow_extrapolate: false,
                },
                2.066118649972704,
            ),
            (
                Tpp2mInput {
                    electron_energy_e_v: 500.0,
                    density_g_cm3: 2.2,
                    molar_mass_g_mol: 60.0843,
                    valence_electrons: 16.0,
                    band_gap_e_v: 9.0,
                    allow_extrapolate: false,
                },
                1.8089998055924568,
            ),
        ];

        for (input, expected) in cases {
            let result = calculate(input).unwrap();
            assert_relative_close(result.imfp_nm, expected);
        }
    }

    #[test]
    fn rejects_energy_outside_recommended_range_without_extrapolation() {
        let error = calculate(silicon_base(5000.0)).unwrap_err();

        assert_eq!(error.code, ErrorCode::OutOfRecommendedRange);
        assert_eq!(error.field.as_deref(), Some("electron_energy_e_v"));
    }

    #[test]
    fn warns_when_extrapolating_energy_outside_recommended_range() {
        let mut input = silicon_base(5000.0);
        input.allow_extrapolate = true;

        let result = calculate(input).unwrap();

        assert_eq!(result.warnings.len(), 1);
        assert_eq!(
            result.warnings[0].code,
            WarningCode::EnergyOutsideRecommendedRange
        );
        assert!(result.imfp_nm.is_finite());
    }

    #[test]
    fn rejects_non_finite_and_non_positive_inputs() {
        let invalid = Tpp2mInput {
            density_g_cm3: f64::NAN,
            ..silicon_base(1000.0)
        };

        let error = calculate(invalid).unwrap_err();

        assert_eq!(error.code, ErrorCode::InvalidInput);
        assert_eq!(error.field.as_deref(), Some("density_g_cm3"));
    }

    #[test]
    fn rejects_negative_band_gap() {
        let invalid = Tpp2mInput {
            band_gap_e_v: -1.0,
            ..silicon_base(1000.0)
        };

        let error = calculate(invalid).unwrap_err();

        assert_eq!(error.code, ErrorCode::InvalidInput);
        assert_eq!(error.field.as_deref(), Some("band_gap_e_v"));
    }

    #[test]
    fn maps_error_codes_to_cli_exit_codes() {
        let cases = [
            (
                ErrorCode::InvalidInput,
                Tpp2mError::invalid_input("density_g_cm3", -1.0).exit_code(),
                1,
            ),
            (
                ErrorCode::OutOfRecommendedRange,
                Tpp2mError::out_of_recommended_range(5000.0, 50.0, 2000.0).exit_code(),
                1,
            ),
            (
                ErrorCode::InvalidSweepRange,
                Tpp2mError::invalid_sweep_range("bad sweep").exit_code(),
                1,
            ),
            (
                ErrorCode::NonPositiveLogArgument,
                Tpp2mError::calculation(
                    ErrorCode::NonPositiveLogArgument,
                    "bad log",
                    serde_json::json!({}),
                )
                .exit_code(),
                2,
            ),
            (
                ErrorCode::OutputFormatConflict,
                Tpp2mError::calculation(
                    ErrorCode::OutputFormatConflict,
                    "bad output",
                    serde_json::json!({}),
                )
                .exit_code(),
                3,
            ),
            (
                ErrorCode::TerminalInitialization,
                Tpp2mError::calculation(
                    ErrorCode::TerminalInitialization,
                    "bad terminal",
                    serde_json::json!({}),
                )
                .exit_code(),
                4,
            ),
            (
                ErrorCode::Internal,
                Tpp2mError::calculation(ErrorCode::Internal, "internal", serde_json::json!({}))
                    .exit_code(),
                70,
            ),
        ];

        for (code, actual, expected) in cases {
            assert_eq!(actual, expected, "{code:?}");
        }
    }

    #[test]
    fn generates_log_sweep_with_fixed_endpoints() {
        let output = sweep(SweepInput {
            material: Tpp2mInput {
                electron_energy_e_v: 1000.0,
                ..silicon_base(1000.0)
            },
            energy_min_e_v: 50.0,
            energy_max_e_v: 2000.0,
            points: 5,
            spacing: Spacing::Log,
        })
        .unwrap();

        assert_eq!(output.points.len(), 5);
        assert_close(output.points[0].electron_energy_e_v, 50.0, 1e-12);
        assert_close(output.points[4].electron_energy_e_v, 2000.0, 1e-10);
        assert!(
            output
                .points
                .windows(2)
                .all(|pair| pair[0].electron_energy_e_v < pair[1].electron_energy_e_v)
        );
    }

    #[test]
    fn generates_linear_sweep_with_fixed_endpoints() {
        let output = sweep(SweepInput {
            material: silicon_base(1000.0),
            energy_min_e_v: 50.0,
            energy_max_e_v: 200.0,
            points: 4,
            spacing: Spacing::Linear,
        })
        .unwrap();

        let energies: Vec<f64> = output
            .points
            .iter()
            .map(|point| point.electron_energy_e_v)
            .collect();
        assert_eq!(energies, vec![50.0, 100.0, 150.0, 200.0]);
    }

    #[test]
    fn rejects_invalid_sweep_ranges_and_point_counts() {
        let base = SweepInput {
            material: silicon_base(1000.0),
            energy_min_e_v: 50.0,
            energy_max_e_v: 2000.0,
            points: 200,
            spacing: Spacing::Log,
        };

        let bad_min = SweepInput {
            energy_min_e_v: 0.0,
            ..base
        };
        assert_eq!(
            sweep(bad_min).unwrap_err().code,
            ErrorCode::InvalidSweepRange
        );

        let bad_max = SweepInput {
            energy_max_e_v: 25.0,
            ..base
        };
        assert_eq!(
            sweep(bad_max).unwrap_err().code,
            ErrorCode::InvalidSweepRange
        );

        let bad_points = SweepInput { points: 1, ..base };
        assert_eq!(
            sweep(bad_points).unwrap_err().code,
            ErrorCode::InvalidSweepRange
        );
    }

    #[test]
    fn produces_log_plot_points_and_physical_axis_labels() {
        let graph = log_plot_points(SweepInput {
            material: silicon_base(1000.0),
            energy_min_e_v: 100.0,
            energy_max_e_v: 1000.0,
            points: 2,
            spacing: Spacing::Log,
        })
        .unwrap();

        assert_eq!(graph.points_log10.len(), 2);
        assert_close(graph.points_log10[0].0, 2.0, 1e-12);
        assert_close(graph.points_log10[1].0, 3.0, 1e-12);
        assert_eq!(graph.x_axis_label, "Electron Energy / eV");
        assert_eq!(graph.y_axis_label, "IMFP / nm");
        assert!(
            graph
                .y_ticks
                .iter()
                .all(|tick| tick.value_log10.is_finite())
        );
    }
}
