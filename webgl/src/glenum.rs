/*
    Copy from https://docs.rs/crate/glenum/0.1.1/source/src/lib.rs

    Documentation taken from https://developer.mozilla.org/en-US/docs/Web/API/WebGL_API/Constants
*/

/// Constants passed to WebGLRenderingContext.vertexAttribPointer()
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum AttributeSize {
    One = 1,
    Two = 2,
    Three = 3,
    Four = 4,
}

/// Constants passed to WebGLRenderingContext.createShader()
#[derive(Debug, Clone, Copy)]
pub enum ShaderKind {
    /// Passed to createShader to define a fragment shader.
    Fragment = 0x8B30,
    /// Passed to createShader to define a vertex shader
    Vertex = 0x8B31,
}

/// Constants passed to WebGLRenderingContext.createShader()
#[derive(Debug, Clone, Copy)]
pub enum ShaderParameter {
    /// Passed to getShaderParamter to get the status of the compilation. Returns false if the shader was not compiled. You can then query getShaderInfoLog to find the exact error
    CompileStatus = 0x8B81,
    /// Passed to getShaderParamter to determine if a shader was deleted via deleteShader. Returns true if it was, false otherwise.
    DeleteStatus = 0x8B80,
    /// Passed to getProgramParameter after calling linkProgram to determine if a program was linked correctly. Returns false if there were errors. Use getProgramInfoLog to find the exact error.
    LinkStatus = 0x8B82,
    /// Passed to getProgramParameter after calling validateProgram to determine if it is valid. Returns false if errors were found.
    ValidateStatus = 0x8B83,
    /// Passed to getProgramParameter after calling attachShader to determine if the shader was attached correctly. Returns false if errors occurred.
    AttachedShaders = 0x8B85,
    /// Passed to getProgramParameter to get the number of attributes active in a program.
    ActiveAttributes = 0x8B89,
    /// Passed to getProgramParamter to get the number of uniforms active in a program.
    ActiveUniforms = 0x8B86,
    /// The maximum number of entries possible in the vertex attribute list.
    MaxVertexAttribs = 0x8869,
    ///
    MaxVertexUniformVectors = 0x8DFB,
    ///
    MaxVaryingVectors = 0x8DFC,
    ///
    MaxCombinedTextureImageUnits = 0x8B4D,
    ///
    MaxVertexTextureImageUnits = 0x8B4C,
    /// Implementation dependent number of maximum texture units. At least 8.
    MaxTextureImageUnits = 0x8872,
    ///
    MaxFragmentUniformVectors = 0x8DFD,
    ///
    ShaderType = 0x8B4F,
    ///
    ShadingLanguageVersion = 0x8B8C,
    ///
    CurrentProgram = 0x8B8D,
}

/// Passed to bindBuffer or bufferData to specify the type of buffer being used.
#[derive(Debug, Clone, Copy)]
pub enum BufferKind {
    Array = 0x8892,
    ElementArray = 0x8893,
}

#[derive(Debug, Clone, Copy)]
pub enum DrawMode {
    /// Passed to bufferData as a hint about whether the contents of the buffer are likely to be used often and not change often.
    Static = 0x88E4,
    /// Passed to bufferData as a hint about whether the contents of the buffer are likely to be used often and change often.
    Dynamic = 0x88E8,
    /// Passed to bufferData as a hint about whether the contents of the buffer are likely to not be used often.
    Stream = 0x88E0,
}

