use ratatui::Frame;
use ratatui::layout::{Constraint, Direction, Layout, Position, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::Line;
use ratatui::widgets::{Block, Borders, Paragraph, Row, Table};

use plotters::coord::types::RangedCoordf64;
use plotters::prelude::*;
use plotters_ratatui_backend::widget_fn;

use crate::app::{AppState, EnergyField, MaterialField, Mode, Pane};

pub fn render(frame: &mut Frame<'_>, state: &AppState) {
    let area = frame.area();
    if area.width < 90 {
        render_stacked(frame, state, area);
    } else {
        render_split(frame, state, area);
    }
}

pub fn pane_titles() -> Vec<&'static str> {
    Pane::ORDER.iter().map(|pane| pane.title()).collect()
}

fn render_split(frame: &mut Frame<'_>, state: &AppState, area: Rect) {
    let columns = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(38), Constraint::Percentage(62)])
        .split(area);
    let left = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(8),
            Constraint::Length(10),
            Constraint::Min(5),
            Constraint::Length(5),
        ])
        .split(columns[0]);
    let right = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(12), Constraint::Length(8)])
        .split(columns[1]);

    render_material(frame, state, left[0]);
    render_energy(frame, state, left[1]);
    render_result(frame, state, left[2]);
    render_help(frame, state, left[3]);
    render_graph(frame, state, right[0]);
    render_messages(frame, state, right[1]);
}

fn render_stacked(frame: &mut Frame<'_>, state: &AppState, area: Rect) {
    let rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(8),
            Constraint::Length(10),
            Constraint::Length(10),
            Constraint::Min(7),
            Constraint::Length(5),
        ])
        .split(area);
    render_material(frame, state, rows[0]);
    render_energy(frame, state, rows[1]);
    render_graph(frame, state, rows[2]);
    render_result(frame, state, rows[3]);
    render_help(frame, state, rows[4]);
}

fn render_material(frame: &mut Frame<'_>, state: &AppState, area: Rect) {
    let preset_marker = field_marker(
        state.focused_pane == Pane::MaterialInput,
        state.selected_material_field == crate::app::MaterialField::Preset,
    );
    let name_marker = field_marker(
        state.focused_pane == Pane::MaterialInput,
        state.selected_material_field == crate::app::MaterialField::Name,
    );
    let text = vec![
        Line::from(format!(
            "{preset_marker}preset: {}",
            material_preset_label(state)
        )),
        Line::from(format!(
            "{name_marker}name: {}",
            material_field_value(state, MaterialField::Name)
        )),
        Line::from(format!(
            "{}density: {} g/cm3",
            field_marker(
                state.focused_pane == Pane::MaterialInput,
                state.selected_material_field == MaterialField::Density,
            ),
            material_field_value(state, MaterialField::Density)
        )),
        Line::from(format!(
            "{}molar mass: {} g/mol",
            field_marker(
                state.focused_pane == Pane::MaterialInput,
                state.selected_material_field == MaterialField::MolarMass,
            ),
            material_field_value(state, MaterialField::MolarMass)
        )),
        Line::from(format!(
            "{}valence electrons: {}",
            field_marker(
                state.focused_pane == Pane::MaterialInput,
                state.selected_material_field == MaterialField::ValenceElectrons,
            ),
            material_field_value(state, MaterialField::ValenceElectrons)
        )),
        Line::from(format!(
            "{}band gap: {} eV",
            field_marker(
                state.focused_pane == Pane::MaterialInput,
                state.selected_material_field == MaterialField::BandGap,
            ),
            material_field_value(state, MaterialField::BandGap)
        )),
    ];
    frame.render_widget(
        Paragraph::new(text).block(block(Pane::MaterialInput, state)),
        area,
    );
    set_edit_cursor(frame, state, area, Pane::MaterialInput);
}

