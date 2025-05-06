pub mod gl_utils;
pub mod renderers;
pub mod image;

pub use gl;

pub type Error = Box<dyn std::error::Error>;
