pub mod app;
pub mod keymap;
pub mod render;

use std::io;
use std::time::Duration;

use crossterm::event::{self, Event};
use crossterm::execute;
use crossterm::terminal::{
    EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode,
};
use ratatui::Terminal;
use ratatui::backend::CrosstermBackend;
use tpp2m_core::Tpp2mInput;

pub use app::{Action, AppState, Mode, Pane, reduce};

pub fn run(
    initial_input: Option<Tpp2mInput>,
    energy_min_e_v: f64,
    energy_max_e_v: f64,
) -> Result<(), String> {
    let mut terminal = setup_terminal().map_err(|error| error.to_string())?;
    let result = run_loop(&mut terminal, initial_input, energy_min_e_v, energy_max_e_v);
    let restore_result = restore_terminal(&mut terminal);

    if let Err(error) = restore_result {
        return Err(error.to_string());
    }
    result
}

fn setup_terminal() -> io::Result<Terminal<CrosstermBackend<io::Stdout>>> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    Terminal::new(CrosstermBackend::new(stdout))
}

fn restore_terminal(terminal: &mut Terminal<CrosstermBackend<io::Stdout>>) -> io::Result<()> {
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()
}

fn run_loop(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    initial_input: Option<Tpp2mInput>,
    energy_min_e_v: f64,
    energy_max_e_v: f64,
) -> Result<(), String> {
    let mut state = AppState::new(initial_input, energy_min_e_v, energy_max_e_v);
    let mut key_state = keymap::KeyInputState::default();

    loop {
        terminal
            .draw(|frame| render::render(frame, &state))
            .map_err(|error| error.to_string())?;

        if state.should_quit {
            return Ok(());
        }

        if event::poll(Duration::from_millis(100)).map_err(|error| error.to_string())? {
            let event = event::read().map_err(|error| error.to_string())?;
            if let Event::Key(key_event) = event
                && let Some(action) = keymap::map_key_event(key_event, &mut key_state)
            {
                state = reduce(state, action);
            }
        }
    }
}