#[derive(Debug, Clone, Copy)]
pub enum BufferParameter {
    /// Passed to getBufferParameter to get a buffer's size.
    Size = 0x8764,
    /// Passed to getBufferParameter to get the hint for the buffer passed in when it was created.
    Usage = 0x8765,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum DataType {
    I8 = 0x1400,
    U8 = 0x1401,
    I16 = 0x1402,
    U16 = 0x1403,
    I32 = 0x1404,
    U32 = 0x1405,
    Float = 0x1406,
}

#[derive(Debug, Clone, Copy)]
pub enum Flag {
    /// Passed to enable/disable to turn on/off blending. Can also be used with getParameter to find the current blending method.
    Blend = 0x0BE2,
    /// Passed to enable/disable to turn on/off the depth test. Can also be used with getParameter to query the depth test.
    DepthTest = 0x0B71,
    /// Passed to enable/disable to turn on/off dithering. Can also be used with getParameter to find the current dithering method.
    Dither = 0x0BD0,
    /// Passed to enable/disable to turn on/off the polygon offset. Useful for rendering hidden-line images, decals, and or solids with highlighted edges. Can also be used with getParameter to query the scissor test.
    PolygonOffsetFill = 0x8037,
    /// Passed to enable/disable to turn on/off the alpha to coverage. Used in multi-sampling alpha channels.
    SampleAlphaToCoverage = 0x809E,
    /// Passed to enable/disable to turn on/off the sample coverage. Used in multi-sampling.
    SampleCoverage = 0x80A0,
    /// Passed to enable/disable to turn on/off the scissor test. Can also be used with getParameter to query the scissor test.
    ScissorTest = 0x0C11,
    /// Passed to enable/disable to turn on/off the stencil test. Can also be used with getParameter to query the stencil test.
    StencilTest = 0x0B90,
}

#[derive(Debug, Clone, Copy)]
pub enum BufferBit {
    /// Passed to clear to clear the current depth buffer.
    Depth = 0x00000100,
    /// Passed to clear to clear the current stencil buffer.
    Stencil = 0x00000400,
    /// Passed to clear to clear the current color buffer.
    Color = 0x00004000,
}

/// Passed to drawElements or drawArrays to draw primitives.
#[derive(Debug, Clone, Copy)]
pub enum Primitives {
    /// Passed to drawElements or drawArrays to draw single points.
    Points = 0x0000,
    /// Passed to drawElements or drawArrays to draw lines. Each vertex connects to the one after it.
    Lines = 0x0001,
    /// Passed to drawElements or drawArrays to draw lines. Each set of two vertices is treated as a separate line segment.
    LineLoop = 0x0002,
    /// Passed to drawElements or drawArrays to draw a connected group of line segments from the first vertex to the last.
    LineStrip = 0x0003,
    /// Passed to drawElements or drawArrays to draw triangles. Each set of three vertices creates a separate triangle.
    Triangles = 0x0004,
    /// Passed to drawElements or drawArrays to draw a connected group of triangles.
    TriangleStrip = 0x0005,
    /// Passed to drawElements or drawArrays to draw a connected group of triangles. Each vertex connects to the previous and the first vertex in the fan.
    TriangleFan = 0x0006,
}

/// Constants passed to WebGLRenderingContext.blendFunc() or WebGLRenderingContext.blendFuncSeparate() to specify the blending mode (for both, RBG and alpha, or separately).
#[derive(Debug, Clone, Copy)]
pub enum BlendMode {
    /// Passed to blendFunc or blendFuncSeparate to turn off a component.
    Zero = 0,
    /// Passed to blendFunc or blendFuncSeparate to turn on a component.
    One = 1,
    /// Passed to blendFunc or blendFuncSeparate to multiply a component by the source elements color.
    SrcColor = 0x0300,
    /// Passed to blendFunc or blendFuncSeparate to multiply a component by one minus the source elements color.
    OneMinusSrcColor = 0x0301,
    /// Passed to blendFunc or blendFuncSeparate to multiply a component by the source's alpha.
    SrcAlpha = 0x0302,
    /// Passed to blendFunc or blendFuncSeparate to multiply a component by one minus the source's alpha.
    OneMinusSrcAlpha = 0x0303,
    /// Passed to blendFunc or blendFuncSeparate to multiply a component by the destination's alpha.
    DstAlpha = 0x0304,
    /// Passed to blendFunc or blendFuncSeparate to multiply a component by one minus the destination's alpha.
    OneMinusDstAlpha = 0x0305,
    /// Passed to blendFunc or blendFuncSeparate to multiply a component by the destination's color.
    DstColor = 0x0306,
    /// Passed to blendFunc or blendFuncSeparate to multiply a component by one minus the destination's color.
    OneMinusDstColor = 0x0307,
    /// Passed to blendFunc or blendFuncSeparate to multiply a component by the minimum of source's alpha or one minus the destination's alpha.
    SrcAlphaSaturate = 0x0308,
    /// Passed to blendFunc or blendFuncSeparate to specify a constant color blend function.
    ConstantColor = 0x8001,
    /// Passed to blendFunc or blendFuncSeparate to specify one minus a constant color blend function.
    OneMinusConstantColor = 0x8002,
    /// Passed to blendFunc or blendFuncSeparate to specify a constant alpha blend function.
    ConstantAlpha = 0x8003,
    /// Passed to blendFunc or blendFuncSeparate to specify one minus a constant alpha blend function.
    OneMinusConstantAlpha = 0x8004,
}

/// Constants passed to WebGLRenderingContext.blendEquation()
/// or WebGLRenderingContext.blendEquationSeparate() to control
/// how the blending is calculated (for both, RBG and alpha, or separately).
#[derive(Debug, Clone, Copy)]
pub enum BlendEquation {
    /// Passed to blendEquation or blendEquationSeparate to set an addition blend function.
    FuncAdd = 0x8006,
    /// Passed to blendEquation or blendEquationSeparate to specify a subtraction blend function (source - destination).
    FuncSubstract = 0x800A,
    /// Passed to blendEquation or blendEquationSeparate to specify a reverse subtraction blend function (destination - source).
    FuncReverseSubtract = 0x800B,
}

/// Constants passed to WebGLRenderingContext.getParameter() to specify what information to return.
#[derive(Debug, Clone, Copy)]
pub enum Parameter {
    /// Passed to getParameter to get the current RGB blend function. same as BlendEquationRgb
    BlendEquation = 0x8009,
    /// Passed to getParameter to get the current alpha blend function. Same as BLEND_EQUATION
    BlendEquationAlpha = 0x883D,
    /// Passed to getParameter to get the current destination RGB blend function.
    BlendDstRgb = 0x80C8,
    /// Passed to getParameter to get the current destination RGB blend function.
    BlendSrcRgb = 0x80C9,
    /// Passed to getParameter to get the current destination alpha blend function.
    BlendDstAlpha = 0x80CA,
    /// Passed to getParameter to get the current source alpha blend function.
    BlendSrcAlpha = 0x80CB,
    /// Passed to getParameter to return a the current blend color.
    BlendColor = 0x8005,
    /// Passed to getParameter to get the array buffer binding.
    ArrayBufferBinding = 0x8894,
    /// Passed to getParameter to get the current element array buffer.
    ElementArrayBufferBinding = 0x8895,
    /// Passed to getParameter to get the current lineWidth (set by the lineWidth method).
    LineWidth = 0x0B21,
    /// Passed to getParameter to get the current size of a point drawn with gl.POINTS
    AliasedPointSizeRange = 0x846D,
    /// Passed to getParameter to get the range of available widths for a line. Returns a length-2 array with the lo value at 0, and hight at 1.
    AliasedLineWidthRange = 0x846E,
    /// Passed to getParameter to get the current value of cullFace. Should return FRONT, BACK, or FRONT_AND_BACK
    CullFaceMode = 0x0B45,
    /// Passed to getParameter to determine the current value of frontFace. Should return CW or CCW.
    FrontFace = 0x0B46,
    /// Passed to getParameter to return a length-2 array of floats giving the current depth range.
    DepthRange = 0x0B70,
    /// Passed to getParameter to determine if the depth write mask is enabled.
    DepthWritemask = 0x0B72,
    /// Passed to getParameter to determine the current depth clear value.
    DepthClearValue = 0x0B73,
    /// Passed to getParameter to get the current depth function. Returns NEVER, ALWAYS, LESS, EQUAL, LEQUAL, GREATER, GEQUAL, or NOTEQUAL.
    DepthFunc = 0x0B74,
    /// Passed to getParameter to get the value the stencil will be cleared to.
    StencilClearValue = 0x0B91,
    /// Passed to getParameter to get the current stencil function. Returns NEVER, ALWAYS, LESS, EQUAL, LEQUAL, GREATER, GEQUAL, or NOTEQUAL.
    StencilFunc = 0x0B92,
    /// Passed to getParameter to get the current stencil fail function. Should return KEEP, REPLACE, INCR, DECR, INVERT, INCR_WRAP, or DECR_WRAP.
    StencilFail = 0x0B94,
    /// Passed to getParameter to get the current stencil fail function should the depth buffer test fail. Should return KEEP, REPLACE, INCR, DECR, INVERT, INCR_WRAP, or DECR_WRAP.
    StencilPassDepthFail = 0x0B95,
    /// Passed to getParameter to get the current stencil fail function should the depth buffer test pass. Should return KEEP, REPLACE, INCR, DECR, INVERT, INCR_WRAP, or DECR_WRAP.
    StencilPassDepthPass = 0x0B96,
    /// Passed to getParameter to get the reference value used for stencil tests.
    StencilRef = 0x0B97,
    ///
    StencilValueMask = 0x0B93,
    ///
    StencilWritemask = 0x0B98,
    ///
    StencilBackFunc = 0x8800,
    ///
    StencilBackFail = 0x8801,
    ///
    StencilBackPassDepthFail = 0x8802,
    ///
    StencilBackPassDepthPass = 0x8803,
    ///
    StencilBackRef = 0x8CA3,
    ///
    StencilBackValueMask = 0x8CA4,
    ///
    StencilBackWritemask = 0x8CA5,
    /// Returns an Int32Array with four elements for the current viewport dimensions.
    Viewport = 0x0BA2,
    /// Returns an Int32Array with four elements for the current scissor box dimensions.
    ScissorBox = 0x0C10,
    ///
    ColorClearValue = 0x0C22,
    ///
    ColorWritemask = 0x0C23,
    ///
    UnpackAlignment = 0x0CF5,
    ///
    PackAlignment = 0x0D05,
    ///
    MaxTextureSize = 0x0D33,
    ///
    MaxViewportDims = 0x0D3A,
    ///
    SubpixelBits = 0x0D50,
    ///
    RedBits = 0x0D52,
    ///
    GreenBits = 0x0D53,
    ///
    BlueBits = 0x0D54,
    ///
    AlphaBits = 0x0D55,
    ///
    DepthBits = 0x0D56,
    ///
    StencilBits = 0x0D57,
    ///
    PolygonOffsetUnits = 0x2A00,
    ///
    PolygonOffsetFactor = 0x8038,
    ///
    TextureBinding2d = 0x8069,
    ///
    SampleBuffers = 0x80A8,
    ///
    Samples = 0x80A9,
    ///
    SampleCoverageValue = 0x80AA,
    ///
    SampleCoverageInvert = 0x80AB,
    ///
    CompressedTextureFormats = 0x86A3,
    ///
    Vendor = 0x1F00,
    ///
    Renderer = 0x1F01,
    ///
    Version = 0x1F02,
    ///
    ImplementationColorReadType = 0x8B9A,
    ///
    ImplementationColorReadFormat = 0x8B9B,
    ///
    BrowserDefaultWebgl = 0x9244,

