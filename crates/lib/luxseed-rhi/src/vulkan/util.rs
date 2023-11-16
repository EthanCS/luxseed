use ash::vk;

use crate::{
    define::{ClearColor, ClearDepthStencil, RenderPassOutput, StencilOpState},
    enums::*,
};

use super::render_pass::VulkanRenderPassOutput;

impl From<Format> for vk::Format {
    fn from(item: Format) -> Self {
        match item {
            Format::B8G8R8A8_UNORM => vk::Format::B8G8R8A8_UNORM,
            Format::B8G8R8A8_SRGB => vk::Format::B8G8R8A8_SRGB,
            Format::R8G8B8A8_UNORM => vk::Format::R8G8B8A8_UNORM,
            Format::R8G8B8A8_SRGB => vk::Format::R8G8B8A8_SRGB,
            Format::B8G8R8_UNORM => vk::Format::B8G8R8_UNORM,
            Format::B8G8R8_SRGB => vk::Format::B8G8R8_SRGB,
            Format::R8G8B8_UNORM => vk::Format::R8G8B8_UNORM,
            Format::R8G8B8_SRGB => vk::Format::R8G8B8_SRGB,
            _ => vk::Format::UNDEFINED,
        }
    }
}

impl From<vk::Format> for Format {
    fn from(item: vk::Format) -> Self {
        match item {
            vk::Format::B8G8R8A8_UNORM => Format::B8G8R8A8_UNORM,
            vk::Format::B8G8R8A8_SRGB => Format::B8G8R8A8_SRGB,
            vk::Format::R8G8B8A8_UNORM => Format::R8G8B8A8_UNORM,
            vk::Format::R8G8B8A8_SRGB => Format::R8G8B8A8_SRGB,
            vk::Format::B8G8R8_UNORM => Format::B8G8R8_UNORM,
            vk::Format::B8G8R8_SRGB => Format::B8G8R8_SRGB,
            vk::Format::R8G8B8_UNORM => Format::R8G8B8_UNORM,
            vk::Format::R8G8B8_SRGB => Format::R8G8B8_SRGB,
            _ => Format::Unknown,
        }
    }
}

impl From<TextureViewAspectMask> for vk::ImageAspectFlags {
    fn from(item: TextureViewAspectMask) -> Self {
        match item {
            TextureViewAspectMask::Color => vk::ImageAspectFlags::COLOR,
            TextureViewAspectMask::Depth => vk::ImageAspectFlags::DEPTH,
            TextureViewAspectMask::Stencil => vk::ImageAspectFlags::STENCIL,
            _ => vk::ImageAspectFlags::empty(),
        }
    }
}

impl From<TextureViewType> for vk::ImageViewType {
    fn from(item: TextureViewType) -> Self {
        match item {
            TextureViewType::Texture1D => vk::ImageViewType::TYPE_1D,
            TextureViewType::Texture2D => vk::ImageViewType::TYPE_2D,
            TextureViewType::Texture3D => vk::ImageViewType::TYPE_3D,
            TextureViewType::TextureCube => vk::ImageViewType::CUBE,
            TextureViewType::Texture1DArray => vk::ImageViewType::TYPE_1D_ARRAY,
            TextureViewType::Texture2DArray => vk::ImageViewType::TYPE_2D_ARRAY,
            TextureViewType::TextureCubeArray => vk::ImageViewType::CUBE_ARRAY,
        }
    }
}

impl From<TextureType> for vk::ImageType {
    fn from(item: TextureType) -> Self {
        match item {
            TextureType::Texture1D => vk::ImageType::TYPE_1D,
            TextureType::Texture2D => vk::ImageType::TYPE_2D,
            TextureType::Texture3D => vk::ImageType::TYPE_3D,
        }
    }
}

impl From<TextureComponentSwizzle> for vk::ComponentSwizzle {
    fn from(item: TextureComponentSwizzle) -> Self {
        match item {
            TextureComponentSwizzle::Identity => vk::ComponentSwizzle::IDENTITY,
            TextureComponentSwizzle::Zero => vk::ComponentSwizzle::ZERO,
            TextureComponentSwizzle::One => vk::ComponentSwizzle::ONE,
            TextureComponentSwizzle::R => vk::ComponentSwizzle::R,
            TextureComponentSwizzle::G => vk::ComponentSwizzle::G,
            TextureComponentSwizzle::B => vk::ComponentSwizzle::B,
            TextureComponentSwizzle::A => vk::ComponentSwizzle::A,
        }
    }
}

