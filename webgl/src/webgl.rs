use std::ops::Deref;
use stdweb::web::*;
use stdweb::unstable::TryInto;
use stdweb::web::html_element::CanvasElement;
use glenum::*;
use common::*;

pub type WebGLContext<'a> = &'a CanvasElement;

impl WebGLRenderingContext {
    pub fn new(canvas: WebGLContext) -> WebGLRenderingContext {
        WebGLRenderingContext {
            common: GLContext::new(&canvas.clone().into()),
        }
    }
}

// impl WebGL2RenderingContext {
//     pub fn new(canvas: &Element) -> WebGL2RenderingContext {
//         WebGL2RenderingContext {
//             common: GLContext::new(canvas, "webgl2"),
//         }
//     }
// }

// Using a "hidden feature" of stdweb to reduce the js serialized overhead
extern "C" {

    fn __js_uniform4vf(
        ctx: i32,
        loc: i32,
        _: f32,
        _: f32,
        _: f32,
        _: f32,
        _: f32,
        _: f32,
        _: f32,
        _: f32,
        _: f32,
        _: f32,
        _: f32,
        _: f32,
        _: f32,
        _: f32,
        _: f32,
        _: f32,
        _: *const u8,
    );
}

// Using a "hidden feature" of stdweb to reduce the js serialized overhead
macro_rules! __js_peel {
    ($($token:tt)*) => {
        _js_impl!( @stringify [@no_return] -> $($token)* )
    };
}

macro_rules! js_raw {
    ({$($token:tt)*}) => {
        __js_raw_asm!(__js_peel!($($token)*))
    };

    ( {$($token:tt)*}, [$($args:tt)*]) => {
        __js_raw_asm!(__js_peel!($($token)*), $($args)*)
    };
}

impl GLContext {
    pub fn log<T: Into<String>>(&self, _msg: T) {
        // js!{ console.log(@{msg.into()})};
    }

    pub fn print<T: Into<String>>(msg: T) {
        js!{ console.log(@{msg.into()})};
    }

    pub fn new<'a>(canvas: &Element) -> GLContext {
        let gl = js!{
            var gl = (@{canvas}).getContext("webgl2");
            if (!gl) {
                gl = (@{canvas}).getContext("webgl");
            }

            // Create gl related objects
            if( !Module.gl) {
                Module.gl = {};
                Module.gl.counter = 1;

                Module.gl.matrix4x4 = new Float32Array([
                    1.0, 0,   0,   0,
                    0,   1.0, 0.0, 0,
                    0,   0,   1.0, 0,
                    0,   0,   0,   1.0
                ]);

                Module.gl.pool = {};
                Module.gl.get = function(id) {
                    return Module.gl.pool[id];
                };
                Module.gl.add = function(o) {
                    var c = Module.gl.counter;
                    Module.gl.pool[c] = o;
                    Module.gl.counter += 1;
                    return c;
                };

                Module.gl.remove = function(id) {
                    delete Module.gl.pool[id];
                    return c;
                };
                console.log("opengl "+gl.getParameter(gl.VERSION));
                console.log("shading language " + gl.getParameter(gl.SHADING_LANGUAGE_VERSION));
                console.log("vendor " + gl.getParameter(gl.VENDOR));
            }

            return Module.gl.add(gl);
        };