    ///
    TextureBindingCubeMap = 0x8514,

    ///
    MaxCubeMapTextureSize = 0x851C,
}

/// Constants passed to WebGLRenderingContext.getVertexAttrib().
#[derive(Debug, Clone, Copy)]
pub enum VertexAttrib {
    /// Passed to getVertexAttrib to read back the current vertex attribute.
    Current = 0x8626,
    ///
    ArrayEnabled = 0x8622,
    ///
    ArraySize = 0x8623,
    ///
    ArrayStride = 0x8624,
    ///
    ArrayType = 0x8625,
    ///
    ArrayNormalized = 0x886A,
    ///
    ArrayPointer = 0x8645,
    ///
    ArrayBufferBinding = 0x889F,
}

/// Constants passed to WebGLRenderingContext.cullFace().
#[derive(Debug, Clone, Copy)]
pub enum Culling {
    /// Passed to enable/disable to turn on/off culling. Can also be used with getParameter to find the current culling method.
    CullFace = 0x0B44,
    /// Passed to cullFace to specify that only front faces should be drawn.
    Front = 0x0404,
    /// Passed to cullFace to specify that only back faces should be drawn.
    Back = 0x0405,
    /// Passed to cullFace to specify that front and back faces should be drawn.
    FrontAndBack = 0x0408,
}

/// Constants returned from WebGLRenderingContext.getError().
#[derive(Debug, Clone, Copy)]
pub enum Error {
    /// Returned from getError.
    NoError = 0,
    /// Returned from getError.
    InvalidEnum = 0x0500,
    /// Returned from getError.
    InvalidValue = 0x0501,
    /// Returned from getError.
    InvalidOperation = 0x0502,
    /// Returned from getError.
    OutOfMemory = 0x0505,
    /// Returned from getError.
    ContextLostWebgl = 0x9242,
}

/// Constants passed to WebGLRenderingContext.frontFace().
#[derive(Debug, Clone, Copy)]
pub enum FrontFaceDirection {
    /// Passed to frontFace to specify the front face of a polygon is drawn in the clockwise direction
    CW = 0x0900,
    /// Passed to frontFace to specify the front face of a polygon is drawn in the counter clockwise direction
    CCW = 0x0901,
}

/// Constants passed to WebGLRenderingContext.depthFunc().
#[derive(Debug, Clone, Copy)]
pub enum DepthTest {
    /// Passed to depthFunction or stencilFunction to specify depth or stencil tests will never pass. i.e. Nothing will be drawn.
    Never = 0x0200,
    /// Passed to depthFunction or stencilFunction to specify depth or stencil tests will always pass. i.e. Pixels will be drawn in the order they are drawn.
    Always = 0x0207,
    /// Passed to depthFunction or stencilFunction to specify depth or stencil tests will pass if the new depth value is less than the stored value.
    Less = 0x0201,
    /// Passed to depthFunction or stencilFunction to specify depth or stencil tests will pass if the new depth value is equals to the stored value.
    Equal = 0x0202,
    /// Passed to depthFunction or stencilFunction to specify depth or stencil tests will pass if the new depth value is less than or equal to the stored value.
    Lequal = 0x0203,
    /// Passed to depthFunction or stencilFunction to specify depth or stencil tests will pass if the new depth value is greater than the stored value.
    Greater = 0x0204,
    /// Passed to depthFunction or stencilFunction to specify depth or stencil tests will pass if the new depth value is greater than or equal to the stored value.
    Gequal = 0x0206,
    /// Passed to depthFunction or stencilFunction to specify depth or stencil tests will pass if the new depth value is not equal to the stored value.
    Notequal = 0x0205,
}

/// Constants passed to WebGLRenderingContext.stencilFunc().
#[derive(Debug, Clone, Copy)]
pub enum StencilTest {
    /// Passed to depthFunction or stencilFunction to specify depth or stencil tests will never pass. i.e. Nothing will be drawn.
    Never = 0x0200,
    /// Passed to depthFunction or stencilFunction to specify depth or stencil tests will always pass. i.e. Pixels will be drawn in the order they are drawn.
    Always = 0x0207,
    /// Passed to depthFunction or stencilFunction to specify depth or stencil tests will pass if the new depth value is less than the stored value.
    Less = 0x0201,
    /// Passed to depthFunction or stencilFunction to specify depth or stencil tests will pass if the new depth value is equals to the stored value.
    Equal = 0x0202,
    /// Passed to depthFunction or stencilFunction to specify depth or stencil tests will pass if the new depth value is less than or equal to the stored value.
    Lequal = 0x0203,
    /// Passed to depthFunction or stencilFunction to specify depth or stencil tests will pass if the new depth value is greater than the stored value.
    Greater = 0x0204,
    /// Passed to depthFunction or stencilFunction to specify depth or stencil tests will pass if the new depth value is greater than or equal to the stored value.
    Gequal = 0x0206,
    /// Passed to depthFunction or stencilFunction to specify depth or stencil tests will pass if the new depth value is not equal to the stored value.
    Notequal = 0x0205,
}

/// Constants passed to WebGLRenderingContext.stencilOp().
#[derive(Debug, Clone, Copy)]
pub enum StencilAction {
    ///
    Keep = 0x1E00,
    ///
    Replace = 0x1E01,
    ///
    Incr = 0x1E02,
    ///
    Decr = 0x1E03,
    ///
    Invert = 0x150A,
    ///
    IncrWrap = 0x8507,
    ///
    DecrWrap = 0x8508,
}

#[derive(Debug, Clone, Copy)]
pub enum PixelType {
    ///
    UnsignedByte = 0x1401,
    ///
    UnsignedShort4444 = 0x8033,
    ///
    UnsignedShort5551 = 0x8034,
    ///
    UnsignedShort565 = 0x8363,
}

#[derive(Debug, Clone, Copy)]
pub enum PixelFormat {
    ///
    DepthComponent = 0x1902,
    ///
    Alpha = 0x1906,
    ///
    Rgb = 0x1907,
    ///
    Rgba = 0x1908,
    ///
    Luminance = 0x1909,
    ///
    LuminanceAlpha = 0x190A,
}

/// Constants passed to WebGLRenderingContext.hint()
#[derive(Debug, Clone, Copy)]
pub enum Hint {
    /// There is no preference for this behavior.
    DontCare = 0x1100,
    /// The most efficient behavior should be used.
    Fastest = 0x1101,
    /// The most correct or the highest quality option should be used.
    Nicest = 0x1102,
    /// Hint for the quality of filtering when generating mipmap images with WebGLRenderingContext.generateMipmap().
    GenerateMipmapHint = 0x8192,
}

/// WebGLRenderingContext.texParameter[fi]() or WebGLRenderingContext.bindTexture() "target" parameter
#[derive(Debug, Clone, Copy)]
pub enum TextureKind {
    ///
    Texture2d = 0x0DE1,
    ///
    TextureCubeMap = 0x8513,
}

/// WebGLRenderingContext.texParameter[fi]() "pname" parameter
#[derive(Debug, Clone, Copy)]
pub enum TextureParameter {
    ///
    TextureMagFilter = 0x2800,
    ///
    TextureMinFilter = 0x2801,
    ///
    TextureWrapS = 0x2802,
    ///
    TextureWrapT = 0x2803,
    ///
    BorderColor = 0x1004,

