use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::sync::mpsc::{self, Receiver, Sender};
use std::thread;

use ratatui::Frame;
use ratatui::layout::{Constraint, Direction, Layout, Position, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::Line;
use ratatui::widgets::{Block, Borders, Paragraph, Row, Table};
use ratatui_image::picker::Picker;
use ratatui_image::protocol::Protocol;
use ratatui_image::{Image as RatatuiImage, Resize};

use crate::app::{AppState, EnergyField, MaterialField, Mode, Pane};

pub struct GraphImageState {
    picker: Picker,
    cache: Option<GraphImageCache>,
    pending_key: Option<GraphImageKey>,
    failed_key: Option<GraphImageKey>,
    result_sender: Sender<GraphImageRenderResult>,
    result_receiver: Receiver<GraphImageRenderResult>,
}

struct GraphImageCache {
    key: GraphImageKey,
    protocol: Protocol,
}

struct GraphImageRenderResult {
    key: GraphImageKey,
    image: Result<image::DynamicImage, String>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct GraphImageKey {
    width: u16,
    height: u16,
    points_len: usize,
    points_hash: u64,
    marker_x: Option<u64>,
}

impl GraphImageState {
    pub fn new(picker: Picker) -> Self {
        let (result_sender, result_receiver) = mpsc::channel();
        Self {
            picker,
            cache: None,
            pending_key: None,
            failed_key: None,
            result_sender,
            result_receiver,
        }
    }
}

pub fn render(frame: &mut Frame<'_>, state: &AppState) {
    render_internal(frame, state, None);
}

pub fn render_with_graph_image(
    frame: &mut Frame<'_>,
    state: &AppState,
    graph_image: &mut GraphImageState,
) {
    render_internal(frame, state, Some(graph_image));
}

fn render_internal(
    frame: &mut Frame<'_>,
    state: &AppState,
    graph_image: Option<&mut GraphImageState>,
) {
    let area = frame.area();
    if area.width < 90 {
        render_stacked(frame, state, area, graph_image);
    } else {
        render_split(frame, state, area, graph_image);
    }
}

pub fn pane_titles() -> Vec<&'static str> {
    Pane::ORDER.iter().map(|pane| pane.title()).collect()
}

fn render_split(
    frame: &mut Frame<'_>,
    state: &AppState,
    area: Rect,
    graph_image: Option<&mut GraphImageState>,
) {
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
    render_graph(frame, state, right[0], graph_image);
    render_messages(frame, state, right[1]);
}

fn render_stacked(
    frame: &mut Frame<'_>,
    state: &AppState,
    area: Rect,
    graph_image: Option<&mut GraphImageState>,
) {
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
    render_graph(frame, state, rows[2], graph_image);
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

fn render_graph(
    frame: &mut Frame<'_>,
    state: &AppState,
    area: Rect,
    graph_image: Option<&mut GraphImageState>,
) {
    if state.graph.is_none() {
        frame.render_widget(
            Paragraph::new("graph unavailable").block(block(Pane::Graph, state)),
            area,
        );
        return;
    }
    let outer = graph_block(state);
    let inner = outer.inner(area);
    frame.render_widget(outer, area);
    frame.render_widget(
        Block::default().style(Style::default().bg(Color::White)),
        inner,
    );
    if let Some(graph_image) = graph_image {
        let _ = render_graph_image(frame, state, inner, graph_image);
    }
}

fn render_graph_image(
    frame: &mut Frame<'_>,
    state: &AppState,
    area: Rect,
    graph_image: &mut GraphImageState,
) -> Result<(), String> {
    let Some(graph) = &state.graph else {
        return Err("graph unavailable".to_string());
    };
    if area.width < 20 || area.height < 8 {
        return Err("graph area too small".to_string());
    }
    let key = graph_image_key(state, area)?;
    receive_graph_image_results(graph_image);
    if graph_image
        .cache
        .as_ref()
        .is_some_and(|cache| cache.key == key)
    {
        let Some(cache) = &graph_image.cache else {
            return Err("image cache unavailable".to_string());
        };
        frame.render_widget(RatatuiImage::new(&cache.protocol), area);
        return Ok(());
    }
    if graph_image.failed_key == Some(key) {
        return Err("image render failed".to_string());
    }
    if graph_image.pending_key != Some(key) {
        spawn_graph_image_render(graph_image, key, area, graph)?;
    }
    Err("image render pending".to_string())
}

fn render_help(frame: &mut Frame<'_>, _state: &AppState, area: Rect) {
    let text = "1-5 focus | Tab panes | hjkl move | gg/G bounds | / search | ? help | q quit";
    frame.render_widget(
        Paragraph::new(text).block(Block::default().borders(Borders::ALL).title("Help")),
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
        Paragraph::new(lines).block(block(Pane::HelpLog, state)),
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
    let title = if state.focused_pane == pane && state.mode == Mode::Visual {
        format!("{} [VISUAL]", pane.title())
    } else {
        pane.title().to_string()
    };
    Block::default()
        .borders(Borders::ALL)
        .title(title)
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

fn graph_marker(state: &AppState) -> Option<f64> {
    (state.energy.electron_energy_e_v > 0.0).then_some(state.energy.electron_energy_e_v.log10())
}

fn graph_image_key(state: &AppState, area: Rect) -> Result<GraphImageKey, String> {
    let graph = state
        .graph
        .as_ref()
        .ok_or_else(|| "graph unavailable".to_string())?;
    if graph.points_log10.is_empty() {
        return Err("graph points unavailable".to_string());
    }
    Ok(GraphImageKey {
        width: area.width,
        height: area.height,
        points_len: graph.points_log10.len(),
        points_hash: graph_points_hash(&graph.points_log10),
        marker_x: graph_marker(state).map(f64::to_bits),
    })
}

fn graph_points_hash(points: &[(f64, f64)]) -> u64 {
    let mut hasher = DefaultHasher::new();
    for (x, y) in points {
        x.to_bits().hash(&mut hasher);
        y.to_bits().hash(&mut hasher);
    }
    hasher.finish()
}

fn receive_graph_image_results(graph_image: &mut GraphImageState) {
    while let Ok(result) = graph_image.result_receiver.try_recv() {
        if graph_image.pending_key != Some(result.key) {
            continue;
        }
        graph_image.pending_key = None;
        match result.image {
            Ok(image) => {
                let protocol = graph_image.picker.new_protocol(
                    image,
                    Rect::new(0, 0, result.key.width, result.key.height),
                    Resize::Fit(None),
                );
                match protocol {
                    Ok(protocol) => {
                        graph_image.failed_key = None;
                        graph_image.cache = Some(GraphImageCache {
                            key: result.key,
                            protocol,
                        });
                    }
                    Err(_) => {
                        graph_image.failed_key = Some(result.key);
                    }
                }
            }
            Err(_) => {
                graph_image.failed_key = Some(result.key);
            }
        }
    }
}

fn spawn_graph_image_render(
    graph_image: &mut GraphImageState,
    key: GraphImageKey,
    area: Rect,
    graph: &tpp2m_core::LogPlotData,
) -> Result<(), String> {
    let font_size = graph_image.picker.font_size();
    let width = u32::from(area.width).saturating_mul(u32::from(font_size.0));
    let height = u32::from(area.height).saturating_mul(u32::from(font_size.1));
    let points = graph.points_log10.clone();
    let x_axis_label = graph.x_axis_label.clone();
    let y_axis_label = graph.y_axis_label.clone();
    let marker_x_log10 = key.marker_x.map(f64::from_bits);
    let result_sender = graph_image.result_sender.clone();
    graph_image.pending_key = Some(key);
    thread::Builder::new()
        .name("tpp2m-tui-graph-render".to_string())
        .spawn(move || {
            let image = render_plot_image(
                width,
                height,
                &points,
                &x_axis_label,
                &y_axis_label,
                marker_x_log10,
            );
            let _ = result_sender.send(GraphImageRenderResult { key, image });
        })
        .map_err(|error| error.to_string())?;
    Ok(())
}

fn render_plot_image(
    width: u32,
    height: u32,
    points: &[(f64, f64)],
    x_axis_label: &str,
    y_axis_label: &str,
    marker_x_log10: Option<f64>,
) -> Result<image::DynamicImage, String> {
    use plotters::prelude as plt;
    use plotters::prelude::{DrawingArea, IntoDrawingArea, IntoFont};
    use plotters::style::Color as PlottersColor;
    use plotters::style::text_anchor::{HPos, Pos, VPos};
    use plotters::style::{FontTransform, IntoTextStyle, TextStyle};
    use plotters_bitmap::BitMapBackend;

    if width < 200 || height < 160 || points.len() < 2 {
        return Err("image plot area too small".to_string());
    }
    let mut buffer = vec![255_u8; width as usize * height as usize * 3];
    {
        let root: DrawingArea<_, _> =
            BitMapBackend::with_buffer(&mut buffer, (width, height)).into_drawing_area();
        root.fill(&plt::WHITE).map_err(|error| error.to_string())?;
        let x_bounds = bounds(points.iter().map(|(x, _)| *x));
        let y_bounds = bounds(points.iter().map(|(_, y)| *y));

        let plot = PlotPixels {
            left: 76,
            right: width as i32 - 28,
            top: 24,
            bottom: height as i32 - 72,
            x_bounds,
            y_bounds,
        };
        if plot.right <= plot.left + 16 || plot.bottom <= plot.top + 16 {
            return Err("image plot area too small".to_string());
        }

        let axis_style = plt::BLACK.stroke_width(1);
        let tick_style = plt::BLACK.stroke_width(1);
        let red_style = plt::RED.stroke_width(3);
        let marker_style = plt::BLUE.mix(0.7).stroke_width(2);

        root.draw(&plt::PathElement::new(
            vec![
                (plot.left, plot.top),
                (plot.right, plot.top),
                (plot.right, plot.bottom),
                (plot.left, plot.bottom),
                (plot.left, plot.top),
            ],
            axis_style,
        ))
        .map_err(|error| error.to_string())?;

        for tick in minor_log_ticks(x_bounds) {
            let x = plot.x(tick);
            draw_line(&root, [(x, plot.bottom), (x, plot.bottom - 7)], tick_style)?;
            draw_line(&root, [(x, plot.top), (x, plot.top + 7)], tick_style)?;
        }
        for tick in minor_log_ticks(y_bounds) {
            let y = plot.y(tick);
            draw_line(&root, [(plot.left, y), (plot.left + 7, y)], tick_style)?;
            draw_line(&root, [(plot.right, y), (plot.right - 7, y)], tick_style)?;
        }
        for exponent in major_log_tick_exponents(x_bounds) {
            let x = plot.x(f64::from(exponent));
            draw_line(&root, [(x, plot.bottom), (x, plot.bottom - 12)], tick_style)?;
            draw_line(&root, [(x, plot.top), (x, plot.top + 12)], tick_style)?;
            draw_power_tick_label(&root, x, plot.bottom + 18, exponent, TickLabelSide::Bottom)?;
        }
        for exponent in major_log_tick_exponents(y_bounds) {
            let y = plot.y(f64::from(exponent));
            draw_line(&root, [(plot.left, y), (plot.left + 12, y)], tick_style)?;
            draw_line(&root, [(plot.right, y), (plot.right - 12, y)], tick_style)?;
            draw_power_tick_label(&root, plot.left - 12, y, exponent, TickLabelSide::Left)?;
        }

        let axis_label_style = ("sans-serif", 18)
            .into_text_style(&root)
            .color(&plt::BLACK)
            .pos(Pos::new(HPos::Center, VPos::Center));
        root.draw_text(
            x_axis_label,
            &axis_label_style,
            ((plot.left + plot.right) / 2, height as i32 - 24),
        )
        .map_err(|error| error.to_string())?;
        let y_label_style = TextStyle::from(("sans-serif", 18).into_font())
            .color(&plt::BLACK)
            .pos(Pos::new(HPos::Center, VPos::Center))
            .transform(FontTransform::Rotate270);
        root.draw_text(
            y_axis_label,
            &y_label_style,
            (22, (plot.top + plot.bottom) / 2),
        )
        .map_err(|error| error.to_string())?;

        let series = points
            .iter()
            .filter(|(x, y)| x.is_finite() && y.is_finite())
            .map(|(x, y)| (plot.x(*x), plot.y(*y)))
            .collect::<Vec<_>>();
        if series.len() >= 2 {
            root.draw(&plt::PathElement::new(series, red_style))
                .map_err(|error| error.to_string())?;
        }

        if let Some(marker_x_log10) = marker_x_log10
            && marker_x_log10 >= x_bounds[0]
            && marker_x_log10 <= x_bounds[1]
        {
            let x = plot.x(marker_x_log10);
            draw_line(&root, [(x, plot.top), (x, plot.bottom)], marker_style)?;
        }
        root.present().map_err(|error| error.to_string())?;
    }
    let image = image::RgbImage::from_raw(width, height, buffer)
        .ok_or_else(|| "invalid plot image buffer".to_string())?;
    Ok(image::DynamicImage::ImageRgb8(image))
}

#[derive(Clone, Copy, Debug)]
struct PlotPixels {
    left: i32,
    right: i32,
    top: i32,
    bottom: i32,
    x_bounds: [f64; 2],
    y_bounds: [f64; 2],
}

impl PlotPixels {
    fn x(self, value: f64) -> i32 {
        let span = (self.x_bounds[1] - self.x_bounds[0]).max(f64::EPSILON);
        let t = ((value - self.x_bounds[0]) / span).clamp(0.0, 1.0);
        self.left + (t * f64::from(self.right - self.left)).round() as i32
    }

    fn y(self, value: f64) -> i32 {
        let span = (self.y_bounds[1] - self.y_bounds[0]).max(f64::EPSILON);
        let t = ((value - self.y_bounds[0]) / span).clamp(0.0, 1.0);
        self.bottom - (t * f64::from(self.bottom - self.top)).round() as i32
    }
}

fn major_log_tick_exponents(bounds: [f64; 2]) -> Vec<i32> {
    let start = bounds[0].ceil() as i32;
    let end = bounds[1].floor() as i32;
    (start..=end).collect()
}

fn minor_log_ticks(bounds: [f64; 2]) -> Vec<f64> {
    let start = bounds[0].floor() as i32;
    let end = bounds[1].ceil() as i32;
    let mut ticks = Vec::new();
    for exponent in start..=end {
        for multiplier in 2..10 {
            let tick = f64::from(exponent) + f64::from(multiplier).log10();
            if tick > bounds[0] && tick < bounds[1] {
                ticks.push(tick);
            }
        }
    }
    ticks
}

fn draw_line<DB: plotters::prelude::DrawingBackend>(
    root: &plotters::prelude::DrawingArea<DB, plotters::coord::Shift>,
    points: [(i32, i32); 2],
    style: plotters::style::ShapeStyle,
) -> Result<(), String> {
    root.draw(&plotters::prelude::PathElement::new(points, style))
        .map_err(|error| error.to_string())
}

#[derive(Clone, Copy, Debug)]
enum TickLabelSide {
    Bottom,
    Left,
}

fn draw_power_tick_label<DB: plotters::prelude::DrawingBackend>(
    root: &plotters::prelude::DrawingArea<DB, plotters::coord::Shift>,
    x: i32,
    y: i32,
    exponent: i32,
    side: TickLabelSide,
) -> Result<(), String> {
    use plotters::prelude as plt;
    use plotters::prelude::IntoFont;
    use plotters::style::text_anchor::{HPos, Pos, VPos};
    use plotters::style::{IntoTextStyle, TextStyle};

    let base_style = ("sans-serif", 15)
        .into_text_style(root)
        .color(&plt::BLACK)
        .pos(Pos::new(HPos::Left, VPos::Top));
    let exponent_style = TextStyle::from(("sans-serif", 10).into_font())
        .color(&plt::BLACK)
        .pos(Pos::new(HPos::Left, VPos::Top));
    let exponent_text = exponent.to_string();
    let label_width = 18 + exponent_text.chars().count() as i32 * 7;
    let (base_x, base_y) = match side {
        TickLabelSide::Bottom => (x - label_width / 2, y),
        TickLabelSide::Left => (x - label_width, y - 8),
    };
    root.draw_text("10", &base_style, (base_x, base_y))
        .map_err(|error| error.to_string())?;
    root.draw_text(&exponent_text, &exponent_style, (base_x + 19, base_y - 7))
        .map_err(|error| error.to_string())
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
                "4 IMFP log-log graph",
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
        assert!(rendered.contains("4 IMFP log-log graph"));
        assert!(rendered.contains("3 Result/Series"));
        assert!(rendered.contains("5 Help/Log"));
        assert!(rendered.contains("Electron energy"));
        assert!(rendered.contains("IMFP at energy"));
        assert!(rendered.contains("range min"));
        assert_eq!(terminal.backend().buffer()[(50, 2)].bg, Color::White);
    }

    #[test]
    fn visual_mode_marks_focused_pane_title() {
        let backend = TestBackend::new(120, 40);
        let terminal = Terminal::new(backend);
        let Ok(mut terminal) = terminal else {
            unreachable!("test backend should be constructible");
        };
        let mut state = AppState::new(None, 50.0, 2000.0);
        state.focused_pane = Pane::HelpLog;
        state.mode = Mode::Visual;

        let result = terminal.draw(|frame| render(frame, &state));

        assert!(result.is_ok());
        let rendered = buffer_text(terminal.backend().buffer());
        assert!(rendered.contains("5 Help/Log [VISUAL]"));
    }

    #[test]
    fn plotters_bitmap_graph_image_contains_non_white_pixels() {
        let points = vec![(1.0, -0.2), (1.5, 0.0), (2.0, 0.3), (3.0, 1.0)];

        let image = render_plot_image(
            320,
            240,
            &points,
            "Electron Energy / eV",
            "IMFP / nm",
            Some(2.0),
        )
        .expect("plot image should render");

        assert_eq!(image.width(), 320);
        assert_eq!(image.height(), 240);
        assert!(
            image
                .to_rgb8()
                .pixels()
                .any(|pixel| pixel.0 != [255, 255, 255])
        );
    }

    #[test]
    fn graph_image_render_starts_background_job_without_blocking_for_cache() {
        let backend = TestBackend::new(120, 40);
        let terminal = Terminal::new(backend);
        let Ok(mut terminal) = terminal else {
            unreachable!("test backend should be constructible");
        };
        let state = AppState::new(None, 50.0, 2000.0);
        let picker = Picker::from_fontsize((10, 20));
        let mut graph_image = GraphImageState::new(picker);

        terminal
            .draw(|frame| render_with_graph_image(frame, &state, &mut graph_image))
            .expect("render should start graph worker");

        assert!(graph_image.pending_key.is_some());
        assert!(graph_image.cache.is_none());
    }

    #[test]
    fn graph_image_render_adopts_completed_background_image() {
        let backend = TestBackend::new(120, 40);
        let terminal = Terminal::new(backend);
        let Ok(mut terminal) = terminal else {
            unreachable!("test backend should be constructible");
        };
        let state = AppState::new(None, 50.0, 2000.0);
        let picker = Picker::from_fontsize((10, 20));
        let mut graph_image = GraphImageState::new(picker);

        terminal
            .draw(|frame| render_with_graph_image(frame, &state, &mut graph_image))
            .expect("render should start graph worker");
        let expected_key = graph_image
            .pending_key
            .expect("first render should leave an image job pending");

        for _ in 0..50 {
            receive_graph_image_results(&mut graph_image);
            if graph_image
                .cache
                .as_ref()
                .is_some_and(|cache| cache.key == expected_key)
            {
                break;
            }
            std::thread::sleep(std::time::Duration::from_millis(10));
        }

        assert!(graph_image.pending_key.is_none());
        assert!(
            graph_image
                .cache
                .as_ref()
                .is_some_and(|cache| cache.key == expected_key)
        );
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
    fn log_plot_ticks_separate_major_exponents_from_minor_positions() {
        assert_eq!(major_log_tick_exponents([1.0, 3.0]), vec![1, 2, 3]);

        let minor = minor_log_ticks([1.0, 2.0]);

        assert_eq!(minor.len(), 8);
        assert!(minor.iter().all(|tick| *tick > 1.0 && *tick < 2.0));
        assert!(
            minor
                .iter()
                .any(|tick| { (*tick - (1.0 + 2.0_f64.log10())).abs() < f64::EPSILON })
        );
    }

    fn buffer_text(buffer: &ratatui::buffer::Buffer) -> String {
        buffer
            .content()
            .iter()
            .map(|cell| cell.symbol())
            .collect::<String>()
    }
}
