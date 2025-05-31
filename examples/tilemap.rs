/// Example demonstrating the TilemapRenderer

use glume::gl;
use gl::types::*;

mod common;

use glenda::renderers::{
    tilemap_renderer::{
        TilemapRenderer, TilesetLayout
    }, Renderer, Viewport
};

type Error = Box<dyn std::error::Error>;

fn main() -> Result<(), Error> {
    common::run_example::<App>()
}

struct App {
    renderer: TilemapRenderer,
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
        let tileset_layout = TilesetLayout {
            tile_count: [4, 4],
            tile_size: [16, 16],
            texture_size: [64, 64],
        };

        #[rustfmt::skip]
        let tile_indices = &[
             1,  12, 11, 10,  9,
             2,  13, 14, 15,  8,
             3,   4,  5,  6,  7,
        ];

        let mut renderer = TilemapRenderer::new(
            [5, 3],
            tile_indices,
            tileset_layout
        )?;

        renderer.set_map_tile_size([0.2, 0.2]);
        renderer.set_map_offset([-1.0, 1.0]);

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
    let size = [64, 64];
    let mut pixels = vec![0u8; size[0] * size[1] * 4];

    let mut set_pixel = |xy:[usize;2], color:[u8;4]| {
        let index = (xy[1] * size[0] + xy[0]) * 4;
        pixels[index] = color[0];
        pixels[index + 1] = color[1];
        pixels[index + 2] = color[2];
        pixels[index + 3] = color[3];
    };

    let mut bkg_colors = [
        [0, 128, 0, 255], // Dark Green
        [0, 255, 255, 255], // Cyan
        [0, 255, 0, 255], // Green
        [128, 128, 128, 255], // Gray
        [128, 0, 0, 255], // Maroon
        [0, 0, 0, 255],   // Black
        [0, 0, 255, 255], // Blue
        [255, 0, 255, 255], // Magenta
        [128, 128, 0, 255], // Olive
        [0, 128, 128, 255], // Teal
        [255, 0, 0, 255], // Red
        [255, 165, 0, 255], // Orange
        [128, 0, 128, 255], // Purple
        [192, 192, 192, 255], // Silver
        [0, 0, 128, 255], // Navy
        [255, 255, 0, 255], // Yellow
    ];

    // darken everything a bit
    bkg_colors.iter_mut().for_each(|color| {
        color[0] /= 3;
        color[1] /= 3;
        color[2] /= 3;
    });

    for ty in 0..4 {
        for tx in 0..4 {
            let px = tx * (size[0] / 4);
            let py = ty * (size[1] / 4);

            let index = ty * 4 + tx;

            let bkg_color = bkg_colors[index % bkg_colors.len()];

            for y in 0..(size[1] / 4) {
                for x in 0..(size[0] / 4) {
                    set_pixel([px + x, py + y], bkg_color);
                }
            }

            let grid_size = (index as f32).sqrt() as usize + 1;
            let grid_color = [255, 255, 255, 255]; // White

            let mut n = 0;
            for y in 0..grid_size {
                for x in 0..grid_size {
                    if n >= index {
                        break;
                    }

                    let grid_x = px + 4 + 2 * x;
                    let grid_y = py + 4 + 2 * y;

                    set_pixel([grid_x, grid_y], grid_color);

                    n += 1;
                }
            }
        }        
    }


    let size = [size[0] as i32, size[1] as i32];
    let texture = glh::create_texture_2d_rgba(size, &pixels)?;
    unsafe {
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::NEAREST as i32);
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::NEAREST as i32);
    }

    Ok(texture)
}
