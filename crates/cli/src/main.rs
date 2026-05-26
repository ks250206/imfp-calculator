use std::io::{self, Write};
use std::process::ExitCode;

use clap::{Parser, Subcommand, ValueEnum};
use serde::Serialize;
use tpp2m_core::{ErrorCode, Spacing, SweepInput, Tpp2mError, Tpp2mInput};

#[derive(Debug, Parser)]
#[command(name = "tpp2m")]
#[command(about = "Calculate electron IMFP with the TPP-2M formula")]
struct Cli {
    #[arg(long)]
    quiet: bool,
    #[arg(long)]
    verbose: bool,
    #[arg(long)]
    no_color: bool,
    #[command(subcommand)]
    command: Option<Command>,
}

#[derive(Debug, Subcommand)]
enum Command {
    Calc(CalcArgs),
    Sweep(SweepArgs),
    Tui(TuiArgs),
}

#[derive(Clone, Debug, Parser)]
struct MaterialArgs {
    #[arg(long, short = 'r')]
    density: f64,
    #[arg(long = "molar-mass", short = 'M')]
    molar_mass: f64,
    #[arg(long = "valence-electrons", short = 'v')]
    valence_electrons: f64,
    #[arg(long = "band-gap", short = 'g', default_value_t = 0.0)]
    band_gap: f64,
    #[arg(long)]
    allow_extrapolate: bool,
}

#[derive(Debug, Parser)]
struct CalcArgs {
    #[command(flatten)]
    material: MaterialArgs,
    #[arg(long, short = 'E')]
    energy: f64,
    #[arg(long)]
    json: bool,
    #[arg(long, default_value_t = 6)]
    precision: usize,
}

#[derive(Debug, Parser)]
struct SweepArgs {
    #[command(flatten)]
    material: MaterialArgs,
    #[arg(long = "energy-min", default_value_t = 50.0)]
    energy_min: f64,
    #[arg(long = "energy-max", default_value_t = 2000.0)]
    energy_max: f64,
    #[arg(long, default_value_t = 200)]
    points: usize,
    #[arg(long, value_enum, default_value_t = CliSpacing::Log)]
    spacing: CliSpacing,
    #[arg(long)]
    json: bool,
    #[arg(long)]
    csv: bool,
}

#[derive(Debug, Parser)]
struct TuiArgs {
    #[command(flatten)]
    material: Option<OptionalMaterialArgs>,
    #[arg(long = "energy-min", default_value_t = 50.0)]
    energy_min: f64,
    #[arg(long = "energy-max", default_value_t = 2000.0)]
    energy_max: f64,
}

#[derive(Clone, Debug, Parser)]
struct OptionalMaterialArgs {
    #[arg(long, short = 'r')]
    density: Option<f64>,
    #[arg(long = "molar-mass", short = 'M')]
    molar_mass: Option<f64>,
    #[arg(long = "valence-electrons", short = 'v')]
    valence_electrons: Option<f64>,
    #[arg(long = "band-gap", short = 'g')]
    band_gap: Option<f64>,
    #[arg(long)]
    allow_extrapolate: bool,
}

#[derive(Clone, Copy, Debug, ValueEnum)]
enum CliSpacing {
    Log,
    Linear,
}

#[derive(Serialize)]
struct CalcJson {
    input: Tpp2mInput,
    output: tpp2m_core::Tpp2mOutput,
    warnings: Vec<tpp2m_core::Warning>,
}

fn main() -> ExitCode {
    let cli = Cli::parse();
    match run(cli) {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            print_error(&error);
            ExitCode::from(error.exit_code() as u8)
        }
    }
}

fn run(cli: Cli) -> Result<(), Tpp2mError> {
    match cli.command {
        Some(Command::Calc(args)) => run_calc(args, cli.verbose),
        Some(Command::Sweep(args)) => run_sweep(args),
        Some(Command::Tui(args)) => run_tui(args),
        None => run_tui(TuiArgs {
            material: None,
            energy_min: 50.0,
            energy_max: 2000.0,
        }),
    }
}

