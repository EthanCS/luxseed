use core::fmt;

#[derive(Default, Clone, Copy)]
pub enum BackendType {
    #[default]
    Unknown,
    Vulkan,
    D3D12,
    Metal,
}

#[derive(Clone, Copy)]
pub enum AdapterType {
    Other,
    IntegratedGPU,
    DiscreteGPU,
    VirtualGPU,
    CPU,
}

#[derive(Clone, Copy, Debug)]
pub enum QueueType {
    Graphics,
    Compute,
    Transfer,
    Present,
}

impl fmt::Display for QueueType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            QueueType::Graphics => write!(f, "Graphics"),
            QueueType::Compute => write!(f, "Compute"),
            QueueType::Transfer => write!(f, "Transfer"),
            QueueType::Present => write!(f, "Present"),
        }
    }
}

#[derive(Default, Clone, Copy, Debug)]
pub enum TextureType {
    Texture1D,
    #[default]
    Texture2D,
    Texture3D,
}

#[derive(Default, Clone, Copy, Debug)]
pub enum TextureViewType {
    Texture1D,
    #[default]
    Texture2D,
    Texture3D,
    TextureCube,
    Texture1DArray,
    Texture2DArray,
    TextureCubeArray,
}

#[derive(Default, Clone, Copy, Debug)]
pub enum TextureViewAspectMask {
    #[default]
    Unknown,
    Color,
    Depth,
    Stencil,
}

pub enum TextureUsage {
    TransferSrc,
    TransferDst,
    Sampled,
    Storage,
    ColorAttachment,
    DepthStencilAttachment,
    TransientAttachment,
    InputAttachment,
}

#[derive(Default, Clone, Copy)]
pub enum TextureTiling {
    #[default]
    Optimal,
    Linear,
}

#[derive(Default, Clone, Copy)]
pub enum TextureComponentSwizzle {
    #[default]
    Identity,
    Zero,
    One,
    R,
    G,
    B,
    A,
}

#[derive(Clone, Copy)]
pub enum ShaderStage {
    None,
    Vertex,
    Fragment,
    Compute,
}

#[derive(Default, Clone, Copy, Debug, Hash)]
#[allow(non_camel_case_types)]
pub enum Format {
    #[default]
    Unknown,
    B8G8R8A8_UNORM,
    B8G8R8A8_SRGB,
    R8G8B8A8_UNORM,
    R8G8B8A8_SRGB,
    B8G8R8_UNORM,
    B8G8R8_SRGB,
    R8G8B8_UNORM,
    R8G8B8_SRGB,
    R32_SFLOAT,
    R32G32_SFLOAT,
    R32G32B32_SFLOAT,
    R32G32B32A32_SFLOAT,
}

#[derive(Default, Clone, Copy, Debug, Hash)]
pub enum ImageLayout {
    #[default]
    Undefined,
    General,
    ColorAttachmentOptimal,
    DepthStencilAttachmentOptimal,
    DepthStencilReadOnlyOptimal,
    ShaderReadOnlyOptimal,
    TransferSrcOptimal,
    TransferDstOptimal,
    Preinitialized,
    PresentSrcKhr,
}

#[derive(Clone, Copy)]
pub enum BlendFactor {
    Zero,
    One,
    SrcColor,
    OneMinusSrcColor,
    DstColor,
    OneMinusDstColor,
    SrcAlpha,
    OneMinusSrcAlpha,
    DstAlpha,
    OneMinusDstAlpha,
    ConstantColor,
    OneMinusConstantColor,
    ConstantAlpha,
    OneMinusConstantAlpha,
    SrcAlphaSaturate,
    Src1Color,
    OneMinusSrc1Color,
    Src1Alpha,
    OneMinusSrc1Alpha,
}

#[derive(Clone, Copy)]
pub enum BlendOp {
    Add,
    Subtract,
    ReverseSubtract,
    Min,
    Max,
}

#[derive(Clone, Copy)]
pub enum CompareOp {
    Never,
    Less,
    Equal,
    LessOrEqual,
    Greater,
    NotEqual,
    GreaterOrEqual,
    Always,
}

#[derive(Clone, Copy)]
pub enum StencilOp {
    Keep,
    Zero,
    Replace,
    IncrementAndClamp,
    DecrementAndClamp,
    Invert,
    IncrementAndWrap,
    DecrementAndWrap,
}

#[derive(Clone, Copy)]
pub enum CullMode {
    None,
    Front,
    Back,
    FrontAndBack,
}

#[derive(Clone, Copy)]
pub enum FrontFace {
    CounterClockwise,
    Clockwise,
}

#[derive(Clone, Copy)]
pub enum PolygonMode {
    Line,
    Fill,
    Point,
}

#[derive(Default, Clone, Copy, Hash)]
pub enum SampleCount {
    #[default]
    Sample1,
    Sample2,
    Sample4,
    Sample8,
    Sample16,
    Sample32,
    Sample64,
}

#[derive(Default, Clone, Copy, Hash)]
pub enum RenderTargetLoadAction {
    #[default]
    Load,
    Clear,
    DontCare,
}

#[derive(Default, Clone, Copy, Hash)]
pub enum RenderTargetStoreAction {
    #[default]
    Store,
    DontCare,
}

#[derive(Default, Clone, Copy, Hash)]
pub enum CommandBufferLevel {
    #[default]
    Primary,
    Secondary,
}

#[derive(Clone, Copy, Hash)]
pub enum PipelineStage {
    TopOfPipe,
    DrawIndirect,
    VertexInput,
    VertexShader,
    TessellationControlShader,
    TessellationEvaluationShader,
    GeometryShader,
    FragmentShader,
    EarlyFragmentTests,
    LateFragmentTests,
    ColorAttachmentOutput,
    ComputeShader,
    Transfer,
    BottomOfPipe,
    Host,
    AllGraphics,
    AllCommands,
}

#[derive(Clone, Copy)]
pub enum VertexInputRate {
    Vertex,
    Instance,
}
