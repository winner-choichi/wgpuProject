#![allow(clippy::module_inception)]

pub mod app;
pub mod physics;
pub mod platform;
pub mod renderer;
pub mod simulation;
pub mod ui;

pub use app::{App, AppError, AppResult};
