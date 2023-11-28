use bitflags::bitflags;

bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
    pub struct BufferUsageFlag : u32 {
        const TRANSFER_SRC = 0b1;
        const TRANSFER_DST = 0b10;
        const UNIFORM_TEXEL_BUFFER = 0b100;
        const STORAGE_TEXEL_BUFFER = 0b1000;
        const UNIFORM_BUFFER = 0b1_0000;
        const STORAGE_BUFFER = 0b10_0000;
        const INDEX_BUFFER = 0b100_0000;
        const VERTEX_BUFFER = 0b1000_0000;
        const INDIRECT_BUFFER = 0b1_0000_0000;
    }
}

bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
    pub struct TextureUsageFlag : u32 {
        const TRANSFER_SRC = 0b1;
        const TRANSFER_DST = 0b10;
        const SAMPLED = 0b100;
        const STORAGE = 0b1000;
        const COLOR_ATTACHMENT = 0b1_0000;
        const DEPTH_STENCIL_ATTACHMENT = 0b10_0000;
        const TRANSIENT_ATTACHMENT = 0b100_0000;
        const INPUT_ATTACHMENT = 0b1000_0000;
    }
}

bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
    pub struct PipelineStageFlag : u32 {
        const TOP_OF_PIPE = 0b1;
        const DRAW_INDIRECT = 0b10;
        const VERTEX_INPUT = 0b100;
        const VERTEX_SHADER = 0b1000;
        const TESSELLATION_CONTROL_SHADER = 0b1_0000;
        const TESSELLATION_EVALUATION_SHADER = 0b10_0000;
        const GEOMETRY_SHADER = 0b100_0000;
        const FRAGMENT_SHADER = 0b1000_0000;
        const EARLY_FRAGMENT_TESTS = 0b1_0000_0000;
        const LATE_FRAGMENT_TESTS = 0b10_0000_0000;
        const COLOR_ATTACHMENT_OUTPUT = 0b100_0000_0000;
        const COMPUTE_SHADER = 0b1000_0000_0000;
        const TRANSFER = 0b1_0000_0000_0000;
        const BOTTOM_OF_PIPE = 0b10_0000_0000_0000;
        const HOST = 0b100_0000_0000_0000;
        const ALL_GRAPHICS = 0b1000_0000_0000_0000;
        const ALL_COMMANDS = 0b1_0000_0000_0000_0000;
    }
}
