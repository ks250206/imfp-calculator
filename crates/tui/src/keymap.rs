use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use crate::app::{Action, Pane};

#[derive(Default)]
pub struct KeyInputState {
    pending_g: bool,
}

pub fn map_key_event(event: KeyEvent, state: &mut KeyInputState) -> Option<Action> {
    if state.pending_g {
        state.pending_g = false;
        if matches!(event.code, KeyCode::Char('g')) {
            return Some(Action::GoTop);
        }
    }

    match event.code {
        KeyCode::Char('1') => Some(Action::Focus(Pane::MaterialInput)),
        KeyCode::Char('2') => Some(Action::Focus(Pane::EnergySweep)),
        KeyCode::Char('3') => Some(Action::Focus(Pane::Graph)),
        KeyCode::Char('4') => Some(Action::Focus(Pane::ResultSeries)),
        KeyCode::Char('5') => Some(Action::Focus(Pane::HelpLog)),
        KeyCode::Tab => Some(Action::NextPane),
        KeyCode::BackTab => Some(Action::PreviousPane),
        KeyCode::Char('h') if event.modifiers.contains(KeyModifiers::CONTROL) => {
            Some(Action::MoveLeft)
        }
        KeyCode::Char('l') if event.modifiers.contains(KeyModifiers::CONTROL) => {
            Some(Action::MoveRight)
        }
        KeyCode::Char('h') => Some(Action::MoveLeft),
        KeyCode::Char('j') => Some(Action::MoveDown),
        KeyCode::Char('k') => Some(Action::MoveUp),
        KeyCode::Char('l') => Some(Action::MoveRight),
        KeyCode::Char('g') => {
            state.pending_g = true;
            None
        }
        KeyCode::Char('G') => Some(Action::GoBottom),
        KeyCode::Char('d') if event.modifiers.contains(KeyModifiers::CONTROL) => {
            Some(Action::HalfPageDown)
        }
        KeyCode::Char('u') if event.modifiers.contains(KeyModifiers::CONTROL) => {
            Some(Action::HalfPageUp)
        }
        KeyCode::Char('/') => Some(Action::StartSearch),
        KeyCode::Char('?') => Some(Action::ToggleHelp),
        KeyCode::Char('q') => Some(Action::Quit),
        KeyCode::Esc => Some(Action::Escape),
        KeyCode::Enter => Some(Action::ConfirmOrEdit),
        KeyCode::Char('i') | KeyCode::Char('a') => Some(Action::StartEdit),
        KeyCode::Char('x') => Some(Action::ClearCurrentField),
        KeyCode::Char('r') => Some(Action::Recalculate),
        KeyCode::Char(':') => Some(Action::StartCommand),
        KeyCode::Char('+') => Some(Action::ZoomIn),
        KeyCode::Char('-') => Some(Action::ZoomOut),
        KeyCode::Char('0') => Some(Action::ResetZoom),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn key(code: KeyCode) -> KeyEvent {
        KeyEvent::new(code, KeyModifiers::NONE)
    }

    #[test]
    fn maps_number_keys_to_focus_actions() {
        let mut state = KeyInputState::default();

        let action = map_key_event(key(KeyCode::Char('4')), &mut state);

        assert_eq!(action, Some(Action::Focus(Pane::ResultSeries)));
    }

    #[test]
    fn maps_gg_sequence_to_go_top() {
        let mut state = KeyInputState::default();

        let first = map_key_event(key(KeyCode::Char('g')), &mut state);
        let second = map_key_event(key(KeyCode::Char('g')), &mut state);

        assert_eq!(first, None);
        assert_eq!(second, Some(Action::GoTop));
    }

    #[test]
    fn maps_ctrl_d_and_ctrl_u() {
        let mut state = KeyInputState::default();

        let down = map_key_event(
            KeyEvent::new(KeyCode::Char('d'), KeyModifiers::CONTROL),
            &mut state,
        );
        let up = map_key_event(
            KeyEvent::new(KeyCode::Char('u'), KeyModifiers::CONTROL),
            &mut state,
        );

        assert_eq!(down, Some(Action::HalfPageDown));
        assert_eq!(up, Some(Action::HalfPageUp));
    }

    #[test]
    fn maps_common_vim_and_mode_keys() {
        let mut state = KeyInputState::default();

        let cases = [
            (KeyCode::Char('h'), Some(Action::MoveLeft)),
            (KeyCode::Char('j'), Some(Action::MoveDown)),
            (KeyCode::Char('k'), Some(Action::MoveUp)),
            (KeyCode::Char('l'), Some(Action::MoveRight)),
            (KeyCode::Char('G'), Some(Action::GoBottom)),
            (KeyCode::Char('/'), Some(Action::StartSearch)),
            (KeyCode::Char('?'), Some(Action::ToggleHelp)),
            (KeyCode::Char('q'), Some(Action::Quit)),
            (KeyCode::Enter, Some(Action::ConfirmOrEdit)),
            (KeyCode::Char('i'), Some(Action::StartEdit)),
            (KeyCode::Char('a'), Some(Action::StartEdit)),
            (KeyCode::Char('x'), Some(Action::ClearCurrentField)),
            (KeyCode::Char('r'), Some(Action::Recalculate)),
            (KeyCode::Char(':'), Some(Action::StartCommand)),
            (KeyCode::Char('+'), Some(Action::ZoomIn)),
            (KeyCode::Char('-'), Some(Action::ZoomOut)),
            (KeyCode::Char('0'), Some(Action::ResetZoom)),
        ];

        for (code, expected) in cases {
            assert_eq!(map_key_event(key(code), &mut state), expected);
        }
    }
}
