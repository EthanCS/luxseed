use bitflags::bitflags;

bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
    pub struct ShaderStageFlags : u32 {
        const VERTEX = 0b1;
        const TESSELLATION_CONTROL = 0b10;
        const TESSELLATION_EVALUATION = 0b100;
        const GEOMETRY = 0b1000;
        const FRAGMENT = 0b1_0000;
        const COMPUTE = 0b10_0000;
        const ALL_GRAPHICS = 0x0000_001F;
        const ALL = 0x7FFF_FFFF;
    }
}

bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
    pub struct BufferUsageFlags : u32 {
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
    pub struct ImageUsageFlags : u32 {
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
    pub struct PipelineStageFlags : u32 {
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

bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
    pub struct ImageAspectFlags : u32 {
        const COLOR = 0b1;
        const DEPTH = 0b10;
        const STENCIL = 0b100;
    }
}

bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
    pub struct AccessFlags : u32 {
        const INDIRECT_COMMAND_READ = 0b1;
        const INDEX_READ = 0b10;
        const VERTEX_ATTRIBUTE_READ = 0b100;
        const UNIFORM_READ = 0b1000;
        const INPUT_ATTACHMENT_READ = 0b1_0000;
        const SHADER_READ = 0b10_0000;
        const SHADER_WRITE = 0b100_0000;
        const COLOR_ATTACHMENT_READ = 0b1000_0000;
        const COLOR_ATTACHMENT_WRITE = 0b1_0000_0000;
        const DEPTH_STENCIL_ATTACHMENT_READ = 0b10_0000_0000;
        const DEPTH_STENCIL_ATTACHMENT_WRITE = 0b100_0000_0000;
        const TRANSFER_READ = 0b1000_0000_0000;
        const TRANSFER_WRITE = 0b1_0000_0000_0000;
        const HOST_READ = 0b10_0000_0000_0000;
        const HOST_WRITE = 0b100_0000_0000_0000;
        const MEMORY_READ = 0b1000_0000_0000_0000;
        const MEMORY_WRITE = 0b1_0000_0000_0000_0000;
    }
}