fn run_calc(args: CalcArgs, verbose: bool) -> Result<(), Tpp2mError> {
    let input = args.material.to_input(args.energy);
    let output = tpp2m_core::calculate(input)?;

    if args.json {
        let json = CalcJson {
            input,
            warnings: output.warnings.clone(),
            output,
        };
        println!(
            "{}",
            serde_json::to_string_pretty(&json).map_err(internal_error)?
        );
    } else if verbose {
        println!(
            "IMFP: {:.*} nm ({:.*} A)",
            args.precision, output.imfp_nm, args.precision, output.imfp_angstrom
        );
        println!("Ep: {:.*} eV", args.precision, output.plasmon_energy_e_v);
        println!("beta: {:.*}", args.precision, output.beta);
        println!(
            "gamma: {:.*} 1/eV",
            args.precision, output.gamma_inverse_e_v
        );
        println!("C: {:.*}", args.precision, output.c);
        println!("D: {:.*}", args.precision, output.d);
    } else {
        println!("IMFP: {:.*} nm", args.precision, output.imfp_nm);
    }
    Ok(())
}

fn run_sweep(args: SweepArgs) -> Result<(), Tpp2mError> {
    if args.json && args.csv {
        return Err(Tpp2mError {
            code: ErrorCode::OutputFormatConflict,
            message: "--json and --csv cannot be used together".to_string(),
            field: None,
            details: serde_json::json!({ "json": true, "csv": true }),
        });
    }

    let input = SweepInput {
        material: args.material.to_input(1000.0),
        energy_min_e_v: args.energy_min,
        energy_max_e_v: args.energy_max,
        points: args.points,
        spacing: args.spacing.into(),
    };
    let output = tpp2m_core::sweep(input)?;

    if args.json {
        println!(
            "{}",
            serde_json::to_string_pretty(&output).map_err(internal_error)?
        );
    } else {
        let mut writer = csv::Writer::from_writer(io::stdout());
        writer
            .write_record(["electron_energy_e_v", "imfp_nm", "imfp_angstrom", "warning"])
            .map_err(internal_error)?;
        for point in output.points {
            writer
                .write_record([
                    format!("{:.12}", point.electron_energy_e_v),
                    format!("{:.12}", point.imfp_nm),
                    format!("{:.12}", point.imfp_angstrom),
                    point.warning.unwrap_or_default(),
                ])
                .map_err(internal_error)?;
        }
        writer.flush().map_err(internal_error)?;
    }

    Ok(())
}

fn run_tui(args: TuiArgs) -> Result<(), Tpp2mError> {
    let initial = args
        .material
        .and_then(OptionalMaterialArgs::into_initial_input);
    tpp2m_tui::run(initial, args.energy_min, args.energy_max).map_err(|message| Tpp2mError {
        code: ErrorCode::TerminalInitialization,
        message,
        field: None,
        details: serde_json::json!({}),
    })
}

fn print_error(error: &Tpp2mError) {
    let _ = writeln!(io::stderr(), "error[{:?}]: {}", error.code, error.message);
    if let Some(field) = &error.field {
        let _ = writeln!(io::stderr(), "  field: {field}");
    }
}

fn internal_error(error: impl std::fmt::Display) -> Tpp2mError {
    Tpp2mError {
        code: ErrorCode::Internal,
        message: error.to_string(),
        field: None,
        details: serde_json::json!({}),
    }
}

impl MaterialArgs {
    fn to_input(&self, energy: f64) -> Tpp2mInput {
        Tpp2mInput {
            electron_energy_e_v: energy,
            density_g_cm3: self.density,
            molar_mass_g_mol: self.molar_mass,
            valence_electrons: self.valence_electrons,
            band_gap_e_v: self.band_gap,
            allow_extrapolate: self.allow_extrapolate,
        }
    }
}

impl OptionalMaterialArgs {
    fn into_initial_input(self) -> Option<Tpp2mInput> {
        Some(Tpp2mInput {
            electron_energy_e_v: 1000.0,
            density_g_cm3: self.density?,
            molar_mass_g_mol: self.molar_mass?,
            valence_electrons: self.valence_electrons?,
            band_gap_e_v: self.band_gap.unwrap_or(0.0),
            allow_extrapolate: self.allow_extrapolate,
        })
    }
}

impl From<CliSpacing> for Spacing {
    fn from(value: CliSpacing) -> Self {
        match value {
            CliSpacing::Log => Self::Log,
            CliSpacing::Linear => Self::Linear,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn maps_cli_spacing_to_core_spacing() {
        assert_eq!(Spacing::from(CliSpacing::Log), Spacing::Log);
        assert_eq!(Spacing::from(CliSpacing::Linear), Spacing::Linear);
    }

    #[test]
    fn command_none_uses_tui_path() {
        let cli = Cli {
            quiet: false,
            verbose: false,
            no_color: false,
            command: None,
        };

        assert!(cli.command.is_none());
    }
}
