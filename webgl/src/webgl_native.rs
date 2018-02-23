use std::ops::Deref;
use glenum::*;
use gl;
use std::os::raw::c_void;

use std::ffi::CString;
use std::ffi::CStr;
use common::*;
use std::ptr;
use std::str;

pub fn check_gl_error(msg: &str) {
    unsafe {
        use gl;
        let err = gl::GetError();
        if err != gl::NO_ERROR {
            panic!(
                "GLError: {} {} ({})",
                msg,
                err,
                match err {
                    gl::INVALID_ENUM => "invalid enum",
                    gl::INVALID_OPERATION => "invalid operation",
                    gl::INVALID_VALUE => "invalid value",
                    gl::OUT_OF_MEMORY => "out of memory",
                    gl::STACK_OVERFLOW => "stack overflow",
                    gl::STACK_UNDERFLOW => "stack underflow",
                    _ => "unknown error",
                }
            );
        }
    }
}

/// gl::GetString convenient wrapper
fn get_string(param: u32) -> String {
    return unsafe {
        let data = CStr::from_ptr(gl::GetString(param) as *const _)
            .to_bytes()
            .to_vec();
        String::from_utf8(data).unwrap()
    };
}

pub type WebGLContext<'p> = Box<'p + for<'a> FnMut(&'a str) -> *const c_void>;

impl WebGLRenderingContext {
    pub fn new<'p>(mut loadfn: WebGLContext<'p>) -> WebGLRenderingContext {
        gl::load_with(move |name| loadfn(name));

        WebGLRenderingContext {
            common: GLContext::new(),
        }
    }
}

impl GLContext {
    pub fn new() -> GLContext {
        //  unsafe { gl::Enable(gl::DEPTH_TEST) };
        println!("opengl {}", get_string(gl::VERSION));
        println!("shading language {}",get_string(gl::SHADING_LANGUAGE_VERSION));
        println!("vendor {}", get_string(gl::VENDOR));
        GLContext { reference: 0 }
    }

    pub fn print(s: &str) {
        print!("{}", s);
    }

    pub fn create_buffer(&self) -> WebGLBuffer {
        let mut buffer = WebGLBuffer(0);
        unsafe {
            gl::GenBuffers(1, &mut buffer.0);
        }
        check_gl_error("create_buffer");
        buffer
    }

    pub fn bind_buffer(&self, kind: BufferKind, buffer: &WebGLBuffer) {
        unsafe {
            gl::BindBuffer(kind as _, buffer.0);
        }
        check_gl_error("bind_buffer");
    }

    pub fn buffer_data(&self, kind: BufferKind, data: &[u8], draw: DrawMode) {
        unsafe {
            gl::BufferData(kind as _, data.len() as _, data.as_ptr() as _, draw as _);
        }
        check_gl_error("buffer_data");
    }

    pub fn buffer_sub_data(&self, kind: BufferKind, offset: u32, data: &[u8]) {
        unsafe {
            gl::BufferSubData(kind as _, offset as _, data.len() as _, data.as_ptr() as _);
        }
        check_gl_error("buffer_data");
    }

    pub fn unbind_buffer(&self, kind: BufferKind) {
        unsafe {
            gl::BindBuffer(kind as _, 0);
        }
        check_gl_error("unbind_buffer");
    }

    pub fn create_shader(&self, kind: ShaderKind) -> WebGLShader {
        check_gl_error("create_shader");
        unsafe { WebGLShader(gl::CreateShader(kind as _)) }
    }

    pub fn shader_source(&self, shader: &WebGLShader, source: &str) {
        let src = CString::new(source).unwrap();
        unsafe {
            use std::ptr;
            gl::ShaderSource(shader.0, 1, &src.as_ptr(), ptr::null());
        }
        check_gl_error("shader_source");
    }

    pub fn compile_shader(&self, shader: &WebGLShader) {
        unsafe {
            gl::CompileShader(shader.0);

            // Get the compile status
            let mut status = gl::FALSE as gl::types::GLint;
            gl::GetShaderiv(shader.0, gl::COMPILE_STATUS, &mut status);

            // Fail on error
            if status != (gl::TRUE as gl::types::GLint) {
                let mut len = 0;
                gl::GetShaderiv(shader.0, gl::INFO_LOG_LENGTH, &mut len);
                let mut buf = Vec::with_capacity(len as usize);
                buf.set_len((len as usize) - 1); // subtract 1 to skip the trailing null character
                gl::GetShaderInfoLog(
                    shader.0,
                    len,
                    ptr::null_mut(),
                    buf.as_mut_ptr() as *mut gl::types::GLchar,
                );

                match String::from_utf8(buf) {
                    Ok(s) => panic!(s),
                    Err(_) => panic!("Compile shader fail, reason unknown"),
                }
            }
        }

        check_gl_error("compile_shader");
    }