impl From<ShaderStage> for vk::ShaderStageFlags {
    fn from(item: ShaderStage) -> Self {
        match item {
            ShaderStage::None => vk::ShaderStageFlags::empty(),
            ShaderStage::Vertex => vk::ShaderStageFlags::VERTEX,
            ShaderStage::Fragment => vk::ShaderStageFlags::FRAGMENT,
            ShaderStage::Compute => vk::ShaderStageFlags::COMPUTE,
        }
    }
}

impl From<BlendFactor> for vk::BlendFactor {
    fn from(item: BlendFactor) -> Self {
        match item {
            BlendFactor::Zero => vk::BlendFactor::ZERO,
            BlendFactor::One => vk::BlendFactor::ONE,
            BlendFactor::SrcColor => vk::BlendFactor::SRC_COLOR,
            BlendFactor::OneMinusSrcColor => vk::BlendFactor::ONE_MINUS_SRC_COLOR,
            BlendFactor::DstColor => vk::BlendFactor::DST_COLOR,
            BlendFactor::OneMinusDstColor => vk::BlendFactor::ONE_MINUS_DST_COLOR,
            BlendFactor::SrcAlpha => vk::BlendFactor::SRC_ALPHA,
            BlendFactor::OneMinusSrcAlpha => vk::BlendFactor::ONE_MINUS_SRC_ALPHA,
            BlendFactor::DstAlpha => vk::BlendFactor::DST_ALPHA,
            BlendFactor::OneMinusDstAlpha => vk::BlendFactor::ONE_MINUS_DST_ALPHA,
            BlendFactor::ConstantColor => vk::BlendFactor::CONSTANT_COLOR,
            BlendFactor::OneMinusConstantColor => vk::BlendFactor::ONE_MINUS_CONSTANT_COLOR,
            BlendFactor::ConstantAlpha => vk::BlendFactor::CONSTANT_ALPHA,
            BlendFactor::OneMinusConstantAlpha => vk::BlendFactor::ONE_MINUS_CONSTANT_ALPHA,
            BlendFactor::SrcAlphaSaturate => vk::BlendFactor::SRC_ALPHA_SATURATE,
            BlendFactor::Src1Color => vk::BlendFactor::SRC1_COLOR,
            BlendFactor::OneMinusSrc1Color => vk::BlendFactor::ONE_MINUS_SRC1_COLOR,
            BlendFactor::Src1Alpha => vk::BlendFactor::SRC1_ALPHA,
            BlendFactor::OneMinusSrc1Alpha => vk::BlendFactor::ONE_MINUS_SRC1_ALPHA,
        }
    }
}

impl From<CullMode> for vk::CullModeFlags {
    fn from(item: CullMode) -> Self {
        match item {
            CullMode::None => vk::CullModeFlags::NONE,
            CullMode::Front => vk::CullModeFlags::FRONT,
            CullMode::Back => vk::CullModeFlags::BACK,
            CullMode::FrontAndBack => vk::CullModeFlags::FRONT_AND_BACK,
        }
    }
}

impl From<FrontFace> for vk::FrontFace {
    fn from(item: FrontFace) -> Self {
        match item {
            FrontFace::CounterClockwise => vk::FrontFace::COUNTER_CLOCKWISE,
            FrontFace::Clockwise => vk::FrontFace::CLOCKWISE,
        }
    }
}

impl From<PolygonMode> for vk::PolygonMode {
    fn from(item: PolygonMode) -> Self {
        match item {
            PolygonMode::Line => vk::PolygonMode::LINE,
            PolygonMode::Fill => vk::PolygonMode::FILL,
            PolygonMode::Point => vk::PolygonMode::POINT,
        }
    }
}

impl From<CompareOp> for vk::CompareOp {
    fn from(item: CompareOp) -> Self {
        match item {
            CompareOp::Never => vk::CompareOp::NEVER,
            CompareOp::Less => vk::CompareOp::LESS,
            CompareOp::Equal => vk::CompareOp::EQUAL,
            CompareOp::LessOrEqual => vk::CompareOp::LESS_OR_EQUAL,
            CompareOp::Greater => vk::CompareOp::GREATER,
            CompareOp::NotEqual => vk::CompareOp::NOT_EQUAL,
            CompareOp::GreaterOrEqual => vk::CompareOp::GREATER_OR_EQUAL,
            CompareOp::Always => vk::CompareOp::ALWAYS,
        }
    }
}