fn render_energy(frame: &mut Frame<'_>, state: &AppState, area: Rect) {
    let text = vec![
        Line::from(format!(
            "{}source: {}",
            field_marker(
                state.focused_pane == Pane::EnergySweep,
                state.selected_energy_field == EnergyField::Source,
            ),
            energy_source_label(state)
        )),
        Line::from(format!(
            "{}Electron energy: {} eV",
            field_marker(
                state.focused_pane == Pane::EnergySweep,
                state.selected_energy_field == EnergyField::ElectronEnergy,
            ),
            energy_field_value(state, EnergyField::ElectronEnergy)
        )),
        Line::from(format!(
            "{}range mode: {:?}",
            field_marker(
                state.focused_pane == Pane::EnergySweep,
                state.selected_energy_field == EnergyField::RangeMode,
            ),
            state.energy.range_mode
        )),
        Line::from(format!(
            "{}range min: {} eV",
            field_marker(
                state.focused_pane == Pane::EnergySweep,
                state.selected_energy_field == EnergyField::EnergyMin,
            ),
            energy_field_value(state, EnergyField::EnergyMin)
        )),
        Line::from(format!(
            "{}range max: {} eV",
            field_marker(
                state.focused_pane == Pane::EnergySweep,
                state.selected_energy_field == EnergyField::EnergyMax,
            ),
            energy_field_value(state, EnergyField::EnergyMax)
        )),
        Line::from(format!(
            "{}points: {}",
            field_marker(
                state.focused_pane == Pane::EnergySweep,
                state.selected_energy_field == EnergyField::Points,
            ),
            energy_field_value(state, EnergyField::Points)
        )),
        Line::from(format!(
            "{}spacing: {:?}",
            field_marker(
                state.focused_pane == Pane::EnergySweep,
                state.selected_energy_field == EnergyField::Spacing,
            ),
            state.energy.spacing
        )),
        Line::from(format!(
            "{}allow extrapolate: {}",
            field_marker(
                state.focused_pane == Pane::EnergySweep,
                state.selected_energy_field == EnergyField::AllowExtrapolate,
            ),
            state.energy.allow_extrapolate
        )),
    ];
    frame.render_widget(
        Paragraph::new(text).block(block(Pane::EnergySweep, state)),
        area,
    );
    set_edit_cursor(frame, state, area, Pane::EnergySweep);
}

