
use crate::Error;

use crate::renderers::{
    Renderer,
    Viewport,
};

const VCODE : &str = r#"
#version 450 core
layout (location = 0) in vec2 inPos;
layout (location = 1) in vec2 inTexCoord;
out vec2 vTexCoord;

void main() {
    gl_Position = vec4(inPos, 0.0, 1.0);
    vTexCoord = inTexCoord;
}
"#;

const FCODE : &str = r#"
#version 450 core
in vec2 vTexCoord;
out vec4 fColor;
uniform sampler2D tex1;
void main() {
    fColor = texture(tex1, vTexCoord);
}
"#;

pub struct TextureRenderer {
    viewport: Viewport,
    program: u32,
    vao: u32,
    vbo: u32,
    texture: Option<u32>,
}

impl TextureRenderer {
    pub fn new() -> Result<Self, Error> {
        let program = glh::ProgramBuilder::new()
            .with_vertex_shader(VCODE)?
            .with_fragment_shader(FCODE)?
            .build()?;

        #[rustfmt::skip]
        let vertices: &[f32] = &[
            // pos,        texCoord
            -1.0,  1.0,    0.0, 0.0,
             1.0,  1.0,    1.0, 0.0,
             1.0, -1.0,    1.0, 1.0,
            -1.0, -1.0,    0.0, 1.0,
        ];

        let component_counts = &[2, 2];

        let buffer = glh::create_buffer(vertices, gl::STATIC_DRAW)?;
        let mut vao = 0;
        unsafe { gl::GenVertexArrays(1, &mut vao); }
        glh::enable_interleaved_vertex_array_attributes(
            vao,
            buffer,
            gl::FLOAT,
            false,
            0,
            component_counts,
        )?;

        unsafe {
            let loc = gl::GetUniformLocation(program, "tex1\0".as_ptr() as *const i8);
            gl::UseProgram(program);
            gl::Uniform1i(loc, 0); // Texture unit 0
        }

        Ok(Self {
            viewport: Viewport::default(),
            program,
            vao,
            vbo: buffer,
            texture: None,
        })
    }

    /// Sets the texture to be used for rendering.
    /// Ownership of the texture is **not** transferred. This essentially acts like a raw pointer.
    /// The texture will not be deleted when this struct is dropped.
    pub fn set_texture(&mut self, texture: Option<u32>) {
        self.texture = texture;
    }
}

impl Renderer for TextureRenderer {
    fn set_viewport(&mut self, viewport: Viewport) {
        self.viewport = viewport;
    }

    fn render(&self) {
        if let Some(tex) = self.texture {
            self.viewport.gl_viewport();
            unsafe {
                gl::UseProgram(self.program);
                gl::BindVertexArray(self.vao);
                gl::ActiveTexture(gl::TEXTURE0);
                gl::BindTexture(gl::TEXTURE_2D, tex);
                gl::DrawArrays(gl::TRIANGLE_FAN, 0, 4);
            }
        }
    }
}

impl Drop for TextureRenderer {
    fn drop(&mut self) {
        unsafe {
            gl::DeleteProgram(self.program);
            gl::DeleteVertexArrays(1, &self.vao);
            gl::DeleteBuffers(1, &self.vbo);
        }
    }
}
