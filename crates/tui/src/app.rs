use tpp2m_core::{LogPlotData, Spacing, SweepInput, SweepOutput, Tpp2mInput, Tpp2mOutput, Warning};

use crate::presets::{MaterialPreset, XRAY_PRESETS, element_presets};

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

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MaterialField {
    Preset,
    Name,
    Density,
    MolarMass,
    ValenceElectrons,
    BandGap,
}

impl MaterialField {
    const ORDER: [Self; 6] = [
        Self::Preset,
        Self::Name,
        Self::Density,
        Self::MolarMass,
        Self::ValenceElectrons,
        Self::BandGap,
    ];
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EnergyField {
    Source,
    ElectronEnergy,
    RangeMode,
    EnergyMin,
    EnergyMax,
    Points,
    Spacing,
    AllowExtrapolate,
}

impl EnergyField {
    const ORDER: [Self; 8] = [
        Self::Source,
        Self::ElectronEnergy,
        Self::RangeMode,
        Self::EnergyMin,
        Self::EnergyMax,
        Self::Points,
        Self::Spacing,
        Self::AllowExtrapolate,
    ];
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EnergySource {
    Custom,
    Xray(usize),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RangeMode {
    Auto,
    Manual,
}

#[derive(Clone, Debug, PartialEq)]
pub struct MaterialForm {
    pub material_name: String,
    pub preset_index: Option<usize>,
    pub density_g_cm3: f64,
    pub molar_mass_g_mol: f64,
    pub valence_electrons: f64,
    pub band_gap_e_v: f64,
}

#[derive(Clone, Debug, PartialEq)]
pub struct EnergyForm {
    pub source: EnergySource,
    pub range_mode: RangeMode,
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
    pub selected_material_field: MaterialField,
    pub selected_energy_field: EnergyField,
    pub edit_buffer: String,
    pub edit_cursor: usize,
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
    StartInsertBefore,
    StartInsertAfter,
    InputChar(char),
    Backspace,
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
                material_name: "Si".to_string(),
                preset_index: preset_index_by_symbol("Si"),
                density_g_cm3: input.density_g_cm3,
                molar_mass_g_mol: input.molar_mass_g_mol,
                valence_electrons: input.valence_electrons,
                band_gap_e_v: input.band_gap_e_v,
            },
            energy: EnergyForm {
                source: EnergySource::Custom,
                range_mode: RangeMode::Manual,
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
            selected_material_field: MaterialField::Preset,
            selected_energy_field: EnergyField::Source,
            edit_buffer: String::new(),
            edit_cursor: 0,
            should_quit: false,
        };
        recalculate(&mut state);
        state
    }

    pub fn current_input(&self) -> Tpp2mInput {
        let allow_extrapolate =
            self.energy.allow_extrapolate || self.energy.electron_energy_e_v > 2000.0;
        Tpp2mInput {
            electron_energy_e_v: self.energy.electron_energy_e_v,
            density_g_cm3: self.material.density_g_cm3,
            molar_mass_g_mol: self.material.molar_mass_g_mol,
            valence_electrons: self.material.valence_electrons,
            band_gap_e_v: self.material.band_gap_e_v,
            allow_extrapolate,
        }
    }

    pub fn current_sweep_input(&self) -> SweepInput {
        let (energy_min_e_v, energy_max_e_v) = self.current_range();
        SweepInput {
            material: self.current_input(),
            energy_min_e_v,
            energy_max_e_v,
            points: self.energy.points,
            spacing: self.energy.spacing,
        }
    }