fn render_result(frame: &mut Frame<'_>, state: &AppState, area: Rect) {
    let rows = state
        .sweep
        .as_ref()
        .map(|sweep| {
            sweep
                .points
                .iter()
                .take(8)
                .map(|point| {
                    Row::new(vec![
                        format!("{:.4}", point.electron_energy_e_v),
                        format!("{:.6}", point.imfp_nm),
                    ])
                })
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();
    let table = Table::new(
        rows,
        [Constraint::Percentage(50), Constraint::Percentage(50)],
    )
    .header(Row::new(vec!["E / eV", "IMFP / nm"]))
    .block(block(Pane::ResultSeries, state));
    frame.render_widget(table, area);
}

fn render_graph(frame: &mut Frame<'_>, state: &AppState, area: Rect) {
    let Some(graph) = &state.graph else {
        frame.render_widget(
            Paragraph::new("graph unavailable").block(block(Pane::Graph, state)),
            area,
        );
        return;
    };
    let points = graph.points_log10.clone();
    let x_axis_label = graph.x_axis_label.clone();
    let y_axis_label = graph.y_axis_label.clone();
    let x_bounds = bounds(points.iter().map(|(x, _)| *x));
    let y_bounds = bounds(points.iter().map(|(_, y)| *y));
    let outer = graph_block(state);
    let inner = outer.inner(area);
    frame.render_widget(outer, area);
    frame.render_widget(
        Block::default().style(Style::default().bg(Color::White)),
        inner,
    );
    let widget = widget_fn(move |drawing_area| {
        drawing_area.fill(&WHITE)?;
        let mut chart = ChartBuilder::on(&drawing_area)
            .margin(4)
            .x_label_area_size(16)
            .y_label_area_size(18)
            .right_y_label_area_size(8)
            .top_x_label_area_size(8)
            .build_cartesian_2d(x_bounds[0]..x_bounds[1], y_bounds[0]..y_bounds[1])?;

        chart
            .configure_mesh()
            .disable_mesh()
            .axis_style(BLACK)
            .label_style(("sans-serif", 10).into_font().color(&BLACK))
            .axis_desc_style(("sans-serif", 10).into_font().color(&BLACK))
            .x_desc(x_axis_label.clone())
            .y_desc(y_axis_label.clone())
            .x_labels(major_ticks(x_bounds).len().max(2))
            .y_labels(major_ticks(y_bounds).len().max(2))
            .x_label_formatter(&|value| superscript_tick(*value))
            .y_label_formatter(&|value| superscript_tick(*value))
            .draw()?;

        draw_axis_ticks(&mut chart, x_bounds, y_bounds)?;
        chart.draw_series(LineSeries::new(points.iter().copied(), &BLUE))?;
        drawing_area.present()
    });

    frame.render_widget(widget, inner);
    frame
        .buffer_mut()
        .set_style(inner, Style::default().bg(Color::White));
}

fn render_help(frame: &mut Frame<'_>, state: &AppState, area: Rect) {
    let text = "1-5 focus | hjkl move | gg/G bounds | / search | ? help | q quit";
    frame.render_widget(
        Paragraph::new(text).block(block(Pane::HelpLog, state)),
        area,
    );
}

fn render_messages(frame: &mut Frame<'_>, state: &AppState, area: Rect) {
    let lines: Vec<Line<'_>> = state
        .messages
        .iter()
        .rev()
        .take(4)
        .map(|message| Line::from(message.text.clone()))
        .collect();
    frame.render_widget(
        Paragraph::new(lines).block(Block::default().borders(Borders::ALL).title("Log")),
        area,
    );
}

fn block(pane: Pane, state: &AppState) -> Block<'static> {
    let style = if state.focused_pane == pane {
        Style::default()
            .fg(Color::Yellow)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default()
    };
    Block::default()
        .borders(Borders::ALL)
        .title(pane.title())
        .border_style(style)
}

fn graph_block(state: &AppState) -> Block<'static> {
    block(Pane::Graph, state).style(Style::default().fg(Color::Black).bg(Color::White))
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

fn field_marker(focused: bool, selected: bool) -> &'static str {
    if focused && selected { "> " } else { "  " }
}

fn material_preset_label(state: &AppState) -> &str {
    if state.material.preset_index.is_some() {
        state.material.material_name.as_str()
    } else {
        "Custom"
    }
}

fn energy_source_label(state: &AppState) -> &str {
    match state.energy.source {
        crate::app::EnergySource::Custom => "Custom",
        crate::app::EnergySource::Xray(index) => crate::presets::XRAY_PRESETS[index].label,
    }
}

fn material_field_value(state: &AppState, field: MaterialField) -> String {
    if state.mode == Mode::Editing
        && state.focused_pane == Pane::MaterialInput
        && state.selected_material_field == field
    {
        return state.edit_buffer.clone();
    }
    match field {
        MaterialField::Name => state.material.material_name.clone(),
        MaterialField::Density => format_float(state.material.density_g_cm3),
        MaterialField::MolarMass => format_float(state.material.molar_mass_g_mol),
        MaterialField::ValenceElectrons => format_float(state.material.valence_electrons),
        MaterialField::BandGap => format_float(state.material.band_gap_e_v),
        MaterialField::Preset => material_preset_label(state).to_string(),
    }
}

