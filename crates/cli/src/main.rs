use std::fs;
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
    Plot(PlotArgs),
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
struct PlotArgs {
    #[command(flatten)]
    material: MaterialArgs,
    #[arg(long = "energy-min", default_value_t = 50.0)]
    energy_min: f64,
    #[arg(long = "energy-max", default_value_t = 2000.0)]
    energy_max: f64,
    #[arg(long, default_value_t = 1000)]
    points: usize,
    #[arg(long, value_enum, default_value_t = CliSpacing::Log)]
    spacing: CliSpacing,
    #[arg(long, short = 'o')]
    output: String,
    #[arg(long, default_value_t = 1280)]
    width: u32,
    #[arg(long, default_value_t = 720)]
    height: u32,
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
        Some(Command::Plot(args)) => run_plot(args),
        Some(Command::Tui(args)) => run_tui(args),
        None => run_tui(TuiArgs {
            material: None,
            energy_min: 50.0,
            energy_max: 2000.0,
        }),
    }
}

fn run_plot(args: PlotArgs) -> Result<(), Tpp2mError> {
    let input = SweepInput {
        material: args.material.to_input(1000.0),
        energy_min_e_v: args.energy_min,
        energy_max_e_v: args.energy_max,
        points: args.points,
        spacing: args.spacing.into(),
    };
    let graph = tpp2m_core::log_plot_points(input)?;
    let svg = render_svg_plot(&graph, args.width, args.height);
    fs::write(&args.output, svg).map_err(internal_error)?;
    Ok(())
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

fn render_svg_plot(graph: &tpp2m_core::LogPlotData, width: u32, height: u32) -> String {
    let width = width.max(360);
    let height = height.max(240);
    let margin_left = 120.0;
    let margin_right = 64.0;
    let margin_top = 56.0;
    let margin_bottom = 96.0;
    let plot_left = margin_left;
    let plot_top = margin_top;
    let plot_width = f64::from(width) - margin_left - margin_right;
    let plot_height = f64::from(height) - margin_top - margin_bottom;
    let x_bounds = bounds(graph.points_log10.iter().map(|(x, _)| *x));
    let y_bounds = bounds(graph.points_log10.iter().map(|(_, y)| *y));

    let mut svg = String::new();
    svg.push_str(&format!(
        r#"<svg xmlns="http://www.w3.org/2000/svg" width="{width}" height="{height}" viewBox="0 0 {width} {height}">"#
    ));
    svg.push_str(r#"<rect width="100%" height="100%" fill="white"/>"#);
    svg.push_str(r#"<g font-family="Arial, Helvetica, sans-serif" fill="black" stroke="black">"#);
    svg.push_str(&format!(
        r#"<rect x="{plot_left:.3}" y="{plot_top:.3}" width="{plot_width:.3}" height="{plot_height:.3}" fill="none" stroke-width="3"/>"#
    ));

    for x in minor_ticks(x_bounds) {
        let px = svg_x(x, x_bounds, plot_left, plot_width);
        svg.push_str(&format!(
            r#"<line x1="{px:.3}" y1="{plot_top:.3}" x2="{px:.3}" y2="{:.3}" stroke-width="2"/>"#,
            plot_top + 14.0
        ));
        svg.push_str(&format!(
            r#"<line x1="{px:.3}" y1="{:.3}" x2="{px:.3}" y2="{:.3}" stroke-width="2"/>"#,
            plot_top + plot_height,
            plot_top + plot_height - 14.0
        ));
    }
    for x in major_ticks(x_bounds) {
        let px = svg_x(x, x_bounds, plot_left, plot_width);
        svg.push_str(&format!(
            r#"<line x1="{px:.3}" y1="{plot_top:.3}" x2="{px:.3}" y2="{:.3}" stroke-width="3"/>"#,
            plot_top + 24.0
        ));
        svg.push_str(&format!(
            r#"<line x1="{px:.3}" y1="{:.3}" x2="{px:.3}" y2="{:.3}" stroke-width="3"/>"#,
            plot_top + plot_height,
            plot_top + plot_height - 24.0
        ));
        svg.push_str(&format!(
            r#"<text x="{px:.3}" y="{:.3}" font-size="42" text-anchor="middle" dominant-baseline="hanging">{}</text>"#,
            plot_top + plot_height + 18.0,
            svg_tick_label(x)
        ));
    }
    for y in minor_ticks(y_bounds) {
        let py = svg_y(y, y_bounds, plot_top, plot_height);
        svg.push_str(&format!(
            r#"<line x1="{plot_left:.3}" y1="{py:.3}" x2="{:.3}" y2="{py:.3}" stroke-width="2"/>"#,
            plot_left + 14.0
        ));
        svg.push_str(&format!(
            r#"<line x1="{:.3}" y1="{py:.3}" x2="{:.3}" y2="{py:.3}" stroke-width="2"/>"#,
            plot_left + plot_width,
            plot_left + plot_width - 14.0
        ));
    }
    for y in major_ticks(y_bounds) {
        let py = svg_y(y, y_bounds, plot_top, plot_height);
        svg.push_str(&format!(
            r#"<line x1="{plot_left:.3}" y1="{py:.3}" x2="{:.3}" y2="{py:.3}" stroke-width="3"/>"#,
            plot_left + 24.0
        ));
        svg.push_str(&format!(
            r#"<line x1="{:.3}" y1="{py:.3}" x2="{:.3}" y2="{py:.3}" stroke-width="3"/>"#,
            plot_left + plot_width,
            plot_left + plot_width - 24.0
        ));
        svg.push_str(&format!(
            r#"<text x="{:.3}" y="{py:.3}" font-size="42" text-anchor="end" dominant-baseline="middle">{}</text>"#,
            plot_left - 24.0,
            svg_tick_label(y)
        ));
    }

    let points = graph
        .points_log10
        .iter()
        .map(|(x, y)| {
            format!(
                "{:.3},{:.3}",
                svg_x(*x, x_bounds, plot_left, plot_width),
                svg_y(*y, y_bounds, plot_top, plot_height)
            )
        })
        .collect::<Vec<_>>()
        .join(" ");
    svg.push_str(&format!(
        r#"<polyline points="{points}" fill="none" stroke="red" stroke-width="5" stroke-linecap="round" stroke-linejoin="round"/>"#
    ));
    svg.push_str(&format!(
        r#"<text x="{:.3}" y="{:.3}" font-size="48" text-anchor="middle">{}</text>"#,
        plot_left + plot_width / 2.0,
        f64::from(height) - 24.0,
        escape_xml(&graph.x_axis_label.replace(" / ", " (").replace("eV", "eV)"))
    ));
    svg.push_str(&format!(
        r#"<text x="48" y="{:.3}" font-size="48" text-anchor="middle" transform="rotate(-90 48 {:.3})">{}</text>"#,
        plot_top + plot_height / 2.0,
        plot_top + plot_height / 2.0,
        escape_xml(&graph.y_axis_label.replace(" / ", " (").replace("nm", "nm)"))
    ));
    svg.push_str("</g></svg>\n");
    svg
}

fn bounds(values: impl Iterator<Item = f64>) -> [f64; 2] {
    let mut min = f64::INFINITY;
    let mut max = f64::NEG_INFINITY;
    for value in values {
        min = min.min(value);
        max = max.max(value);
    }
    if min.is_finite() && max.is_finite() && min < max {
        [min, max]
    } else {
        [0.0, 1.0]
    }
}

fn svg_x(value: f64, bounds: [f64; 2], plot_left: f64, plot_width: f64) -> f64 {
    plot_left + ((value - bounds[0]) / (bounds[1] - bounds[0])).clamp(0.0, 1.0) * plot_width
}

fn svg_y(value: f64, bounds: [f64; 2], plot_top: f64, plot_height: f64) -> f64 {
    plot_top + (1.0 - ((value - bounds[0]) / (bounds[1] - bounds[0])).clamp(0.0, 1.0)) * plot_height
}

fn major_ticks(bounds: [f64; 2]) -> Vec<f64> {
    let start = bounds[0].ceil() as i32;
    let end = bounds[1].floor() as i32;
    (start..=end).map(f64::from).collect()
}

fn minor_ticks(bounds: [f64; 2]) -> Vec<f64> {
    let start = bounds[0].floor() as i32;
    let end = bounds[1].ceil() as i32;
    let mut ticks = Vec::new();
    for power in start..=end {
        for multiplier in 2..10 {
            let tick = f64::from(power) + f64::from(multiplier).log10();
            if tick > bounds[0] && tick < bounds[1] {
                ticks.push(tick);
            }
        }
    }
    ticks
}

fn svg_tick_label(value: f64) -> String {
    let power = value.round() as i32;
    format!(r#"10<tspan baseline-shift="super" font-size="65%">{power}</tspan>"#)
}

fn escape_xml(text: &str) -> String {
    text.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
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
