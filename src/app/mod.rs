use crate::physics::elements::Element;
use crate::renderer::cloud::CloudVertex;
use crate::renderer::renderer::Renderer;
use crate::simulation::atom::Atom;
use crate::simulation::solver::{CloudSample, MonteCarloSampler, SampleConfig};
use crate::ui::UiState;
use winit::dpi::PhysicalSize;

pub type AppError = Box<dyn std::error::Error + Send + Sync>;
pub type AppResult<T> = Result<T, AppError>;

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
        let mut renderer = Renderer::new(window).await?;
        let element = Element::hydrogen();
        let atom = Atom::new(element.clone());
        let mut sampler = MonteCarloSampler::new();
        let sample_config = SampleConfig::new(50_000);
        let cloud_vertices = Self::generate_cloud(&mut sampler, &atom, sample_config);
        renderer.update_cloud(&cloud_vertices);

        let ui_state = UiState::new(
            element.atomic_number,
            sample_config.samples,
            atom.active_orbital().clone(),
        );

        let surface_format = renderer.surface_config().format;
        let ui_layer = UiLayer::new(window, renderer.device(), surface_format);
        let camera_controller = CameraController::new(renderer.camera());
        let last_frame_time = Instant::now();

        Ok(Self {
            renderer,
            atom,
            sampler,
            sample_config,
            cloud_vertices,
            ui_state,
            ui_layer,
            camera_controller,
            last_frame_time,
        })
    }

    #[cfg(target_arch = "wasm32")]
    pub async fn initialize(canvas: &web_sys::HtmlCanvasElement) -> AppResult<Self> {
        let mut renderer = Renderer::new(canvas).await?;
        let element = Element::hydrogen();
        let atom = Atom::new(element.clone());
        let mut sampler = MonteCarloSampler::new();
        let sample_config = SampleConfig::new(50_000);
        let cloud_vertices = Self::generate_cloud(&mut sampler, &atom, sample_config);
        renderer.update_cloud(&cloud_vertices);

        let ui_state = UiState::new(
            element.atomic_number,
            sample_config.samples,
            atom.active_orbital().clone(),
        );

        Ok(Self {
            renderer,
            atom,
            sampler,
            sample_config,
            cloud_vertices,
            ui_state,
        })
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
        self.update_camera();

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
        self.renderer.update_cloud(&self.cloud_vertices);
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

        if self.ui_state.take_resample_request() || resample_needed {
            self.resample();
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
            .default_width(240.0)
            .resizable(false)
            .show(ctx, |ui| {
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
            });
    }

    #[cfg(not(target_arch = "wasm32"))]
    fn update_camera(&mut self) {
        let now = Instant::now();
        let dt = (now - self.last_frame_time).as_secs_f32();
        self.last_frame_time = now;
        self.camera_controller.update(&mut self.renderer, dt);
    }
}

fn cloud_vertex_from_sample(sample: CloudSample) -> CloudVertex {
    CloudVertex::new(sample.position, sample.weight)
}
