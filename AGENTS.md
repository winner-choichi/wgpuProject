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

## Simulation & Quantum Model

The Monte Carlo pipeline (`src/simulation/solver.rs`) turns hydrogenic wavefunctions into draw-ready `CloudVertex` buffers. We blend analytic sampling with rejection fallback so the common orbitals are fast while the code stays extensible.

- **Analytic draws**: 1s and 2s states use their closed-form radial CDFs (gamma distribution over `r` plus uniform solid angle). For 2p (`n = 2`, `l = 1`, `m ∈ {-1,0,1}`) we sample `r` ~ Γ(k=3, θ=a₀/2) and project along the correct angular factor (`Y₁m`). We store weights as √(ρ/ρₘₐₓ) to preserve relative opacity without saturating bright regions.
- **Fallback rejection**: Orbitals without an explicit sampler draw inside a cubic bounding box sized by `Orbital::bounding_radius`. We expand the box up to four times if acceptance stalls, then back-fill with zero-weight samples (look for the warning in logs when that happens).
- **Quantum reference**: The hydrogenic radial functions implemented match textbook forms: R₁₀(r)=2 a₀^{-3/2} e^{-r/a₀}, R₂₀(r)=(1/2√2)a₀^{-3/2}(2−r/a₀)e^{-r/2a₀}, R₂₁(r)=(1/2√6)a₀^{-3/2}(r/a₀)e^{-r/2a₀}. Effective Bohr radius scales as a₀/Z, so Helium renders tighter clouds than Hydrogen.

## Runtime Features & Workflows

The egui panel (desktop build) exposes atomic number, quantum numbers (`n`, `l`, `m`), and sample count; any change triggers resampling and feeds fresh vertices into the renderer (`App::render`). The renderer composites nucleus spheres, probabilistic clouds, and the UI in sequence. Clamped surface sizes ensure we stay within adapter texture limits even on ultra-wide windows, and warnings are logged when the requested extent exceeds the GPU’s maximum dimension.

### Performance Tips

- Default sample count is 20 000; lower it while developing UI flows, then ramp it up for screenshots or demos.
- Monte Carlo loops pre-allocate exact capacities and reuse RNG state to avoid per-frame allocations. Rejection boxes expand adaptively (up to 4×) before falling back to filler samples.
- Camera input is velocity-smoothed (`acceleration = damping = 12`), so quick taps create gentle nudges while longer holds reach cruise speed. Right-click drag updates yaw/pitch; WASD+Space/Shift move in the camera’s local basis; scroll performs a dolly with clamped near/far distances.

## Testing Guidelines

Unit tests belong next to the code under test (`#[cfg(test)] mod tests`). Current coverage validates Monte Carlo determinism. Extend with integration tests under `tests/` for renderer/platform interactions when we add headless drawing. Always run `cargo test --all-targets` before pushing. GPU-heavy tests should gate browser-only paths behind `cfg(target_arch = "wasm32")`.

## Commit & Pull Request Guidelines

Keep commits scoped and written in imperative present tense under 72 characters (e.g. `Expose orbital selector to egui`). PRs should describe motivation, outline architecture touches (renderer, physics, UI), and include manual verification (`cargo run`, `cargo test`). Attach screenshots or short clips for UI adjustments and request peer review before merge.
