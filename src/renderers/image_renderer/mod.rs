use crate::image::ImageRef;

use crate::renderers::{Renderer, Viewport};

type Error = Box<dyn std::error::Error>;

/// A texture configured for display in a window, rather than on a 3D model.
pub struct ImageTexture {
    texture_id: u32,
    size: (u32, u32),
}

impl ImageTexture {
    pub fn new(image: ImageRef) -> Self {
        let texture_id = image.create_texture().unwrap();

        unsafe {
            gl::BindTexture(gl::TEXTURE_2D, texture_id);

            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_S, gl::CLAMP_TO_EDGE as i32);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_T, gl::CLAMP_TO_EDGE as i32);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::LINEAR as i32);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::LINEAR as i32);
        }

        Self {
            texture_id,
            size: image.size(),
        }
    }

    pub fn size(&self) -> (u32, u32) {
        self.size
    }
}

impl Drop for ImageTexture {
    fn drop(&mut self) {
        unsafe {
            gl::DeleteTextures(1, &self.texture_id);
        }
    }
}

pub struct ImageRenderer {
    renderer: _ImageRenderer,
    viewport: Viewport,
    texture: Option<ImageTexture>,
}

impl ImageRenderer {
    pub fn new(viewport: Viewport) -> Result<Self, Error> {
        let renderer = _ImageRenderer::new()?;

        Ok(Self {
            renderer,
            viewport,
            texture: None,
        })
    }

    pub fn set_render_quad(&mut self, vertices: &[f32]) {
        self.renderer.set_render_quad(vertices);
    }

    pub fn reset_render_quad(&mut self) {
        self.renderer.reset_render_quad();
    }

    pub fn replace_texture(&mut self, texture: ImageTexture) -> Option<ImageTexture> {
        let old_texture = self.texture.take();
        self.texture = Some(texture);
        old_texture
    }
}

impl Renderer for ImageRenderer {
    fn set_viewport(&mut self, viewport: Viewport) {
        self.viewport = viewport;
    }

    fn render(&self) {
        if let Some(ref texture) = self.texture {
            self.viewport.gl_viewport();
            self.renderer.render(texture);
        }
    }
}

struct _ImageRenderer {
    program: u32,
    vao: u32,
    vbo: u32,
}

impl _ImageRenderer {
    pub fn new() -> Result<Self, Error> {
        use crate::gl_utils::vertex_array::create_buffer;
        use crate::gl_utils::shader::ShaderProgramBuilder;

        let vcode = include_str!("shaders/vertex_shader.glsl");
        let fcode = include_str!("shaders/fragment_shader.glsl");

        let mut builder = ShaderProgramBuilder::new();
        builder.add_vertex_shader(vcode)?;
        builder.add_fragment_shader(fcode)?;
        let program = builder.build()?;

        let vertices: &[f32] = &[
            // positions
            -1.0, 1.0,
            1.0, 1.0,
            1.0, -1.0,
            -1.0, -1.0,

            // texture coords
            0.0, 0.0,
            1.0, 0.0,
            1.0, 1.0,
            0.0, 1.0,
        ];

        let vbo = create_buffer(vertices, gl::DYNAMIC_DRAW)?;

        let mut vao = 0;
        unsafe {
            gl::GenVertexArrays(1, &mut vao);
            gl::BindVertexArray(vao);

            gl::BindBuffer(gl::ARRAY_BUFFER, vbo);

            let stride = 0;

            let offset = 0 as *const _;
            gl::VertexAttribPointer(0, 2, gl::FLOAT, gl::FALSE, stride, offset);
            gl::EnableVertexAttribArray(0);

            let offset = (8 * std::mem::size_of::<f32>()) as *const _;
            gl::VertexAttribPointer(1, 2, gl::FLOAT, gl::FALSE, stride, offset);
            gl::EnableVertexAttribArray(1);
        }

        Ok(Self {
            program,
            vao,
            vbo,
        })
    }

    pub unsafe fn render_raw_texture(&self, texture_id: u32) {
        unsafe {
            gl::UseProgram(self.program);
            gl::BindTexture(gl::TEXTURE_2D, texture_id);
            gl::BindVertexArray(self.vao);
            gl::DrawArrays(gl::TRIANGLE_FAN, 0, 4);
        }
    }

    pub fn render(&self, texture: &ImageTexture) {
        unsafe {
            self.render_raw_texture(texture.texture_id);
        }
    }

    pub fn set_render_quad(&mut self, vertices: &[f32]) {
        if vertices.len() != 8 {
            panic!("Invalid number of vertices");
        }

        unsafe {
            gl::BindBuffer(gl::ARRAY_BUFFER, self.vbo);
            gl::BufferSubData(gl::ARRAY_BUFFER, 0, (vertices.len() * std::mem::size_of::<f32>()) as isize, vertices.as_ptr() as _);
        }
    }

    pub fn reset_render_quad(&mut self) {
        let vertices: &[f32] = &[
            // positions
            -1.0, 1.0,
            1.0, 1.0,
            1.0, -1.0,
            -1.0, -1.0,
        ];

        self.set_render_quad(vertices);
    }
}

impl Drop for _ImageRenderer {
    fn drop(&mut self) {
        unsafe {
            gl::DeleteProgram(self.program);
            gl::DeleteVertexArrays(1, &self.vao);
            gl::DeleteBuffers(1, &self.vbo);
        }
    }
}
