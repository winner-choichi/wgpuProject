use crate::renderer::camera::Camera;
use crate::renderer::renderer::Renderer;
use glam::{Vec2, Vec3};
use std::f32::consts::FRAC_PI_2;
use winit::event::{ElementState, KeyEvent, MouseButton, MouseScrollDelta};
use winit::keyboard::{KeyCode, PhysicalKey};

#[derive(Debug)]
pub struct CameraController {
    position: Vec3,
    yaw: f32,
    pitch: f32,
    velocity: Vec3,
    move_forward: bool,
    move_backward: bool,
    move_left: bool,
    move_right: bool,
    move_up: bool,
    move_down: bool,
    drag_active: bool,
    last_cursor: Option<Vec2>,
    speed: f32,
    acceleration: f32,
    damping: f32,
    sensitivity: f32,
    zoom_step: f32,
}

impl CameraController {
    pub fn new(camera: &Camera) -> Self {
        let position = camera.eye;
        let forward = (camera.target - camera.eye).normalize_or_zero();
        let yaw = forward.x.atan2(forward.z);
        let pitch = forward.y.asin();

        Self {
            position,
            yaw,
            pitch,
            velocity: Vec3::ZERO,
            move_forward: false,
            move_backward: false,
            move_left: false,
            move_right: false,
            move_up: false,
            move_down: false,
            drag_active: false,
            last_cursor: None,
            speed: 4.0,
            acceleration: 12.0,
            damping: 12.0,
            sensitivity: 0.0025,
            zoom_step: 4.0,
        }
    }

    pub fn handle_keyboard(&mut self, event: &KeyEvent) -> bool {
        let pressed = event.state == ElementState::Pressed;
        let changed = match event.physical_key {
            PhysicalKey::Code(KeyCode::KeyW) => {
                self.move_forward = pressed;
                true
            }
            PhysicalKey::Code(KeyCode::KeyS) => {
                self.move_backward = pressed;
                true
            }
            PhysicalKey::Code(KeyCode::KeyA) => {
                self.move_left = pressed;
                true
            }
            PhysicalKey::Code(KeyCode::KeyD) => {
                self.move_right = pressed;
                true
            }
            PhysicalKey::Code(KeyCode::Space) => {
                self.move_up = pressed;
                true
            }
            PhysicalKey::Code(KeyCode::ShiftLeft) | PhysicalKey::Code(KeyCode::ShiftRight) => {
                self.move_down = pressed;
                true
            }
            _ => false,
        };

        changed
    }

    pub fn handle_mouse_button(&mut self, state: ElementState, button: MouseButton) -> bool {
        if button != MouseButton::Right {
            return false;
        }

        self.drag_active = state == ElementState::Pressed;
        if self.drag_active {
            self.last_cursor = None;
        }
        true
    }

    pub fn handle_cursor_move(&mut self, position: (f64, f64)) -> bool {
        if !self.drag_active {
            return false;
        }

        let current = Vec2::new(position.0 as f32, position.1 as f32);
        if let Some(last) = self.last_cursor.replace(current) {
            let delta = current - last;
            self.yaw -= delta.x * self.sensitivity;
            self.pitch -= delta.y * self.sensitivity;
            self.pitch = self.pitch.clamp(-FRAC_PI_2 + 0.01, FRAC_PI_2 - 0.01);
            return true;
        }

        false
    }

    pub fn handle_scroll(&mut self, delta: &MouseScrollDelta) -> bool {
        let scroll = match delta {
            MouseScrollDelta::LineDelta(_, y) => *y,
            MouseScrollDelta::PixelDelta(pos) => pos.y as f32 * 0.1,
        };

        if scroll.abs() <= f32::EPSILON {
            return false;
        }

        let forward = self.forward_vector();
        self.position += forward * scroll * self.zoom_step;
        true
    }

    pub fn update(&mut self, renderer: &mut Renderer, dt: f32) {
        let dt = dt.clamp(0.0, 0.1);
        if dt <= f32::EPSILON {
            self.apply_transform(renderer);
            return;
        }

        let forward = self.forward_vector();
        let mut right = forward.cross(Vec3::Y).normalize_or_zero();
        if right.length_squared() <= 1e-4 {
            right = Vec3::X;
        }

        let mut up_dir = right.cross(forward).normalize_or_zero();
        if up_dir.length_squared() <= 1e-4 {
            up_dir = Vec3::Y;
        }

        let mut direction = Vec3::ZERO;
        if self.move_forward {
            direction += forward;
        }
        if self.move_backward {
            direction -= forward;
        }
        if self.move_right {
            direction += right;
        }
        if self.move_left {
            direction -= right;
        }
        if self.move_up {
            direction += up_dir;
        }
        if self.move_down {
            direction -= up_dir;
        }

        let desired_velocity = if direction.length_squared() > f32::EPSILON {
            direction.normalize() * self.speed
        } else {
            Vec3::ZERO
        };

        let blend = 1.0 - f32::exp(-self.acceleration * dt);
        self.velocity = self.velocity.lerp(desired_velocity, blend);

        if desired_velocity.length_squared() <= f32::EPSILON {
            let damping = f32::exp(-self.damping * dt);
            self.velocity *= damping;
        }

        self.position += self.velocity * dt;

        self.apply_transform(renderer);
    }

    fn forward_vector(&self) -> Vec3 {
        let cos_pitch = self.pitch.cos();
        Vec3::new(
            self.yaw.sin() * cos_pitch,
            self.pitch.sin(),
            self.yaw.cos() * cos_pitch,
        )
        .normalize_or_zero()
    }

    fn apply_transform(&self, renderer: &mut Renderer) {
        let forward = self.forward_vector();
        if forward.length_squared() <= f32::EPSILON {
            return;
        }

        let camera = renderer.camera_mut();
        camera.eye = self.position;
        camera.target = self.position + forward;
        camera.up = Vec3::Y;
    }
}