impl From<StencilOp> for vk::StencilOp {
    fn from(item: StencilOp) -> Self {
        match item {
            StencilOp::Keep => vk::StencilOp::KEEP,
            StencilOp::Zero => vk::StencilOp::ZERO,
            StencilOp::Replace => vk::StencilOp::REPLACE,
            StencilOp::IncrementAndClamp => vk::StencilOp::INCREMENT_AND_CLAMP,
            StencilOp::DecrementAndClamp => vk::StencilOp::DECREMENT_AND_CLAMP,
            StencilOp::Invert => vk::StencilOp::INVERT,
            StencilOp::IncrementAndWrap => vk::StencilOp::INCREMENT_AND_WRAP,
            StencilOp::DecrementAndWrap => vk::StencilOp::DECREMENT_AND_WRAP,
        }
    }
}

impl From<StencilOpState> for vk::StencilOpState {
    fn from(item: StencilOpState) -> Self {
        vk::StencilOpState {
            fail_op: item.fail_op.into(),
            pass_op: item.pass_op.into(),
            depth_fail_op: item.depth_fail_op.into(),
            compare_op: item.compare_op.into(),
            compare_mask: item.compare_mask.into(),
            write_mask: item.write_mask.into(),
            reference: item.reference.into(),
        }
    }
}

impl From<BlendOp> for vk::BlendOp {
    fn from(item: BlendOp) -> Self {
        match item {
            BlendOp::Add => vk::BlendOp::ADD,
            BlendOp::Subtract => vk::BlendOp::SUBTRACT,
            BlendOp::ReverseSubtract => vk::BlendOp::REVERSE_SUBTRACT,
            BlendOp::Min => vk::BlendOp::MIN,
            BlendOp::Max => vk::BlendOp::MAX,
        }
    }
}

impl From<SampleCount> for vk::SampleCountFlags {
    fn from(item: SampleCount) -> Self {
        match item {
            SampleCount::Sample1 => vk::SampleCountFlags::TYPE_1,
            SampleCount::Sample2 => vk::SampleCountFlags::TYPE_2,
            SampleCount::Sample4 => vk::SampleCountFlags::TYPE_4,
            SampleCount::Sample8 => vk::SampleCountFlags::TYPE_8,
            SampleCount::Sample16 => vk::SampleCountFlags::TYPE_16,
            SampleCount::Sample32 => vk::SampleCountFlags::TYPE_32,
            SampleCount::Sample64 => vk::SampleCountFlags::TYPE_64,
        }
    }
}

impl From<RenderTargetLoadAction> for vk::AttachmentLoadOp {
    fn from(item: RenderTargetLoadAction) -> Self {
        match item {
            RenderTargetLoadAction::Load => vk::AttachmentLoadOp::LOAD,
            RenderTargetLoadAction::Clear => vk::AttachmentLoadOp::CLEAR,
            RenderTargetLoadAction::DontCare => vk::AttachmentLoadOp::DONT_CARE,
        }
    }
}

impl From<TextureUsage> for vk::ImageUsageFlags {
    fn from(value: TextureUsage) -> Self {
        match value {
            TextureUsage::TransferSrc => vk::ImageUsageFlags::TRANSFER_SRC,
            TextureUsage::TransferDst => vk::ImageUsageFlags::TRANSFER_DST,
            TextureUsage::Sampled => vk::ImageUsageFlags::SAMPLED,
            TextureUsage::Storage => vk::ImageUsageFlags::STORAGE,
            TextureUsage::ColorAttachment => vk::ImageUsageFlags::COLOR_ATTACHMENT,
            TextureUsage::DepthStencilAttachment => vk::ImageUsageFlags::DEPTH_STENCIL_ATTACHMENT,
            TextureUsage::TransientAttachment => vk::ImageUsageFlags::TRANSIENT_ATTACHMENT,
            TextureUsage::InputAttachment => vk::ImageUsageFlags::INPUT_ATTACHMENT,
        }
    }
}

impl From<CommandBufferLevel> for vk::CommandBufferLevel {
    fn from(value: CommandBufferLevel) -> Self {
        match value {
            CommandBufferLevel::Primary => vk::CommandBufferLevel::PRIMARY,
            CommandBufferLevel::Secondary => vk::CommandBufferLevel::SECONDARY,
        }
    }
}

