use crate::physics::elements::Element;
use crate::renderer::cloud::CloudVertex;
use crate::renderer::renderer::{Renderer, SphereStyle};
use crate::simulation::atom::Atom;
use crate::simulation::ozone::{OzoneParameters, OzoneResult, OzoneSimulation};
use crate::simulation::solver::{CloudSample, MonteCarloSampler, SampleConfig};
use crate::ui::{UiState, VisualizationMode};
use glam::Vec3;
use rand::{Rng, SeedableRng, rngs::StdRng};
use winit::dpi::PhysicalSize;

pub type AppError = Box<dyn std::error::Error + Send + Sync>;
pub type AppResult<T> = Result<T, AppError>;

#[derive(Clone, Debug)]
struct PhotonParticle {
    position: Vec3,
    velocity: Vec3,
    brightness: f32,
    passed_shell: bool,
    absorbed: bool,
}

#[cfg(not(target_arch = "wasm32"))]
mod camera;
#[cfg(not(target_arch = "wasm32"))]
use crate::ui::desktop::{UiFrame, UiLayer};
#[cfg(not(target_arch = "wasm32"))]
use camera::CameraController;
#[cfg(not(target_arch = "wasm32"))]
use std::time::Instant;
#[cfg(not(target_arch = "wasm32"))]
use winit::{event::WindowEvent, window::Window};

pub struct App {
    renderer: Renderer,
    atom: Atom,
    sampler: MonteCarloSampler,
    sample_config: SampleConfig,
    cloud_vertices: Vec<CloudVertex>,
    render_vertices: Vec<CloudVertex>,
    ozone_simulation: OzoneSimulation,
    ozone_params: OzoneParameters,
    ozone_result: OzoneResult,
    current_mode: VisualizationMode,
    chapter3_shell_points: Vec<CloudVertex>,
    chapter3_photons: Vec<PhotonParticle>,
    chapter3_transmission: f32,
    chapter3_shell_radius: f32,
    chapter3_rng: StdRng,
    ui_state: UiState,
    #[cfg(not(target_arch = "wasm32"))]
    ui_layer: UiLayer,
    #[cfg(not(target_arch = "wasm32"))]
    camera_controller: CameraController,
    #[cfg(not(target_arch = "wasm32"))]
    last_frame_time: Instant,
}

impl App {
    #[cfg(not(target_arch = "wasm32"))]
    pub async fn initialize(window: &Window) -> AppResult<Self> {
        let renderer = Renderer::new(window).await?;
        let element = Element::hydrogen();
        let atom = Atom::new(element.clone());
        let mut sampler = MonteCarloSampler::new();
        let sample_config = SampleConfig::new(50_000);
        let cloud_vertices = Self::generate_cloud(&mut sampler, &atom, sample_config);
        let render_vertices = cloud_vertices.clone();

        let ozone_params = OzoneParameters::default();
        let ozone_simulation = OzoneSimulation::new();
        let ozone_result = OzoneResult::default();

        let ui_state = UiState::new(
            element.atomic_number,
            sample_config.samples,
            atom.active_orbital().clone(),
            ozone_params,
        );

        let surface_format = renderer.surface_config().format;
        let ui_layer = UiLayer::new(window, renderer.device(), surface_format);
        let camera_controller = CameraController::new(renderer.camera());
        let last_frame_time = Instant::now();

        let mut app = Self {
            renderer,
            atom,
            sampler,
            sample_config,
            cloud_vertices,
            render_vertices,
            ozone_simulation,
            ozone_params,
            ozone_result,
            current_mode: ui_state.visualization_mode,
            chapter3_shell_points: Vec::new(),
            chapter3_photons: Vec::new(),
            chapter3_transmission: 1.0,
            chapter3_shell_radius: 0.9,
            chapter3_rng: StdRng::seed_from_u64(1),
            ui_state,
            ui_layer,
            camera_controller,
            last_frame_time,
        };

        app.rebuild_render_vertices();
        Ok(app)
    }

