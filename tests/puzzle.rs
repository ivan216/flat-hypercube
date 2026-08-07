use flat_hypercube::puzzle::{Puzzle, PuzzleTurn, SideTurn, Turn};

fn sample_turns() -> Vec<Turn> {
    vec![
        Turn::Side(SideTurn {
            side: 0,
            layer_min: 2,
            layer_max: 2,
            from: 1,
            to: 2,
        }),
        Turn::Side(SideTurn {
            side: !0,
            layer_min: -2,
            layer_max: -2,
            from: 1,
            to: 3,
        }),
        Turn::Puzzle(PuzzleTurn { from: 0, to: 2 }),
        Turn::Side(SideTurn {
            side: 3,
            layer_min: 0,
            layer_max: 0,
            from: !0,
            to: 1,
        }),
        Turn::Puzzle(PuzzleTurn { from: 1, to: 3 }),
    ]
}

fn labeled_puzzle() -> Puzzle {
    let mut puzzle = Puzzle::make_solved(3, 4);
    let positions: Vec<Vec<i16>> = puzzle.stickers.keys().cloned().collect();
    for (idx, pos) in positions.into_iter().enumerate() {
        puzzle.stickers.insert(pos, idx as i16);
    }
    puzzle
}

#[test]
fn batch_turns_match_sequential_turns() {
    let turns = sample_turns();
    let mut sequential = labeled_puzzle();
    let mut batch = sequential.clone();

    for turn in &turns {
        assert!(sequential.turn(turn.clone()).is_some());
    }
    assert!(batch.apply_turns_batch(&turns).is_some());

    assert_eq!(sequential.stickers, batch.stickers);
}

#[test]
fn batch_turns_match_sequential_inverse_turns() {
    let turns: Vec<Turn> = sample_turns()
        .iter()
        .rev()
        .map(|turn| turn.inverse())
        .collect();
    let mut sequential = labeled_puzzle();
    let mut batch = sequential.clone();

    for turn in &turns {
        assert!(sequential.turn(turn.clone()).is_some());
    }
    assert!(batch.apply_turns_batch(&turns).is_some());

    assert_eq!(sequential.stickers, batch.stickers);
}

#[test]
fn batch_positions_match_sequential_positions() {
    let puzzle = Puzzle::make_solved(3, 4);
    let turns = sample_turns();
    let mut sequential = vec![
        vec![2, 2, 0, 0],
        vec![2, 0, 2, 0],
        vec![0, 0, 0, 0],
        vec![2, 2, 0, 0],
    ];
    let mut batch = sequential.clone();

    for turn in &turns {
        for pos in &mut sequential {
            assert!(puzzle.turn_position(pos, turn).is_some());
        }
    }
    assert!(puzzle
        .apply_turns_to_positions_batch(&mut batch, &turns)
        .is_some());

    assert_eq!(sequential, batch);
}
