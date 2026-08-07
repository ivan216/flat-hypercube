#![allow(dead_code)]

use flat_hypercube::prefs::Prefs;
use flat_hypercube::state::AppState;

pub fn prefs_json(
    select: &str,
    side: &str,
    axis_key: &str,
    layer0: &str,
    reset_mode: &str,
) -> String {
    let json = r#"{
    "axes": [
        {
            "pos": {
                "name": "R",
                "color": "ff0000",
                "keys": {
                    "select": "__SELECT__",
                    "side": "__SIDE__"
                }
            },
            "neg": {
                "name": "L",
                "color": "00ff00",
                "keys": {
                    "select": "a",
                    "side": "b"
                }
            },
            "axis_key": "__AXIS_KEY__"
        }
    ],
    "global_keys": {
        "layers": [
            "__LAYER0__"
        ],
        "rotate": "x",
        "scramble": "=",
        "reset": "-",
        "keybind_mode": "\\",
        "axis_mode": "|",
        "undo": "z",
        "redo": "Z",
        "next_filter": "K",
        "prev_filter": "J",
        "live_filter_mode": "F",
        "reset_mode": "__RESET_MODE__",
        "save": "S",
        "rev_start": "Q",
        "rev_stop": "W",
        "rev_unwind": "E",
        "rev_commutator": "R"
    },
    "global_colors": {
        "piece": "808080",
        "filtered": "505050",
        "alert": "d86c6c",
        "clicked": "b0b0b0"
    },
    "damage_repeat": 5,
    "alert_frames": 4
}"#;

    json.replace("__SELECT__", select)
        .replace("__SIDE__", side)
        .replace("__AXIS_KEY__", axis_key)
        .replace("__LAYER0__", layer0)
        .replace("__RESET_MODE__", reset_mode)
}

pub fn test_state() -> AppState {
    AppState::new(
        Some(3),
        Some(3),
        Prefs::load_default().expect("default prefs"),
    )
    .expect("valid test state")
}
