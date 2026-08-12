#![allow(dead_code)]
use crossterm::style::Color;
use serde::Deserializer;
use serde::de::Error;
use std::num::ParseIntError;

use rgb2ansi256::rgb_to_ansi256;
use serde::Deserialize;

pub const ESCAPE_CODE: char = '\x1b';
pub const BACKSPACE_CODE: char = '\x08';
pub const DISABLED_KEY_CODE: char = '\0';
pub const DEFAULT_FILE_PATH_STR: &str = include_str!("../default_prefs.json");

#[derive(Debug, Clone, Deserialize)]
pub struct Prefs {
    pub axes: Vec<Axis>,
    pub global_keys: GlobalKeys,
    pub global_colors: GlobalColors,
    pub damage_repeat: u8,
    pub alert_frames: u8,
}

impl Prefs {
    pub fn load_default() -> Result<Self, Box<dyn std::error::Error>> {
        Ok(serde_json::from_str(DEFAULT_FILE_PATH_STR)?)
    }

    pub fn pos_keys(&self) -> impl Iterator<Item = char> + '_ {
        self.axes.iter().map(|side| side.pos.keys.select)
    }

    pub fn max_dim(&self) -> u16 {
        self.axes.len() as u16
    }

    pub fn max_layers(&self) -> i16 {
        (self.global_keys.layers.len() * 2 + 1) as i16
    }

    pub fn validate(&self) -> Result<(), String> {
        use std::collections::HashSet;

        // Same field across different (axis + direction) must be unique.
        // select and side each span all pos/neg directions; axis_key spans all axes.
        // Different fields (select vs side vs axis_key) never conflict.
        let mut selects = HashSet::new();
        let mut sides = HashSet::new();
        for (i, ax) in self.axes.iter().enumerate() {
            for (dir, keys) in [("pos", &ax.pos.keys), ("neg", &ax.neg.keys)] {
                if !is_disabled_key(keys.select) && !selects.insert(keys.select) {
                    return Err(format!(
                        "duplicate select key '{0}' in axis {i} {dir}",
                        keys.select
                    ));
                }
                if !is_disabled_key(keys.side) && !sides.insert(keys.side) {
                    return Err(format!(
                        "duplicate side key '{0}' in axis {i} {dir}",
                        keys.side
                    ));
                }
            }
        }

        let mut axis_keys = HashSet::new();
        for (i, ax) in self.axes.iter().enumerate() {
            if !is_disabled_key(ax.axis_key) && !axis_keys.insert(ax.axis_key) {
                return Err(format!("duplicate axis_key '{0}' in axis {i}", ax.axis_key));
            }
        }

        // Any axis key must not conflict with global keys
        let all: HashSet<char> = selects
            .iter()
            .chain(sides.iter())
            .chain(axis_keys.iter())
            .copied()
            .collect();
        let gk = &self.global_keys;
        for (i, &ch) in gk.layers.iter().enumerate() {
            if all.contains(&ch) {
                return Err(format!(
                    "key '{ch}' in layer key[{i}] conflicts with an axis key"
                ));
            }
        }
        for (label, ch) in [
            ("global rotate", gk.rotate),
            ("global scramble", gk.scramble),
            ("global reset", gk.reset),
            ("global keybind_mode", gk.keybind_mode),
            ("global axis_mode", gk.axis_mode),
            ("global undo", gk.undo),
            ("global redo", gk.redo),
            ("global next_filter", gk.next_filter),
            ("global prev_filter", gk.prev_filter),
            ("global live_filter_mode", gk.live_filter_mode),
            ("global reset_mode", gk.reset_mode),
            ("global save", gk.save),
            ("global rev_start", gk.rev_start),
            ("global rev_stop", gk.rev_stop),
            ("global rev_unwind", gk.rev_unwind),
            ("global rev_commutator", gk.rev_commutator),
        ] {
            if all.contains(&ch) {
                return Err(format!("key '{ch}' in {label} conflicts with an axis key"));
            }
        }

        Ok(())
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct Axis {
    pub pos: Side,
    pub neg: Side,
    #[serde(deserialize_with = "de_key")]
    pub axis_key: char,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Side {
    pub name: char,
    #[serde(deserialize_with = "de_color")]
    pub color: Color,
    pub keys: Keys,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Keys {
    #[serde(deserialize_with = "de_key")]
    pub select: char,
    #[serde(deserialize_with = "de_key")]
    pub side: char,
}

#[derive(Debug, Clone, Deserialize)]
pub struct GlobalColors {
    #[serde(deserialize_with = "de_color")]
    pub piece: Color,
    #[serde(deserialize_with = "de_color")]
    pub filtered: Color,
    #[serde(deserialize_with = "de_color")]
    pub alert: Color,
    #[serde(deserialize_with = "de_color")]
    pub clicked: Color,
}

#[derive(Debug, Clone, Deserialize)]
pub struct GlobalKeys {
    #[serde(deserialize_with = "de_key_vec")]
    pub layers: Vec<char>,
    #[serde(deserialize_with = "de_key")]
    pub rotate: char,
    #[serde(deserialize_with = "de_key")]
    pub scramble: char,
    #[serde(deserialize_with = "de_key")]
    pub reset: char,
    #[serde(deserialize_with = "de_key")]
    pub keybind_mode: char,
    #[serde(deserialize_with = "de_key")]
    pub axis_mode: char,
    #[serde(deserialize_with = "de_key")]
    pub undo: char,
    #[serde(deserialize_with = "de_key")]
    pub redo: char,
    #[serde(deserialize_with = "de_key")]
    pub next_filter: char,
    #[serde(deserialize_with = "de_key")]
    pub prev_filter: char,
    #[serde(deserialize_with = "de_key")]
    pub live_filter_mode: char,
    #[serde(deserialize_with = "de_key")]
    pub reset_mode: char,
    #[serde(deserialize_with = "de_key")]
    pub save: char,
    #[serde(deserialize_with = "de_key")]
    pub rev_start: char,
    #[serde(deserialize_with = "de_key")]
    pub rev_stop: char,
    #[serde(deserialize_with = "de_key")]
    pub rev_unwind: char,
    #[serde(deserialize_with = "de_key")]
    pub rev_commutator: char,
}

fn is_disabled_key(ch: char) -> bool {
    ch == DISABLED_KEY_CODE
}

fn parse_key(st: &str) -> Result<char, String> {
    match st {
        "Esc" | "Escape" | "\\e" | "\\x1b" | "\\u001b" => return Ok(ESCAPE_CODE),
        "Backspace" | "\\b" | "\\x08" | "\\u0008" => return Ok(BACKSPACE_CODE),
        "None" | "Disabled" | "\\0" | "\\x00" | "\\u0000" => return Ok(DISABLED_KEY_CODE),
        _ => {}
    }

    let mut chars = st.chars();
    match (chars.next(), chars.next()) {
        (Some('⎋'), None) => Ok(ESCAPE_CODE),
        (Some('⌫'), None) => Ok(BACKSPACE_CODE),
        (Some('∅'), None) => Ok(DISABLED_KEY_CODE),
        (Some(ch), None) => Ok(ch),
        _ => Err(format!(
            "key must be one character or a supported key escape, got {st:?}"
        )),
    }
}

fn de_key<'de, D>(deserializer: D) -> Result<char, D::Error>
where
    D: Deserializer<'de>,
{
    let st = String::deserialize(deserializer)?;
    parse_key(&st).map_err(D::Error::custom)
}

fn de_key_vec<'de, D>(deserializer: D) -> Result<Vec<char>, D::Error>
where
    D: Deserializer<'de>,
{
    let keys = Vec::<String>::deserialize(deserializer)?;
    keys.into_iter()
        .map(|st| parse_key(&st).map_err(D::Error::custom))
        .collect()
}

fn hex(st: &str) -> Result<Color, ParseIntError> {
    let hex = u32::from_str_radix(&st, 16)?;
    Ok(Color::AnsiValue(rgb_to_ansi256(
        ((hex >> 16) & 0xff) as u8,
        ((hex >> 8) & 0xff) as u8,
        ((hex >> 0) & 0xff) as u8,
    )))
}

fn de_color<'de, D>(deserializer: D) -> Result<Color, D::Error>
where
    D: Deserializer<'de>,
{
    let st = String::deserialize(deserializer)?;
    hex(&st).map_err(D::Error::custom)
}
