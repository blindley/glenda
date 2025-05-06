
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
        use crate::gl_utils::{
            shader::ShaderProgramBuilder,
            vertex_array::create_interleaved_f32_vertex_array
        };

        let mut builder = ShaderProgramBuilder::new();
        builder.add_vertex_shader(VCODE)?;
        builder.add_fragment_shader(FCODE)?;
        let program = builder.build()?;

        let vertices: &[f32] = &[
            // pos, texCoord
            -1.0, 1.0, 0.0, 0.0,
            1.0, 1.0, 1.0, 0.0,
            1.0, -1.0, 1.0, 1.0,
            -1.0, -1.0, 0.0, 1.0,
        ];

        let component_counts = &[2, 2];

        let v = create_interleaved_f32_vertex_array(
            vertices,
            component_counts,
            gl::STATIC_DRAW,
        )?;

        unsafe {
            let loc = gl::GetUniformLocation(program, "tex1\0".as_ptr() as *const i8);
            gl::UseProgram(program);
            gl::Uniform1i(loc, 0); // Texture unit 0
        }

        Ok(Self {
            viewport: Viewport::default(),
            program,
            vao: v.vao,
            vbo: v.buffers[0],
            texture: None,
        })
    }

    /// Sets the texture to be used for rendering.
    /// Ownership of the texture is **not** transferred. This essentially acts like a raw pointer.
    /// The texture will not be deleted when this struct is dropped.
    pub unsafe fn set_texture(&mut self, texture: Option<u32>) {
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
