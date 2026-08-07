mod common;

use common::test_state;

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
