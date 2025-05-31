/// Example demonstrating the TextureRenderer

use glume::gl;
use gl::types::*;

mod common;

use glenda::renderers::{
    Renderer,
    Viewport,
    texture_renderer::TextureRenderer,
};

type Error = Box<dyn std::error::Error>;

fn main() -> Result<(), Error> {
    common::run_example::<App>()
}

struct App {
    renderer: TextureRenderer,
    texture: GLuint,
}

impl common::Application for App {
    fn window_configuration() -> glume::window::WindowConfiguration {
        glume::window::WindowConfiguration {
            title: "Texture Renderer Example".to_string(),
            size: (800, 600),
            gl_version: (4, 5),
        }
    }

    fn new() -> Result<Self, Error> {
        let renderer = TextureRenderer::new()?;
        let texture = sample_texture()?;

        unsafe {
            gl::ActiveTexture(gl::TEXTURE0);
            gl::BindTexture(gl::TEXTURE_2D, texture);
        }

        Ok(Self { renderer, texture })
    }
}

impl Drop for App {
    fn drop(&mut self) {
        unsafe {
            gl::DeleteTextures(1, &self.texture);
        }
    }
}

impl Renderer for App {
    fn render(&self) {
        self.renderer.render();
    }

    fn set_viewport(&mut self, viewport: Viewport) {
        self.renderer.set_viewport(viewport);
    }
}

fn sample_texture() -> Result<GLuint, Error> {
    let size = [512, 512];
    let mut pixels = vec![0u8; size[0] * size[1] * 4];

    let center = [size[0] / 2, size[1] / 2];

    for y in 0..size[1] {
        for x in 0..size[0] {
            let idx = (y * size[0] + x) * 4;

            let dx = 1.0 * (x as f32 - center[0] as f32) / center[0] as f32;
            let dy = 1.0 * (y as f32 - center[1] as f32) / center[1] as f32;
            let r = (dx * dx + dy * dy).sqrt();

            if r < 0.8 {
                pixels[idx] = (255.0 * (1.0 - r)) as u8; // Red channel
                pixels[idx + 1] = (255.0 * (1.0 - r)) as u8; // Green channel
                pixels[idx + 2] = (255.0 * (1.0 - r)) as u8; // Blue channel
                pixels[idx + 3] = 255; // Alpha channel
            } else {
                pixels[idx] = (255 * x / size[0]) as u8; // Red channel
                pixels[idx + 1] = (255 * y / size[1]) as u8; // Green channel
                pixels[idx + 2] = 0; // Blue channel
                pixels[idx + 3] = 255; // Alpha channel
            }
        }
    }

    let size = [size[0] as i32, size[1] as i32];
    let texture = glh::create_texture_2d_rgba(size, &pixels)?;

    Ok(texture)
}
