use tpp2m_core::{LogPlotData, Spacing, SweepInput, SweepOutput, Tpp2mInput, Tpp2mOutput};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Pane {
    MaterialInput,
    EnergySweep,
    Graph,
    ResultSeries,
    HelpLog,
}

impl Pane {
    pub const ORDER: [Self; 5] = [
        Self::MaterialInput,
        Self::EnergySweep,
        Self::Graph,
        Self::ResultSeries,
        Self::HelpLog,
    ];

    pub fn from_number(number: u8) -> Option<Self> {
        match number {
            1 => Some(Self::MaterialInput),
            2 => Some(Self::EnergySweep),
            3 => Some(Self::Graph),
            4 => Some(Self::ResultSeries),
            5 => Some(Self::HelpLog),
            _ => None,
        }
    }

    pub fn title(self) -> &'static str {
        match self {
            Self::MaterialInput => "1 Material/Input",
            Self::EnergySweep => "2 Energy/Sweep",
            Self::Graph => "3 IMFP log-log graph",
            Self::ResultSeries => "4 Result/Series",
            Self::HelpLog => "5 Help/Log",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Mode {
    Normal,
    Editing,
    Search,
    Help,
    Command,
    ConfirmQuit,
}

#[derive(Clone, Debug, PartialEq)]
pub struct MaterialForm {
    pub density_g_cm3: f64,
    pub molar_mass_g_mol: f64,
    pub valence_electrons: f64,
    pub band_gap_e_v: f64,
}

#[derive(Clone, Debug, PartialEq)]
pub struct EnergyForm {
    pub electron_energy_e_v: f64,
    pub energy_min_e_v: f64,
    pub energy_max_e_v: f64,
    pub points: usize,
    pub spacing: Spacing,
    pub allow_extrapolate: bool,
}

#[derive(Clone, Debug, PartialEq)]
pub enum MessageLevel {
    Info,
    Warning,
    Error,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Message {
    pub level: MessageLevel,
    pub text: String,
}

#[derive(Clone, Debug, PartialEq)]
pub struct AppState {
    pub focused_pane: Pane,
    pub mode: Mode,
    pub material: MaterialForm,
    pub energy: EnergyForm,
    pub result: Option<Tpp2mOutput>,
    pub sweep: Option<SweepOutput>,
    pub graph: Option<LogPlotData>,
    pub messages: Vec<Message>,
    pub selected_row: usize,
    pub should_quit: bool,
}

#[derive(Clone, Debug, PartialEq)]
pub enum Action {
    Focus(Pane),
    NextPane,
    PreviousPane,
    MoveLeft,
    MoveRight,
    MoveDown,
    MoveUp,
    GoTop,
    GoBottom,
    HalfPageDown,
    HalfPageUp,
    StartSearch,
    ToggleHelp,
    StartEdit,
    ClearCurrentField,
    ConfirmOrEdit,
    Escape,
    Recalculate,
    StartCommand,
    Quit,
    ZoomIn,
    ZoomOut,
    ResetZoom,
}

impl AppState {
    pub fn new(
        initial_input: Option<Tpp2mInput>,
        energy_min_e_v: f64,
        energy_max_e_v: f64,
    ) -> Self {
        let input = initial_input.unwrap_or(Tpp2mInput {
            electron_energy_e_v: 1000.0,
            density_g_cm3: 2.3296,
            molar_mass_g_mol: 28.0855,
            valence_electrons: 4.0,
            band_gap_e_v: 1.12,
            allow_extrapolate: false,
        });

        let mut state = Self {
            focused_pane: Pane::MaterialInput,
            mode: Mode::Normal,
            material: MaterialForm {
                density_g_cm3: input.density_g_cm3,
                molar_mass_g_mol: input.molar_mass_g_mol,
                valence_electrons: input.valence_electrons,
                band_gap_e_v: input.band_gap_e_v,
            },
            energy: EnergyForm {
                electron_energy_e_v: input.electron_energy_e_v,
                energy_min_e_v,
                energy_max_e_v,
                points: 200,
                spacing: Spacing::Log,
                allow_extrapolate: input.allow_extrapolate,
            },
            result: None,
            sweep: None,
            graph: None,
            messages: Vec::new(),
            selected_row: 0,
            should_quit: false,
        };
        recalculate(&mut state);
        state
    }

    pub fn current_input(&self) -> Tpp2mInput {
        Tpp2mInput {
            electron_energy_e_v: self.energy.electron_energy_e_v,
            density_g_cm3: self.material.density_g_cm3,
            molar_mass_g_mol: self.material.molar_mass_g_mol,
            valence_electrons: self.material.valence_electrons,
            band_gap_e_v: self.material.band_gap_e_v,
            allow_extrapolate: self.energy.allow_extrapolate,
        }
    }

    pub fn current_sweep_input(&self) -> SweepInput {
        SweepInput {
            material: self.current_input(),
            energy_min_e_v: self.energy.energy_min_e_v,
            energy_max_e_v: self.energy.energy_max_e_v,
            points: self.energy.points,
            spacing: self.energy.spacing,
        }
    }
}

pub fn reduce(mut state: AppState, action: Action) -> AppState {
    match action {
        Action::Focus(pane) => {
            if state.mode == Mode::Normal {
                state.focused_pane = pane;
            }
        }
        Action::NextPane => cycle_pane(&mut state, 1),
        Action::PreviousPane => cycle_pane(&mut state, -1),
        Action::MoveLeft => move_horizontal(&mut state, -1),
        Action::MoveRight => move_horizontal(&mut state, 1),
        Action::MoveDown => move_selection(&mut state, 1),
        Action::MoveUp => move_selection(&mut state, -1),
        Action::GoTop => state.selected_row = 0,
        Action::GoBottom => {
            state.selected_row = state
                .sweep
                .as_ref()
                .map(|sweep| sweep.points.len().saturating_sub(1))
                .unwrap_or(0);
        }
        Action::HalfPageDown => move_selection(&mut state, 10),
        Action::HalfPageUp => move_selection(&mut state, -10),
        Action::StartSearch => state.mode = Mode::Search,
        Action::ToggleHelp => {
            state.mode = if state.mode == Mode::Help {
                Mode::Normal
            } else {
                Mode::Help
            };
        }
        Action::StartEdit | Action::ConfirmOrEdit => state.mode = Mode::Editing,
        Action::ClearCurrentField => clear_current_field(&mut state),
        Action::Escape => state.mode = Mode::Normal,
        Action::Recalculate => recalculate(&mut state),
        Action::StartCommand => state.mode = Mode::Command,
        Action::Quit => {
            if state.mode == Mode::Help {
                state.mode = Mode::Normal;
            } else {
                state.should_quit = true;
            }
        }
        Action::ZoomIn => zoom(&mut state, 0.8),
        Action::ZoomOut => zoom(&mut state, 1.25),
        Action::ResetZoom => {
            state.energy.energy_min_e_v = 50.0;
            state.energy.energy_max_e_v = 2000.0;
            recalculate(&mut state);
        }
    }
    state
}

fn cycle_pane(state: &mut AppState, direction: isize) {
    if state.mode != Mode::Normal {
        return;
    }
    let current = Pane::ORDER
        .iter()
        .position(|pane| *pane == state.focused_pane)
        .unwrap_or(0);
    let len = Pane::ORDER.len() as isize;
    let next = (current as isize + direction).rem_euclid(len) as usize;
    state.focused_pane = Pane::ORDER[next];
}

fn move_horizontal(state: &mut AppState, direction: isize) {
    match (state.focused_pane, direction) {
        (Pane::MaterialInput | Pane::EnergySweep | Pane::ResultSeries, 1) => {
            state.focused_pane = Pane::Graph;
        }
        (Pane::Graph, -1) => state.focused_pane = Pane::MaterialInput,
        _ => move_selection(state, direction),
    }
}

fn move_selection(state: &mut AppState, direction: isize) {
    let max = state
        .sweep
        .as_ref()
        .map(|sweep| sweep.points.len().saturating_sub(1))
        .unwrap_or(0);
    if direction >= 0 {
        state.selected_row = state
            .selected_row
            .saturating_add(direction as usize)
            .min(max);
    } else {
        state.selected_row = state.selected_row.saturating_sub(direction.unsigned_abs());
    }
}

fn clear_current_field(state: &mut AppState) {
    match state.focused_pane {
        Pane::MaterialInput => state.material.band_gap_e_v = 0.0,
        Pane::EnergySweep => state.energy.electron_energy_e_v = 50.0,
        _ => {}
    }
    recalculate(state);
}

fn zoom(state: &mut AppState, scale: f64) {
    let center = (state.energy.energy_min_e_v + state.energy.energy_max_e_v) / 2.0;
    let half_width = (state.energy.energy_max_e_v - state.energy.energy_min_e_v) * scale / 2.0;
    state.energy.energy_min_e_v = (center - half_width).max(1.0);
    state.energy.energy_max_e_v = center + half_width;
    recalculate(state);
}

fn recalculate(state: &mut AppState) {
    state.messages.clear();
    match tpp2m_core::calculate(state.current_input()) {
        Ok(result) => {
            for warning in &result.warnings {
                state.messages.push(Message {
                    level: MessageLevel::Warning,
                    text: warning.message.clone(),
                });
            }
            state.result = Some(result);
        }
        Err(error) => {
            state.result = None;
            state.messages.push(Message {
                level: MessageLevel::Error,
                text: error.message,
            });
        }
    }

    match tpp2m_core::sweep(state.current_sweep_input()) {
        Ok(sweep) => state.sweep = Some(sweep),
        Err(error) => {
            state.sweep = None;
            state.messages.push(Message {
                level: MessageLevel::Error,
                text: error.message,
            });
        }
    }

    match tpp2m_core::log_plot_points(state.current_sweep_input()) {
        Ok(graph) => state.graph = Some(graph),
        Err(error) => {
            state.graph = None;
            state.messages.push(Message {
                level: MessageLevel::Error,
                text: error.message,
            });
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn number_focus_actions_move_to_matching_pane() {
        let state = AppState::new(None, 50.0, 2000.0);

        let state = reduce(state, Action::Focus(Pane::HelpLog));

        assert_eq!(state.focused_pane, Pane::HelpLog);
    }

    #[test]
    fn tab_actions_cycle_panes() {
        let state = AppState::new(None, 50.0, 2000.0);

        let state = reduce(state, Action::NextPane);
        let state = reduce(state, Action::PreviousPane);

        assert_eq!(state.focused_pane, Pane::MaterialInput);
    }

    #[test]
    fn vim_navigation_moves_selected_row() {
        let state = AppState::new(None, 50.0, 2000.0);

        let state = reduce(state, Action::MoveDown);
        let state = reduce(state, Action::MoveDown);
        let state = reduce(state, Action::MoveUp);

        assert_eq!(state.selected_row, 1);
    }

    #[test]
    fn gg_and_g_jump_to_bounds() {
        let state = AppState::new(None, 50.0, 2000.0);

        let state = reduce(state, Action::GoBottom);
        let bottom = state.selected_row;
        let state = reduce(state, Action::GoTop);

        assert!(bottom > 0);
        assert_eq!(state.selected_row, 0);
    }

    #[test]
    fn calculation_errors_are_stored_as_messages() {
        let mut state = AppState::new(None, 50.0, 2000.0);
        state.energy.electron_energy_e_v = 5000.0;

        let state = reduce(state, Action::Recalculate);

        assert!(state.result.is_none());
        assert!(state.messages.iter().any(|message| {
            message.level == MessageLevel::Error
                && message.text.contains("electron_energy_e_v must be within")
        }));
    }

    #[test]
    fn edit_search_help_command_and_escape_update_modes() {
        let state = AppState::new(None, 50.0, 2000.0);

        let state = reduce(state, Action::StartEdit);
        assert_eq!(state.mode, Mode::Editing);

        let state = reduce(state, Action::Escape);
        assert_eq!(state.mode, Mode::Normal);

        let state = reduce(state, Action::StartSearch);
        assert_eq!(state.mode, Mode::Search);

        let state = reduce(state, Action::ToggleHelp);
        assert_eq!(state.mode, Mode::Help);

        let state = reduce(state, Action::ToggleHelp);
        assert_eq!(state.mode, Mode::Normal);

        let state = reduce(state, Action::StartCommand);
        assert_eq!(state.mode, Mode::Command);
    }

    #[test]
    fn focus_actions_are_ignored_while_not_in_normal_mode() {
        let state = AppState::new(None, 50.0, 2000.0);
        let state = reduce(state, Action::StartEdit);
        let state = reduce(state, Action::Focus(Pane::Graph));

        assert_eq!(state.focused_pane, Pane::MaterialInput);
    }

    #[test]
    fn quit_closes_help_before_exiting() {
        let state = AppState::new(None, 50.0, 2000.0);
        let state = reduce(state, Action::ToggleHelp);
        let state = reduce(state, Action::Quit);

        assert_eq!(state.mode, Mode::Normal);
        assert!(!state.should_quit);

        let state = reduce(state, Action::Quit);
        assert!(state.should_quit);
    }

    #[test]
    fn graph_zoom_and_reset_recalculate_data() {
        let state = AppState::new(None, 50.0, 2000.0);
        let state = reduce(state, Action::ZoomIn);

        assert!(state.energy.energy_min_e_v > 50.0);
        assert!(state.graph.is_some());

        let zoomed_width = state.energy.energy_max_e_v - state.energy.energy_min_e_v;
        let state = reduce(state, Action::ZoomOut);
        let widened_width = state.energy.energy_max_e_v - state.energy.energy_min_e_v;
        assert!(widened_width > zoomed_width);

        let state = reduce(state, Action::ResetZoom);
        assert_eq!(state.energy.energy_min_e_v, 50.0);
        assert_eq!(state.energy.energy_max_e_v, 2000.0);
    }

    #[test]
    fn clearing_current_field_uses_pane_specific_defaults() {
        let state = AppState::new(None, 50.0, 2000.0);
        let state = reduce(state, Action::ClearCurrentField);
        assert_eq!(state.material.band_gap_e_v, 0.0);

        let state = reduce(state, Action::Focus(Pane::EnergySweep));
        let state = reduce(state, Action::ClearCurrentField);
        assert_eq!(state.energy.electron_energy_e_v, 50.0);
    }
}
