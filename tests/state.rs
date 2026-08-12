mod common;

use common::test_state;
use crossterm::event::KeyCode;
use flat_hypercube::state::AppMode;

#[test]
fn rev_unwind_updates_clicked_positions() {
    let mut state = test_state();
    let clicked = vec![2, 2, 0];
    state.clicked.push(clicked.clone());

    state.rev_start();
    assert!(state.perform_turn(0, 1, 2).is_some());
    assert_ne!(state.clicked[0], clicked);
    state.rev_stop();

    state.rev_unwind();

    assert_eq!(state.clicked[0], clicked);
    assert!(state.puzzle.is_solved());
    assert_eq!(state.message.as_deref(), Some("solved!"));
}

#[test]
fn rev_commutator_updates_clicked_positions() {
    let mut state = test_state();
    let clicked = vec![2, 2, 0];
    state.clicked.push(clicked.clone());

    state.rev_start();
    assert!(state.perform_turn(0, 1, 2).is_some());
    state.rev_stop();
    assert!(state.perform_turn(0, 1, 2).is_some());
    assert_ne!(state.clicked[0], clicked);

    state.rev_commutator();

    assert_eq!(state.clicked[0], clicked);
    assert!(state.puzzle.is_solved());
    assert_eq!(state.message.as_deref(), Some("solved!"));
}

#[test]
fn slash_help_writes_output_and_exits_command_mode() {
    let mut state = test_state();

    state.process_key('/');
    assert_eq!(state.mode, AppMode::Command);
    for ch in "help".chars() {
        state.process_command_key(KeyCode::Char(ch));
    }
    state.process_command_key(KeyCode::Enter);

    assert_eq!(state.mode, AppMode::Turn);
    assert_eq!(state.command.history, vec!["help"]);
    assert_eq!(state.output.hint.as_deref(), Some("commands:"));
    assert!(state.output.lines.iter().any(|line| line == "commands:"));
    assert!(
        state
            .output
            .lines
            .iter()
            .any(|line| line.starts_with("/status"))
    );
}

#[test]
fn slash_clear_clears_output_panel() {
    let mut state = test_state();

    state.process_key('/');
    for ch in "help".chars() {
        state.process_command_key(KeyCode::Char(ch));
    }
    state.process_command_key(KeyCode::Enter);
    assert!(!state.output.lines.is_empty());

    state.process_key('/');
    for ch in "clear".chars() {
        state.process_command_key(KeyCode::Char(ch));
    }
    state.process_command_key(KeyCode::Enter);

    assert!(state.output.lines.is_empty());
}

#[test]
fn panel_toggle_keeps_output_lines() {
    let mut state = test_state();

    state.process_key('/');
    for ch in "help".chars() {
        state.process_command_key(KeyCode::Char(ch));
    }
    state.process_command_key(KeyCode::Enter);
    let output_after_help = state.output.lines.clone();

    state.process_key('/');
    for ch in "panel".chars() {
        state.process_command_key(KeyCode::Char(ch));
    }
    state.process_command_key(KeyCode::Enter);

    assert!(state.output.open);
    assert!(
        output_after_help
            .iter()
            .all(|line| state.output.lines.contains(line))
    );
    assert_eq!(state.message.as_deref(), Some("output panel shown"));

    state.process_key('/');
    for ch in "panel".chars() {
        state.process_command_key(KeyCode::Char(ch));
    }
    state.process_command_key(KeyCode::Enter);

    assert!(!state.output.open);
    assert!(
        output_after_help
            .iter()
            .all(|line| state.output.lines.contains(line))
    );
    assert_eq!(state.message.as_deref(), Some("output panel hidden"));
}

#[test]
fn slash_does_not_open_hidden_output_panel() {
    let mut state = test_state();

    state.process_key('/');
    for ch in "help".chars() {
        state.process_command_key(KeyCode::Char(ch));
    }
    state.process_command_key(KeyCode::Enter);

    state.process_key('/');
    for ch in "panel".chars() {
        state.process_command_key(KeyCode::Char(ch));
    }
    state.process_command_key(KeyCode::Enter);
    assert!(state.output.open);

    state.process_key('/');
    assert!(matches!(state.mode, AppMode::Command));
    assert!(state.output.open);
    state.process_command_key(KeyCode::Esc);

    state.process_key('/');
    for ch in "panel".chars() {
        state.process_command_key(KeyCode::Char(ch));
    }
    state.process_command_key(KeyCode::Enter);
    assert!(!state.output.open);

    state.process_key('/');
    assert!(!state.output.open);
}

#[test]
fn question_mark_toggles_output_panel() {
    let mut state = test_state();

    state.process_key('/');
    for ch in "help".chars() {
        state.process_command_key(KeyCode::Char(ch));
    }
    state.process_command_key(KeyCode::Enter);
    assert!(!state.output.open);

    state.process_key('?');
    assert!(state.output.open);
    assert!(state.output.hint.is_none());

    state.process_key('?');
    assert!(!state.output.open);
}

#[test]
fn command_output_sets_status_hint_when_panel_is_closed() {
    let mut state = test_state();

    state.process_key('/');
    for ch in "status".chars() {
        state.process_command_key(KeyCode::Char(ch));
    }
    state.process_command_key(KeyCode::Enter);

    assert!(!state.output.open);
    assert_eq!(state.output.hint.as_deref(), Some("puzzle: 3^3"));
}

#[test]
fn command_history_up_recalls_last_command() {
    let mut state = test_state();

    state.process_key('/');
    for ch in "status".chars() {
        state.process_command_key(KeyCode::Char(ch));
    }
    state.process_command_key(KeyCode::Enter);

    state.process_key('/');
    state.process_command_key(KeyCode::Up);

    assert_eq!(state.command.buffer, "status");
}

#[test]
fn slash_reset_reuses_reset_behavior() {
    let mut state = test_state();
    assert!(state.perform_turn(0, 1, 2).is_some());
    assert!(!state.undo_history.is_empty());

    state.process_key('/');
    for ch in "reset".chars() {
        state.process_command_key(KeyCode::Char(ch));
    }
    state.process_command_key(KeyCode::Enter);

    assert!(state.puzzle.is_solved());
    assert!(state.undo_history.is_empty());
    assert!(state.redo_history.is_empty());
    assert!(state.rev_stack.is_empty());
    assert!(state.output.lines.iter().any(|line| line == "puzzle reset"));
}