    pub fn create_program(&self) -> WebGLProgram {
        let p = unsafe { WebGLProgram(gl::CreateProgram()) };
        check_gl_error("create_program");
        p
    }

    pub fn link_program(&self, program: &WebGLProgram) {
        unsafe {
            gl::LinkProgram(program.0);
        }
        check_gl_error("link_program");
    }

    pub fn use_program(&self, program: &WebGLProgram) {
        unsafe {
            gl::UseProgram(program.0);
        }
        check_gl_error("use_program");
    }

    pub fn attach_shader(&self, program: &WebGLProgram, shader: &WebGLShader) {
        unsafe {
            gl::AttachShader(program.0, shader.0);
        }
        check_gl_error("attach_shader");
    }

    pub fn get_attrib_location(&self, program: &WebGLProgram, name: &str) -> Option<u32> {
        let c_name = CString::new(name).unwrap();
        unsafe {
            let location = gl::GetAttribLocation(program.0 as _, c_name.as_ptr());
            check_gl_error("get_attrib_location");
            if location == -1 {
                return None;
            }
            return Some(location as _);
        }
    }

    pub fn get_uniform_location(
        &self,
        program: &WebGLProgram,
        name: &str,
    ) -> Option<WebGLUniformLocation> {
        let c_name = CString::new(name).unwrap();
        unsafe {
            let location = gl::GetUniformLocation(program.0 as _, c_name.as_ptr());
            check_gl_error("get_uniform_location");
            if location == -1 {
                return None;
            }
            return Some(WebGLUniformLocation {
                reference: location as _,
                name: name.into(),
            });
        }
    }

    pub fn vertex_attrib_pointer(
        &self,
        location: u32,
        size: AttributeSize,
        kind: DataType,
        normalized: bool,
        stride: u32,
        offset: u32,
    ) {
        unsafe {
            gl::VertexAttribPointer(
                location as _,
                size as _,
                kind as _,
                normalized as _,
                stride as _,
                offset as _,
            );
        }
        // println!(
        //     "{:?} {:?} {:?} {:?} {:?} {:?} {:?}",
        //     location, size, kind, kind as u32, normalized, stride, offset
        // );
        check_gl_error("vertex_attrib_pointer");
    }

    pub fn enable_vertex_attrib_array(&self, location: u32) {
        unsafe {
            gl::EnableVertexAttribArray(location as _);
        }
        check_gl_error("enable_vertex_attrib_array");
    }

    pub fn clear_color(&self, r: f32, g: f32, b: f32, a: f32) {
        unsafe {
            gl::ClearColor(r, g, b, a);
        }
        check_gl_error("clear_color");
    }

    pub fn enable(&self, flag: i32) {
        unsafe {
            gl::Enable(flag as _);
        }
        check_gl_error("enable");
    }

    pub fn disable(&self, flag: i32) {
        unsafe {
            gl::Disable(flag as _);
        }
        check_gl_error("disable");
    }

    pub fn cull_face(&self, flag: Culling) {
        unsafe {
            gl::CullFace(flag as _);
        }
        check_gl_error("cullface");
    }

    pub fn clear(&self, bit: BufferBit) {
        unsafe {
            gl::Clear(bit as _);
        }
        check_gl_error("clear");
    }

    pub fn viewport(&self, x: i32, y: i32, width: u32, height: u32) {
        unsafe {
            gl::Viewport(x, y, width as _, height as _);
        };
        check_gl_error("viewport");
    }

    pub fn draw_elements(&self, mode: Primitives, count: usize, kind: DataType, offset: u32) {
        unsafe {
            gl::DrawElements(mode as _, count as _, kind as _, offset as _);
        };
        check_gl_error("draw_elements");
    }

    pub fn draw_arrays(&self, mode: Primitives, count: usize) {
        unsafe {
            gl::DrawArrays(mode as _, 0, count as _);
        };
        check_gl_error("draw_arrays");
    }

    pub fn pixel_storei(&self, storage: PixelStorageMode, value: i32) {
        unsafe {
            gl::PixelStorei(storage as _, value);
        }
    }

