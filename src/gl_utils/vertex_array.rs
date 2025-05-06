
use gl;
use gl::types::*;

use crate::Error;

fn validate_usage_enum(usage: GLenum) -> Result<(), Error> {
    const ALLOWED_USAGE: [GLenum; 9] = [
        gl::STREAM_DRAW,
        gl::STATIC_DRAW,
        gl::DYNAMIC_DRAW,
        gl::STREAM_READ,
        gl::STATIC_READ,
        gl::DYNAMIC_READ,
        gl::STREAM_COPY,
        gl::STATIC_COPY,
        gl::DYNAMIC_COPY,
    ];

    if !ALLOWED_USAGE.contains(&usage) {
        return Err(Error::from("Invalid buffer usage. Allowed values are: \
            STREAM_DRAW, STATIC_DRAW, DYNAMIC_DRAW, \
            STREAM_READ, STATIC_READ, DYNAMIC_READ, \
            STREAM_COPY, STATIC_COPY, DYNAMIC_COPY."));
    }

    Ok(())
}

pub fn create_buffer<T: Copy>(data: &[T], usage: GLenum) -> Result<GLuint, Error> {
    validate_usage_enum(usage)?;
    if data.is_empty() {
        return Err(Error::from("Data array is empty."));
    }

    let mut buffer = 0;
    let data_size = (data.len() * std::mem::size_of::<T>()) as GLsizeiptr;
    let data_ptr = data.as_ptr() as *const std::ffi::c_void;
    unsafe {
        gl::CreateBuffers(1, &mut buffer);
        gl::NamedBufferData(
            buffer,
            data_size,
            data_ptr,
            usage,
        );
    }

    Ok(buffer)
}

pub struct CreateVertexArrayResult {
    pub vao: GLuint,
    pub buffers: Vec<GLuint>,
    pub vcount: usize,
}

pub fn create_interleaved_f32_vertex_array(
    data: &[f32],
    component_counts: &[usize],
    usage: GLenum,
) -> Result<CreateVertexArrayResult, Error>
{
    let total_components = component_counts.iter().sum::<usize>();
    if total_components == 0 {
        return Err(Error::from("Total components is zero."));
    }

    if data.len() % total_components != 0 {
        return Err(Error::from("Data length is not a multiple of total components."));
    }

    let buffer = create_buffer(data, usage)?;

    unsafe {
        let mut vao = 0;
        gl::CreateVertexArrays(1, &mut vao);
        gl::BindVertexArray(vao);

        gl::BindBuffer(gl::ARRAY_BUFFER, buffer);

        let stride = (std::mem::size_of::<f32>() * total_components) as GLsizei;
        let mut offset = 0;
        for (i, &count) in component_counts.iter().enumerate() {
            let count = count as GLint;
            gl::EnableVertexAttribArray(i as GLuint);
            gl::VertexAttribPointer(
                i as GLuint,
                count,
                gl::FLOAT,
                gl::FALSE,
                stride,
                offset as *const std::ffi::c_void,
            );
            offset += count as usize * std::mem::size_of::<f32>();
        }

        Ok(CreateVertexArrayResult {
            vao,
            buffers: vec![buffer],
            vcount: data.len() / total_components,
        })
    }
}
