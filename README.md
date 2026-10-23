# Rust Wars // Frontier 09

Rust Wars is a playable graphical battle slice for the demo: a turn-based fight between human forces and AI war bots over a larger, scrollable 16×12 isometric frontier map. The 2.5D presentation uses code-drawn pixel-style terrain, raised forests, faceted mountains and units, so the project needs no external art assets.

## Run

Install Rust with rustup, then launch the game:

```sh
cargo run --release
```

## Battle controls

- Click a human unit to select it.
- Click a highlighted tile to move. Forests cost extra movement; mountains block movement.
- Click any bot to lock it as your target; press `Tab` to cycle through enemies and center the map on each one.
- With a human unit selected, click an enemy in range to attack. Infantry and armor attack adjacent tiles. Artillery attacks at range 2–3.
- Infantry captures a command site by moving onto it.
- Click **End Turn** or press `E` to let the war bots act.
- Press `Esc` or right-click to clear selection.
- Press `P` to toggle pan mode, then left-drag the map. Middle-drag and arrow keys also pan; press `Home` to return to deployment.
- Press `M` to mute or restore the looping chiptune battle theme.
- Selection, movement, firing, impacts and captures have synthesized sound effects.
- Capture three of four command sites or eliminate the enemy army to win.
- Click or press `Enter` after the battle to redeploy.

## Units

| Unit | Movement | Weapon |
| --- | ---: | --- |
| Infantry | 3 | Adjacent; captures sites |
| Armor | 4 | Adjacent; strong against infantry |
| Artillery | 3 | Range 2–3; cannot fire adjacent |

The battle slice uses [Macroquad](https://macroquad.rs/) for cross-platform 2D rendering.
