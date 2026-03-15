# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

Craft-rs is a Minecraft clone ported from C to Rust, using software rendering via a local `portablegl-rs` crate (no GPU required). It uses SDL2 for windowing/input and Simplex noise for procedural terrain generation.

## Build Commands

```bash
cargo build          # Build the project
cargo run            # Build and run
cargo build --release  # Release build
```

**Platform note:** Currently targets `x86_64-pc-windows-gnu` (configured in `.cargo/config.toml`). The build script (`build.rs`) copies `SDL2.dll` to the target directory automatically. SDL2 static/import libraries are vendored in `/lib`.

**Key dependency:** `portablegl` is a local crate referenced via `path = ../portablegl-rs` in Cargo.toml — it must exist as a sibling directory.

## Architecture

**Entry point:** `main.rs` — contains the game loop, SDL2/OpenGL initialization, event handling, rendering orchestration, and player physics.

**Core modules:**

- **chunk.rs** — Chunk management (32x32x256 blocks), mesh generation, ambient occlusion computation, light propagation via flood fill
- **world.rs** — Procedural terrain generation using multi-octave Simplex noise (grass, sand, stone, trees, flowers, clouds)
- **render.rs** — Matrix setup, buffer management, frustum culling
- **shaders.rs** — GLSL-style vertex/fragment shaders for blocks, sky, text, and lines (executed on CPU by PortableGL)
- **player.rs** — Player state, WASD movement, gravity, collision detection, camera interpolation
- **cube.rs** — Geometric mesh generation for block faces, plants, and character models
- **item.rs** — Block type constants (64 types) and texture atlas UV mappings
- **map.rs** — Hash-based spatial map for storing block data per chunk
- **config.rs** — Game constants (window size 1024x768, chunk radius, etc.)
- **util.rs** — PNG texture loading, FPS counter, text rendering utilities
- **sign.rs** — Sign data structures

**Rendering pipeline:** All rendering is software-based through PortableGL's `GlContext`. Shaders receive a custom uniforms struct. The render loop: clear → sky sphere → terrain chunks (with frustum culling) → wireframe overlay → HUD (crosshairs, item preview, text). Day/night cycle runs on a 600-second timer affecting lighting and fog.

**Chunk system:** Chunks use a hash map (`Map`) for sparse block storage. Dirty chunks get their mesh buffers rebuilt. A worker thread pool (4 threads) infrastructure exists for async chunk generation.

## Textures

Block textures use a 16x16 tile atlas (`textures/texture.png`). Font, sky, and sign textures are separate PNGs in the `textures/` directory.
