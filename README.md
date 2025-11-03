# Atomic Orbital Visualizer

A cross-platform atomic orbital and molecular visualization application built with Rust and wgpu. This educational tool demonstrates quantum mechanics, molecular chemistry, and atmospheric physics through GPU-accelerated real-time rendering.

![Platform](https://img.shields.io/badge/platform-Windows%20%7C%20macOS%20%7C%20Linux%20%7C%20Web-blue)
![Rust](https://img.shields.io/badge/rust-2024-orange)
![wgpu](https://img.shields.io/badge/wgpu-0.20-green)

## Features

### Chapter 1: Quantum Orbital Visualization

- **Hydrogenic Orbital Rendering**: Visualize electron probability distributions using Monte Carlo sampling
- **Quantum Number Control**: Interactive selection of n (principal), l (azimuthal), and m (magnetic) quantum numbers
- **Multiple Elements**: Support for Hydrogen (Z=1) and Helium (Z=2)
- **Implemented Orbitals**:
  - 1s (ground state) - analytic Gamma distribution sampling
  - 2s - radial nodes with analytic sampling
  - 2p (m = -1, 0, 1) - angular dependencies
  - Higher orbitals - adaptive rejection sampling
- **Accurate Physics**: Wavefunction probability densities |ψ|² with proper Bohr radius scaling

### Chapter 2: Molecular Bond Visualization

- **Dual-Atom System**: Visualize electron cloud overlap between bonded atoms
- **Covalent Bonding**: Bridge electrons showing shared electron pairs between nuclei
- **Color-Coded Orbitals**:
  - Purple-ish: Left atom orbital
  - Cyan: Right atom orbital
  - Teal: Bridge electrons
  - White/Yellow: Nucleus particles

### Chapter 3: Ozone Layer Simulation

- **UV Radiation Physics**: Simplified atmospheric ozone filtering simulation
- **Beer-Lambert Absorption**: Realistic UV-A and UV-B photon tracking
- **Interactive Parameters**:
  - Ozone concentration (100-600 Dobson Units)
  - Photon count (1,000-50,000)
- **Real-time Statistics**: Surface transmission percentages and absorption rates
- **Visual Components**:
  - Earth sphere (blue, radius 0.5)
  - Ozone shell (cyan translucent, thickness varies with concentration)
  - Animated photons with absorption highlights

## Technologies

- **wgpu 0.20**: Modern GPU API abstraction (Vulkan, Metal, DX12, WebGL2)
- **winit 0.29**: Cross-platform windowing
- **egui 0.28**: Immediate mode GUI for desktop controls
- **glam 0.27**: SIMD-optimized vector/matrix math
- **rand**: Monte Carlo sampling with ChaCha8 PRNG
- **WebAssembly**: Browser support via wasm-bindgen

## Building and Running

### Native Desktop

```bash
cargo run --release
```

### WebAssembly

```bash
# Install wasm-pack if not already installed
cargo install wasm-pack

# Build for web
wasm-pack build --target web --out-dir www/pkg

# Serve locally (requires a web server)
# For example, using Python:
cd www
python3 -m http.server 8080
```

Then open `http://localhost:8080` in your browser.

## Controls

### Camera (Desktop)
- **WASD**: Move forward/back and strafe left/right
- **Space/Shift**: Move up/down
- **Right-click + Drag**: Rotate camera (pitch/yaw)
- **Mouse Wheel**: Zoom in/out

### UI Panel
- **Scene Selection**: Choose between quantum orbitals, molecular bonds, or ozone simulation
- **Element Picker**: Select hydrogen or helium
- **Quantum Numbers**: Adjust n, l, m values via sliders
- **Sample Count**: Control cloud density (5,000-250,000 points)
- **Resample**: Regenerate electron cloud with new parameters

## Architecture

### Project Structure

```
src/
├── main.rs              # Entry point (native)
├── lib.rs               # Public API (library/WASM)
├── platform.rs          # Cross-platform surface abstraction
├── constants.rs         # Physical constants
├── app/
│   └── mod.rs          # Main application orchestrator
├── renderer/
│   ├── renderer.rs     # Core wgpu renderer
│   ├── camera.rs       # Camera system with view-projection
│   ├── cloud.rs        # Point cloud renderer
│   ├── mesh.rs         # UV sphere mesh generation
│   └── vertex.rs       # Vertex data structures
├── physics/
│   ├── electron.rs     # Orbital wavefunctions
│   ├── elements.rs     # Chemical element metadata
│   ├── nucleus.rs      # Nucleus with Fibonacci sphere packing
│   └── particle.rs     # Common particle trait
├── simulation/
│   ├── atom.rs         # Atomic structure
│   ├── solver.rs       # Monte Carlo sampler
│   └── mod.rs          # Ozone layer simulation
├── ui/
│   └── mod.rs          # egui integration
└── shaders/
    ├── sphere.wgsl     # Solid sphere rendering
    └── cloud.wgsl      # Point cloud with vertex coloring
```

### Key Design Patterns

- **Separation of Concerns**: Clear boundaries between physics, rendering, simulation, and UI
- **Trait Abstractions**: `Particle` trait, `SurfaceProvider` for platform independence
- **Adaptive Algorithms**: Monte Carlo sampler with analytic fast-paths and rejection fallback
- **GPU-First Design**: Heavy computation in shaders for performance

## Performance

- **Sample Generation**: 50,000 points typically render in sub-second time
- **Frame Rate**: 60+ FPS with smooth camera movement
- **Memory**: Dynamic buffer resizing with power-of-2 capacity growth
- **Physics**: 60 Hz fixed timestep for photon simulation

## Implementation Highlights

### Monte Carlo Sampling Strategy

Two-tier approach for efficiency:
1. **Analytic Sampling** (fast): Gamma distribution for 1s/2s orbitals, projected spherical harmonics for 2p
2. **Rejection Sampling** (fallback): Adaptive cubic bounding box with expansion for higher orbitals

### Rendering Pipeline

Dual-pipeline architecture:
1. **Sphere Pipeline**: Indexed triangle meshes with alpha blending for nucleus and shells
2. **Cloud Pipeline**: Point list rendering with per-vertex kind/weight attributes for electron clouds

### Quantum Mechanics Accuracy

- Proper hydrogenic wavefunction probability densities
- Effective Bohr radius scaling: a₀/Z (accounts for nuclear charge)
- Weight visualization using √(ρ/ρₘₐₓ) for balanced opacity

## License

This is a personal educational project.

## Acknowledgments

Built with the amazing Rust graphics ecosystem, particularly the wgpu team for providing excellent cross-platform GPU abstractions.
