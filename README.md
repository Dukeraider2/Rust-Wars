# Rust Wars // Frontier 09

Rust Wars is a playable graphical battle slice for the demo: a turn-based fight between human forces and AI war bots over a pan-able 16x12 isometric frontier map. Its industrial frontier look combines textured terrain, stepped elevation, raised forests, faceted ridges and an original pixel-art unit atlas inspired by classic sci-fi strategy games.

## Run

Install Rust with rustup, then launch the game:

```sh
cargo run --release
```

## Battle controls

- Click a human unit to select it.
- Click a highlighted tile to move. Forests cost extra movement and grant cover; each uphill level costs extra movement, and units can climb one level per tile.
- High ground adds attack damage and defense. Mountain ridges block movement and artillery fire; artillery needs a clear straight firing lane.
- Click any bot to lock it as your target; press `Tab` to cycle through enemies and center the map on each one.
- With a human unit selected, click an enemy in range to attack. Infantry and armor attack adjacent tiles. Artillery attacks at range 2â€“3.
- Infantry captures a command site by moving onto it.
- Click **End Turn** or press `E` to let the war bots act.
- Press `Esc` or right-click to clear selection.
- Press `P` to toggle pan mode, then left-drag the map. Middle-drag and arrow keys also pan; press `Home` to return to deployment.
- Press `M` to mute or restore the looping CC0 sci-fi battle theme.
- Selection, movement, firing, impacts and captures have synthesized sound effects.
- Capture three of five command sites or eliminate the enemy army to win.
- Click or press `Enter` after the battle to redeploy.

## Units

| Unit | Movement | Weapon |
| --- | ---: | --- |
| Infantry | 3 | Adjacent; captures sites |
| Armor | 4 | Adjacent; strong against infantry |
| Artillery | 3 | Range 2â€“3; cannot fire adjacent |

The battle slice uses [Macroquad](https://macroquad.rs/) for cross-platform 2D rendering.

## Music credit

Noisy Battle by Some Weirdo (CC0), [OpenGameArt.org](https://opengameart.org/content/noisy-battle).

