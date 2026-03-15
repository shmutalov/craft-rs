# Craft-rs

A Minecraft clone written in Rust, ported from [Craft](https://github.com/fogleman/Craft) by Michael Fogleman. Uses software rendering via [portablegl-rs](https://github.com/shmutalov/portablegl-rs) — no GPU required.

## Features

- Procedural terrain generation using Simplex noise
- 64+ block types (grass, sand, stone, brick, wood, flowers, etc.)
- Block placement, destruction, and eyedropper picking
- Ambient occlusion and dynamic lighting
- Day/night cycle with smooth lighting transitions
- Frustum culling for efficient rendering
- Flying mode and collision-based walking
- Software-rendered — runs without a GPU

## Controls

| Key | Action |
|-----|--------|
| WASD | Move |
| Space | Jump |
| Left Click | Break block |
| Right Click | Place block |
| Middle Click | Pick block type |
| Mouse Wheel / E / R | Cycle inventory |
| 1-9 | Select inventory slot |
| Tab | Toggle flying mode |
| F | Toggle orthographic view |
| Shift | Zoom |
| Esc | Release mouse / exit |

## Prerequisites

- Rust toolchain (edition 2021)
- `x86_64-pc-windows-gnu` target (configured in `.cargo/config.toml`)
- [portablegl-rs](https://github.com/shmutalov/portablegl-rs) cloned as a sibling directory (`../portablegl-rs`)

## Building and Running

```bash
cargo run
```

SDL2 libraries are vendored in the `lib/` directory and `SDL2.dll` is automatically copied to the build output by the build script.

## Dependencies

| Crate | Purpose |
|-------|---------|
| [portablegl](https://github.com/shmutalov/portablegl-rs) | Software OpenGL renderer |
| [sdl2](https://crates.io/crates/sdl2) | Window creation and input handling |
| [noise](https://crates.io/crates/noise) | Simplex noise for terrain generation |
| [rusqlite](https://crates.io/crates/rusqlite) | SQLite database (bundled) |
| [png](https://crates.io/crates/png) | Texture loading |

## License

MIT License. See [LICENSE.md](LICENSE.md) for details.

## Acknowledgments

This project is a Rust port of [Craft](https://github.com/fogleman/Craft) by Michael Fogleman.