    pub fn current_range(&self) -> (f64, f64) {
        match self.energy.range_mode {
            RangeMode::Auto => (10.0, self.energy.electron_energy_e_v),
            RangeMode::Manual => (self.energy.energy_min_e_v, self.energy.energy_max_e_v),
        }
    }
}

pub fn reduce(mut state: AppState, action: Action) -> AppState {
    match action {
        Action::Focus(pane) => {
            if state.mode == Mode::Normal {
                state.focused_pane = pane;
                reset_cursor_for_current_field(&mut state);
            }
        }
        Action::NextPane => cycle_pane_or_commit_then_cycle(&mut state, 1),
        Action::PreviousPane => cycle_pane_or_commit_then_cycle(&mut state, -1),
        Action::MoveLeft => move_horizontal(&mut state, -1),
        Action::MoveRight => move_horizontal(&mut state, 1),
        Action::MoveDown => move_down_or_field(&mut state, 1),
        Action::MoveUp => move_down_or_field(&mut state, -1),
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
        Action::StartEdit | Action::StartInsertBefore => start_edit(&mut state, InsertMode::Before),
        Action::StartInsertAfter => start_edit(&mut state, InsertMode::After),
        Action::InputChar(ch) => {
            if state.mode == Mode::Editing {
                insert_char(&mut state, ch);
            }
        }
        Action::Backspace => {
            if state.mode == Mode::Editing {
                backspace(&mut state);
            }
        }
        Action::ConfirmOrEdit => confirm_or_edit(&mut state),
        Action::ClearCurrentField => clear_current_field(&mut state),
        Action::Escape => {
            state.mode = Mode::Normal;
            state.edit_buffer.clear();
            reset_cursor_for_current_field(&mut state);
        }
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
            state.energy.range_mode = RangeMode::Auto;
            apply_auto_range(&mut state);
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
    reset_cursor_for_current_field(state);
}

fn cycle_pane_or_commit_then_cycle(state: &mut AppState, direction: isize) {
    if state.mode == Mode::Editing {
        commit_edit(state);
    }
    cycle_pane(state, direction);
}

fn move_horizontal(state: &mut AppState, direction: isize) {
    if state.mode == Mode::Normal {
        match state.focused_pane {
            Pane::MaterialInput if state.selected_material_field == MaterialField::Preset => {
                cycle_material_preset(state, direction);
                return;
            }
            Pane::MaterialInput if current_field_is_editable(state) => {
                move_edit_cursor(state, direction);
                return;
            }
            Pane::EnergySweep => match state.selected_energy_field {
                EnergyField::Source => {
                    cycle_energy_source(state, direction);
                    return;
                }
                EnergyField::RangeMode => {
                    cycle_range_mode(state);
                    return;
                }
                EnergyField::Spacing => {
                    cycle_spacing(state);
                    return;
                }
                EnergyField::AllowExtrapolate => {
                    state.energy.allow_extrapolate = !state.energy.allow_extrapolate;
                    recalculate(state);
                    return;
                }
                _ if current_field_is_editable(state) => {
                    move_edit_cursor(state, direction);
                    return;
                }
                _ => {}
            },
            _ => {}
        }
    }

    move_selection(state, direction);
}

fn move_down_or_field(state: &mut AppState, direction: isize) {
    match state.focused_pane {
        Pane::MaterialInput if state.mode == Mode::Normal => cycle_material_field(state, direction),
        Pane::EnergySweep if state.mode == Mode::Normal => cycle_energy_field(state, direction),
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
        Pane::MaterialInput => match state.selected_material_field {
            MaterialField::Name => {
                state.material.material_name = "Custom".to_string();
                state.material.preset_index = None;
            }
            MaterialField::BandGap => mark_custom_from_preset(state, |state| {
                state.material.band_gap_e_v = 0.0;
            }),
            _ => {}
        },
        Pane::EnergySweep => match state.selected_energy_field {
            EnergyField::ElectronEnergy => {
                state.energy.source = EnergySource::Custom;
                state.energy.electron_energy_e_v = 50.0;
                apply_auto_range(state);
            }
            EnergyField::EnergyMin | EnergyField::EnergyMax => {
                state.energy.range_mode = RangeMode::Manual;
                state.energy.energy_min_e_v = 10.0;
                state.energy.energy_max_e_v = state.energy.electron_energy_e_v;
            }
            _ => {}
        },
        _ => {}
    }
    recalculate(state);
}

fn zoom(state: &mut AppState, scale: f64) {
    state.energy.range_mode = RangeMode::Manual;
    let center = (state.energy.energy_min_e_v + state.energy.energy_max_e_v) / 2.0;
    let half_width = (state.energy.energy_max_e_v - state.energy.energy_min_e_v) * scale / 2.0;
    state.energy.energy_min_e_v = (center - half_width).max(1.0);
    state.energy.energy_max_e_v = center + half_width;
    recalculate(state);
}

fn cycle_material_field(state: &mut AppState, direction: isize) {
    let current = MaterialField::ORDER
        .iter()
        .position(|field| *field == state.selected_material_field)
        .unwrap_or(0);
    let len = MaterialField::ORDER.len() as isize;
    let next = (current as isize + direction).rem_euclid(len) as usize;
    state.selected_material_field = MaterialField::ORDER[next];
    reset_cursor_for_current_field(state);
}

fn cycle_energy_field(state: &mut AppState, direction: isize) {
    let current = EnergyField::ORDER
        .iter()
        .position(|field| *field == state.selected_energy_field)
        .unwrap_or(0);
    let len = EnergyField::ORDER.len() as isize;
    let next = (current as isize + direction).rem_euclid(len) as usize;
    state.selected_energy_field = EnergyField::ORDER[next];
    reset_cursor_for_current_field(state);
}

fn cycle_material_preset(state: &mut AppState, direction: isize) {
    let presets = element_presets();
    let current = state.material.preset_index.unwrap_or_else(|| {
        presets
            .iter()
            .position(|preset| preset.material_name == "Si")
            .unwrap_or(0)
    });
    let len = presets.len() as isize;
    let next = (current as isize + direction).rem_euclid(len) as usize;
    apply_material_preset(state, presets[next], next);
}

fn apply_material_preset(state: &mut AppState, preset: MaterialPreset, index: usize) {
    state.material.material_name = preset.material_name.to_string();
    state.material.preset_index = Some(index);
    state.material.density_g_cm3 = preset.density_g_cm3;
    state.material.molar_mass_g_mol = preset.molar_mass_g_mol;
    state.material.valence_electrons = preset.valence_electrons;
    state.material.band_gap_e_v = preset.band_gap_e_v;
    recalculate(state);
}

fn cycle_energy_source(state: &mut AppState, direction: isize) {
    let current = match state.energy.source {
        EnergySource::Custom => 0,
        EnergySource::Xray(index) => index + 1,
    };
    let len = XRAY_PRESETS.len() as isize + 1;
    let next = (current as isize + direction).rem_euclid(len) as usize;
    state.energy.source = if next == 0 {
        EnergySource::Custom
    } else {
        let index = next - 1;
        state.energy.electron_energy_e_v = XRAY_PRESETS[index].electron_energy_e_v;
        EnergySource::Xray(index)
    };
    apply_auto_range(state);
    recalculate(state);
}

fn cycle_range_mode(state: &mut AppState) {
    state.energy.range_mode = match state.energy.range_mode {
        RangeMode::Auto => RangeMode::Manual,
        RangeMode::Manual => RangeMode::Auto,
    };
    apply_auto_range(state);
    recalculate(state);
}

fn cycle_spacing(state: &mut AppState) {
    state.energy.spacing = match state.energy.spacing {
        Spacing::Log => Spacing::Linear,
        Spacing::Linear => Spacing::Log,
    };
    recalculate(state);
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum InsertMode {
    Before,
    After,
}

fn start_edit(state: &mut AppState, insert_mode: InsertMode) {
    if !current_field_is_editable(state) {
        return;
    }
    state.mode = Mode::Editing;
    state.edit_buffer = current_field_text(state);
    clamp_edit_cursor(state);
    if insert_mode == InsertMode::After {
        state.edit_cursor = state
            .edit_cursor
            .saturating_add(1)
            .min(state.edit_buffer.chars().count());
    }
}

fn confirm_or_edit(state: &mut AppState) {
    if state.mode == Mode::Editing {
        commit_edit(state);
    } else {
        match state.focused_pane {
            Pane::MaterialInput if state.selected_material_field == MaterialField::Preset => {
                cycle_material_preset(state, 1);
            }
            Pane::EnergySweep => match state.selected_energy_field {
                EnergyField::Source => cycle_energy_source(state, 1),
                EnergyField::RangeMode => cycle_range_mode(state),
                EnergyField::Spacing => cycle_spacing(state),
                EnergyField::AllowExtrapolate => {
                    state.energy.allow_extrapolate = !state.energy.allow_extrapolate;
                    recalculate(state);
                }
                _ => start_edit(state, InsertMode::Before),
            },
            _ => start_edit(state, InsertMode::Before),
        }
    }
}

fn commit_edit(state: &mut AppState) {
    let buffer = state.edit_buffer.trim().to_string();
    let mut accepted = true;
    match state.focused_pane {
        Pane::MaterialInput => match state.selected_material_field {
            MaterialField::Name => {
                state.material.material_name = if buffer.is_empty() {
                    "Custom".to_string()
                } else {
                    buffer
                };
                state.material.preset_index = None;
            }
            MaterialField::Density => {
                accepted = apply_f64(&buffer, |value| {
                    mark_custom_from_preset(state, |state| state.material.density_g_cm3 = value);
                });
            }
            MaterialField::MolarMass => {
                accepted = apply_f64(&buffer, |value| {
                    mark_custom_from_preset(state, |state| state.material.molar_mass_g_mol = value);
                });
            }
            MaterialField::ValenceElectrons => {
                accepted = apply_f64(&buffer, |value| {
                    mark_custom_from_preset(state, |state| {
                        state.material.valence_electrons = value
                    });
                });
            }
            MaterialField::BandGap => {
                accepted = apply_f64(&buffer, |value| {
                    mark_custom_from_preset(state, |state| state.material.band_gap_e_v = value);
                });
            }
            MaterialField::Preset => {}
        },
        Pane::EnergySweep => match state.selected_energy_field {
            EnergyField::ElectronEnergy => {
                accepted = apply_f64(&buffer, |value| {
                    state.energy.source = EnergySource::Custom;
                    state.energy.electron_energy_e_v = value;
                    apply_auto_range(state);
                });
            }
            EnergyField::EnergyMin => {
                accepted = apply_f64(&buffer, |value| {
                    state.energy.range_mode = RangeMode::Manual;
                    state.energy.energy_min_e_v = value;
                });
            }
            EnergyField::EnergyMax => {
                accepted = apply_f64(&buffer, |value| {
                    state.energy.range_mode = RangeMode::Manual;
                    state.energy.energy_max_e_v = value;
                });
            }
            EnergyField::Points => {
                accepted = buffer.parse::<usize>().is_ok_and(|value| {
                    state.energy.points = value;
                    true
                });
            }
            _ => {}
        },
        _ => {}
    }

    state.mode = Mode::Normal;
    state.edit_buffer.clear();
    reset_cursor_for_current_field(state);
    if accepted {
        recalculate(state);
    } else {
        state.messages.push(Message {
            level: MessageLevel::Error,
            text: "invalid field value".to_string(),
        });
    }
}

fn current_field_is_editable(state: &AppState) -> bool {
    match state.focused_pane {
        Pane::MaterialInput => !matches!(state.selected_material_field, MaterialField::Preset),
        Pane::EnergySweep => matches!(
            state.selected_energy_field,
            EnergyField::ElectronEnergy
                | EnergyField::EnergyMin
                | EnergyField::EnergyMax
                | EnergyField::Points
        ),
        _ => false,
    }
}

fn current_field_text(state: &AppState) -> String {
    match state.focused_pane {
        Pane::MaterialInput => match state.selected_material_field {
            MaterialField::Preset => current_material_preset_label(state).to_string(),
            MaterialField::Name => state.material.material_name.clone(),
            MaterialField::Density => format_float(state.material.density_g_cm3),
            MaterialField::MolarMass => format_float(state.material.molar_mass_g_mol),
            MaterialField::ValenceElectrons => format_float(state.material.valence_electrons),
            MaterialField::BandGap => format_float(state.material.band_gap_e_v),
        },
        Pane::EnergySweep => match state.selected_energy_field {
            EnergyField::Source => current_energy_source_label(state).to_string(),
            EnergyField::ElectronEnergy => format_float(state.energy.electron_energy_e_v),
            EnergyField::RangeMode => format!("{:?}", state.energy.range_mode),
            EnergyField::EnergyMin => format_float(state.energy.energy_min_e_v),
            EnergyField::EnergyMax => format_float(state.energy.energy_max_e_v),
            EnergyField::Points => state.energy.points.to_string(),
            EnergyField::Spacing => format!("{:?}", state.energy.spacing),
            EnergyField::AllowExtrapolate => state.energy.allow_extrapolate.to_string(),
        },
        _ => String::new(),
    }
}

fn insert_char(state: &mut AppState, ch: char) {
    let index = byte_index_at_char(&state.edit_buffer, state.edit_cursor);
    state.edit_buffer.insert(index, ch);
    state.edit_cursor += 1;
}

fn backspace(state: &mut AppState) {
    if state.edit_cursor == 0 {
        return;
    }
    let start = byte_index_at_char(&state.edit_buffer, state.edit_cursor - 1);
    let end = byte_index_at_char(&state.edit_buffer, state.edit_cursor);
    state.edit_buffer.replace_range(start..end, "");
    state.edit_cursor -= 1;
}

fn move_edit_cursor(state: &mut AppState, direction: isize) {
    let len = active_field_text(state).chars().count();
    if direction < 0 {
        state.edit_cursor = state.edit_cursor.saturating_sub(direction.unsigned_abs());
    } else {
        state.edit_cursor = state
            .edit_cursor
            .saturating_add(direction as usize)
            .min(len);
    }
}

fn reset_cursor_for_current_field(state: &mut AppState) {
    state.edit_cursor = if current_field_is_editable(state) {
        active_field_text(state).chars().count()
    } else {
        0
    };
}

fn clamp_edit_cursor(state: &mut AppState) {
    state.edit_cursor = state
        .edit_cursor
        .min(active_field_text(state).chars().count());
}

fn active_field_text(state: &AppState) -> String {
    if state.mode == Mode::Editing {
        state.edit_buffer.clone()
    } else {
        current_field_text(state)
    }
}

fn byte_index_at_char(text: &str, char_index: usize) -> usize {
    text.char_indices()
        .nth(char_index)
        .map(|(index, _)| index)
        .unwrap_or(text.len())
}

fn current_material_preset_label(state: &AppState) -> &str {
    state
        .material
        .preset_index
        .and_then(|index| {
            element_presets()
                .get(index)
                .map(|preset| preset.material_name)
        })
        .unwrap_or("Custom")
}

fn current_energy_source_label(state: &AppState) -> &str {
    match state.energy.source {
        EnergySource::Custom => "Custom",
        EnergySource::Xray(index) => XRAY_PRESETS[index].label,
    }
}

fn apply_auto_range(state: &mut AppState) {
    if state.energy.range_mode == RangeMode::Auto {
        state.energy.energy_min_e_v = 10.0;
        state.energy.energy_max_e_v = state.energy.electron_energy_e_v;
    }
}

fn mark_custom_from_preset(state: &mut AppState, edit: impl FnOnce(&mut AppState)) {
    let base = state.material.material_name.clone();
    edit(state);
    if state.material.preset_index.is_some() {
        state.material.material_name = format!("Custom from {base}");
        state.material.preset_index = None;
    }
}

fn parse_f64(text: &str) -> Option<f64> {
    text.parse::<f64>().ok().filter(|value| value.is_finite())
}

fn apply_f64(text: &str, apply: impl FnOnce(f64)) -> bool {
    if let Some(value) = parse_f64(text) {
        apply(value);
        true
    } else {
        false
    }
}

fn format_float(value: f64) -> String {
    format!("{value:.6}")
        .trim_end_matches('0')
        .trim_end_matches('.')
        .to_string()
}

fn preset_index_by_symbol(symbol: &str) -> Option<usize> {
    element_presets()
        .iter()
        .position(|preset| preset.material_name == symbol)
}

fn recalculate(state: &mut AppState) {
    state.messages.clear();
    match tpp2m_core::calculate(state.current_input()) {
        Ok(result) => {
            append_warning_messages(state, &result.warnings, "calculation");
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
        Ok(sweep) => {
            append_warning_messages(state, &sweep.warnings, "sweep");
            state.sweep = Some(sweep);
        }
        Err(error) => {
            state.sweep = None;
            state.messages.push(Message {
                level: MessageLevel::Error,
                text: error.message,
            });
        }
    }

    match tpp2m_core::log_plot_points(state.current_sweep_input()) {
        Ok(graph) => {
            append_warning_messages(state, &graph.warnings, "graph");
            state.graph = Some(graph);
        }
        Err(error) => {
            state.graph = None;
            state.messages.push(Message {
                level: MessageLevel::Error,
                text: error.message,
            });
        }
    }
}

fn append_warning_messages(state: &mut AppState, warnings: &[Warning], context: &str) {
    if warnings.is_empty() {
        return;
    }
    let first = &warnings[0].message;
    let text = if warnings.len() == 1 {
        format!("{context}: {first}")
    } else {
        format!("{context}: {first} ({} points)", warnings.len())
    };
    if !state.messages.iter().any(|message| message.text == text) {
        state.messages.push(Message {
            level: MessageLevel::Warning,
            text,
        });
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
        let state = reduce(state, Action::Focus(Pane::ResultSeries));

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
        state.material.density_g_cm3 = -1.0;

        let state = reduce(state, Action::Recalculate);

        assert!(state.result.is_none());
        assert!(state.messages.iter().any(|message| {
            message.level == MessageLevel::Error && message.text.contains("density_g_cm3")
        }));
    }

    #[test]
    fn extrapolated_sweep_warnings_are_visible_in_messages() {
        let mut state = AppState::new(None, 10.0, 1000.0);
        state.energy.allow_extrapolate = true;

        let state = reduce(state, Action::Recalculate);

        assert!(state.sweep.is_some());
        assert!(state.messages.iter().any(|message| {
            message.level == MessageLevel::Warning
                && message
                    .text
                    .contains("outside the recommended 50..=2000 eV range")
        }));
    }

    #[test]
    fn edit_search_help_command_and_escape_update_modes() {
        let state = AppState::new(None, 50.0, 2000.0);
        let state = reduce(state, Action::MoveDown);

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
        let state = reduce(state, Action::MoveDown);
        let state = reduce(state, Action::StartInsertBefore);
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
        assert_eq!(state.energy.range_mode, RangeMode::Auto);
        assert_eq!(state.energy.energy_min_e_v, 10.0);
        assert_eq!(
            state.energy.energy_max_e_v,
            state.energy.electron_energy_e_v
        );
    }

    #[test]
    fn clearing_current_field_uses_pane_specific_defaults() {
        let state = AppState::new(None, 50.0, 2000.0);
        let state = reduce(state, Action::Focus(Pane::MaterialInput));
        let state = reduce(state, Action::MoveUp);
        let state = reduce(state, Action::ClearCurrentField);
        assert_eq!(state.material.band_gap_e_v, 0.0);

        let state = reduce(state, Action::Focus(Pane::EnergySweep));
        let state = reduce(state, Action::MoveDown);
        let state = reduce(state, Action::ClearCurrentField);
        assert_eq!(state.energy.electron_energy_e_v, 50.0);
    }

    #[test]
    fn tab_always_moves_between_panes() {
        let state = AppState::new(None, 50.0, 2000.0);

        let state = reduce(state, Action::NextPane);

        assert_eq!(state.focused_pane, Pane::EnergySweep);
        assert_eq!(state.selected_material_field, MaterialField::Preset);
    }

    #[test]
    fn jk_moves_between_fields_inside_input_panes() {
        let state = AppState::new(None, 50.0, 2000.0);

        let state = reduce(state, Action::MoveDown);
        let state = reduce(state, Action::MoveUp);

        assert_eq!(state.focused_pane, Pane::MaterialInput);
        assert_eq!(state.selected_material_field, MaterialField::Preset);
    }

    #[test]
    fn inline_edit_updates_material_value_and_marks_custom_from_preset() {
        let state = AppState::new(None, 50.0, 2000.0);
        let state = reduce(state, Action::MoveDown);
        let state = reduce(state, Action::MoveDown);
        let state = reduce(state, Action::StartInsertAfter);
        let state = reduce(state, Action::InputChar('9'));
        let state = reduce(state, Action::ConfirmOrEdit);

        assert_eq!(state.mode, Mode::Normal);
        assert_eq!(state.material.material_name, "Custom from Si");
        assert!(state.material.preset_index.is_none());
        assert_eq!(state.material.density_g_cm3, 2.32969);
    }

    #[test]
    fn normal_hl_moves_edit_cursor_on_editable_fields() {
        let state = AppState::new(None, 50.0, 2000.0);
        let state = reduce(state, Action::MoveDown);
        let end = state.edit_cursor;

        let state = reduce(state, Action::MoveLeft);
        let state = reduce(state, Action::StartInsertBefore);
        let state = reduce(state, Action::InputChar('X'));
        let state = reduce(state, Action::ConfirmOrEdit);

        assert_eq!(end, 2);
        assert_eq!(state.material.material_name, "SXi");
    }

    #[test]
    fn a_enters_insert_mode_after_current_cursor_position() {
        let state = AppState::new(None, 50.0, 2000.0);
        let state = reduce(state, Action::MoveDown);
        let state = reduce(state, Action::MoveLeft);
        let state = reduce(state, Action::StartInsertAfter);
        let state = reduce(state, Action::InputChar('X'));
        let state = reduce(state, Action::ConfirmOrEdit);

        assert_eq!(state.material.material_name, "SiX");
    }

    #[test]
    fn tab_commits_edit_and_moves_pane() {
        let state = AppState::new(None, 50.0, 2000.0);
        let state = reduce(state, Action::MoveDown);
        let state = reduce(state, Action::StartInsertAfter);
        let state = reduce(state, Action::InputChar('X'));
        let state = reduce(state, Action::NextPane);

        assert_eq!(state.mode, Mode::Normal);
        assert_eq!(state.focused_pane, Pane::EnergySweep);
        assert_eq!(state.material.material_name, "SiX");
    }

    #[test]
    fn xray_preset_sets_electron_energy_and_auto_range() {
        let state = AppState::new(None, 50.0, 2000.0);
        let state = reduce(state, Action::Focus(Pane::EnergySweep));
        let state = reduce(state, Action::MoveRight);
        let state = reduce(state, Action::MoveRight);

        assert_eq!(state.energy.source, EnergySource::Xray(1));
        assert_eq!(state.energy.electron_energy_e_v, 1253.6);

        let state = reduce(state, Action::MoveDown);
        let state = reduce(state, Action::MoveDown);
        let state = reduce(state, Action::MoveRight);
        assert_eq!(state.energy.range_mode, RangeMode::Auto);
        assert_eq!(state.energy.energy_min_e_v, 10.0);
        assert_eq!(state.energy.energy_max_e_v, 1253.6);
    }

    #[test]
    fn manual_range_edit_switches_range_mode() {
        let state = AppState::new(None, 50.0, 2000.0);
        let state = reduce(state, Action::Focus(Pane::EnergySweep));
        let state = reduce(state, Action::MoveDown);
        let state = reduce(state, Action::MoveDown);
        let state = reduce(state, Action::MoveRight);
        let state = reduce(state, Action::MoveDown);
        let state = reduce(state, Action::StartInsertAfter);
        let state = reduce(state, Action::InputChar('0'));
        let state = reduce(state, Action::ConfirmOrEdit);

        assert_eq!(state.energy.range_mode, RangeMode::Manual);
        assert_eq!(state.energy.energy_min_e_v, 100.0);
    }
}
