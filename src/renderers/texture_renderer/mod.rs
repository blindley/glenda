
use crate::Error;
use gl;
use gl::types::*;

use crate::renderers::{
    Renderer,
    Viewport,
    Transformable,
    Mat4,
};

const VCODE : &str = r#"
#version 450 core
layout (location = 0) in vec2 in_pos;
layout (location = 1) in vec2 in_uv;
out vec2 v_uv;
uniform mat4 u_transform;

void main() {
    gl_Position = u_transform * vec4(in_pos, 0.0, 1.0);
    v_uv = in_uv;
}
"#;

const FCODE : &str = r#"
#version 450 core
in vec2 v_uv;
out vec4 fColor;
uniform sampler2D u_tex1;
void main() {
    fColor = texture(u_tex1, v_uv);
}
"#;

pub struct TextureRenderer {
    viewport: Viewport,
    program: u32,
    vao: u32,
    buffer: u32,
    uloc_tex1: GLint,
    uloc_transform: GLint,
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

        let uloc_tex1;
        let uloc_transform;

        unsafe {
            uloc_tex1 = gl::GetUniformLocation(program, "u_tex1\0".as_ptr() as *const i8);
            uloc_transform = gl::GetUniformLocation(program, "u_transform\0".as_ptr() as *const i8);
        }

        let mut self_ = Self {
            viewport: Viewport::default(),
            program,
            vao,
            buffer,
            uloc_tex1,
            uloc_transform,
        };

        self_.set_texture_unit(0); // Default to texture unit 0
        self_.clear_transform();

        Ok(self_)
    }

    /// This only sets the texture unit to use for the shader.
    /// The texture itself must managed separately, and bound
    /// to the specified texture unit before rendering. Learn
    /// more about texture units in OpenGL, they'r dumb and
    /// confusing.
    pub fn set_texture_unit(&mut self, texture_unit: GLint) {
        unsafe {
            gl::UseProgram(self.program);
            gl::Uniform1i(self.uloc_tex1, texture_unit);
        }
    }
}

impl Renderer for TextureRenderer {
    fn set_viewport(&mut self, viewport: Viewport) {
        self.viewport = viewport;
    }

    fn render(&self) {
        self.viewport.gl_viewport();
        unsafe {
            gl::UseProgram(self.program);
            gl::BindVertexArray(self.vao);
            gl::DrawArrays(gl::TRIANGLE_FAN, 0, 4);
        }
    }
}

impl Transformable for TextureRenderer {
    fn set_transform(&mut self, transform: Mat4) {
        unsafe {
            gl::UseProgram(self.program);
            gl::UniformMatrix4fv(self.uloc_transform, 1, gl::FALSE, transform.as_ptr());
        }
    }
}

impl Drop for TextureRenderer {
    fn drop(&mut self) {
        unsafe {
            gl::DeleteProgram(self.program);
            gl::DeleteVertexArrays(1, &self.vao);
            gl::DeleteBuffers(1, &self.buffer);
        }
    }
}
