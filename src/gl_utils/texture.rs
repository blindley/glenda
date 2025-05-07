
use crate::Error;
use gl;
use gl::types::*;

pub fn create_texture(format: GLenum, size: (u32, u32), data: &[u8])
    -> Result<u32, Error>
{
    let mut texture = 0;
    unsafe {
        gl::GenTextures(1, &mut texture);
        gl::BindTexture(gl::TEXTURE_2D, texture);
        gl::TexImage2D(
            gl::TEXTURE_2D,
            0,
            format as i32,
            size.0 as i32,
            size.1 as i32,
            0,
            format,
            gl::UNSIGNED_BYTE,
            data.as_ptr() as _,
        );

        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::LINEAR as i32);
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::LINEAR as i32);
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_S, gl::CLAMP_TO_EDGE as i32);
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_T, gl::CLAMP_TO_EDGE as i32);
    }


    Ok(texture)
}

pub fn create_texture_rgb(size: (u32, u32), data: &[u8]) -> Result<u32, Error> {
    create_texture(gl::RGB, size, data)
}

pub fn create_texture_rgba(size: (u32, u32), data: &[u8]) -> Result<u32, Error> {
    create_texture(gl::RGBA, size, data)
}
