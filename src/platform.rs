use wgpu::{Instance, Surface};
use winit::dpi::PhysicalSize;

/// 플랫폼별 Surface 생성을 추상화하는 트레이트
pub trait SurfaceProvider {
    fn create_surface(
        &self,
        instance: &Instance,
    ) -> Result<(Option<Surface<'static>>, PhysicalSize<u32>), Box<dyn std::error::Error>>;
}

/// 네이티브 윈도우용 SurfaceProvider 구현
#[cfg(not(target_arch = "wasm32"))]
impl SurfaceProvider for winit::window::Window {
    fn create_surface(
        &self,
        instance: &Instance,
    ) -> Result<(Option<Surface<'static>>, PhysicalSize<u32>), Box<dyn std::error::Error>> {
        let surface = instance.create_surface(self)?;
        let size = self.inner_size();
        // Surface를 'static으로 변환하기 위해 unsafe 사용
        let static_surface = unsafe { std::mem::transmute(surface) };
        Ok((Some(static_surface), size))
    }
}

/// 네이티브 플랫폼 시작 함수
#[cfg(not(target_arch = "wasm32"))]
pub fn start() {
    use crate::create_renderer;
    use pollster::block_on;
    use winit::{event::*, event_loop::EventLoop, window::WindowBuilder};

    env_logger::init();

    let event_loop = EventLoop::new().unwrap();
    let window = WindowBuilder::new()
        .with_title("WGPU Triangle")
        .build(&event_loop)
        .unwrap();

    // 네이티브에서는 winit window를 사용해서 renderer 생성
    let mut renderer = block_on(create_renderer(&window)).unwrap();

    let _ = event_loop.run(move |event, target| match event {
        Event::WindowEvent {
            event: WindowEvent::CloseRequested,
            ..
        } => {
            target.exit();
        }
        Event::WindowEvent {
            event: WindowEvent::Resized(new_size),
            ..
        } => {
            renderer.resize(new_size);
        }
        Event::WindowEvent {
            event: WindowEvent::RedrawRequested,
            ..
        } => match renderer.render() {
            Ok(_) => {}
            Err(wgpu::SurfaceError::Lost) => renderer.resize(renderer.size()),
            Err(wgpu::SurfaceError::OutOfMemory) => target.exit(),
            Err(e) => eprintln!("Render error: {:?}", e),
        },
        Event::AboutToWait => {
            // 지속적인 렌더링을 위해 redraw 요청
        }
        _ => {}
    });
}

// wasm32 타겟에서 필요한 import들
#[cfg(target_arch = "wasm32")]
use {wasm_bindgen::JsCast, wasm_bindgen::prelude::*, wasm_bindgen_futures::spawn_local};

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen(start)]
pub fn start() {
    console_log::init_with_level(log::Level::Debug).expect("Couldn't initialize logger");
    console_error_panic_hook::set_once();

    // DOM이 로드될 때까지 기다림
    let window = web_sys::window().unwrap();
    let document = window.document().unwrap();
    let canvas = document.get_element_by_id("canvas").unwrap();
    let canvas: web_sys::HtmlCanvasElement = canvas.dyn_into().unwrap();

    // 캔버스 크기 설정 (HTML과 일치시킴)
    canvas.set_width(640);
    canvas.set_height(480);

    spawn_local(async move {
        match crate::create_renderer(&canvas).await {
            Ok(mut renderer) => {
                log::info!("Renderer created successfully!");
                // 첫 번째 렌더링 수행
                match renderer.render() {
                    Ok(_) => log::info!("Triangle rendered successfully!"),
                    Err(e) => log::error!("Render failed: {:?}", e),
                }
            }
            Err(e) => {
                log::error!("Failed to create renderer: {:?}", e);
            }
        }
    });
}

/// 웹 캔버스용 SurfaceProvider 구현
#[cfg(target_arch = "wasm32")]
impl SurfaceProvider for web_sys::HtmlCanvasElement {
    fn create_surface(
        &self,
        instance: &Instance,
    ) -> Result<(Option<Surface<'static>>, PhysicalSize<u32>), Box<dyn std::error::Error>> {
        // wasm32에서는 캔버스를 직접 사용 (OffscreenCanvas 대신)
        let surface = instance.create_surface(wgpu::SurfaceTarget::Canvas(self.clone()))?;
        let static_surface = unsafe { std::mem::transmute(surface) };

        let size = PhysicalSize::new(self.width(), self.height());
        Ok((Some(static_surface), size))
    }
}

/// 헤드리스 모드용 (Surface 없이 실행)
pub struct HeadlessProvider {
    pub width: u32,
    pub height: u32,
}

impl SurfaceProvider for HeadlessProvider {
    fn create_surface(
        &self,
        _instance: &Instance,
    ) -> Result<(Option<Surface<'static>>, PhysicalSize<u32>), Box<dyn std::error::Error>> {
        let size = PhysicalSize::new(self.width, self.height);
        Ok((None, size))
    }
}
