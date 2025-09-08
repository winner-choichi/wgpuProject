#[cfg(not(target_arch = "wasm32"))]
fn main() {
    wgpu_triangle::platform::start();
}

#[cfg(target_arch = "wasm32")]
fn main() {
    // wasm32에서는 #[wasm_bindgen(start)]가 자동으로 호출되므로 main은 비어있음
}
