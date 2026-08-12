mod common;

use common::prefs_json;
use flat_hypercube::prefs::{BACKSPACE_CODE, DISABLED_KEY_CODE, ESCAPE_CODE, Prefs};

fn parse_prefs(json: String) -> Prefs {
    serde_json::from_str(&json).expect("prefs json")
}

#[test]
fn standard_escapes_parse() {
    let prefs = parse_prefs(prefs_json(
        r"\u001b", r"\b", r"\u0000", r"\u001b", r"\u001b",
    ));

    assert_eq!(prefs.global_keys.reset_mode, ESCAPE_CODE);
    assert_eq!(prefs.global_keys.layers[0], ESCAPE_CODE);
    assert_eq!(prefs.axes[0].pos.keys.select, ESCAPE_CODE);
    assert_eq!(prefs.axes[0].pos.keys.side, BACKSPACE_CODE);
    assert_eq!(prefs.axes[0].axis_key, DISABLED_KEY_CODE);
}

#[test]
fn named_aliases_parse() {
    let prefs = parse_prefs(prefs_json(
        "Escape",
        "Backspace",
        "Disabled",
        "Backspace",
        "Escape",
    ));

    assert_eq!(prefs.global_keys.reset_mode, ESCAPE_CODE);
    assert_eq!(prefs.global_keys.layers[0], BACKSPACE_CODE);
    assert_eq!(prefs.axes[0].pos.keys.select, ESCAPE_CODE);
    assert_eq!(prefs.axes[0].pos.keys.side, BACKSPACE_CODE);
    assert_eq!(prefs.axes[0].axis_key, DISABLED_KEY_CODE);
}

#[test]
fn legacy_symbols_parse() {
    let prefs = parse_prefs(prefs_json("⎋", "⌫", "∅", "n", "⎋"));

    assert_eq!(prefs.global_keys.reset_mode, ESCAPE_CODE);
    assert_eq!(prefs.axes[0].pos.keys.select, ESCAPE_CODE);
    assert_eq!(prefs.axes[0].pos.keys.side, BACKSPACE_CODE);
    assert_eq!(prefs.axes[0].axis_key, DISABLED_KEY_CODE);
}
