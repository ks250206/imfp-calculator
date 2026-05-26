use ratatui::Frame;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::symbols;
use ratatui::text::Line;
use ratatui::widgets::{Axis, Block, Borders, Chart, Dataset, Paragraph, Row, Table};

use crate::app::{AppState, Pane};

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
            Constraint::Length(7),
            Constraint::Length(7),
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
            Constraint::Length(7),
            Constraint::Length(7),
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
    let text = vec![
        Line::from(format!(
            "density: {:.6} g/cm3",
            state.material.density_g_cm3
        )),
        Line::from(format!(
            "molar mass: {:.6} g/mol",
            state.material.molar_mass_g_mol
        )),
        Line::from(format!(
            "valence electrons: {:.6}",
            state.material.valence_electrons
        )),
        Line::from(format!("band gap: {:.6} eV", state.material.band_gap_e_v)),
    ];
    frame.render_widget(
        Paragraph::new(text).block(block(Pane::MaterialInput, state)),
        area,
    );
}

fn render_energy(frame: &mut Frame<'_>, state: &AppState, area: Rect) {
    let text = vec![
        Line::from(format!(
            "energy: {:.6} eV",
            state.energy.electron_energy_e_v
        )),
        Line::from(format!(
            "range: {:.6}..{:.6} eV",
            state.energy.energy_min_e_v, state.energy.energy_max_e_v
        )),
        Line::from(format!(
            "points: {} / spacing: {:?}",
            state.energy.points, state.energy.spacing
        )),
    ];
    frame.render_widget(
        Paragraph::new(text).block(block(Pane::EnergySweep, state)),
        area,
    );
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
    let dataset = Dataset::default()
        .name("IMFP")
        .marker(symbols::Marker::Braille)
        .style(Style::default().fg(Color::Cyan))
        .data(&graph.points_log10);
    let x_bounds = bounds(graph.points_log10.iter().map(|(x, _)| *x));
    let y_bounds = bounds(graph.points_log10.iter().map(|(_, y)| *y));
    let chart = Chart::new(vec![dataset])
        .block(block(Pane::Graph, state))
        .x_axis(
            Axis::default()
                .title(graph.x_axis_label.clone())
                .bounds(x_bounds),
        )
        .y_axis(
            Axis::default()
                .title(graph.y_axis_label.clone())
                .bounds(y_bounds),
        );
    frame.render_widget(chart, area);
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
    }
}