    #[cfg(target_arch = "wasm32")]
    pub async fn initialize(canvas: &web_sys::HtmlCanvasElement) -> AppResult<Self> {
        let renderer = Renderer::new(canvas).await?;
        let element = Element::hydrogen();
        let atom = Atom::new(element.clone());
        let mut sampler = MonteCarloSampler::new();
        let sample_config = SampleConfig::new(50_000);
        let cloud_vertices = Self::generate_cloud(&mut sampler, &atom, sample_config);
        let render_vertices = cloud_vertices.clone();

        let ozone_params = OzoneParameters::default();
        let ozone_simulation = OzoneSimulation::new();
        let ozone_result = OzoneResult::default();

        let ui_state = UiState::new(
            element.atomic_number,
            sample_config.samples,
            atom.active_orbital().clone(),
            ozone_params,
        );

        let mut app = Self {
            renderer,
            atom,
            sampler,
            sample_config,
            cloud_vertices,
            render_vertices,
            ozone_simulation,
            ozone_params,
            ozone_result,
            current_mode: ui_state.visualization_mode,
            chapter3_shell_points: Vec::new(),
            chapter3_photons: Vec::new(),
            chapter3_transmission: 1.0,
            chapter3_shell_radius: 0.9,
            chapter3_rng: StdRng::seed_from_u64(1),
            ui_state,
        };

        app.rebuild_render_vertices();
        Ok(app)
    }

