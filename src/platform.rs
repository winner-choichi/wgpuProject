use wgpu::{Instance, Surface};
use winit::dpi::PhysicalSize;

use crate::App;

/// Platform-specific ability to construct a surface for rendering.
pub trait SurfaceProvider {
    fn create_surface(
        &self,
        instance: &Instance,
    ) -> Result<
        (Option<Surface<'static>>, PhysicalSize<u32>),
        Box<dyn std::error::Error + Send + Sync>,
    >;
}

#[cfg(not(target_arch = "wasm32"))]
impl SurfaceProvider for winit::window::Window {
    fn create_surface(
        &self,
        instance: &Instance,
    ) -> Result<
        (Option<Surface<'static>>, PhysicalSize<u32>),
        Box<dyn std::error::Error + Send + Sync>,
    > {
        let surface = instance.create_surface(self)?;
        let size = self.inner_size();
        let static_surface = unsafe { std::mem::transmute(surface) };
        Ok((Some(static_surface), size))
    }
}

#[cfg(not(target_arch = "wasm32"))]
pub fn start() {
    use pollster::block_on;
    use winit::event::{Event, WindowEvent};
    use winit::event_loop::{ControlFlow, EventLoop};
    use winit::window::WindowBuilder;

    let _ = env_logger::builder().is_test(true).try_init();

    let event_loop = EventLoop::new().expect("Failed to create event loop");
    let window = WindowBuilder::new()
        .with_title("Atomic Orbital Visualizer")
        .build(&event_loop)
        .expect("Failed to create window");

    let mut app = block_on(App::initialize(&window)).expect("Failed to initialise renderer app");
    window.request_redraw();

    let _ = event_loop.run(move |event, target| {
        target.set_control_flow(ControlFlow::Poll);

        match event {
            Event::WindowEvent { event, .. } => {
                app.handle_event(&window, &event);

                match event {
                    WindowEvent::CloseRequested => target.exit(),
                    WindowEvent::Resized(size) => app.resize(size),
                    WindowEvent::ScaleFactorChanged { .. } => {
                        app.resize(window.inner_size());
                    }
                    WindowEvent::RedrawRequested => {
                        if let Err(err) = app.render(&window) {
                            match err {
                                wgpu::SurfaceError::Lost => app.resize(app.size()),
                                wgpu::SurfaceError::OutOfMemory => target.exit(),
                                wgpu::SurfaceError::Outdated | wgpu::SurfaceError::Timeout => {}
                            }
                        }
                    }
                    _ => {}
                }
            }
            Event::AboutToWait => {
                window.request_redraw();
            }
            _ => {}
        }
    });
}

#[cfg(target_arch = "wasm32")]
use {wasm_bindgen::JsCast, wasm_bindgen::prelude::*, wasm_bindgen_futures::spawn_local};

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen(start)]
pub fn start() {
    console_log::init_with_level(log::Level::Debug).expect("Couldn't initialize logger");
    console_error_panic_hook::set_once();

    let window = web_sys::window().expect("no global `window` exists");
    let document = window.document().expect("should have a document on window");
    let canvas = document
        .get_element_by_id("canvas")
        .expect("canvas element");
    let canvas: web_sys::HtmlCanvasElement = canvas.dyn_into().unwrap();

    canvas.set_width(800);
    canvas.set_height(600);

    spawn_local(async move {
        match App::initialize(&canvas).await {
            Ok(mut app) => {
                if let Err(err) = app.render() {
                    log::error!("Initial render failed: {:?}", err);
                }
            }
            Err(err) => {
                log::error!("Failed to initialise app: {:?}", err);
            }
        }
    });
}

#[cfg(target_arch = "wasm32")]
impl SurfaceProvider for web_sys::HtmlCanvasElement {
    fn create_surface(
        &self,
        instance: &Instance,
    ) -> Result<
        (Option<Surface<'static>>, PhysicalSize<u32>),
        Box<dyn std::error::Error + Send + Sync>,
    > {
        let surface = instance.create_surface(wgpu::SurfaceTarget::Canvas(self.clone()))?;
        let static_surface = unsafe { std::mem::transmute(surface) };
        let size = PhysicalSize::new(self.width(), self.height());
        Ok((Some(static_surface), size))
    }
}

pub struct HeadlessProvider {
    pub width: u32,
    pub height: u32,
}

impl SurfaceProvider for HeadlessProvider {
    fn create_surface(
        &self,
        _instance: &Instance,
    ) -> Result<
        (Option<Surface<'static>>, PhysicalSize<u32>),
        Box<dyn std::error::Error + Send + Sync>,
    > {
        let size = PhysicalSize::new(self.width, self.height);
        Ok((None, size))
    }
}