impl From<ImageLayout> for vk::ImageLayout {
    fn from(value: ImageLayout) -> Self {
        match value {
            ImageLayout::Undefined => vk::ImageLayout::UNDEFINED,
            ImageLayout::General => vk::ImageLayout::GENERAL,
            ImageLayout::ColorAttachmentOptimal => vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL,
            ImageLayout::DepthStencilAttachmentOptimal => {
                vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL
            }
            ImageLayout::DepthStencilReadOnlyOptimal => {
                vk::ImageLayout::DEPTH_STENCIL_READ_ONLY_OPTIMAL
            }
            ImageLayout::ShaderReadOnlyOptimal => vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL,
            ImageLayout::TransferSrcOptimal => vk::ImageLayout::TRANSFER_SRC_OPTIMAL,
            ImageLayout::TransferDstOptimal => vk::ImageLayout::TRANSFER_DST_OPTIMAL,
            ImageLayout::Preinitialized => vk::ImageLayout::PREINITIALIZED,
            ImageLayout::PresentSrcKhr => vk::ImageLayout::PRESENT_SRC_KHR,
        }
    }
}

impl From<PipelineStage> for vk::PipelineStageFlags {
    fn from(value: PipelineStage) -> Self {
        match value {
            PipelineStage::TopOfPipe => vk::PipelineStageFlags::TOP_OF_PIPE,
            PipelineStage::DrawIndirect => vk::PipelineStageFlags::DRAW_INDIRECT,
            PipelineStage::VertexInput => vk::PipelineStageFlags::VERTEX_INPUT,
            PipelineStage::VertexShader => vk::PipelineStageFlags::VERTEX_SHADER,
            PipelineStage::TessellationControlShader => {
                vk::PipelineStageFlags::TESSELLATION_CONTROL_SHADER
            }
            PipelineStage::TessellationEvaluationShader => {
                vk::PipelineStageFlags::TESSELLATION_EVALUATION_SHADER
            }
            PipelineStage::GeometryShader => vk::PipelineStageFlags::GEOMETRY_SHADER,
            PipelineStage::FragmentShader => vk::PipelineStageFlags::FRAGMENT_SHADER,
            PipelineStage::EarlyFragmentTests => vk::PipelineStageFlags::EARLY_FRAGMENT_TESTS,
            PipelineStage::LateFragmentTests => vk::PipelineStageFlags::LATE_FRAGMENT_TESTS,
            PipelineStage::ColorAttachmentOutput => vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT,
            PipelineStage::ComputeShader => vk::PipelineStageFlags::COMPUTE_SHADER,
            PipelineStage::Transfer => vk::PipelineStageFlags::TRANSFER,
            PipelineStage::BottomOfPipe => vk::PipelineStageFlags::BOTTOM_OF_PIPE,
            PipelineStage::Host => vk::PipelineStageFlags::HOST,
            PipelineStage::AllGraphics => vk::PipelineStageFlags::ALL_GRAPHICS,
            PipelineStage::AllCommands => vk::PipelineStageFlags::ALL_COMMANDS,
        }
    }
}

impl From<RenderPassOutput> for VulkanRenderPassOutput {
    fn from(value: RenderPassOutput) -> Self {
        let mut ret = Self {
            num_colors: value.num_colors,
            depth_stencil_format: value.depth_stencil_format.into(),
            depth_stencil_final_layout: value.depth_stencil_final_layout.into(),
            depth_stencil_samples: value.depth_stencil_samples.into(),
            depth_load: value.depth_load.into(),
            stencil_load: value.stencil_load.into(),
            ..Default::default()
        };
        for i in 0..value.num_colors {
            ret.color_formats[i as usize] = value.color_formats[i as usize].into();
            ret.color_final_layouts[i as usize] = value.color_final_layouts[i as usize].into();
            ret.color_load[i as usize] = value.color_loads[i as usize].into();
            ret.color_samples[i as usize] = value.color_samples[i as usize].into();
        }
        ret
    }
}

impl From<ClearColor> for vk::ClearValue {
    fn from(value: ClearColor) -> Self {
        vk::ClearValue { color: vk::ClearColorValue { float32: value.value } }
    }
}

impl From<ClearDepthStencil> for vk::ClearValue {
    fn from(value: ClearDepthStencil) -> Self {
        vk::ClearValue {
            depth_stencil: vk::ClearDepthStencilValue {
                depth: value.depth,
                stencil: value.stencil,
            },
        }
    }
}
