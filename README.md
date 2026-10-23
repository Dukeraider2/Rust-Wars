# Rust Wars

A small, dependency-free Rust tactics demo inspired by 16-bit turn-based strategy games. Human forces and AI war bots fight over a frontier map. Capture three command sites to win, or destroy every enemy unit.

## Run

Install Rust with [rustup](https://rustup.rs/), then run:

```sh
cargo run --release
```

## Play

Select one of your units by its ID, then move or attack. Each unit gets one action per round. Infantry can seize a command site by moving onto it.

```text
units                  list units and IDs
select <id>            select a human unit (example: select 1)
move <x> <y>           move the selected unit (example: move 4 3)
attack <x> <y>         attack a target in range
end                    end your turn; the war bots act
help                   show commands
quit                   quit the demo
```

Coordinates are shown on the map, with `(0, 0)` at the top left. Forests slow movement and protect units. Mountains block movement. Artillery can fire from range but cannot shoot adjacent targets.

## Map legend

| Mark | Meaning |
| --- | --- |
| `.` | Plains |
| `♣` | Forest |
| `▲` | Mountain |
| `H` / `T` / `A` | Human infantry / armor / artillery |
| `b` / `t` / `a` | Bot infantry / armor / artillery |
| `⌂` | Neutral command site |
| Cyan / red `⌂` | Human / bot command site |
