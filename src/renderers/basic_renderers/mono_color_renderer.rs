use crate::renderers::{Renderer, Viewport};

use crate::Error;

const VCODE: &str = r#"
#version 450 core
const vec2 vertices[4] = vec2[4](
    vec2(-1.0, -1.0),
    vec2(-1.0, 1.0),
    vec2(1.0, 1.0),
    vec2(1.0, -1.0)
);

void main() {
    gl_Position = vec4(vertices[gl_VertexID], 0.0, 1.0);
}
"#;

const FCODE: &str = r#"
#version 450 core
out vec4 fColor;
uniform vec4 color;
void main() {
    fColor = color;
}
"#;

pub struct MonoColorRenderer {
    viewport: Viewport,
    program: u32,
}

impl MonoColorRenderer {
    pub fn new(color: [f32; 4]) -> Result<Self, Error> {
        let program = glh::ProgramBuilder::new()
            .with_vertex_shader(VCODE)?
            .with_fragment_shader(FCODE)?
            .build()?;

        let mut _self = Self {
            viewport: Viewport::default(),
            program,
        };

        _self.set_color(color);

        Ok(_self)
    }

    pub fn set_color(&mut self, color: [f32; 4]) {
        unsafe {
            let loc = gl::GetUniformLocation(self.program, "color\0".as_ptr() as *const i8);
            gl::UseProgram(self.program);
            gl::Uniform4f(loc, color[0], color[1], color[2], color[3]);
        };
    }
}

impl Renderer for MonoColorRenderer {
    fn set_viewport(&mut self, viewport: Viewport) {
        self.viewport = viewport;
    }

    fn render(&self) {
        self.viewport.gl_viewport();
        unsafe {
            gl::UseProgram(self.program);
            gl::DrawArrays(gl::TRIANGLE_FAN, 0, 4);
        }
    }
}

impl Drop for MonoColorRenderer {
    fn drop(&mut self) {
        unsafe {
            gl::DeleteProgram(self.program);
        }
    }
}
