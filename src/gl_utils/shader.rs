use gl;
use gl::types::*;

type Error = Box<dyn std::error::Error>;

pub fn compile_shader(src: &str, ty: u32) -> Result<u32, Error> {
    let ty_str = detail::shader_type_as_str(ty)
        .ok_or(Error::from("Invalid shader type"))?;

    unsafe {
        let shader = gl::CreateShader(ty);
        detail::shader_source(shader, src);

        gl::CompileShader(shader);

        let mut success = 0;
        gl::GetShaderiv(shader, gl::COMPILE_STATUS, &mut success);
        if success == 0 {
            let mut len = 0;
            gl::GetShaderiv(shader, gl::INFO_LOG_LENGTH, &mut len);
            let mut buffer = vec![0; len as usize];
            gl::GetShaderInfoLog(shader, len, std::ptr::null_mut(), buffer.as_mut_ptr() as _);
            let log = std::str::from_utf8(&buffer).unwrap();
            let msg = format!("Failed to compile {} shader: {}", ty_str, log);

            gl::DeleteShader(shader);
            Err(msg.into())
        } else {
            Ok(shader)
        }
    }
}

pub fn link_shader_program(shaders: &[u32]) -> Result<u32, Error> {
    unsafe {
        let program = gl::CreateProgram();
        for &shader in shaders {
            gl::AttachShader(program, shader);
        }

        gl::LinkProgram(program);

        for &shader in shaders {
            gl::DetachShader(program, shader);
        }

        let mut success = 0;
        gl::GetProgramiv(program, gl::LINK_STATUS, &mut success);
        if success == 0 {
            let mut len = 0;
            gl::GetProgramiv(program, gl::INFO_LOG_LENGTH, &mut len);
            let mut buffer = vec![0; len as usize];
            gl::GetProgramInfoLog(program, len, std::ptr::null_mut(), buffer.as_mut_ptr() as _);
            let log = std::str::from_utf8(&buffer).unwrap();
            let msg = format!("Failed to link shader program: {}", log);

            gl::DeleteProgram(program);
            Err(msg.into())
        } else {
            Ok(program)
        }
    }
}

pub struct ShaderProgramBuilder {
    shaders: Vec<detail::CompiledShader>,
}

impl ShaderProgramBuilder {
    pub fn new() -> Self {
        Self {
            shaders: Vec::new(),
        }
    }

    fn add_shader(&mut self, src: &str, shader_type: GLenum) -> Result<(), Error> {
        let shader = detail::CompiledShader::new(src, shader_type)?;
        self.shaders.push(shader);
        Ok(())
    }

    pub fn add_vertex_shader(&mut self, src: &str) -> Result<(), Error> {
        self.add_shader(src, gl::VERTEX_SHADER)
    }

    pub fn add_fragment_shader(&mut self, src: &str) -> Result<(), Error> {
        self.add_shader(src, gl::FRAGMENT_SHADER)
    }

    pub fn add_geometry_shader(&mut self, src: &str) -> Result<(), Error> {
        self.add_shader(src, gl::GEOMETRY_SHADER)
    }

    pub fn add_tess_control_shader(&mut self, src: &str) -> Result<(), Error> {
        self.add_shader(src, gl::TESS_CONTROL_SHADER)
    }

    pub fn add_tess_evaluation_shader(&mut self, src: &str) -> Result<(), Error> {
        self.add_shader(src, gl::TESS_EVALUATION_SHADER)
    }

    pub fn add_compute_shader(&mut self, src: &str) -> Result<(), Error> {
        self.add_shader(src, gl::COMPUTE_SHADER)
    }

    pub fn build(self) -> Result<GLuint, Error> {
        let shader_ids: Vec<_> = self.shaders.iter().map(|s| s.0).collect();
        let program = link_shader_program(&shader_ids)?;
        Ok(program)
    }
}

mod detail {
    use super::*;

    pub fn shader_source(shader: u32, src: &str) {
        let src = std::ffi::CString::new(src).unwrap();
        let src_ptr = src.as_ptr();
    
        unsafe {
            gl::ShaderSource(shader, 1, &src_ptr, std::ptr::null());
        }
    }
    
    pub fn shader_type_as_str(ty: u32) -> Option<&'static str> {
        match ty {
            gl::VERTEX_SHADER => Some("vertex"),
            gl::TESS_CONTROL_SHADER => Some("tess control"),
            gl::TESS_EVALUATION_SHADER => Some("tess evaluation"),
            gl::GEOMETRY_SHADER => Some("geometry"),
            gl::FRAGMENT_SHADER => Some("fragment"),
            gl::COMPUTE_SHADER => Some("compute"),
            _ => None,
        }
    }
    pub struct CompiledShader(pub GLuint);

    impl CompiledShader {
        pub fn new(src: &str, shader_type: GLenum) -> Result<Self, Error> {
            let shader = compile_shader(src, shader_type)?;
            Ok(Self(shader))
        }
    }

    impl Drop for CompiledShader {
        fn drop(&mut self) {
            unsafe {
                gl::DeleteShader(self.0);
            }
        }
    }
}