fn energy_field_value(state: &AppState, field: EnergyField) -> String {
    if state.mode == Mode::Editing
        && state.focused_pane == Pane::EnergySweep
        && state.selected_energy_field == field
    {
        return state.edit_buffer.clone();
    }
    let (range_min, range_max) = state.current_range();
    match field {
        EnergyField::Source => energy_source_label(state).to_string(),
        EnergyField::ElectronEnergy => format_float(state.energy.electron_energy_e_v),
        EnergyField::RangeMode => format!("{:?}", state.energy.range_mode),
        EnergyField::EnergyMin => format_float(range_min),
        EnergyField::EnergyMax => format_float(range_max),
        EnergyField::Points => state.energy.points.to_string(),
        EnergyField::Spacing => format!("{:?}", state.energy.spacing),
        EnergyField::AllowExtrapolate => state.energy.allow_extrapolate.to_string(),
    }
}

fn format_float(value: f64) -> String {
    format!("{value:.6}")
        .trim_end_matches('0')
        .trim_end_matches('.')
        .to_string()
}

fn set_edit_cursor(frame: &mut Frame<'_>, state: &AppState, area: Rect, pane: Pane) {
    let Some((row, prefix_len)) = edit_field_location(state, pane) else {
        return;
    };
    let inner_x = area.x.saturating_add(1);
    let inner_y = area.y.saturating_add(1);
    let max_x = area.right().saturating_sub(2);
    let x = inner_x
        .saturating_add(prefix_len)
        .saturating_add(state.edit_buffer.chars().count() as u16)
        .min(max_x);
    let y = inner_y.saturating_add(row);
    if y < area.bottom().saturating_sub(1) {
        frame.set_cursor_position(Position { x, y });
    }
}

fn edit_field_location(state: &AppState, pane: Pane) -> Option<(u16, u16)> {
    if state.mode != Mode::Editing || state.focused_pane != pane {
        return None;
    }
    match pane {
        Pane::MaterialInput => match state.selected_material_field {
            MaterialField::Name => Some((1, 8)),
            MaterialField::Density => Some((2, 11)),
            MaterialField::MolarMass => Some((3, 14)),
            MaterialField::ValenceElectrons => Some((4, 21)),
            MaterialField::BandGap => Some((5, 12)),
            MaterialField::Preset => None,
        },
        Pane::EnergySweep => match state.selected_energy_field {
            EnergyField::ElectronEnergy => Some((1, 19)),
            EnergyField::EnergyMin => Some((3, 13)),
            EnergyField::EnergyMax => Some((4, 13)),
            EnergyField::Points => Some((5, 10)),
            _ => None,
        },
        _ => None,
    }
}

fn superscript_tick(value: f64) -> String {
    let rounded = value.round();
    if (value - rounded).abs() > 0.25 {
        String::new()
    } else {
        format!("10{}", to_superscript(rounded as i32))
    }
}

