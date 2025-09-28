# Repository Guidelines

## Project Structure & Module Organization
The project delivers a cross-platform atomic orbital visualizer. `src/main.rs` starts a native window, while `src/platform.rs` abstracts surface creation for native, web, and headless providers. Shared exports are re-exported from `src/lib.rs`. The rendering stack lives under `src/renderer/` (camera, mesh generation, cloud point pipeline, and WGSL shaders in `src/shaders/`). Domain logic is split into `src/physics/` (element data, nucleus modelling, orbital math) and `src/simulation/` (atom aggregation plus Monte Carlo solvers). `src/app/` orchestrates renderer + simulation state, and `src/ui/` integrates the egui control panel. Build artefacts collect in `target/` and must remain untracked.

## Build, Test, and Development Commands
- `cargo check` — fast sanity pass across all targets.
- `cargo run` — launch the native app with interactive controls.
- `cargo build --release` — emit an optimized build for demos or profiling.
- `cargo build --target wasm32-unknown-unknown` — compile the WebGL/WebGPU variant (served via your bundler).
- `RUST_LOG=wgpu=trace cargo run` — surface device-level debugging while diagnosing GPU issues.

## Coding Style & Naming Conventions
Format with `cargo fmt --all` and lint via `cargo clippy --all-targets -- -D warnings` before sending patches. Use four-space indentation, <100 character lines, `snake_case` for items, `PascalCase` for types, and `SCREAMING_SNAKE_CASE` for constants. Keep WGSL filenames lowercase with hyphens (`sphere.wgsl`, `cloud.wgsl`). Prefer explicit visibility, add `label:`s to GPU resources for debugging, and keep the egui UI layout logic in dedicated helpers.

## Runtime Features & Workflows
The Monte Carlo rejection sampler (`src/simulation/solver.rs`) produces weighted electron cloud vertices for the currently selected element and orbital. The egui panel (desktop build) exposes atomic number, quantum numbers (`n`, `l`, `m`), and sample count; any change triggers resampling and feeds fresh vertices into the renderer (`App::render`). The renderer composites nucleus spheres and the probabilistic cloud, then overlays the UI using a secondary render pass.

## Testing Guidelines
Unit tests belong next to the code under test (`#[cfg(test)] mod tests`). Current coverage validates Monte Carlo determinism. Extend with integration tests under `tests/` for renderer/platform interactions when we add headless drawing. Always run `cargo test --all-targets` before pushing. GPU-heavy tests should gate browser-only paths behind `cfg(target_arch = "wasm32")`.

## Commit & Pull Request Guidelines
Keep commits scoped and written in imperative present tense under 72 characters (e.g. `Expose orbital selector to egui`). PRs should describe motivation, outline architecture touches (renderer, physics, UI), and include manual verification (`cargo run`, `cargo test`). Attach screenshots or short clips for UI adjustments and request peer review before merge.
