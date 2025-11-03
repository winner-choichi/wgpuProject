use crate::physics::electron::Orbital;
use crate::simulation::ozone::{OzoneParameters, RadiationFlux};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum VisualizationMode {
    Chapter1,
    Chapter2,
    Chapter3,
}

impl VisualizationMode {
    pub const ALL: [Self; 3] = [Self::Chapter1, Self::Chapter2, Self::Chapter3];

    pub fn label(self) -> &'static str {
        match self {
            Self::Chapter1 => "Chapter 1 — Atomic Orbital",
            Self::Chapter2 => "Chapter 2 — Bond Visualization",
            Self::Chapter3 => "Chapter 3 — Ozone Shield",
        }
    }
}

#[derive(Clone, Debug)]
pub struct UiState {
    pub selected_atomic_number: u8,
    pub principal_n: u8,
    pub angular_l: u8,
    pub magnetic_m: i8,
    pub sample_count: usize,
    pub visualization_mode: VisualizationMode,
    pub ozone_dobson_units: f32,
    pub ozone_photon_count: usize,
    pub ozone_flux: RadiationFlux,
    resample_requested: bool,
    ozone_update_requested: bool,
}

impl UiState {
    pub fn new(
        selected_atomic_number: u8,
        sample_count: usize,
        orbital: Orbital,
        ozone_params: OzoneParameters,
    ) -> Self {
        Self {
            selected_atomic_number,
            principal_n: orbital.n,
            angular_l: orbital.l,
            magnetic_m: orbital.m,
            sample_count,
            visualization_mode: VisualizationMode::Chapter1,
            ozone_dobson_units: ozone_params.dobson_units,
            ozone_photon_count: ozone_params.photon_count,
            ozone_flux: RadiationFlux::default(),
            resample_requested: false,
            ozone_update_requested: false,
        }
    }

    pub fn current_orbital(&self) -> Orbital {
        Orbital::new(self.principal_n, self.angular_l, self.magnetic_m)
    }

    pub fn request_resample(&mut self) {
        self.resample_requested = true;
    }

    pub fn take_resample_request(&mut self) -> bool {
        let requested = self.resample_requested;
        self.resample_requested = false;
        requested
    }

    pub fn request_ozone_update(&mut self) {
        self.ozone_update_requested = true;
    }

    pub fn take_ozone_update_request(&mut self) -> bool {
        let requested = self.ozone_update_requested;
        self.ozone_update_requested = false;
        requested
    }

    pub fn set_ozone_flux(&mut self, flux: RadiationFlux) {
        self.ozone_flux = flux;
    }

    pub fn sync_quantum_numbers(&mut self) {
        if self.principal_n == 0 {
            self.principal_n = 1;
        }

        if self.angular_l >= self.principal_n {
            self.angular_l = self.principal_n.saturating_sub(1);
        }

        let max_m = self.angular_l as i8;
        if max_m > 0 {
            self.magnetic_m = self.magnetic_m.clamp(-max_m, max_m);
        } else {
            self.magnetic_m = 0;
        }
    }
}

#[cfg(not(target_arch = "wasm32"))]
pub mod desktop {
    use egui::ClippedPrimitive;
    use egui_wgpu::{Renderer, ScreenDescriptor};
    use egui_winit::{State as EguiWinitState, pixels_per_point};
    use wgpu::{CommandEncoder, Device, Queue, TextureFormat, TextureView};
    use winit::{dpi::PhysicalSize, event::WindowEvent, window::Window};

    pub struct UiLayer {
        ctx: egui::Context,
        state: EguiWinitState,
        renderer: Renderer,
        screen_desc: ScreenDescriptor,
    }

    pub struct UiFrame {
        pub shapes: Vec<ClippedPrimitive>,
        pub textures_delta: egui::TexturesDelta,
    }

    impl UiLayer {
        pub fn new(window: &Window, device: &Device, surface_format: TextureFormat) -> Self {
            let ctx = egui::Context::default();
            let state = EguiWinitState::new(
                ctx.clone(),
                egui::ViewportId::ROOT,
                window,
                Some(window.scale_factor() as f32),
                None,
            );

            let mut layer = Self {
                ctx,
                state,
                renderer: Renderer::new(device, surface_format, None, 1),
                screen_desc: ScreenDescriptor {
                    size_in_pixels: [1, 1],
                    pixels_per_point: 1.0,
                },
            };
            layer.update_screen_descriptor(window);
            layer
        }

        pub fn handle_event(&mut self, window: &Window, event: &WindowEvent) -> bool {
            let response = self.state.on_window_event(window, event);
            if response.repaint {
                window.request_redraw();
            }
            response.consumed
        }

        pub fn prepare<F>(
            &mut self,
            window: &Window,
            surface_size: PhysicalSize<u32>,
            mut build_ui: F,
        ) -> UiFrame
        where
            F: FnMut(&egui::Context),
        {
            self.update_screen_descriptor(window);
            self.screen_desc.size_in_pixels =
                [surface_size.width.max(1), surface_size.height.max(1)];
            let raw_input = self.state.take_egui_input(window);
            let full_output = self.ctx.run(raw_input, |ctx| build_ui(ctx));
            self.state
                .handle_platform_output(window, full_output.platform_output);

            self.screen_desc.pixels_per_point = full_output.pixels_per_point;

            let shapes = self
                .ctx
                .tessellate(full_output.shapes, self.screen_desc.pixels_per_point);

            UiFrame {
                shapes,
                textures_delta: full_output.textures_delta,
            }
        }

        pub fn paint(
            &mut self,
            device: &Device,
            queue: &Queue,
            encoder: &mut CommandEncoder,
            view: &TextureView,
            frame: UiFrame,
        ) {
            let UiFrame {
                shapes,
                mut textures_delta,
            } = frame;

            for (id, image_delta) in textures_delta.set.drain(..) {
                self.renderer
                    .update_texture(device, queue, id, &image_delta);
            }

            let callback_buffers =
                self.renderer
                    .update_buffers(device, queue, encoder, &shapes, &self.screen_desc);

            if !callback_buffers.is_empty() {
                queue.submit(callback_buffers);
            }

            {
                let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                    label: Some("egui-ui-pass"),
                    color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                        view,
                        resolve_target: None,
                        ops: wgpu::Operations {
                            load: wgpu::LoadOp::Load,
                            store: wgpu::StoreOp::Store,
                        },
                    })],
                    depth_stencil_attachment: None,
                    timestamp_writes: None,
                    occlusion_query_set: None,
                });
                self.renderer
                    .render(&mut render_pass, &shapes, &self.screen_desc);
            }

            for id in textures_delta.free.drain(..) {
                self.renderer.free_texture(&id);
            }
        }

        pub fn ctx(&self) -> &egui::Context {
            &self.ctx
        }

        fn update_screen_descriptor(&mut self, window: &Window) {
            let size = window.inner_size();
            self.screen_desc.pixels_per_point = pixels_per_point(&self.ctx, window);
            self.screen_desc.size_in_pixels = [size.width.max(1), size.height.max(1)];
        }
    }
}