fn to_superscript(value: i32) -> String {
    value
        .to_string()
        .chars()
        .map(|ch| match ch {
            '-' => '⁻',
            '0' => '⁰',
            '1' => '¹',
            '2' => '²',
            '3' => '³',
            '4' => '⁴',
            '5' => '⁵',
            '6' => '⁶',
            '7' => '⁷',
            '8' => '⁸',
            '9' => '⁹',
            _ => ch,
        })
        .collect()
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

fn draw_axis_ticks<DB: DrawingBackend>(
    chart: &mut ChartContext<'_, DB, Cartesian2d<RangedCoordf64, RangedCoordf64>>,
    x_bounds: [f64; 2],
    y_bounds: [f64; 2],
) -> Result<(), DrawingAreaErrorKind<DB::ErrorType>> {
    let x_major_len = (y_bounds[1] - y_bounds[0]) * 0.035;
    let x_minor_len = x_major_len * 0.55;
    let y_major_len = (x_bounds[1] - x_bounds[0]) * 0.035;
    let y_minor_len = y_major_len * 0.55;

    for x in minor_ticks(x_bounds) {
        chart.draw_series([
            PathElement::new(
                vec![(x, y_bounds[0]), (x, y_bounds[0] + x_minor_len)],
                BLACK,
            ),
            PathElement::new(
                vec![(x, y_bounds[1]), (x, y_bounds[1] - x_minor_len)],
                BLACK,
            ),
        ])?;
    }
    for x in major_ticks(x_bounds) {
        chart.draw_series([
            PathElement::new(
                vec![(x, y_bounds[0]), (x, y_bounds[0] + x_major_len)],
                BLACK,
            ),
            PathElement::new(
                vec![(x, y_bounds[1]), (x, y_bounds[1] - x_major_len)],
                BLACK,
            ),
        ])?;
    }
    for y in minor_ticks(y_bounds) {
        chart.draw_series([
            PathElement::new(
                vec![(x_bounds[0], y), (x_bounds[0] + y_minor_len, y)],
                BLACK,
            ),
            PathElement::new(
                vec![(x_bounds[1], y), (x_bounds[1] - y_minor_len, y)],
                BLACK,
            ),
        ])?;
    }
    for y in major_ticks(y_bounds) {
        chart.draw_series([
            PathElement::new(
                vec![(x_bounds[0], y), (x_bounds[0] + y_major_len, y)],
                BLACK,
            ),
            PathElement::new(
                vec![(x_bounds[1], y), (x_bounds[1] - y_major_len, y)],
                BLACK,
            ),
        ])?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::Terminal;
    use ratatui::backend::TestBackend;

    #[test]
    fn pane_titles_include_required_numbers_and_names() {
        assert_eq!(
            pane_titles(),
            vec![
                "1 Material/Input",
                "2 Energy/Sweep",
                "3 IMFP log-log graph",
                "4 Result/Series",
                "5 Help/Log"
            ]
        );
    }

    #[test]
    fn render_smoke_test_for_five_pane_ui() {
        let backend = TestBackend::new(120, 40);
        let terminal = Terminal::new(backend);
        let Ok(mut terminal) = terminal else {
            unreachable!("test backend should be constructible");
        };
        let state = AppState::new(None, 50.0, 2000.0);
        let result = terminal.draw(|frame| render(frame, &state));

        assert!(result.is_ok());
        let rendered = buffer_text(terminal.backend().buffer());
        assert!(rendered.contains("1 Material/Input"));
        assert!(rendered.contains("2 Energy/Sweep"));
        assert!(rendered.contains("3 IMFP log-log graph"));
        assert!(rendered.contains("Electron energy"));
        assert!(rendered.contains("range min"));
        assert_eq!(terminal.backend().buffer()[(50, 2)].bg, Color::White);
    }

    #[test]
    fn editing_material_field_places_cursor_at_buffer_end() {
        let backend = TestBackend::new(120, 40);
        let terminal = Terminal::new(backend);
        let Ok(mut terminal) = terminal else {
            unreachable!("test backend should be constructible");
        };
        let mut state = AppState::new(None, 50.0, 2000.0);
        state.mode = Mode::Editing;
        state.focused_pane = Pane::MaterialInput;
        state.selected_material_field = MaterialField::Density;
        state.edit_buffer = "123".to_string();

        let result = terminal.draw(|frame| render(frame, &state));

        assert!(result.is_ok());
        terminal
            .backend_mut()
            .assert_cursor_position(Position { x: 15, y: 3 });
    }

    #[test]
    fn formats_log_ticks_with_superscript_exponents() {
        assert_eq!(superscript_tick(0.0), "10⁰");
        assert_eq!(superscript_tick(3.0), "10³");
        assert_eq!(superscript_tick(-1.0), "10⁻¹");
        assert_eq!(superscript_tick(0.4), "");
    }

    #[test]
    fn computes_minor_ticks_inside_decades() {
        let ticks = minor_ticks([0.0, 1.0]);

        assert_eq!(ticks.len(), 8);
        assert!(ticks.iter().all(|tick| *tick > 0.0 && *tick < 1.0));
    }

    fn buffer_text(buffer: &ratatui::buffer::Buffer) -> String {
        buffer
            .content()
            .iter()
            .map(|cell| cell.symbol())
            .collect::<String>()
    }
}
