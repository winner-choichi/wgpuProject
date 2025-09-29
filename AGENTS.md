# Repository Guidelines

## Project Structure & Module Organization

The project delivers a cross-platform atomic orbital visualizer. `src/main.rs` starts a native window, while `src/platform.rs` abstracts surface creation for native, web, and headless providers. Shared exports are re-exported from `src/lib.rs`. The rendering stack lives under `src/renderer/` (camera, mesh generation, cloud point pipeline, and WGSL shaders in `src/shaders/`). Domain logic is split into `src/physics/` (element data, nucleus modelling, orbital math) and `src/simulation/` (atom aggregation plus Monte Carlo solvers). `src/app/` orchestrates renderer + simulation state, and `src/ui/` integrates the egui control panel. Build artefacts collect in `target/` and must remain untracked.

## Quick Start

- `cargo run` launches the desktop visualizer with egui controls.
- `cargo check` offers a fast sanity pass across all targets while you iterate.
- `cargo test --all-targets` is the pre-push gate; new code and docs should keep it green.
- `cargo build --release` emits an optimized native build for demos or profiling sessions.
- `cargo build --target wasm32-unknown-unknown` compiles the WebGL/WebGPU binary (serve with your bundler).
- Need verbose GPU state? Run with `RUST_LOG=wgpu=trace cargo run` to surface device-level debugging.

## Runtime Controls

- **Camera translation**: `W`/`S` walk forward/backward, `A`/`D` strafe left/right; updates are immediate and request a redraw.
- **Zoom**: use the mouse wheel (or trackpad scroll). We clamp the camera distance between 0.5 Å and ~95% of `zfar` to avoid clipping the nucleus.
- **Quantum selection**: change the atomic number plus `n`, `l`, `m` in the egui panel; the sampler resynthesizes the point cloud on every confirmed change.
- **Orientation cues**: p-orbitals map to cardinal axes—`m = 0`→z, `m = 1`→x, `m = -1`→y—so rotate the camera to inspect nodal planes.

## Coding Style & Naming Conventions

Format with `cargo fmt --all` and lint via `cargo clippy --all-targets -- -D warnings` before sending patches. Use four-space indentation, <100 character lines, `snake_case` for items, `PascalCase` for types, and `SCREAMING_SNAKE_CASE` for constants. Keep WGSL filenames lowercase with hyphens (`sphere.wgsl`, `cloud.wgsl`). Prefer explicit visibility, add `label:`s to GPU resources for debugging, and keep the egui UI layout logic in dedicated helpers.

## Runtime Features & Workflows

The Monte Carlo rejection sampler (`src/simulation/solver.rs`) produces weighted electron cloud vertices for the currently selected element and orbital. The egui panel (desktop build) exposes atomic number, quantum numbers (`n`, `l`, `m`), and sample count; any change triggers resampling and feeds fresh vertices into the renderer (`App::render`). The renderer composites nucleus spheres and the probabilistic cloud, then overlays the UI using a secondary render pass.

### Sampling Notes

- Hydrogenic 1s and 2s orbitals are sampled via closed-form cumulative distributions for speed.
- The 2p family (`n = 2`, `l = 1`) now uses exact hydrogenic densities, so the visualization shows the expected sandglass lobes and nodal planes without resorting to isotropic fallbacks.
- Higher orbitals still fall back to radial Gaussians; extend `Orbital::probability_density` when you need more exact shapes.

### Performance Tips

- Default sample count is 20 000; lower it while developing UI flows, then ramp it up for screenshots or demos.
- The rejection sampler expands the bounding box adaptively (up to 4×) before giving up and filling remaining points with zero-weight samples. Monitor the log for the warning if you see sparse clouds.

## Testing Guidelines

Unit tests belong next to the code under test (`#[cfg(test)] mod tests`). Current coverage validates Monte Carlo determinism. Extend with integration tests under `tests/` for renderer/platform interactions when we add headless drawing. Always run `cargo test --all-targets` before pushing. GPU-heavy tests should gate browser-only paths behind `cfg(target_arch = "wasm32")`.

## Commit & Pull Request Guidelines

Keep commits scoped and written in imperative present tense under 72 characters (e.g. `Expose orbital selector to egui`). PRs should describe motivation, outline architecture touches (renderer, physics, UI), and include manual verification (`cargo run`, `cargo test`). Attach screenshots or short clips for UI adjustments and request peer review before merge.
