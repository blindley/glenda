use gl::{self, types::*};

use crate::renderers::{
    Renderer,
    Viewport,
    Transformable,
    Mat4,
};

type Error = Box<dyn std::error::Error>;

pub struct TilemapRenderer {
    viewport: Viewport,
    program: GLuint,

    vao: GLuint,
    buffer: GLuint,

    /// Size of the tilemap in tiles
    map_size: [usize; 2],

    uloc_transform: GLint,
    uloc_tileset_texture_unit: GLint,
    uloc_map_tile_size: GLint,
    uloc_map_offset: GLint,
}

pub struct TilesetLayout {
    pub texture_size: [usize; 2],
    pub tile_size: [usize; 2],
    pub tile_count: [usize; 2],
}

impl TilemapRenderer {
    pub fn new(
        map_size: [usize;2],
        tile_indices: &[u16],
        tileset_layout: TilesetLayout
    ) -> Result<Self, Error>
    {
        let VaoAndBuffer { vao, buffer } =
            create_tilemap_vao(map_size, tile_indices, tileset_layout)?;

        let program = create_tilemap_shader_program()?;

        let uloc_transform;
        let uloc_tileset_texture_unit;
        let uloc_map_tile_size;
        let uloc_map_offset;
        unsafe {
            uloc_transform = gl::GetUniformLocation(program, "u_transform\0".as_ptr() as _);
            uloc_tileset_texture_unit = gl::GetUniformLocation(program, "u_tileset_texture\0".as_ptr() as _);
            uloc_map_tile_size = gl::GetUniformLocation(program, "u_map_tile_size\0".as_ptr() as _);
            uloc_map_offset = gl::GetUniformLocation(program, "u_map_offset\0".as_ptr() as _);
        }

        let mut self_ = Self {
            viewport: Viewport::default(),
            program,
            vao,
            buffer,
            map_size,

            uloc_transform,
            uloc_tileset_texture_unit,
            uloc_map_tile_size,
            uloc_map_offset,
        };

        self_.set_tileset_texture_unit(0);
        self_.set_map_tile_size([1.0, 1.0]);
        self_.set_map_offset([0.0, 0.0]);
        self_.clear_transform();

        Ok(self_)
    }

    /// Sets the texture unit for the tileset texture.
    /// Note that binding of the texture must be done separately
    pub fn set_tileset_texture_unit(&mut self, texture_unit: GLint) {
        unsafe {
            gl::UseProgram(self.program);
            gl::Uniform1i(self.uloc_tileset_texture_unit, texture_unit);
        }
    }

    /// Sets the size of each tile in the map, in normalized device coordinates.
    /// This is independent of the size of the tiles in the tileset texture.
    pub fn set_map_tile_size(&mut self, tile_size: [f32; 2]) {
        unsafe {
            gl::UseProgram(self.program);
            gl::Uniform2f(self.uloc_map_tile_size, tile_size[0], tile_size[1]);
        }
    }

    pub fn set_map_offset(&mut self, offset: [f32; 2]) {
        unsafe {
            gl::UseProgram(self.program);
            gl::Uniform2f(self.uloc_map_offset, offset[0], offset[1]);
        }
    }
}

impl Drop for TilemapRenderer {
    fn drop(&mut self) {
        unsafe {
            gl::DeleteVertexArrays(1, &self.vao);
            gl::DeleteBuffers(1, &self.buffer);
            gl::DeleteProgram(self.program);
        }
    }
}

impl Renderer for TilemapRenderer {
    fn render(&self) {
        self.viewport.gl_viewport();
        let vcount = (self.map_size[0] * self.map_size[1] * 6) as i32;
        unsafe {
            gl::UseProgram(self.program);
            gl::BindVertexArray(self.vao);
            gl::DrawArrays(gl::TRIANGLES, 0, vcount);
        }
    }

    fn set_viewport(&mut self, viewport: Viewport) {
        self.viewport = viewport;
    }
}

impl Transformable for TilemapRenderer {
    fn set_transform(&mut self, transform: Mat4) {
        unsafe {
            gl::UseProgram(self.program);
            gl::UniformMatrix4fv(self.uloc_transform, 1, gl::FALSE, transform.as_ptr());
        }
    }
}

fn create_tilemap_shader_program() -> Result<GLuint, Error> {
    const TILEMAP_VCODE: &str = r#"
        #version 450 core
        layout(location = 0) in vec2 pos;
        layout(location = 1) in vec2 uv;
        out vec2 v_uv;

        uniform mat4 u_transform;
        uniform vec2 u_map_tile_size;
        uniform vec2 u_map_offset;

        void main() {
            v_uv = uv;
            vec2 p = pos * u_map_tile_size + u_map_offset;
            gl_Position = u_transform * vec4(p, 0.0, 1.0);
        }
        "#;

    const TILEMAP_FCODE: &str = r#"
        #version 450 core
        in vec2 v_uv;
        out vec4 f_color;
        uniform sampler2D u_tileset_texture;
        void main() {
            f_color = texture(u_tileset_texture, v_uv);
        }
        "#;

    let program = glh::ProgramBuilder::new()
        .with_vertex_shader(TILEMAP_VCODE)?
        .with_fragment_shader(TILEMAP_FCODE)?
        .build()?;

    Ok(program)
}

struct VaoAndBuffer {
    vao: GLuint,
    buffer: GLuint,
}

fn create_tilemap_vao(
    map_size: [usize; 2],
    tile_indices: &[u16],
    tileset_layout: TilesetLayout,
) -> Result<VaoAndBuffer, Error>
{
    if tile_indices.len() != map_size[0] * map_size[1] {
        return Err("Tile indices length does not match map size".into());
    }

    let tile_size_uv = {
        let size_u = tileset_layout.tile_size[0] as f32 / tileset_layout.texture_size[0] as f32;
        let size_v = tileset_layout.tile_size[1] as f32 / tileset_layout.texture_size[1] as f32;
        [size_u, size_v]
    };

    let mut vertices = Vec::new();
    for (i, &tile_index) in tile_indices.iter().enumerate() {
        let mx = i % map_size[0];
        let my = i / map_size[0];

        let x1 = mx as f32;
        let y1 = -(my as f32);
        let x2 = x1 + 1.0;
        let y2 = y1 - 1.0;

        let tx = tile_index as usize % tileset_layout.tile_count[0];
        let ty = tile_index as usize / tileset_layout.tile_count[0];

        let u1 = tx as f32 * tile_size_uv[0];
        let v1 = ty as f32 * tile_size_uv[1];
        let u2 = u1 + tile_size_uv[0];
        let v2 = v1 + tile_size_uv[1];

        let tl = &[x1, y1, u1, v1];
        let tr = &[x2, y1, u2, v1];
        let bl = &[x1, y2, u1, v2];
        let br = &[x2, y2, u2, v2];

        vertices.extend_from_slice(bl);
        vertices.extend_from_slice(tl);
        vertices.extend_from_slice(br);
        vertices.extend_from_slice(tl);
        vertices.extend_from_slice(tr);
        vertices.extend_from_slice(br);
    }

    let buffer = glh::create_buffer(&vertices, gl::STATIC_DRAW)?;
    let mut vao = 0;
    unsafe {
        gl::GenVertexArrays(1, &mut vao);
    }

    glh::enable_interleaved_vertex_array_attributes(vao, buffer, gl::FLOAT, false, 0, &[2, 2])?;

    Ok(VaoAndBuffer {
        vao,
        buffer,
    })
}