    pub fn resize(&mut self, new_size: PhysicalSize<u32>) {
        self.renderer.resize(new_size);
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub fn handle_event(&mut self, window: &Window, event: &WindowEvent) -> bool {
        if self.ui_layer.handle_event(window, event) {
            return true;
        }

        let mut consumed = false;

        match event {
            WindowEvent::KeyboardInput { event, .. } => {
                consumed |= self.camera_controller.handle_keyboard(event);
            }
            WindowEvent::MouseWheel { delta, .. } => {
                consumed |= self.camera_controller.handle_scroll(delta);
            }
            WindowEvent::MouseInput { state, button, .. } => {
                consumed |= self.camera_controller.handle_mouse_button(*state, *button);
            }
            WindowEvent::CursorMoved { position, .. } => {
                consumed |= self
                    .camera_controller
                    .handle_cursor_move((position.x, position.y));
            }
            _ => {}
        }

        if consumed {
            window.request_redraw();
        }

        consumed
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub fn render(&mut self, window: &Window) -> Result<(), wgpu::SurfaceError> {
        let dt = self.update_camera();
        if self.current_mode == VisualizationMode::Chapter3 {
            self.update_chapter3_physics(dt);
        }

        let ui_frame: UiFrame = {
            let (ui_layer, ui_state) = (&mut self.ui_layer, &mut self.ui_state);
            let surface_size = self.renderer.size();
            ui_layer.prepare(window, surface_size, |ctx| Self::build_ui(ctx, ui_state))
        };

        self.apply_ui_changes();

        let mut pending_frame = Some(ui_frame);
        let (renderer, ui_layer) = (&mut self.renderer, &mut self.ui_layer);
        renderer.render_with_ui(|device, queue, encoder, view| {
            if let Some(frame) = pending_frame.take() {
                ui_layer.paint(device, queue, encoder, view, frame);
            }
        })
    }

    #[cfg(target_arch = "wasm32")]
    pub fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
        if self.current_mode == VisualizationMode::Chapter3 {
            self.update_chapter3_physics(1.0 / 60.0);
        }
        self.renderer.render()
    }

    pub fn size(&self) -> PhysicalSize<u32> {
        self.renderer.size()
    }

    pub fn atom(&self) -> &Atom {
        &self.atom
    }

    pub fn resample(&mut self) {
        self.cloud_vertices =
            Self::generate_cloud(&mut self.sampler, &self.atom, self.sample_config);
    }

    fn apply_ui_changes(&mut self) {
        self.ui_state.sync_quantum_numbers();

        let mut resample_needed = false;

        let desired_atomic_number = self.ui_state.selected_atomic_number;
        if desired_atomic_number != self.atom.element().atomic_number {
            if let Some(element) = Element::by_atomic_number(desired_atomic_number) {
                self.atom = Atom::new(element.clone());
                resample_needed = true;
            } else {
                self.ui_state.selected_atomic_number = self.atom.element().atomic_number;
            }
        }

        if self.sample_config.samples != self.ui_state.sample_count {
            self.sample_config = SampleConfig::new(self.ui_state.sample_count);
            resample_needed = true;
        }

        let desired_orbital = self.ui_state.current_orbital();
        if self.atom.active_orbital() != &desired_orbital {
            self.atom.set_active_orbital(desired_orbital);
            resample_needed = true;
        }

        let mut rebuild_needed = false;
        if self.ui_state.take_resample_request() || resample_needed {
            self.resample();
            rebuild_needed = true;
        }

        if self.ui_state.take_ozone_update_request() {
            self.ozone_params.dobson_units = self.ui_state.ozone_dobson_units.clamp(100.0, 600.0);
            self.ozone_params.photon_count = self.ui_state.ozone_photon_count.max(100);
            if self.current_mode == VisualizationMode::Chapter3 {
                rebuild_needed = true;
            }
        }

        if self.ui_state.visualization_mode != self.current_mode {
            self.current_mode = self.ui_state.visualization_mode;
            rebuild_needed = true;
        }

        if rebuild_needed {
            self.rebuild_render_vertices();
        }
    }

    fn generate_cloud(
        sampler: &mut MonteCarloSampler,
        atom: &Atom,
        config: SampleConfig,
    ) -> Vec<CloudVertex> {
        sampler
            .sample_orbital(atom.element(), atom.active_orbital(), config)
            .into_iter()
            .map(cloud_vertex_from_sample)
            .collect()
    }

    #[cfg(not(target_arch = "wasm32"))]
    fn build_ui(ctx: &egui::Context, ui_state: &mut UiState) {
        use egui::{ComboBox, Slider};

        egui::Window::new("Simulation Controls")
            .default_width(260.0)
            .resizable(false)
            .show(ctx, |ui| {
                ComboBox::from_label("Scene")
                    .selected_text(ui_state.visualization_mode.label())
                    .show_ui(ui, |ui| {
                        for mode in VisualizationMode::ALL {
                            ui.selectable_value(
                                &mut ui_state.visualization_mode,
                                mode,
                                mode.label(),
                            );
                        }
                    });

                ui.separator();

                let elements = Element::all();
                let current_label = elements
                    .iter()
                    .find(|element| element.atomic_number == ui_state.selected_atomic_number)
                    .map(|element| format!("{} ({})", element.name(), element.symbol()))
                    .unwrap_or_else(|| "Unknown".to_owned());

                ComboBox::from_label("Element")
                    .selected_text(current_label)
                    .show_ui(ui, |ui| {
                        for element in elements {
                            let label = format!("{} ({})", element.name(), element.symbol());
                            if ui
                                .selectable_label(
                                    ui_state.selected_atomic_number == element.atomic_number,
                                    label,
                                )
                                .clicked()
                            {
                                ui_state.selected_atomic_number = element.atomic_number;
                                ui_state.request_resample();
                            }
                        }
                    });

                ui.separator();

                let mut principal = ui_state.principal_n;
                if ui
                    .add(Slider::new(&mut principal, 1..=6).text("Principal (n)"))
                    .changed()
                {
                    ui_state.principal_n = principal;
                    ui_state.sync_quantum_numbers();
                    ui_state.request_resample();
                }

                let l_max = ui_state.principal_n.saturating_sub(1);
                let mut angular = ui_state.angular_l.min(l_max);
                if ui
                    .add(Slider::new(&mut angular, 0..=l_max).text("Azimuthal (l)"))
                    .changed()
                {
                    ui_state.angular_l = angular;
                    ui_state.sync_quantum_numbers();
                    ui_state.request_resample();
                }

                let m_limit = ui_state.angular_l as i8;
                let mut magnetic = ui_state.magnetic_m.clamp(-m_limit, m_limit);
                if ui
                    .add(Slider::new(&mut magnetic, -m_limit..=m_limit).text("Magnetic (m)"))
                    .changed()
                {
                    ui_state.magnetic_m = magnetic;
                    ui_state.request_resample();
                }

                ui.separator();

                let mut samples = ui_state.sample_count.max(5_000);
                if ui
                    .add(Slider::new(&mut samples, 5_000..=250_000).text("Samples"))
                    .changed()
                {
                    ui_state.sample_count = samples;
                    ui_state.request_resample();
                }

                if ui.button("Resample").clicked() {
                    ui_state.request_resample();
                }

                if ui_state.visualization_mode == VisualizationMode::Chapter3 {
                    ui.separator();
                    ui.heading("Ozone Filter");

                    let mut dobson = ui_state.ozone_dobson_units;
                    if ui
                        .add(Slider::new(&mut dobson, 100.0..=600.0).text("Ozone (DU)"))
                        .changed()
                    {
                        ui_state.ozone_dobson_units = dobson;
                        ui_state.request_ozone_update();
                    }

                    let mut photons = ui_state.ozone_photon_count as u32;
                    if ui
                        .add(Slider::new(&mut photons, 1_000..=50_000).text("Photon Samples"))
                        .changed()
                    {
                        ui_state.ozone_photon_count = photons as usize;
                        ui_state.request_ozone_update();
                    }

                    ui.separator();
                    let flux = ui_state.ozone_flux;
                    let incident_total = flux.incident_total();
                    let transmitted_total = flux.transmitted_total();
                    let percent = if incident_total > 0.0 {
                        (transmitted_total / incident_total) * 100.0
                    } else {
                        0.0
                    };
                    ui.label(format!("Incident UVA: {:.0}", flux.incident_uva));
                    ui.label(format!("Incident UVB: {:.0}", flux.incident_uvb));
                    ui.label(format!("Transmitted UVA: {:.0}", flux.transmitted_uva));
                    ui.label(format!("Transmitted UVB: {:.0}", flux.transmitted_uvb));
                    ui.label(format!("Surface transmission: {:.1}%", percent));
                }
            });
    }

    fn rebuild_render_vertices(&mut self) {
        self.render_vertices = match self.current_mode {
            VisualizationMode::Chapter1 => self.build_chapter1_vertices(),
            VisualizationMode::Chapter2 => self.build_chapter2_vertices(),
            VisualizationMode::Chapter3 => self.build_chapter3_vertices(),
        };
        self.renderer.update_cloud(&self.render_vertices);
    }

    fn build_chapter1_vertices(&mut self) -> Vec<CloudVertex> {
        self.renderer.update_earth_style(SphereStyle::default());
        self.renderer.update_shell_style(SphereStyle::hidden());
        self.chapter3_shell_points.clear();
        self.chapter3_photons.clear();

        let mut vertices = self.cloud_vertices.clone();
        for vertex in &mut vertices {
            vertex.kind = 0;
            vertex.weight = vertex.weight.clamp(0.0, 1.0);
        }
        self.ozone_result = OzoneResult::default();
        self.ui_state.set_ozone_flux(self.ozone_result.flux);
        vertices
    }

    fn build_chapter2_vertices(&mut self) -> Vec<CloudVertex> {
        self.renderer.update_earth_style(SphereStyle::hidden());
        self.renderer.update_shell_style(SphereStyle::hidden());
        self.chapter3_shell_points.clear();
        self.chapter3_photons.clear();

        let mut result = Vec::with_capacity(self.cloud_vertices.len() * 3 + 32);
        for (index, base) in self.cloud_vertices.iter().enumerate() {
            let base_vec = Vec3::from_array(base.position) * 0.85;
            let left = base_vec + Vec3::new(-1.2, 0.0, 0.0);
            let right = base_vec + Vec3::new(1.2, 0.0, 0.0);
            let weight = base.weight.clamp(0.0, 1.0);
            result.push(CloudVertex::new(left, weight, 1));
            result.push(CloudVertex::new(right, weight, 2));

            if index % 6 == 0 {
                let bridge = base_vec * 0.35;
                let bridge_weight = (base.weight * 0.6 + 0.25).clamp(0.0, 0.8);
                result.push(CloudVertex::new(bridge, bridge_weight, 3));
            }
        }

        let nucleus_positions = [Vec3::new(-1.2, 0.0, 0.0), Vec3::new(1.2, 0.0, 0.0)];
        for pos in nucleus_positions {
            for &offset in &[
                Vec3::ZERO,
                Vec3::new(0.05, 0.05, 0.0),
                Vec3::new(-0.05, -0.04, 0.02),
                Vec3::new(0.03, -0.05, -0.03),
            ] {
                result.push(CloudVertex::new(pos + offset, 1.3, 4));
            }
        }

        self.ozone_result = OzoneResult::default();
        self.ui_state.set_ozone_flux(self.ozone_result.flux);
        result
    }

    fn build_chapter3_vertices(&mut self) -> Vec<CloudVertex> {
        self.renderer.update_earth_style(SphereStyle {
            visible: true,
            radius: 0.5,
            color: [0.15, 0.35, 0.95, 1.0],
        });

        let thickness = (self.ozone_params.dobson_units / 600.0).clamp(0.2, 1.0);
        self.chapter3_shell_radius = 0.75 + thickness * 0.25;
        self.renderer.update_shell_style(SphereStyle {
            visible: true,
            radius: self.chapter3_shell_radius,
            color: [0.35, 0.85, 1.0, 0.35 + thickness * 0.4],
        });

        let shell_lat = 40usize;
        let shell_lon = 80usize;
        self.chapter3_shell_points = Vec::with_capacity(shell_lat * shell_lon);
        for lat in 0..shell_lat {
            let v = lat as f32 / shell_lat as f32;
            let theta = v * std::f32::consts::PI;
            let sin_theta = theta.sin();
            let cos_theta = theta.cos();
            for lon in 0..shell_lon {
                let u = lon as f32 / shell_lon as f32;
                let phi = u * std::f32::consts::TAU;
                let dir = Vec3::new(sin_theta * phi.cos(), cos_theta, sin_theta * phi.sin());
                let pos = dir * self.chapter3_shell_radius;
                self.chapter3_shell_points
                    .push(CloudVertex::new(pos, 0.4, 6));
            }
        }

        let ozone_result = self
            .ozone_simulation
            .run(self.ozone_params, self.chapter3_shell_points.len());
        self.ozone_result = ozone_result.clone();
        self.ui_state.set_ozone_flux(ozone_result.flux);

        let absorption = ozone_result.flux.absorbed_fraction();
        let highlight_strength = (1.3 + absorption * 0.9).clamp(1.3, 2.4);
        for (idx, vertex) in self.chapter3_shell_points.iter_mut().enumerate() {
            vertex.weight = (0.25 + absorption * 0.55).clamp(0.2, 1.0);
            if ozone_result.highlight_indices.contains(&idx) {
                vertex.kind = 7;
                vertex.weight = highlight_strength;
            }
        }

        let transmission = if ozone_result.flux.incident_total() > 0.0 {
            ozone_result.flux.transmitted_total() / ozone_result.flux.incident_total()
        } else {
            1.0
        }
        .clamp(0.0, 1.0);
        self.chapter3_transmission = transmission;

        let photon_count =
            (self.ozone_params.photon_count as f32 / 400.0).clamp(60.0, 400.0) as usize;
        self.chapter3_photons = (0..photon_count)
            .map(|_| {
                Self::spawn_photon_particle_from_rng(
                    &mut self.chapter3_rng,
                    self.chapter3_shell_radius,
                )
            })
            .collect();

        self.compose_chapter3_vertices()
    }
    fn compose_chapter3_vertices(&self) -> Vec<CloudVertex> {
        let mut vertices = self.chapter3_shell_points.clone();
        for photon in &self.chapter3_photons {
            let kind = if photon.absorbed { 7 } else { 8 };
            let weight = photon.brightness.clamp(0.0, 2.0);
            vertices.push(CloudVertex::new(photon.position, weight, kind));
        }
        vertices
    }

    fn spawn_photon_particle_from_rng(rng: &mut StdRng, shell_radius: f32) -> PhotonParticle {
        use std::f32::consts::TAU;
        let angle = rng.gen_range(0.0..TAU);
        let radial = rng.gen_range(0.2..0.7);
        let start_height = shell_radius + rng.gen_range(0.3..0.6);
        let x = radial * angle.cos();
        let z = radial * angle.sin();
        let position = Vec3::new(x, start_height, z);
        let direction = (-position).normalize_or_zero();
        let speed = rng.gen_range(0.6..1.4);
        PhotonParticle {
            position,
            velocity: direction * speed,
            brightness: 1.2,
            passed_shell: false,
            absorbed: false,
        }
    }

    fn update_chapter3_physics(&mut self, dt: f32) {
        if self.current_mode != VisualizationMode::Chapter3 {
            return;
        }

        if self.chapter3_photons.is_empty() {
            return;
        }

        let shell_radius = self.chapter3_shell_radius.max(0.1);
        let absorb_prob = (1.0 - self.chapter3_transmission).clamp(0.0, 1.0);
        let transmission = self.chapter3_transmission;
        let rng = &mut self.chapter3_rng;

        for photon in &mut self.chapter3_photons {
            if photon.absorbed {
                photon.brightness *= (1.0 - dt * 1.5).max(0.0);
                if photon.brightness < 0.1 {
                    *photon = Self::spawn_photon_particle_from_rng(rng, shell_radius);
                }
                continue;
            }

            photon.position += photon.velocity * dt;
            let radius = photon.position.length();

            if !photon.passed_shell && radius <= shell_radius {
                if rng.r#gen::<f32>() < absorb_prob {
                    photon.absorbed = true;
                    photon.passed_shell = true;
                    photon.velocity = Vec3::ZERO;
                    if radius > 0.0 {
                        photon.position = photon.position.normalize() * shell_radius;
                    }
                    photon.brightness = 1.8;
                    continue;
                } else {
                    photon.passed_shell = true;
                    photon.brightness *= transmission.max(0.05);
                }
            }

            if photon.passed_shell {
                photon.brightness *= (1.0 - dt * (1.0 - transmission)).clamp(0.2, 1.0);
            }

            if radius <= 0.55 || radius > 3.0 {
                *photon = Self::spawn_photon_particle_from_rng(rng, shell_radius);
            }
        }

        self.render_vertices = self.compose_chapter3_vertices();
        self.renderer.update_cloud(&self.render_vertices);
    }

    #[cfg(not(target_arch = "wasm32"))]
    fn update_camera(&mut self) -> f32 {
        let now = Instant::now();
        let dt = (now - self.last_frame_time).as_secs_f32();
        self.last_frame_time = now;
        self.camera_controller.update(&mut self.renderer, dt);
        dt
    }
}

fn cloud_vertex_from_sample(sample: CloudSample) -> CloudVertex {
    CloudVertex::new(sample.position, sample.weight, 0)
}
