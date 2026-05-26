use ratatui::Frame;
use ratatui::buffer::Buffer;
use ratatui::layout::{Constraint, Direction, Layout, Position, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::Line;
use ratatui::widgets::{Block, Borders, Paragraph, Row, Table};

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
            Constraint::Length(11),
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
            Constraint::Length(11),
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
        Line::from(format!("  IMFP at energy: {}", single_imfp_label(state))),
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
    let visible_rows = area.height.saturating_sub(3) as usize;
    let rows = state
        .sweep
        .as_ref()
        .map(|sweep| {
            let start = if visible_rows == 0 {
                0
            } else {
                state
                    .selected_row
                    .min(sweep.points.len().saturating_sub(1))
                    .saturating_sub(visible_rows.saturating_sub(1))
            };
            sweep
                .points
                .iter()
                .enumerate()
                .skip(start)
                .take(visible_rows)
                .map(|point| {
                    let (index, point) = point;
                    let row = Row::new(vec![
                        format!("{index:>5}"),
                        format!("{:>14.4}", point.electron_energy_e_v),
                        format!("{:>14.6}", point.imfp_nm),
                    ]);
                    if index == state.selected_row {
                        row.style(Style::default().fg(Color::Yellow))
                    } else {
                        row
                    }
                })
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();
    let table = Table::new(
        rows,
        [
            Constraint::Length(7),
            Constraint::Length(16),
            Constraint::Length(16),
        ],
    )
    .header(Row::new(vec!["index", "E / eV", "IMFP / nm"]))
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
    let outer = graph_block(state);
    let inner = outer.inner(area);
    frame.render_widget(outer, area);
    frame.render_widget(
        Block::default().style(Style::default().bg(Color::White)),
        inner,
    );
    draw_graph_buffer(
        frame.buffer_mut(),
        inner,
        &graph.points_log10,
        &graph.x_axis_label,
        &graph.y_axis_label,
        graph_marker(state),
    );
}

fn render_help(frame: &mut Frame<'_>, state: &AppState, area: Rect) {
    let text = "1-4 focus | Tab panes | hjkl move | gg/G bounds | / search | ? help | q quit";
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

fn single_imfp_label(state: &AppState) -> String {
    state
        .result
        .as_ref()
        .map(|result| format!("{} nm", format_float(result.imfp_nm)))
        .unwrap_or_else(|| "unavailable".to_string())
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
        .saturating_add(state.edit_cursor as u16)
        .min(max_x);
    let y = inner_y.saturating_add(row);
    if y < area.bottom().saturating_sub(1) {
        frame.set_cursor_position(Position { x, y });
    }
}

fn edit_field_location(state: &AppState, pane: Pane) -> Option<(u16, u16)> {
    if state.focused_pane != pane {
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
            EnergyField::EnergyMin => Some((4, 13)),
            EnergyField::EnergyMax => Some((5, 13)),
            EnergyField::Points => Some((6, 10)),
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

fn draw_graph_buffer(
    buffer: &mut Buffer,
    area: Rect,
    points: &[(f64, f64)],
    x_axis_label: &str,
    y_axis_label: &str,
    marker_x_log10: Option<f64>,
) {
    buffer.set_style(area, Style::default().fg(Color::Black).bg(Color::White));
    if area.width < 20 || area.height < 8 || points.len() < 2 {
        return;
    }

    let plot = Rect {
        x: area.x.saturating_add(7),
        y: area.y.saturating_add(2),
        width: area.width.saturating_sub(10),
        height: area.height.saturating_sub(5),
    };
    if plot.width < 8 || plot.height < 4 {
        return;
    }

    let x_bounds = bounds(points.iter().map(|(x, _)| *x));
    let y_bounds = bounds(points.iter().map(|(_, y)| *y));
    draw_plot_frame(buffer, plot);
    draw_plot_ticks(buffer, plot, x_bounds, y_bounds);
    draw_plot_labels(
        buffer,
        area,
        plot,
        x_bounds,
        y_bounds,
        x_axis_label,
        y_axis_label,
    );
    if let Some(marker_x_log10) = marker_x_log10 {
        draw_energy_marker(buffer, plot, x_bounds, marker_x_log10);
    }
    draw_plot_series(buffer, plot, x_bounds, y_bounds, points);
}

fn graph_marker(state: &AppState) -> Option<f64> {
    (state.energy.electron_energy_e_v > 0.0).then_some(state.energy.electron_energy_e_v.log10())
}

fn draw_plot_frame(buffer: &mut Buffer, plot: Rect) {
    let style = Style::default().fg(Color::Black).bg(Color::White);
    let left = plot.x;
    let right = plot.right().saturating_sub(1);
    let top = plot.y;
    let bottom = plot.bottom().saturating_sub(1);

    for x in left.saturating_add(1)..right {
        set_symbol(buffer, x, top, "─", style);
        set_symbol(buffer, x, bottom, "─", style);
    }
    for y in top.saturating_add(1)..bottom {
        set_symbol(buffer, left, y, "│", style);
        set_symbol(buffer, right, y, "│", style);
    }
    set_symbol(buffer, left, top, "┌", style);
    set_symbol(buffer, right, top, "┐", style);
    set_symbol(buffer, left, bottom, "└", style);
    set_symbol(buffer, right, bottom, "┘", style);
}

fn draw_plot_ticks(buffer: &mut Buffer, plot: Rect, x_bounds: [f64; 2], y_bounds: [f64; 2]) {
    let style = Style::default().fg(Color::Black).bg(Color::White);
    let left = plot.x;
    let right = plot.right().saturating_sub(1);
    let top = plot.y;
    let bottom = plot.bottom().saturating_sub(1);

    for x in minor_ticks(x_bounds) {
        let col = x_to_col(x, x_bounds, plot);
        set_symbol(buffer, col, top, "┬", style);
        set_symbol(buffer, col, bottom, "┴", style);
    }
    for x in major_ticks(x_bounds) {
        let col = x_to_col(x, x_bounds, plot);
        set_symbol(buffer, col, top, "┬", style);
        set_symbol(buffer, col, top.saturating_add(1), "│", style);
        set_symbol(buffer, col, bottom, "┴", style);
        set_symbol(buffer, col, bottom.saturating_sub(1), "│", style);
    }

    for y in minor_ticks(y_bounds) {
        let row = y_to_row(y, y_bounds, plot);
        set_symbol(buffer, left, row, "├", style);
        set_symbol(buffer, right, row, "┤", style);
    }
    for y in major_ticks(y_bounds) {
        let row = y_to_row(y, y_bounds, plot);
        set_symbol(buffer, left, row, "├", style);
        set_symbol(buffer, left.saturating_add(1), row, "─", style);
        set_symbol(buffer, right, row, "┤", style);
        set_symbol(buffer, right.saturating_sub(1), row, "─", style);
    }
}

fn draw_plot_labels(
    buffer: &mut Buffer,
    area: Rect,
    plot: Rect,
    x_bounds: [f64; 2],
    y_bounds: [f64; 2],
    x_axis_label: &str,
    y_axis_label: &str,
) {
    let style = Style::default().fg(Color::Black).bg(Color::White);
    let bottom = plot.bottom().saturating_sub(1);

    for x in major_ticks(x_bounds) {
        let label = superscript_tick(x);
        let col = centered_label_x(x_to_col(x, x_bounds, plot), label.chars().count() as u16);
        put_string(buffer, col, bottom.saturating_add(1), &label, style);
    }
    for y in major_ticks(y_bounds) {
        let label = superscript_tick(y);
        let row = y_to_row(y, y_bounds, plot);
        let col = plot.x.saturating_sub(label.chars().count() as u16 + 1);
        put_string(buffer, col, row, &label, style);
    }

    let x_label_x = centered_label_x(
        plot.x.saturating_add(plot.width / 2),
        x_axis_label.chars().count() as u16,
    );
    put_string(
        buffer,
        x_label_x,
        area.bottom().saturating_sub(1),
        x_axis_label,
        style,
    );
    put_string(
        buffer,
        area.x.saturating_add(1),
        plot.y.saturating_sub(1),
        y_axis_label,
        style,
    );
}

fn draw_plot_series(
    buffer: &mut Buffer,
    plot: Rect,
    x_bounds: [f64; 2],
    y_bounds: [f64; 2],
    points: &[(f64, f64)],
) {
    let style = Style::default().fg(Color::Red).bg(Color::White);
    let mut cells = std::collections::BTreeMap::<(u16, u16), u8>::new();
    for (x, y) in points {
        let Some((col, row, mask)) = point_to_braille_cell(*x, *y, x_bounds, y_bounds, plot) else {
            continue;
        };
        for (dot_col, dot_row, dot_mask) in expanded_braille_dot(col, row, mask, plot) {
            cells
                .entry((dot_col, dot_row))
                .and_modify(|existing| *existing |= dot_mask)
                .or_insert(dot_mask);
        }
    }
    for ((col, row), mask) in cells {
        if let Some(symbol) = char::from_u32(0x2800 + u32::from(mask)) {
            set_symbol(buffer, col, row, &symbol.to_string(), style);
        }
    }
}

fn draw_energy_marker(buffer: &mut Buffer, plot: Rect, x_bounds: [f64; 2], x: f64) {
    if x < x_bounds[0] || x > x_bounds[1] {
        return;
    }
    let style = Style::default().fg(Color::Blue).bg(Color::White);
    let col = x_to_col(x, x_bounds, plot);
    for row in plot.y.saturating_add(1)..plot.bottom().saturating_sub(1) {
        set_symbol(buffer, col, row, "│", style);
    }
}

fn point_to_braille_cell(
    x: f64,
    y: f64,
    x_bounds: [f64; 2],
    y_bounds: [f64; 2],
    plot: Rect,
) -> Option<(u16, u16, u8)> {
    let inner_width = plot.width.checked_sub(2)?;
    let inner_height = plot.height.checked_sub(2)?;
    let x_span = (x_bounds[1] - x_bounds[0]).max(f64::EPSILON);
    let y_span = (y_bounds[1] - y_bounds[0]).max(f64::EPSILON);
    let x_t = ((x - x_bounds[0]) / x_span).clamp(0.0, 1.0);
    let y_t = ((y - y_bounds[0]) / y_span).clamp(0.0, 1.0);
    let sub_width = inner_width.saturating_mul(2).saturating_sub(1);
    let sub_height = inner_height.saturating_mul(4).saturating_sub(1);
    let sub_x = (x_t * f64::from(sub_width)).round() as u16;
    let sub_y = ((1.0 - y_t) * f64::from(sub_height)).round() as u16;
    let col = plot.x.saturating_add(1).saturating_add(sub_x / 2);
    let row = plot.y.saturating_add(1).saturating_add(sub_y / 4);
    let mask = braille_dot_mask((sub_x % 2) as u8, (sub_y % 4) as u8);
    Some((col, row, mask))
}

fn braille_dot_mask(x: u8, y: u8) -> u8 {
    match (x, y) {
        (0, 0) => 0b0000_0001,
        (0, 1) => 0b0000_0010,
        (0, 2) => 0b0000_0100,
        (0, 3) => 0b0100_0000,
        (1, 0) => 0b0000_1000,
        (1, 1) => 0b0001_0000,
        (1, 2) => 0b0010_0000,
        (1, 3) => 0b1000_0000,
        _ => 0,
    }
}

fn expanded_braille_dot(
    col: u16,
    row: u16,
    mask: u8,
    plot: Rect,
) -> impl Iterator<Item = (u16, u16, u8)> {
    let Some((sub_x, sub_y)) = braille_mask_position(mask) else {
        return Vec::new().into_iter();
    };
    let base_x = i32::from(col.saturating_sub(plot.x.saturating_add(1))) * 2 + i32::from(sub_x);
    let base_y = i32::from(row.saturating_sub(plot.y.saturating_add(1))) * 4 + i32::from(sub_y);
    let max_x = i32::from(plot.width.saturating_sub(2)) * 2;
    let max_y = i32::from(plot.height.saturating_sub(2)) * 4;
    let mut dots = Vec::new();
    for (dx, dy) in [(0, 0), (1, 0), (-1, 0), (0, 1), (0, -1)] {
        let x = base_x + dx;
        let y = base_y + dy;
        if x < 0 || y < 0 || x >= max_x || y >= max_y {
            continue;
        }
        let cell_col = plot.x.saturating_add(1).saturating_add((x / 2) as u16);
        let cell_row = plot.y.saturating_add(1).saturating_add((y / 4) as u16);
        dots.push((
            cell_col,
            cell_row,
            braille_dot_mask((x % 2) as u8, (y % 4) as u8),
        ));
    }
    dots.into_iter()
}

fn braille_mask_position(mask: u8) -> Option<(u8, u8)> {
    match mask {
        0b0000_0001 => Some((0, 0)),
        0b0000_0010 => Some((0, 1)),
        0b0000_0100 => Some((0, 2)),
        0b0100_0000 => Some((0, 3)),
        0b0000_1000 => Some((1, 0)),
        0b0001_0000 => Some((1, 1)),
        0b0010_0000 => Some((1, 2)),
        0b1000_0000 => Some((1, 3)),
        _ => None,
    }
}

fn x_to_col(value: f64, bounds: [f64; 2], plot: Rect) -> u16 {
    let span = (bounds[1] - bounds[0]).max(f64::EPSILON);
    let t = ((value - bounds[0]) / span).clamp(0.0, 1.0);
    plot.x
        .saturating_add(1)
        .saturating_add((t * f64::from(plot.width.saturating_sub(3))).round() as u16)
}

fn y_to_row(value: f64, bounds: [f64; 2], plot: Rect) -> u16 {
    let span = (bounds[1] - bounds[0]).max(f64::EPSILON);
    let t = ((value - bounds[0]) / span).clamp(0.0, 1.0);
    plot.y
        .saturating_add(1)
        .saturating_add(((1.0 - t) * f64::from(plot.height.saturating_sub(3))).round() as u16)
}

fn centered_label_x(center: u16, width: u16) -> u16 {
    center.saturating_sub(width / 2)
}

fn put_string(buffer: &mut Buffer, x: u16, y: u16, text: &str, style: Style) {
    for (offset, ch) in text.chars().enumerate() {
        set_symbol(
            buffer,
            x.saturating_add(offset as u16),
            y,
            &ch.to_string(),
            style,
        );
    }
}

fn set_symbol(buffer: &mut Buffer, x: u16, y: u16, symbol: &str, style: Style) {
    if x >= buffer.area().right() || y >= buffer.area().bottom() {
        return;
    }
    buffer[(x, y)].set_symbol(symbol).set_style(style);
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
                "3 Result/Series",
                "4 IMFP log-log graph"
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
        assert!(rendered.contains("4 IMFP log-log graph"));
        assert!(rendered.contains("3 Result/Series"));
        assert!(rendered.contains("Electron energy"));
        assert!(rendered.contains("IMFP at energy"));
        assert!(rendered.contains("range min"));
        assert_eq!(terminal.backend().buffer()[(50, 2)].bg, Color::White);
    }

    #[test]
    fn graph_uses_solid_black_axes_and_high_resolution_red_series_without_top_or_right_labels() {
        let mut buffer = Buffer::empty(Rect::new(0, 0, 80, 24));
        let points = vec![(1.0, -1.0), (1.6, -0.2), (2.2, 0.4), (3.0, 1.0)];

        draw_graph_buffer(
            &mut buffer,
            Rect::new(0, 0, 80, 24),
            &points,
            "Electron Energy / eV",
            "IMFP / nm",
            Some(2.2),
        );

        let rendered = buffer_text(&buffer);
        assert!(rendered.contains("┌"));
        assert!(rendered.contains("┬"));
        assert!(!rendered.contains('.'));
        assert!(rendered.chars().any(is_braille));
        assert!(!rendered.contains('╱'));
        assert!(!rendered.contains('╲'));
        assert!(
            buffer
                .content()
                .iter()
                .filter(|cell| cell.fg == Color::Red)
                .count()
                > 0
        );
        assert!(
            buffer
                .content()
                .iter()
                .filter(|cell| cell.fg == Color::Blue)
                .count()
                > 0
        );
        assert!(!row_text(&buffer, 0).contains("Electron Energy / eV"));
        assert!(!right_edge_text(&buffer, 16).contains("IMFP / nm"));
    }

    #[test]
    fn graph_series_uses_braille_subcell_resolution() {
        let point =
            point_to_braille_cell(1.25, 0.25, [1.0, 2.0], [0.0, 1.0], Rect::new(0, 0, 12, 8))
                .expect("point should fit plot");

        assert_ne!(point.2, braille_dot_mask(0, 0));
    }

    #[test]
    fn graph_series_expands_each_braille_dot_for_visibility() {
        let dots: Vec<_> =
            expanded_braille_dot(5, 5, braille_dot_mask(1, 2), Rect::new(0, 0, 12, 8)).collect();

        assert!(dots.len() > 1);
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
        state.edit_cursor = 3;

        let result = terminal.draw(|frame| render(frame, &state));

        assert!(result.is_ok());
        terminal
            .backend_mut()
            .assert_cursor_position(Position { x: 15, y: 3 });
    }

    #[test]
    fn normal_mode_shows_cursor_on_editable_field() {
        let backend = TestBackend::new(120, 40);
        let terminal = Terminal::new(backend);
        let Ok(mut terminal) = terminal else {
            unreachable!("test backend should be constructible");
        };
        let mut state = AppState::new(None, 50.0, 2000.0);
        state.focused_pane = Pane::MaterialInput;
        state.selected_material_field = MaterialField::Name;
        state.edit_cursor = 1;

        let result = terminal.draw(|frame| render(frame, &state));

        assert!(result.is_ok());
        terminal
            .backend_mut()
            .assert_cursor_position(Position { x: 10, y: 2 });
    }

    #[test]
    fn result_table_can_scroll_to_last_sweep_point() {
        let backend = TestBackend::new(120, 40);
        let terminal = Terminal::new(backend);
        let Ok(mut terminal) = terminal else {
            unreachable!("test backend should be constructible");
        };
        let mut state = AppState::new(None, 50.0, 2000.0);
        state.focused_pane = Pane::ResultSeries;
        state.selected_row = state
            .sweep
            .as_ref()
            .map(|sweep| sweep.points.len().saturating_sub(1))
            .unwrap_or(0);

        let result = terminal.draw(|frame| render(frame, &state));

        assert!(result.is_ok());
        let rendered = buffer_text(terminal.backend().buffer());
        assert!(rendered.contains("  199"));
        assert!(
            terminal
                .backend()
                .buffer()
                .content()
                .iter()
                .any(|cell| cell.fg == Color::Yellow && cell.symbol() == "1")
        );
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

    fn row_text(buffer: &ratatui::buffer::Buffer, y: u16) -> String {
        (0..buffer.area().width)
            .map(|x| buffer[(x, y)].symbol())
            .collect::<String>()
    }

    fn right_edge_text(buffer: &ratatui::buffer::Buffer, width: u16) -> String {
        let start = buffer.area().right().saturating_sub(width);
        buffer
            .content()
            .chunks(buffer.area().width as usize)
            .map(|row| {
                row.iter()
                    .skip(start as usize)
                    .map(|cell| cell.symbol())
                    .collect::<String>()
            })
            .collect::<Vec<_>>()
            .join("\n")
    }

    fn is_braille(ch: char) -> bool {
        ('\u{2800}'..='\u{28ff}').contains(&ch)
    }
}