    pub fn tex_image2d(
        &self,
        target: TextureBindPoint,
        level: u8,
        width: u16,
        height: u16,
        format: PixelFormat,
        kind: DataType,
        pixels: &[u8],
    ) {
        let p: *const c_void;

        if pixels.len() > 0 {
            p = pixels.as_ptr() as _;
        } else {
            p = 0 as _;
        }

        unsafe {
            gl::TexImage2D(
                target as _,
                level as _,
                format as _,
                width as _,
                height as _,
                0,
                format as _,
                kind as _,
                p as _,
            );
        }
    }

    pub fn tex_sub_image2d(
        &self,
        target: TextureBindPoint,
        level: u8,
        xoffset: u16,
        yoffset: u16,
        width: u16,
        height: u16,
        format: PixelFormat,
        kind: DataType,
        pixels: &[u8],
    ) {
        unsafe {
            gl::TexSubImage2D(
                target as _,
                level as _,
                xoffset as _,
                yoffset as _,
                width as _,
                height as _,
                format as _,
                kind as _,
                pixels.as_ptr() as _,
            );
        }
    }

    pub fn compressed_tex_image2d(
        &self,
        target: TextureBindPoint,
        level: u8,
        compression: TextureCompression,
        width: u16,
        height: u16,
        data: &[u8],
    ) {
        unsafe {
            gl::CompressedTexImage2D(
                target as _,
                level as _,
                compression as _,
                width as _,
                height as _,
                0,
                (data.len() - 128) as _, //gl::UNSIGNED_BYTE as _,
                &data[128] as *const u8 as _,
            );
        }
    }

    pub fn get_program_parameter(&self, program: &WebGLProgram, pname: ShaderParameter) -> i32 {
        let mut res = 0;
        unsafe {
            gl::GetProgramiv(program.0, pname as _, &mut res);
        }
        res
    }

    // pub fn get_active_uniform(&self, program: &WebGLProgram, location: u32) -> WebGLActiveInfo {
    //     let mut name: Vec<u8> = Vec::with_capacity(NAME_SIZE);
    //     let mut size = 0i32;
    //     let mut len = 0i32;
    //     let mut kind = 0u32;

    //     unsafe {
    //         gl::GetActiveUniform(
    //             program.0,
    //             location as _,
    //             NAME_SIZE as _,
    //             &mut len,
    //             &mut size,
    //             &mut kind,
    //             name.as_mut_ptr() as _,
    //         );
    //         name.set_len(len as _);
    //     };

    //     use std::mem;

    //     WebGLActiveInfo::new(
    //         String::from_utf8(name).unwrap(),
    //         //location as _,
    //         size as _,
    //         unsafe { mem::transmute::<u16, UniformType>(kind as _) },
    //         0
    //         //unsafe { mem::transmute::<u16, DataType>(kind as _) },
    //     )
    // }

    // pub fn get_active_attrib(&self, program: &WebGLProgram, location: u32) -> WebGLActiveInfo {
    //     let mut name: Vec<u8> = Vec::with_capacity(NAME_SIZE);
    //     let mut size = 0i32;
    //     let mut len = 0i32;
    //     let mut kind = 0u32;

    //     unsafe {
    //         gl::GetActiveAttrib(
    //             program.0,
    //             location as _,
    //             NAME_SIZE as _,
    //             &mut len,
    //             &mut size,
    //             &mut kind,
    //             name.as_mut_ptr() as _,
    //         );
    //         name.set_len(len as _);
    //     }
    //     println!("name {:?}", name);
    //     use std::mem;
    //     //let c_name = unsafe { CString::from_raw(name[0..(len+1)].as_mut_ptr())};
    //     WebGLActiveInfo::new(
    //         String::from_utf8(name).expect("utf8 parse failed"),
    //         //location,
    //         size as _,
    //         //DataType::Float
    //         unsafe { mem::transmute::<u16, UniformType>(kind as _) },
    //         0,
    //     )
    // }

    ///
    pub fn create_texture(&self) -> WebGLTexture {
        let mut handle = WebGLTexture(0);
        unsafe {
            gl::GenTextures(1, &mut handle.0);
        }
        handle
    }

    pub fn delete_texture(&self, texture: &WebGLTexture) {
        unsafe {
            gl::DeleteTextures(1, texture.0 as _);
        }
    }

