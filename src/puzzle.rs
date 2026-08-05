use itertools::Itertools;
use rand::prelude::*;
use rand::rngs::ThreadRng;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, Hash)]
pub struct SideTurn {
    pub side: i16,
    pub layer_min: i16,
    pub layer_max: i16,
    pub from: i16,
    pub to: i16,
}

impl SideTurn {
    pub fn inverse(&self) -> Self {
        SideTurn {
            from: self.to,
            to: self.from,
            side: self.side,
            layer_min: self.layer_min,
            layer_max: self.layer_max,
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, Hash)]
pub struct PuzzleTurn {
    pub from: i16,
    pub to: i16,
}

impl PuzzleTurn {
    pub fn inverse(&self) -> Self {
        PuzzleTurn {
            from: self.to,
            to: self.from,
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, Hash)]
pub enum Turn {
    Side(SideTurn),
    Puzzle(PuzzleTurn),
}

impl Turn {
    pub fn inverse(&self) -> Self {
        match self {
            Self::Side(t) => Self::Side(t.inverse()),
            Self::Puzzle(t) => Self::Puzzle(t.inverse()),
        }
    }
}

mod serde_map {
    use serde::{Deserialize, Deserializer, Serialize, Serializer};
    use std::collections::HashMap;

    pub(super) fn serialize<K, V, S>(
        value: &HashMap<K, V>,
        serializer: S,
    ) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
        K: Serialize,
        V: Serialize,
    {
        value.iter().collect::<Vec<_>>().serialize(serializer)
    }