        GLContext {
            reference: gl.try_into().unwrap(),
        }
    }

    pub fn create_buffer(&self) -> WebGLBuffer {
        self.log("create_buffer");
        let value = js_raw!({
            var ctx = Module.gl.get(@{a0});
            return Module.gl.add(ctx.createBuffer());
        }, [self.reference] );
        WebGLBuffer(value)
    }

    pub fn buffer_data(&self, kind: BufferKind, data: &[u8], draw: DrawMode) {
        self.log("buffer_data");

        js! {
            @(no_return)
            var ctx = Module.gl.get(@{self.reference});
            ctx.bufferData(@{kind as u32},@{ TypedArray::from(data) }, @{draw as u32})
        };
    }

    pub fn bind_buffer(&self, kind: BufferKind, buffer: &WebGLBuffer) {
        self.log("bind_buffer");
        js! {
            @(no_return)
            var ctx = Module.gl.get(@{self.reference});
            var buf = Module.gl.get(@{buffer.deref()});

            ctx.bindBuffer(@{kind as u32},buf)
        };
    }

    pub fn unbind_buffer(&self, kind: BufferKind) {
        self.log("unbind_buffer");
        js! {
            @(no_return)
            var ctx = Module.gl.get(@{&self.reference});
            ctx.bindBuffer(@{kind as u32},null);
        }
    }

    pub fn create_shader(&self, kind: ShaderKind) -> WebGLShader {
        self.log("create_shader");
        let value = js! {
            var ctx = Module.gl.get(@{&self.reference});
            return Module.gl.add( ctx.createShader(@{ kind as u32 }) );
        };

        WebGLShader(value.try_into().unwrap())
    }

    pub fn shader_source(&self, shader: &WebGLShader, code: &str) {
        self.log("shader_source");
        js! {
            @(no_return)
            var ctx = Module.gl.get(@{&self.reference});
            var shader = Module.gl.get(@{shader.deref()});
            ctx.shaderSource(shader,@{ code })
        };
    }

    pub fn compile_shader(&self, shader: &WebGLShader) {
        self.log("compile_shader");
        js! {
            var ctx = Module.gl.get(@{&self.reference});
            var shader = Module.gl.get(@{shader.deref()});
            ctx.compileShader(shader);

            var compiled = ctx.getShaderParameter(shader, 0x8B81);
            console.log("Shader compiled successfully:", compiled);
            var compilationLog = ctx.getShaderInfoLog(shader);
            console.log("Shader compiler log:",compilationLog);
        };
    }

    pub fn create_program(&self) -> WebGLProgram {
        self.log("create_program");
        let value = js! {
            var ctx = Module.gl.get(@{&self.reference});
            var h = {};
            h.prog = ctx.createProgram();
            h.uniform_names = {};
            return Module.gl.add(h);
        };
        WebGLProgram(value.try_into().unwrap())
    }

    pub fn link_program(&self, program: &WebGLProgram) {
        self.log("link_program");
        js! {
            var ctx = Module.gl.get(@{&self.reference});
            var h = Module.gl.get(@{program.deref()});
            ctx.linkProgram(h.prog);
        };
    }

    pub fn use_program(&self, program: &WebGLProgram) {
        self.log("use_program");
        js! {
            var ctx = Module.gl.get(@{&self.reference});
            var h = Module.gl.get(@{program.deref()});
            ctx.useProgram(h.prog)
        };
    }

    pub fn attach_shader(&self, program: &WebGLProgram, shader: &WebGLShader) {
        self.log("attach_shader");
        js! {
            var ctx = Module.gl.get(@{&self.reference});
            var h = Module.gl.get(@{program.deref()});
            var shader = Module.gl.get(@{shader.deref()});
            ctx.attachShader(h.prog, shader)
        };
    }

    pub fn get_attrib_location(&self, program: &WebGLProgram, name: &str) -> Option<u32> {
        self.log("get_attrib_location");
        let value = js! {
            var ctx = Module.gl.get(@{&self.reference});
            var h = Module.gl.get(@{program.deref()});
            var r = ctx.getAttribLocation(h.prog,@{name});
            return r >= 0 ? r : null;
        };
        value.try_into().ok() as _
    }

    pub fn get_uniform_location(
        &self,
        program: &WebGLProgram,
        name: &str,
    ) -> Option<WebGLUniformLocation> {
        self.log("get_uniform_location");
        let value = js! {
            var ctx = Module.gl.get(@{&self.reference});
            var h = Module.gl.get(@{program.deref()});

            var name = @{name};
            var uniform = h.uniform_names[name];
            if(name in h.uniform_names) return h.uniform_names[name];

            uniform = Module.gl.add(ctx.getUniformLocation(h.prog,name));
            h.uniform_names[name] = uniform;

            return uniform;
        };

        value.try_into().ok().map(|uni| WebGLUniformLocation {
            reference: uni,
            name: name.into(),
        })
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
        self.log("vertex_attribute_pointer");
        let params = js! { return [@{location},@{size as u16},@{kind as i32},@{normalized}] };
        js! {
            var p = @{params};
            var ctx = Module.gl.get(@{&self.reference});

            ctx.vertexAttribPointer(p[0],p[1],p[2],p[3],@{stride},@{offset});
        };
    }

    pub fn enable_vertex_attrib_array(&self, location: u32) {
        self.log("enabled_vertex_attrib_array");
        js! {
            var ctx = Module.gl.get(@{&self.reference});
            ctx.enableVertexAttribArray(@{location})
        };
    }

    pub fn clear_color(&self, r: f32, g: f32, b: f32, a: f32) {
        self.log("clear_color");

        js! {
            var p = [@{r},@{g},@{b},@{a}];
            var ctx = Module.gl.get(@{&self.reference});
            ctx.clearColor(p[0],p[1],p[2],p[3]);
        };
    }

    pub fn enable(&self, flag: i32) {
        self.log("enable");
        js! {
            var ctx = Module.gl.get(@{&self.reference});
            ctx.enable(@{flag as i32})
        };
    }

    pub fn disable(&self, flag: i32) {
        self.log("disable");
        js! {
            var ctx = Module.gl.get(@{&self.reference});
            ctx.disable(@{flag as i32})
        };
    }

    pub fn cull_face(&self, flag: Culling) {
        self.log("cull_face");
        js! {
            var ctx = Module.gl.get(@{&self.reference});
            ctx.cullFace(@{flag as i32})
        };
    }

    pub fn clear(&self, bit: BufferBit) {
        self.log("clear");
        js! {
            var ctx = Module.gl.get(@{&self.reference});
            ctx.clear(@{bit as i32})
        };
    }

    pub fn viewport(&self, x: i32, y: i32, width: u32, height: u32) {
        self.log("viewport");
        let params = js! { return [@{x},@{y},@{width},@{height}] };
        js! {
            var ctx = Module.gl.get(@{&self.reference});
            var p = @{params};
            ctx.viewport(p[0],p[1],p[2],p[3]);
        };
    }

    pub fn draw_elements(&self, mode: Primitives, count: usize, kind: DataType, offset: u32) {
        self.log("draw_elemnts");
        js_raw!({
            var ctx = Module.gl.get(@{0});
            ctx.drawElements(@{1},@{2},@{3},@{4});
        }, [self.reference, mode as i32, count as i32, kind as i32, offset as i32 ]);
    }

    pub fn draw_arrays(&self, mode: Primitives, count: usize) {
        self.log("draw_arrays");
        js! {
            var ctx = Module.gl.get(@{&self.reference});
            ctx.drawArrays(@{mode as i32},0,@{count as i32});
        };
    }

    pub fn pixel_storei(&self, storage: PixelStorageMode, value: i32) {
        self.log("pixel_storei");
        js!{
            var ctx = Module.gl.get(@{&self.reference});
            ctx.pixelStorei(@{storage as i32},@{value});
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
        self.log("tex_img2d");
        let params1 = js! { return [@{target as u32},@{level as u32},@{format as u32}] };
        let params2 =
            js! { return [@{width as u32},@{height as u32},@{format as u32},@{kind as u32}] };

        if pixels.len() > 0 {
            js!{
                var p = @{params1}.concat(@{params2});
                var ctx = Module.gl.get(@{&self.reference});

                ctx.texImage2D(p[0],p[1], p[2] ,p[3],p[4],0,p[2],p[6],@{TypedArray::from(pixels)});
            };
        } else {
            js!{
                var p = @{params1}.concat(@{params2});
                var ctx = Module.gl.get(@{&self.reference});

                ctx.texImage2D(p[0],p[1], p[2] ,p[3],p[4],0,p[2],p[6],null);
            };
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
        self.log("sub_tex_img2d");
        let params1 =
            js! { return [@{target as u32},@{level as u32},@{xoffset as u32},@{yoffset as u32}] };
        let params2 =
            js! { return [@{width as u32},@{height as u32},@{format as u32},@{kind as u32}] };
        js!{
            var p = @{params1}.concat(@{params2});
            var ctx = Module.gl.get(@{&self.reference});
            ctx.texSubImage2D(p[0],p[1],p[2],p[3],p[4],p[5],p[6],p[7],@{TypedArray::from(pixels)});
        };
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
        self.log("compressed_tex_img2d");
        let params =
            js! { return [@{target as u32},@{level as u32},@{width as u32},@{height as u32}] };
        // for some reason this needs to be called otherwise invalid format error, extension initialization?
        js! {
            var ctx = Module.gl.get(@{&self.reference});

            // for some reason this needs to be called otherwise invalid format error, extension initialization?
            (ctx.getExtension("WEBGL_compressed_texture_s3tc") ||
                ctx.getExtension("MOZ_WEBGL_compressed_texture_s3tc") ||
                ctx.getExtension("WEBKIT_WEBGL_compressed_texture_s3tc"));

            var p = @{params};

            ctx.compressedTexImage2D(
                p[0],
                p[1],
                @{compression as u16},
                p[2],
                p[3],
                0,
                @{TypedArray::from(data)}
            );

            return 0;
        }
        self.log("compressed_tex_img2d end");
    }

    ///
    pub fn create_texture(&self) -> WebGLTexture {
        self.log("create_tex");
        let handle = js!{
            var ctx = Module.gl.get(@{&self.reference});
            return Module.gl.add(ctx.createTexture()) ;
        };
        WebGLTexture(handle.try_into().unwrap())
    }

    pub fn delete_texture(&self, texture: &WebGLTexture) {
        self.log("delete_tex");
        js!{
            var ctx = Module.gl.get(@{&self.reference});
            var tex = Module.gl.get(@{&texture.0});
            ctx.deleteTexture(tex);
            Module.gl.remove(tex);
        }
    }

    pub fn active_texture(&self, active: u32) {
        self.log("active_texture");
        js!{
            var ctx = Module.gl.get(@{&self.reference});
            ctx.activeTexture(ctx.TEXTURE0 + @{active})
        }
    }

    pub fn bind_texture(&self, texture: &WebGLTexture) {
        self.log("bind_tex");
        js!{
            var ctx = Module.gl.get(@{&self.reference});
            var tex = Module.gl.get(@{&texture.0});
            ctx.bindTexture(@{TextureBindPoint::Texture2d as u32 }, tex)
        }
    }

    pub fn unbind_texture(&self) {
        self.log("unbind_tex");
        js!{
            var ctx = Module.gl.get(@{&self.reference});
            ctx.bindTexture(@{TextureBindPoint::Texture2d as u32 },null)
        }
    }

    pub fn blend_func(&self, b1: BlendMode, b2: BlendMode) {
        self.log("blend_func");
        js!{
            var ctx = Module.gl.get(@{&self.reference});
            ctx.blendFunc(@{b1 as u32},@{b2 as u32})
        }
    }

    pub fn uniform_matrix_4fv(&self, location: &WebGLUniformLocation, value: &[[f32; 4]; 4]) {
        self.log("uniform_matrix_4fv");
        use std::mem;
        let array = unsafe { mem::transmute::<&[[f32; 4]; 4], &[f32; 16]>(value) as &[f32] };

        unsafe {
            __js_uniform4vf(
                self.reference,
                *location.deref(),
                array[0],
                array[1],
                array[2],
                array[3],
                array[4],
                array[5],
                array[6],
                array[7],
                array[8],
                array[9],
                array[10],
                array[11],
                array[12],
                array[13],
                array[14],
                array[15],
                r###"
            var ctx = Module.gl.get($0);
            var loc = Module.gl.get($1);
            var m = Module.gl.matrix4x4;
            m[0] = $2;
            m[1] = $3;
            m[2] = $4;
            m[3] = $5;
            m[4] = $6;
            m[5] = $7;
            m[6] = $8;
            m[7] = $9;
            m[8] = $10;
            m[9] = $11;
            m[10] = $12;
            m[11] = $13;
            m[12] = $14;
            m[13] = $15;
            m[14] = $16;
            m[15] = $17;

            return ctx.uniformMatrix4fv(loc,false, m);
        "### as *const _ as *const u8,
            );
        }
    }

    pub fn uniform_matrix_3fv(&self, location: &WebGLUniformLocation, value: &[[f32; 3]; 3]) {
        self.log("uniform_matrix_3fv");
        use std::mem;
        let array = unsafe { mem::transmute::<&[[f32; 3]; 3], &[f32; 9]>(value) as &[f32] };
        js!{
            var ctx = Module.gl.get(@{self.reference});
            var loc = Module.gl.get(@{location.deref()});
            ctx.uniformMatrix3fv(loc,false,@{&array})
        }
    }

    pub fn uniform_matrix_2fv(&self, location: &WebGLUniformLocation, value: &[[f32; 2]; 2]) {
        use std::mem;
        let array = unsafe { mem::transmute::<&[[f32; 2]; 2], &[f32; 4]>(value) as &[f32] };
        js!{
            var ctx = Module.gl.get(@{self.reference});
            var loc = Module.gl.get(@{location.deref()});
            ctx.uniformMatrix2fv(loc,false,@{&array})
        }
    }

    pub fn uniform_1i(&self, location: &WebGLUniformLocation, value: i32) {
        js!{
            var ctx = Module.gl.get(@{self.reference});
            var loc = Module.gl.get(@{location.deref()});
            ctx.uniform1i(loc,@{value})
        }
    }

    pub fn uniform_1f(&self, location: &WebGLUniformLocation, value: f32) {
        js!{
            var ctx = Module.gl.get(@{self.reference});
            var loc = Module.gl.get(@{location.deref()});
            ctx.uniform1f(loc,@{value});
        }
    }

    pub fn uniform_2f(&self, location: &WebGLUniformLocation, value: (f32, f32)) {
        js!{
            var p = [@{value.0},@{value.1}];
            var ctx = Module.gl.get(@{self.reference});
            var loc = Module.gl.get(@{location.deref()});

            ctx.uniform2f(loc,p[0],p[1])
        }
    }

    pub fn uniform_3f(&self, location: &WebGLUniformLocation, value: (f32, f32, f32)) {
        js!{
            var p = [@{value.0},@{value.1},@{value.2}];
            var ctx = Module.gl.get(@{self.reference});
            var loc = Module.gl.get(@{location.deref()});

            ctx.uniform3f(loc,p[0],p[1],p[2])
        }
    }

    pub fn uniform_4f(&self, location: &WebGLUniformLocation, value: (f32, f32, f32, f32)) {
        js!{
            var p = [@{value.0},@{value.1},@{value.2},@{value.3}];
            var ctx = Module.gl.get(@{self.reference});
            var loc = Module.gl.get(@{location.deref()});

            ctx.uniform4f(loc,p[0],p[1],p[2],p[3])
        }
    }

    pub fn create_vertex_array(&self) -> WebGLVertexArray {
        self.log("create_vertex_array");
        let val = js! {
            var ctx = Module.gl.get(@{self.reference});
            if (ctx.createVertexArray) {
                return Module.gl.add(ctx.createVertexArray());
            } else {
                return 0;
            }
        };
        WebGLVertexArray(val.try_into().unwrap())
    }

    pub fn bind_vertex_array(&self, vao: &WebGLVertexArray) {
        self.log("bind_vertex_array");
        js! {
            var ctx = Module.gl.get(@{self.reference});
            if (ctx.bindVertexArray) {
                var vao = Module.gl.get(@{vao.deref()});
                ctx.bindVertexArray(vao);
            }
        }
    }

    pub fn unbind_vertex_array(&self) {
        self.log("unbind_vertex_array");
        js! {
            var ctx = Module.gl.get(@{self.reference});
            if (ctx.unbindVertexArray) {
                ctx.unbindVertexArray(0);
            }
        }
    }

    pub fn get_program_parameter(&self, program: &WebGLProgram, pname: ShaderParameter) -> i32 {
        let res = js! {
            var h = Module.gl.get(@{program.deref()});
            var ctx = Module.gl.get(@{self.reference});

            return ctx.getProgramParameter(h.prog,@{pname as u32});
        };

        res.try_into().unwrap()
    }

    // pub fn get_active_uniform(&self, program: &WebGLProgram, location: u32) -> WebGLActiveInfo {
    //     let res = js! {
    //         var h = Module.gl.get(@{program.deref()});
    //         var ctx = Module.gl.get(@{self.reference});

    //         return ctx.getActiveUniform(h.prog,@{location})
    //     };

    //     let name = js! { return @{&res}.name };
    //     let size = js!{ return @{&res}.size };
    //     let kind = js!{ return @{&res}.type };
    //     let k: u32 = kind.try_into().unwrap();
    //     use std::mem;
    //     WebGLActiveInfo::new(
    //         name.into_string().unwrap(),
    //         size.try_into().unwrap(),
    //         unsafe { mem::transmute::<u16, UniformType>(k as _) },
    //         res.into_reference().unwrap(),
    //     )
    // }

    // pub fn get_active_attrib(&self, program: &WebGLProgram, location: u32) -> WebGLActiveInfo {
    //     let res = js! {
    //         var h = Module.gl.programs[@{program.deref()}];
    //         return @{self.reference}.getActiveAttrib(h.prog,@{location})
    //     };
    //     let name = js! { return @{&res}.name };
    //     let size = js!{ return @{&res}.size };
    //     let kind = js!{ return @{&res}.type };
    //     let k: u32 = kind.try_into().unwrap();
    //     use std::mem;
    //     WebGLActiveInfo::new(
    //         name.into_string().unwrap(),
    //         size.try_into().unwrap(),
    //         unsafe { mem::transmute::<u16, UniformType>(k as _) },
    //         res.into_reference().unwrap(),
    //     )
    // }

    pub fn tex_parameteri(&self, pname: TextureParameter, param: i32) {
        js! {
            var ctx = Module.gl.get(@{self.reference});
            return ctx.texParameteri(@{TextureBindPoint::Texture2d as u32},@{pname as u32},@{param})
        };
    }

    pub fn tex_parameterfv(&self, pname: TextureParameter, param: f32) {
        js! {
            var ctx = Module.gl.get(@{self.reference});
            return ctx.texParameterf(@{TextureBindPoint::Texture2d as u32},@{pname as u32},@{param})
        };
    }

    pub fn create_framebuffer(&self) -> WebGLFrameBuffer {
        let val = js! {
            var ctx = Module.gl.get(@{self.reference});
            return Module.gl.add(ctx.createFramebuffer());
        };
        WebGLFrameBuffer(val.try_into().unwrap())
    }

    pub fn bind_framebuffer(&self, buffer: Buffers, fb: &WebGLFrameBuffer) {
        js! {
            var ctx = Module.gl.get(@{self.reference});
            var fb = Module.gl.get(@{fb.deref()});
            ctx.bindFramebuffer(@{buffer as u32}, fb);
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
        js! {
            var ctx = Module.gl.get(@{self.reference});
            var tex = Module.gl.get(@{&texture.0});
            ctx.framebufferTexture2D(@{target as u32},@{attachment as u32},@{textarget as u32},tex,@{level});
        }
    }

    pub fn unbind_framebuffer(&self, buffer: Buffers) {
        self.log("unbind_framebuffer");
        js!{
            var ctx = Module.gl.get(@{&self.reference});
            ctx.bindFramebuffer(@{buffer as u32},null)
        }
    }
}