    pub fn active_texture(&self, active: u32) {
        unsafe {
            gl::ActiveTexture(gl::TEXTURE0 + active);
        }
    }

    pub fn bind_texture(&self, texture: &WebGLTexture) {
        unsafe {
            gl::BindTexture(gl::TEXTURE_2D, texture.0);
        }
    }

    pub fn unbind_texture(&self) {
        unsafe {
            gl::BindTexture(gl::TEXTURE_2D, 0);
        }
    }

    pub fn blend_func(&self, b1: BlendMode, b2: BlendMode) {
        unsafe {
            gl::BlendFunc(b1 as _, b2 as _);
        }
    }

    pub fn uniform_matrix_4fv(&self, location: &WebGLUniformLocation, value: &[[f32; 4]; 4]) {
        unsafe {
            gl::UniformMatrix4fv(*location.deref() as i32, 1, false as _, &value[0] as _);
        }
    }

    pub fn uniform_matrix_3fv(&self, location: &WebGLUniformLocation, value: &[[f32; 3]; 3]) {
        unsafe {
            gl::UniformMatrix3fv(*location.deref() as i32, 1, false as _, &value[0] as _);
        }
    }

    pub fn uniform_matrix_2fv(&self, location: &WebGLUniformLocation, value: &[[f32; 2]; 2]) {
        unsafe {
            gl::UniformMatrix2fv(*location.deref() as i32, 1, false as _, &value[0] as _);
        }
    }

    pub fn uniform_1i(&self, location: &WebGLUniformLocation, value: i32) {
        unsafe {
            gl::Uniform1i(*location.deref() as i32, value as _);
        }
    }

    pub fn uniform_1f(&self, location: &WebGLUniformLocation, value: f32) {
        unsafe {
            gl::Uniform1f(*location.deref() as i32, value as _);
        }
    }

    pub fn uniform_2f(&self, location: &WebGLUniformLocation, value: (f32, f32)) {
        unsafe {
            gl::Uniform2f(*location.deref() as _, value.0, value.1);
        }
    }

    pub fn uniform_3f(&self, location: &WebGLUniformLocation, value: (f32, f32, f32)) {
        unsafe {
            gl::Uniform3f(*location.deref() as _, value.0, value.1, value.2);
        }
    }

    pub fn uniform_4f(&self, location: &WebGLUniformLocation, value: (f32, f32, f32, f32)) {
        unsafe {
            gl::Uniform4f(*location.deref() as _, value.0, value.1, value.2, value.3);
        }
    }

    pub fn tex_parameteri(&self, pname: TextureParameter, param: i32) {
        unsafe {
            gl::TexParameteri(gl::TEXTURE_2D, pname as _, param);
        }
    }

    pub fn tex_parameterfv(&self, pname: TextureParameter, param: f32) {
        unsafe {
            gl::TexParameterfv(gl::TEXTURE_2D, pname as _, &param);
        }
    }

    pub fn create_vertex_array(&self) -> WebGLVertexArray {
        let mut vao = WebGLVertexArray(0);
        unsafe {
            gl::GenVertexArrays(1, &mut vao.0);
        }
        check_gl_error("create_vertex_array");
        vao
    }

    pub fn bind_vertex_array(&self, vao: &WebGLVertexArray) {
        unsafe {
            gl::BindVertexArray(vao.0);
        }
        check_gl_error("bind_vertex_array");
    }

    pub fn unbind_vertex_array(&self) {
        unsafe {
            gl::BindVertexArray(0);
        }
    }

    pub fn create_framebuffer(&self) -> WebGLFrameBuffer {
        let mut fb = WebGLFrameBuffer(0);
        unsafe {
            gl::GenFramebuffers(1, &mut fb.0);
        }
        fb
    }

    pub fn bind_framebuffer(&self, buffer: Buffers, fb: &WebGLFrameBuffer) {
        unsafe {
            gl::BindFramebuffer(buffer as u32, fb.0);
        }
    }

    pub fn framebuffer_texture2d(
        &self,
        target: Buffers,
        attachment: Buffers,
        textarget: TextureBindPoint,
        texture: &WebGLTexture,
        level: i32,
    ) {
        unsafe {
            gl::FramebufferTexture2D(
                target as u32,
                attachment as u32,
                textarget as u32,
                texture.0,
                level,
            );
        }
    }

    pub fn unbind_framebuffer(&self, buffer: Buffers) {
        unsafe {
            gl::BindFramebuffer(buffer as u32, 0);
        }
    }
}