    /// WebGL 2.0 only
    TextureWrapR = 32882,
}

/// WebGLRenderingContext.texImage2D() "target" parameter
#[derive(Debug, Clone, Copy)]
pub enum TextureBindPoint {
    ///
    Texture2d = 0x0DE1,
    ///
    TextureCubeMapPositiveX = 0x8515,
    ///
    TextureCubeMapNegativeX = 0x8516,
    ///
    TextureCubeMapPositiveY = 0x8517,
    ///
    TextureCubeMapNegativeY = 0x8518,
    ///
    TextureCubeMapPositiveZ = 0x8519,
    ///
    TextureCubeMapNegativeZ = 0x851A,
}

/// WebGLRenderingContext.texParameter[fi]() "param" parameter
#[derive(Debug, Clone, Copy)]
pub enum TextureMagFilter {
    ///
    Nearest = 0x2600,
    ///
    Linear = 0x2601,
}

/// WebGLRenderingContext.texParameter[fi]() "param" parameter
#[derive(Debug, Clone, Copy)]
pub enum TextureMinFilter {
    ///
    Nearest = 0x2600,
    ///
    Linear = 0x2601,
    ///
    NearestMipmapNearest = 0x2700,
    ///
    LinearMipmapNearest = 0x2701,
    ///
    NearestMipmapLinear = 0x2702,
    ///
    LinearMipmapLinear = 0x2703,
}

/// WebGLRenderingContext.texParameter[fi]() "param" parameter
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TextureWrap {
    ///
    Repeat = 0x2901,
    ///
    ClampToEdge = 0x812F,
    ///
    MirroredRepeat = 0x8370,
}

/// Constants passed to WebGLRenderingContext.hint()
#[derive(Debug, Clone, Copy)]
pub enum Buffers {
    ///
    Framebuffer = 0x8D40,
    ///
    Renderbuffer = 0x8D41,
    ///
    Rgba4 = 0x8056,
    ///
    Rgb5A1 = 0x8057,
    ///
    Rgb565 = 0x8D62,
    ///
    DepthComponent16 = 0x81A5,
    ///
    StencilIndex = 0x1901,
    ///
    StencilIndex8 = 0x8D48,
    ///
    DepthStencil = 0x84F9,
    ///
    RenderbufferWidth = 0x8D42,
    ///
    RenderbufferHeight = 0x8D43,
    ///
    RenderbufferInternalFormat = 0x8D44,
    ///
    RenderbufferRedSize = 0x8D50,
    ///
    RenderbufferGreenSize = 0x8D51,
    ///
    RenderbufferBlueSize = 0x8D52,
    ///
    RenderbufferAlphaSize = 0x8D53,
    ///
    RenderbufferDepthSize = 0x8D54,
    ///
    RenderbufferStencilSize = 0x8D55,
    ///
    FramebufferAttachmentObjectType = 0x8CD0,
    ///
    FramebufferAttachmentObjectName = 0x8CD1,
    ///
    FramebufferAttachmentTextureLevel = 0x8CD2,
    ///
    FramebufferAttachmentTextureCubeMapFace = 0x8CD3,
    ///
    ColorAttachment0 = 0x8CE0,
    ///
    DepthAttachment = 0x8D00,
    ///
    StencilAttachment = 0x8D20,
    ///
    DepthStencilAttachment = 0x821A,
    ///
    None = 0,
    ///
    FramebufferComplete = 0x8CD5,
    ///
    FramebufferIncompleteAttachment = 0x8CD6,
    ///
    FramebufferIncompleteMissingAttachment = 0x8CD7,
    ///
    FramebufferIncompleteDimensions = 0x8CD9,
    ///
    FramebufferUnsupported = 0x8CDD,
    ///
    FramebufferBinding = 0x8CA6,
    ///
    RenderbufferBinding = 0x8CA7,
    ///
    MaxRenderbufferSize = 0x84E8,
    ///
    InvalidFramebufferOperation = 0x0506,
}

/// Constants passed to WebGLRenderingContext.hint()
#[derive(Debug, Clone, Copy)]
pub enum PixelStorageMode {
    ///
    UnpackFlipYWebgl = 0x9240,
    ///
    UnpackPremultiplyAlphaWebgl = 0x9241,
    ///
    UnpackColorspaceConversionWebgl = 0x9243,
    /// Packing of pixel data into memory.
    /// Can be 1, 2, 4, 8 defaults to 4
    PackAlignment = 0x0D05,
    /// Unpacking of pixel data from memory
    /// Can be 1, 2, 4, 8 defaults to 4
    UnpackAlignment = 0x0CF5,
}

///
#[derive(Debug, Clone, Copy)]
pub enum ShaderPrecision {
    ///
    LowFloat = 0x8DF0,
    ///
    MediumFloat = 0x8DF1,
    ///
    HighFloat = 0x8DF2,
    ///
    LowInt = 0x8DF3,
    ///
    MediumInt = 0x8DF4,
    ///
    HighInt = 0x8DF5,
}

/// Constants passed to WebGLRenderingContext.hint()
#[derive(Debug, Clone, Copy)]
pub enum UniformType {
    ///
    FloatVec2 = 0x8B50,
    ///
    FloatVec3 = 0x8B51,
    ///
    FloatVec4 = 0x8B52,
    ///
    IntVec2 = 0x8B53,
    ///
    IntVec3 = 0x8B54,
    ///
    IntVec4 = 0x8B55,
    ///
    Bool = 0x8B56,
    ///
    BoolVec2 = 0x8B57,
    ///
    BoolVec3 = 0x8B58,
    ///
    BoolVec4 = 0x8B59,
    ///
    FloatMat2 = 0x8B5A,
    ///
    FloatMat3 = 0x8B5B,
    ///
    FloatMat4 = 0x8B5C,
    ///
    Sampler2d = 0x8B5E,
    ///
    SamplerCube = 0x8B60,
}

///
#[derive(Debug, Clone, Copy)]
pub enum TextureCompression {
    /// A DXT1-compressed image in an RGB image format.
    RgbDxt1 = 0x83F0,
    /// A DXT1-compressed image in an RGB image format with a simple on/off alpha value.
    RgbaDxt1 = 0x83F1,
    ///	A DXT3-compressed image in an RGBA image format.
    /// Compared to a 32-bit RGBA texture, it offers 4:1 compression.
    RgbaDxt3 = 0x83F2,
    /// A DXT5-compressed image in an RGBA image format.
    /// It also provides a 4:1 compression,
    /// but differs to the DXT3 compression in how the alpha compression is done.
    RgbaDxt5 = 0x83F3,
}