    pub(super) fn deserialize<'de, K, V, D>(deserializer: D) -> Result<HashMap<K, V>, D::Error>
    where
        D: Deserializer<'de>,
        K: Deserialize<'de> + std::hash::Hash + Eq,
        V: Deserialize<'de>,
    {
        Ok(HashMap::from_iter(<Vec<(K, V)>>::deserialize(
            deserializer,
        )?))
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Puzzle {
    pub n: i16,
    pub d: u16,
    // map from coordinate vector (only contains -n+1, n-1 every other, and ±n)
    // to side (sides related by ! are opposite)
    #[serde(with = "serde_map")]
    pub stickers: HashMap<Vec<i16>, i16>,
}

pub fn ax(s: i16) -> i16 {
    s.max(!s)
}

#[derive(Debug, Clone, Copy)]
enum NormalizedTurn {
    Side {
        side: usize,
        layer_min: i16,
        layer_max: i16,
        from: usize,
        to: usize,
    },
    Puzzle {
        from: usize,
        to: usize,
    },
}

impl Puzzle {
    pub fn make_solved(n: i16, d: u16) -> Puzzle {
        if d == 1 {
            // i think multi_cartesian_product returns empty iterator for the empty product

            return Puzzle {
                n,
                d,
                stickers: HashMap::from([(vec![-n], !0), (vec![n], 0)]),
            };
        }

        let mut stickers = HashMap::new();
        for (side, coords) in [n, -n].into_iter().cartesian_product(
            (0..d - 1)
                .map(|_| (-n + 1..n).step_by(2))
                .multi_cartesian_product(),
        ) {
            let mut pos = vec![side];
            pos.extend(&coords);
            for f in 0..(d as i16) {
                stickers.insert(pos.clone(), if side >= 0 { f } else { !f });
                pos.rotate_right(1);
            }
        }
        Puzzle { n, d, stickers }
    }

    pub fn is_solved(&self) -> bool {
        let mut side_colors = HashMap::new();
        for (pos, &color) in &self.stickers {
            let side = pos
                .iter()
                .position(|x| x.abs() == self.n)
                .expect("should be on a face");
            let side = if pos[side] < 0 { !side } else { side };
            let old_color = side_colors.insert(side, color);
            match old_color {
                Some(c) if c != color => return false,
                _ => (),
            }
        }
        true
    }

    fn axis_index(&self, axis: i16) -> Option<usize> {
        let axis = ax(axis);
        if axis < 0 || axis as u16 >= self.d {
            return None;
        }
        Some(axis as usize)
    }

    fn normalize_turn(&self, turn: &Turn) -> Option<NormalizedTurn> {
        match turn {
            Turn::Side(turn) => {
                let SideTurn {
                    side,
                    layer_min,
                    layer_max,
                    mut from,
                    mut to,
                } = *turn;
                if side == from
                    || side == !from
                    || side == to
                    || side == !to
                    || from == to
                    || from == !to
                {
                    return None;
                }

                let to_swap = (from < 0) != (to < 0);
                if from < 0 {
                    from = !from
                }
                if to < 0 {
                    to = !to
                }
                if to_swap {
                    std::mem::swap(&mut from, &mut to)
                }

                Some(NormalizedTurn::Side {
                    side: self.axis_index(side)?,
                    layer_min,
                    layer_max,
                    from: self.axis_index(from)?,
                    to: self.axis_index(to)?,
                })
            }
            Turn::Puzzle(turn) => {
                let from = self.axis_index(turn.from)?;
                let to = self.axis_index(turn.to)?;
                if from == to {
                    return None;
                }
                Some(NormalizedTurn::Puzzle { from, to })
            }
        }
    }

    fn source_position(&self, turn: NormalizedTurn, pos: &[i16]) -> Option<Vec<i16>> {
        match turn {
            NormalizedTurn::Side {
                side,
                layer_min,
                layer_max,
                from,
                to,
            } => {
                let layer_range = layer_min - 1..=layer_max + 1;
                if !layer_range.contains(&pos[side]) {
                    return None;
                }
                let mut source = pos.to_vec();
                source[from] = pos[to];
                source[to] = -pos[from];
                Some(source)
            }
            NormalizedTurn::Puzzle { from, to } => {
                let mut source = pos.to_vec();
                source[from] = pos[to];
                source[to] = -pos[from];
                Some(source)
            }
        }
    }

    fn apply_normalized_turn(&mut self, turn: NormalizedTurn) {
        let mut new_stickers = HashMap::new();
        for pos in self.stickers.keys() {
            if let Some(source) = self.source_position(turn, pos) {
                new_stickers.insert(pos.clone(), self.stickers[&source]);
            }
        }
        self.stickers.extend(new_stickers);
    }

    fn normalized_turn_cycles(
        &self,
        turn: NormalizedTurn,
        positions: &[Vec<i16>],
        pos_to_idx: &HashMap<Vec<i16>, usize>,
        source_indices: &mut [usize],
        visited: &mut [bool],
        cycle: &mut Vec<usize>,
    ) -> Vec<Vec<usize>> {
        let len = positions.len();
        for (idx, source_idx) in source_indices.iter_mut().enumerate() {
            *source_idx = idx;
        }
        visited.fill(false);
        let mut changed = false;

        for (dest_idx, dest) in positions.iter().enumerate() {
            if let Some(source) = self.source_position(turn, dest) {
                let source_idx = pos_to_idx[&source];
                source_indices[dest_idx] = source_idx;
                changed |= source_idx != dest_idx;
            }
        }
        if !changed {
            return Vec::new();
        }

        let mut cycles = Vec::new();
        for start in 0..len {
            if visited[start] {
                continue;
            }

            cycle.clear();
            let mut current = start;
            while !visited[current] {
                visited[current] = true;
                cycle.push(current);
                current = source_indices[current];
            }

            if cycle.len() > 1 {
                cycles.push(cycle.clone());
            }
        }
        cycles
    }

    fn apply_cycles_to_permutation(cycles: &[Vec<usize>], perm: &mut [usize]) {
        for cycle in cycles {
            let first = perm[cycle[0]];
            for i in 0..cycle.len() - 1 {
                perm[cycle[i]] = perm[cycle[i + 1]];
            }
            perm[*cycle.last().expect("non-empty cycle")] = first;
        }
    }

    pub fn turn(&mut self, turn: Turn) -> Option<()> {
        let turn = self.normalize_turn(&turn)?;
        self.apply_normalized_turn(turn);
        Some(())
    }

    pub fn turn_position(&self, pos: &mut Vec<i16>, turn: &Turn) -> Option<()> {
        let turn = self.normalize_turn(turn)?;
        match turn {
            NormalizedTurn::Side {
                side,
                layer_min,
                layer_max,
                from,
                to,
            } => {
                let layer_range = layer_min - 1..=layer_max + 1;
                if layer_range.contains(&pos[side]) {
                    pos.swap(from, to);
                    pos[from] *= -1;
                }
            }
            NormalizedTurn::Puzzle { from, to } => {
                pos.swap(from, to);
                pos[from] *= -1;
            }
        }
        Some(())
    }

    pub fn apply_turns_batch(&mut self, turns: &[Turn]) -> Option<()> {
        let turns: Vec<(Turn, NormalizedTurn)> = turns
            .iter()
            .map(|turn| Some((turn.clone(), self.normalize_turn(turn)?)))
            .collect::<Option<_>>()?;
        if turns.is_empty() {
            return Some(());
        }

        let positions: Vec<Vec<i16>> = self.stickers.keys().cloned().collect();
        let pos_to_idx: HashMap<Vec<i16>, usize> = positions
            .iter()
            .cloned()
            .enumerate()
            .map(|(idx, pos)| (pos, idx))
            .collect();
        let old_colors: Vec<i16> = positions.iter().map(|pos| self.stickers[pos]).collect();
        let mut perm: Vec<usize> = (0..positions.len()).collect();
        let mut source_indices: Vec<usize> = vec![0; positions.len()];
        let mut visited = vec![false; positions.len()];
        let mut cycle = Vec::new();
        let mut cycle_cache: HashMap<Turn, Vec<Vec<usize>>> = HashMap::new();

        for (turn, normalized) in turns {
            let cycles = cycle_cache.entry(turn).or_insert_with(|| {
                self.normalized_turn_cycles(
                    normalized,
                    &positions,
                    &pos_to_idx,
                    &mut source_indices,
                    &mut visited,
                    &mut cycle,
                )
            });
            Self::apply_cycles_to_permutation(cycles, &mut perm);
        }

        for (dest_idx, pos) in positions.iter().enumerate() {
            *self.stickers.get_mut(pos).expect("known position") = old_colors[perm[dest_idx]];
        }
        Some(())
    }

    pub fn piece_body(&self, piece: &[i16]) -> Vec<i16> {
        if let Some(ind) = piece.iter().position(|x| x.abs() == self.n) {
            let mut piece_body = piece.to_vec();
            if piece[ind] == self.n {
                piece_body[ind] -= 1;
            } else {
                piece_body[ind] += 1;
            }
            piece_body
        } else {
            piece.to_vec()
        }
    }

    fn piece_body_stickers(&self, piece: &[i16]) -> Vec<i16> {
        let mut colors = vec![];
        for (ind, x) in piece.iter().enumerate() {
            let mut piece = piece.to_vec();
            if *x == self.n - 1 {
                piece[ind] += 1;
            } else if *x == -(self.n - 1) {
                piece[ind] -= 1;
            } else {
                continue;
            }
            colors.push(self.stickers[&piece]);
            if self.n == 1 {
                // the piece of a 1^d has two stickers per axis
                colors.push(!self.stickers[&piece]);
            }
        }
        colors
    }

    pub fn stickers(&self, piece: &[i16]) -> Vec<i16> {
        self.piece_body_stickers(&self.piece_body(piece))
    }

    pub fn scramble(&mut self, rng: &mut ThreadRng) {
        for _ in 0..5000 {
            let mut axes: Vec<i16> = (0..self.d as i16).collect();
            axes.shuffle(rng);
            let layer = self.n - 1 - 2 * rng.gen_range(0..self.n);
            self.turn(Turn::Side(SideTurn {
                side: axes[0],
                layer_min: layer,
                layer_max: layer,
                from: axes[1],
                to: axes[2],
            }));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
}
