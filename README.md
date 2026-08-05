# Flat Hypercube

This repository is a personal fork of the original Flat Hypercube project. It is kept for my own use and experimentation, and it is not intended to continue tracking upstream changes.

The upstream baseline for this fork is commit `d1c60c8`. Changes in this repository should be understood relative to that commit, not relative to the current upstream main branch.

## Differences From Upstream `d1c60c8`

This fork includes the following local changes:

- Added pan view controls, including mouse dragging, arrow-key panning, wheel scrolling, and touchpad-style horizontal or vertical scrolling.
- Added reversion-block shortcuts for commutator and conjugator workflows:
  - `F1` starts a reversion block.
  - `F2` ends the current block.
  - `F3` applies the inverse of the block.
  - `F4` applies the commutator-style reverse sequence.
- Refactored the turn system around explicit turn objects and shared move history handling.
- Added keybind hints for `n = 2` puzzles.
- Improved `Ctrl+C` handling so the first press exits cleanly.
- Removed hard-coded layout gap limits that restricted higher-dimensional layouts.
- Added semi-compact display mode for a middle ground between the default layout and fully compact mode.

## What It Does

Flat Hypercube is a terminal-based simulator for solving `n^d` hypercubes using a flat recursive projection. A custom preferences file can define axes, colors, and keybindings for higher-dimensional puzzles.

The layout is recursive in the number of dimensions. Adding a dimension creates copies of the lower-dimensional layout, with caps representing the two new facets. The projection was inspired by Don Hatch's layout in MagicCubeNdSolve.

## Running

Build and run with Cargo:

```bash
cargo run -- [n] [d]
```

For example:

```bash
cargo run -- 3 4
```

Useful display options:

```bash
cargo run -- 3 4 -c        # semi-compact mode
cargo run -- 3 4 --compact # compact mode
cargo run -- 3 4 --vertical
cargo run -- 3 4 --boxes
```

Other options:

```bash
cargo run -- 3 4 --prefs path/to/prefs.json
cargo run -- 3 4 --filters path/to/filters.txt
cargo run -- --log path/to/log.json
```

## Basic Controls

Global controls:

- `Ctrl+C`: quit cleanly; press again to force quit.
- `=` five times: scramble the puzzle.
- `-` five times: reset the puzzle.
- `Z`: undo.
- `Shift+Z`: redo.
- `Esc`: clear the current input or status mode.
- `Shift+S`: save the current session.

View controls:

- Drag with the mouse to pan.
- Use arrow keys to pan.
- Use the mouse wheel or touchpad scrolling to pan.
- Hold `Shift` while scrolling vertically to pan horizontally.
- Hold `Ctrl` with arrow keys for fine panning.

## Turn Input

The program supports multiple keybinding systems. Press `\` to cycle turn modes. Press `Shift+\` to toggle axis mode and side mode.

Layer keys `1` through `9` select a layer before a turn. `X` starts a whole-puzzle rotation.

The default keybindings are defined in `default_prefs.json`. Custom preferences files can define different keys and additional axes.

## Reversion Blocks

Reversion blocks are intended for commutator and conjugator-style workflows.

- `F1`: start a block.
- `F2`: end a block.
- `F3`: apply the inverse of the block in reverse order.
- `F4`: apply the inverse of the block, then the inverse of the moves after the block.

The current block structure is displayed as `RevStack` near the bottom of the terminal. Undo and redo adjust block indices when they cross block boundaries.

After `F3` or `F4`, the puzzle is checked for the solved state just like after a normal turn. If the puzzle is solved, the status line prints `solved!`.

## Marking Stickers

Click a sticker to mark it and nearby stickers with brackets. Marking happens on mouse release, so dragging can be used for panning without accidentally marking stickers.

Double-click empty space to clear all marks.

## Filters

Piece filters can be loaded with `--filters`. Each line in the filter file is one filter expression.

Live filter creation is available with `Shift+F`. Confirm with `Enter` and cancel with `Esc`.

Use `Shift+K` and `Shift+J` to move between loaded filters.

## Notes

This fork is maintained for personal workflows. Compatibility with future upstream changes is not a goal.
