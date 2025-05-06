pub mod basic_renderers;
pub mod image_renderer;
pub mod system_text;
pub mod texture_renderer;

use crate::gl;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Viewport {
    pub pos: [i32; 2],
    pub size: [i32; 2],
}

impl Viewport {
    pub fn new(pos: [i32; 2], size: [i32; 2]) -> Self {
        Self { pos, size }
    }

    pub fn gl_viewport(&self) {
        unsafe {
            gl::Viewport(self.pos[0], self.pos[1], self.size[0], self.size[1]);
            gl::Scissor(self.pos[0], self.pos[1], self.size[0], self.size[1]);
            gl::Enable(gl::SCISSOR_TEST);
        }
    }
}

impl Default for Viewport {
    fn default() -> Self {
        Self::new([0, 0], [0, 0])
    }
}

impl From<[[i32; 2]; 2]> for Viewport {
    fn from(arr: [[i32; 2]; 2]) -> Self {
        Self::new(arr[0], arr[1])
    }
}

impl From<[i32; 4]> for Viewport {
    fn from(arr: [i32; 4]) -> Self {
        Self::new([arr[0], arr[1]], [arr[2], arr[3]])
    }
}

impl From<(i32, i32, i32, i32)> for Viewport {
    fn from(arr: (i32, i32, i32, i32)) -> Self {
        Self::new([arr.0, arr.1], [arr.2, arr.3])
    }
}

impl From<[i32;2]> for Viewport {
    fn from(arr: [i32;2]) -> Self {
        Self::new([0, 0], arr)
    }
}

impl From<(i32, i32)> for Viewport {
    fn from(arr: (i32, i32)) -> Self {
        Self::new([0, 0], [arr.0, arr.1])
    }
}

pub trait Renderer {
    fn set_viewport(&mut self, viewport: Viewport);
    fn render(&self);
}
