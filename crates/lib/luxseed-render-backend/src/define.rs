use luxseed_utility::pool::Handle;
use raw_window_handle::{RawDisplayHandle, RawWindowHandle};
use smallvec::SmallVec;

use crate::{enums::*, flag::*};

pub const ERR_MSG_DEVICE_NOT_CREATED: &str = "Device not created.";
pub const ERR_MSG_QUEUE_NOT_FOUND: &str = "Queue not found.";

pub const MAX_DESCRIPTORS_PER_SET: usize = 16;
pub const MAX_RENDER_TARGETS: usize = 8;
pub const MAX_SHADER_STAGES: usize = 5;

#[derive(Clone)]
pub struct AdapterInfo {
    pub api_version: u32,
    pub driver_version: u32,
    pub vendor_id: u32,
    pub device_id: u32,
    pub device_type: AdapterType,
    pub device_name: String,
}

#[derive(Clone, Copy)]
pub struct RenderBackendCreateDesc<'a> {
    pub enable_debugging: bool,
    pub app_name: &'a str,
    pub app_version: u32,
    pub raw_display_handle: RawDisplayHandle,
}

#[derive(Clone, Copy)]
pub struct SurfaceCreateDesc {
    pub raw_display_handle: RawDisplayHandle,
    pub raw_window_handle: RawWindowHandle,
}

#[derive(Clone, Copy)]
pub struct SwapchainCreateDesc {
    pub width: u32,
    pub height: u32,
    pub surface: Handle<Surface>,
    pub vsync: bool,
    pub format: Format,
}

pub const TEXTURE_DEFAULT_NAME: &str = "Texture_Default";

pub struct ImageCreateDesc<'a> {
    pub name: &'a str,
    pub format: Format,
    pub extent: [u32; 3],
    pub type_: ImageType,
    pub usage: ImageUsageFlags,
    pub tiling: ImageTiling,
    pub mip_levels: u32,
    pub array_layers: u32,
    pub samples: SampleCount,
    pub initial_layout: ImageLayout,
}

impl<'a> ImageCreateDesc<'a> {
    pub fn new_2d(name: &'a str, format: Format, width: u32, height: u32) -> Self {
        Self {
            name,
            format,
            extent: [width, height, 1],
            type_: ImageType::Texture2D,
            usage: ImageUsageFlags::SAMPLED | ImageUsageFlags::TRANSFER_DST,
            tiling: ImageTiling::Optimal,
            mip_levels: 1,
            array_layers: 1,
            samples: SampleCount::Sample1,
            initial_layout: ImageLayout::Undefined,
        }
    }

    pub fn new_depth(name: &'a str, format: Format, width: u32, height: u32) -> Self {
        Self {
            name,
            format,
            extent: [width, height, 1],
            type_: ImageType::Texture2D,
            usage: ImageUsageFlags::DEPTH_STENCIL_ATTACHMENT,
            tiling: ImageTiling::Optimal,
            mip_levels: 1,
            array_layers: 1,
            samples: SampleCount::Sample1,
            initial_layout: ImageLayout::Undefined,
        }
    }
}

#[derive(Clone, Copy)]
pub struct ImageViewCreateDesc {
    pub format: Option<Format>,
    pub view_type: TextureViewType,
    pub base_mip_level: u8,
    pub level_count: u8,
    pub base_array_layer: u8,
    pub layer_count: u8,
    pub aspect_mask: ImageAspectFlags,
    pub component_r: TextureComponentSwizzle,
    pub component_g: TextureComponentSwizzle,
    pub component_b: TextureComponentSwizzle,
    pub component_a: TextureComponentSwizzle,
}

impl ImageViewCreateDesc {
    pub fn new_2d(
        override_format: Option<Format>,
        aspect_mask: ImageAspectFlags,
    ) -> ImageViewCreateDesc {
        Self {
            format: override_format,
            view_type: TextureViewType::Texture2D,
            base_mip_level: 0,
            level_count: 1,
            base_array_layer: 0,
            layer_count: 1,
            aspect_mask,
            component_r: TextureComponentSwizzle::Identity,
            component_g: TextureComponentSwizzle::Identity,
            component_b: TextureComponentSwizzle::Identity,
            component_a: TextureComponentSwizzle::Identity,
        }
    }
}

impl Default for ImageViewCreateDesc {
    fn default() -> Self {
        Self {
            format: None,
            view_type: TextureViewType::Texture2D,
            base_mip_level: 0,
            level_count: 1,
            base_array_layer: 0,
            layer_count: 1,
            aspect_mask: ImageAspectFlags::COLOR,
            component_r: TextureComponentSwizzle::Identity,
            component_g: TextureComponentSwizzle::Identity,
            component_b: TextureComponentSwizzle::Identity,
            component_a: TextureComponentSwizzle::Identity,
        }
    }
}

#[derive(Clone, Copy, Default)]
pub struct ClearColor {
    pub value: [f32; 4],
}

impl ClearColor {
    pub fn new(value: [f32; 4]) -> Self {
        Self { value }
    }
}

#[derive(Clone, Copy, Default)]
pub struct ClearDepthStencil {
    pub depth: f32,
    pub stencil: u32,
}

pub struct ShaderModuleCreation<'a> {
    pub name: &'a str,
    pub code: &'a [u32],
    pub stage: ShaderStageFlags,
    pub entry: &'a str,
}

#[derive(Clone, Copy)]
pub struct StencilOpState {
    pub fail_op: StencilOp,
    pub pass_op: StencilOp,
    pub depth_fail_op: StencilOp,
    pub compare_op: CompareOp,
    pub compare_mask: u8,
    pub write_mask: u8,
    pub reference: u8,
}

impl Default for StencilOpState {
    fn default() -> Self {
        Self {
            fail_op: StencilOp::Keep,
            pass_op: StencilOp::Keep,
            depth_fail_op: StencilOp::Keep,
            compare_op: CompareOp::Always,
            compare_mask: 0,
            write_mask: 0,
            reference: 0,
        }
    }
}

pub struct DepthState {
    pub depth_test_enable: bool,
    pub depth_write_enable: bool,
    pub depth_compare_mode: CompareOp,

    pub stencil_test_enable: bool,
    pub stencil_read_mask: u8,
    pub stencil_write_mask: u8,

    pub stencil_front: StencilOpState,
    pub stencil_back: StencilOpState,
}

impl Default for DepthState {
    fn default() -> Self {
        Self {
            depth_test_enable: true,
            depth_write_enable: true,
            depth_compare_mode: CompareOp::LessOrEqual,

            stencil_test_enable: false,
            stencil_read_mask: 0,
            stencil_write_mask: 0,

            stencil_front: StencilOpState::default(),
            stencil_back: StencilOpState::default(),
        }
    }
}

#[derive(Clone, Copy)]
pub struct BlendState {
    pub source_color: BlendFactor,
    pub destination_color: BlendFactor,
    pub color_op: BlendOp,

    pub source_alpha: BlendFactor,
    pub destination_alpha: BlendFactor,
    pub alpha_op: BlendOp,

    pub blend_enable: bool,
}

impl Default for BlendState {
    fn default() -> Self {
        Self {
            source_color: BlendFactor::One,
            destination_color: BlendFactor::One,
            color_op: BlendOp::Add,

            source_alpha: BlendFactor::One,
            destination_alpha: BlendFactor::One,
            alpha_op: BlendOp::Add,

            blend_enable: false,
        }
    }
}

pub struct RasterState {
    pub cull_mode: CullMode,
    pub front_face: FrontFace,
    pub fill_mode: PolygonMode,
}

impl Default for RasterState {
    fn default() -> Self {
        Self {
            cull_mode: CullMode::Back,
            front_face: FrontFace::Clockwise,
            fill_mode: PolygonMode::Fill,
        }
    }
}

pub struct PipelineLayoutCreateDesc<'a> {
    pub descriptor_set_layouts: &'a [Handle<DescriptorSetLayout>],
}

pub struct RasterPipelineCreateDesc<'a> {
    pub vertex_input_bindings: Option<&'a [VertexInputBinding<'a>]>,
    pub raster_state: RasterState,
    pub depth_state: DepthState,
    pub blend_states: &'a [BlendState],
    pub shader_stages: &'a [Handle<Shader>],
    pub render_pass_output: RenderPassOutput,
    pub pipeline_layout: Handle<PipelineLayout>,
}

#[derive(Clone, Copy)]
pub struct ColorAttachment {
    pub view: Handle<ImageView>,
    pub clear_value: Option<[f32; 4]>,
    pub load_op: RenderTargetLoadAction,
    pub store_op: RenderTargetStoreAction,
}

#[derive(Clone, Copy)]
pub struct DepthStencilAttachment {
    pub view: Handle<ImageView>,
    pub depth_clear_value: Option<f32>,
    pub depth_load_op: RenderTargetLoadAction,
    pub depth_store_op: RenderTargetStoreAction,
    pub stencil_clear_value: Option<u8>,
    pub stencil_load_op: RenderTargetLoadAction,
    pub stencil_store_op: RenderTargetStoreAction,
}

#[derive(Default, Clone, Copy)]
pub struct RenderPassOutput {
    pub num_colors: u8,
    pub color_formats: [Format; MAX_RENDER_TARGETS],
    pub color_final_layouts: [ImageLayout; MAX_RENDER_TARGETS],
    pub color_loads: [RenderTargetLoadAction; MAX_RENDER_TARGETS],
    pub color_samples: [SampleCount; MAX_RENDER_TARGETS],
    pub depth_stencil_format: Format,
    pub depth_stencil_final_layout: ImageLayout,
    pub depth_load: RenderTargetLoadAction,
    pub stencil_load: RenderTargetLoadAction,
    pub depth_stencil_samples: SampleCount,
}

impl RenderPassOutput {
    pub fn builder() -> RenderPassOutputBuilder {
        RenderPassOutputBuilder::default()
    }
}

#[derive(Default)]
pub struct RenderPassOutputBuilder {
    pub color_formats: Vec<Format>,
    pub color_final_layouts: Vec<ImageLayout>,
    pub color_loads: Vec<RenderTargetLoadAction>,
    pub color_samples: Vec<SampleCount>,
    pub depth_stencil_format: Format,
    pub depth_stencil_final_layout: ImageLayout,
    pub depth_load: RenderTargetLoadAction,
    pub stencil_load: RenderTargetLoadAction,
    pub depth_stencil_samples: SampleCount,
}

impl RenderPassOutputBuilder {
    pub fn reset(mut self) -> Self {
        self.color_formats.clear();
        self.color_final_layouts.clear();
        self.color_loads.clear();
        self.color_samples.clear();
        self.depth_stencil_format = Format::Unknown;
        self.depth_stencil_final_layout = ImageLayout::Undefined;
        self.depth_load = RenderTargetLoadAction::DontCare;
        self.stencil_load = RenderTargetLoadAction::DontCare;
        self.depth_stencil_samples = SampleCount::Sample1;
        self
    }

    pub fn add_color(
        mut self,
        color_formats: Format,
        color_final_layouts: ImageLayout,
        color_load: RenderTargetLoadAction,
        color_samples: SampleCount,
    ) -> Self {
        self.color_formats.push(color_formats);
        self.color_final_layouts.push(color_final_layouts);
        self.color_loads.push(color_load);
        self.color_samples.push(color_samples);
        self
    }

    pub fn set_depth_stencil(
        mut self,
        depth_stencil_format: Format,
        depth_stencil_final_layout: ImageLayout,
        depth_load: RenderTargetLoadAction,
        stencil_load: RenderTargetLoadAction,
        depth_stencil_samples: SampleCount,
    ) -> Self {
        self.depth_stencil_format = depth_stencil_format;
        self.depth_stencil_final_layout = depth_stencil_final_layout;
        self.depth_load = depth_load;
        self.stencil_load = stencil_load;
        self.depth_stencil_samples = depth_stencil_samples;
        self
    }

    pub fn build(self) -> RenderPassOutput {
        let num_colors = self.color_formats.len() as u8;
        let mut color_formats = [Default::default(); MAX_RENDER_TARGETS];
        let mut color_final_layouts = [Default::default(); MAX_RENDER_TARGETS];
        let mut color_loads = [RenderTargetLoadAction::default(); MAX_RENDER_TARGETS];
        let mut color_samples = [Default::default(); MAX_RENDER_TARGETS];
        for i in 0..num_colors {
            color_formats[i as usize] = *self.color_formats.get(i as usize).unwrap();
            color_final_layouts[i as usize] = *self.color_final_layouts.get(i as usize).unwrap();
            color_loads[i as usize] = *self.color_loads.get(i as usize).unwrap();
            color_samples[i as usize] = *self.color_samples.get(i as usize).unwrap();
        }

        RenderPassOutput {
            num_colors,
            color_formats,
            color_final_layouts,
            color_loads,
            color_samples,
            depth_stencil_format: self.depth_stencil_format,
            depth_stencil_final_layout: self.depth_stencil_final_layout,
            depth_load: self.depth_load,
            stencil_load: self.stencil_load,
            depth_stencil_samples: self.depth_stencil_samples,
        }
    }
}

#[derive(Clone, Copy)]
pub struct FramebufferCreateDesc<'a> {
    pub render_pass: Handle<RenderPass>,
    pub color_views: &'a [Handle<ImageView>],
    pub depth_stencil_view: Option<Handle<ImageView>>,
}

pub struct QueueSubmitDesc<'a> {
    pub wait_semaphore: Option<&'a [Handle<Semaphore>]>,
    pub wait_stage: Option<&'a [PipelineStageFlags]>,
    pub command_buffer: &'a [Handle<CommandBuffer>],
    pub finish_semaphore: Option<&'a [Handle<Semaphore>]>,
    pub fence: Option<Handle<Fence>>,
}

pub struct QueuePresentDesc<'a> {
    pub wait_semaphores: &'a [Handle<Semaphore>],
    pub swapchain: Handle<Swapchain>,
    pub image_index: u32,
}

pub struct VertexInputBinding<'a> {
    pub stride: usize,
    pub attributes: &'a [VertexInputAttribute],
    pub input_rate: VertexInputRate,
}

pub struct VertexInputAttribute {
    pub offset: usize,
    pub format: Format,
}

pub struct BufferCreateDesc<'a> {
    pub name: &'a str,
    pub size: u64,
    pub usage: BufferUsageFlags,
    pub memory: MemoryLocation,
    pub initial_data: Option<&'a [u8]>,
}

#[derive(Default, Clone, Copy)]
pub struct BufferCopyRegion {
    pub src_offset: u64,
    pub dst_offset: u64,
    pub size: u64,
}

#[derive(Clone, Copy)]
pub struct BufferImageCopyRegion {
    pub buffer_offset: u64,
    pub buffer_row_length: u32,
    pub buffer_image_height: u32,
    pub aspect_mask: ImageAspectFlags,
    pub mip_level: u32,
    pub base_array_layer: u32,
    pub layer_count: u32,
    pub image_offset: [i32; 3],
    pub image_extent: [u32; 3],
}

pub struct DescriptorPoolSize {
    pub descriptor_type: DescriptorType,
    pub descriptor_count: u32,
}

pub struct DescriptorPoolCreateDesc<'a> {
    pub max_sets: u32,
    pub pool_sizes: &'a [DescriptorPoolSize],
}

#[derive(Clone, Copy)]
pub struct DescriptorSetLayoutBinding {
    pub binding: u32,
    pub descriptor_type: DescriptorType,
    pub descriptor_count: u32,
    pub stage_flags: ShaderStageFlags,
}

#[derive(Clone, Copy)]
pub struct DescriptorBindingInfo {
    pub index: u16,
    pub type_: DescriptorType,
    pub count: u16,
    pub stage_flags: ShaderStageFlags,
}

pub struct DescriptorSetLayoutCreateDesc {
    pub bindings: SmallVec<[DescriptorBindingInfo; MAX_DESCRIPTORS_PER_SET]>,
}

impl DescriptorSetLayoutCreateDesc {
    pub fn new() -> Self {
        Self { bindings: SmallVec::new() }
    }

    pub fn add_binding_info(mut self, binding: DescriptorBindingInfo) -> Self {
        self.bindings.push(binding);
        self
    }
}

#[derive(Clone, Copy)]
pub struct DescriptorBindingData {
    pub binding: u16,
    pub buffer: Option<Handle<Buffer>>,
    pub sampler: Option<Handle<Sampler>>,
    pub image_view: Option<Handle<ImageView>>,
}

pub struct DescriptorSetCreateDesc {
    pub pool: Handle<DescriptorPool>,
    pub layout: Handle<DescriptorSetLayout>,
    pub bindings: SmallVec<[DescriptorBindingData; MAX_DESCRIPTORS_PER_SET]>,
}

impl DescriptorSetCreateDesc {
    pub fn new(pool: Handle<DescriptorPool>, layout: Handle<DescriptorSetLayout>) -> Self {
        Self { pool, layout, bindings: SmallVec::new() }
    }

    pub fn bind_image_view(mut self, binding: u16, image_view: Handle<ImageView>) -> Self {
        self.bindings.push(DescriptorBindingData {
            binding: binding,
            buffer: None,
            sampler: None,
            image_view: Some(image_view),
        });
        self
    }

    pub fn bind_combined_image_sampler(
        mut self,
        binding: u16,
        image_view: Handle<ImageView>,
        sampler: Handle<Sampler>,
    ) -> Self {
        self.bindings.push(DescriptorBindingData {
            binding: binding,
            buffer: None,
            sampler: Some(sampler),
            image_view: Some(image_view),
        });
        self
    }

    pub fn bind_uniform_buffer(mut self, binding: u16, buffer: Handle<Buffer>) -> Self {
        self.bindings.push(DescriptorBindingData {
            binding: binding,
            buffer: Some(buffer),
            sampler: None,
            image_view: None,
        });
        self
    }
}

pub struct MemoryBarrier {
    pub src_queue_family_index: u32,
    pub dst_queue_family_index: u32,
    pub src_access_mask: AccessFlag,
    pub dst_access_mask: AccessFlag,
}

pub struct BufferMemoryBarrier {
    pub buffer: Handle<Buffer>,
    pub src_queue_family_index: u32,
    pub dst_queue_family_index: u32,
    pub src_access_mask: AccessFlag,
    pub dst_access_mask: AccessFlag,
}

pub struct ImageMemoryBarrier {
    pub image: Handle<Image>,
    pub aspect_mask: ImageAspectFlags,
    pub base_mip_level: u32,
    pub level_count: u32,
    pub base_array_layer: u32,
    pub layer_count: u32,
    pub old_layout: ImageLayout,
    pub new_layout: ImageLayout,
    pub src_queue_family_index: Option<u32>,
    pub dst_queue_family_index: Option<u32>,
    pub src_access_mask: AccessFlags,
    pub dst_access_mask: AccessFlags,
}

pub struct SamplerCreateDesc {
    pub min_filter: FilterType,
    pub mag_filter: FilterType,
    pub mipmap_mode: SamplerMipmapMode,
    pub address_mode_u: SamplerAddressMode,
    pub address_mode_v: SamplerAddressMode,
    pub address_mode_w: SamplerAddressMode,
    pub mip_lod_bias: f32,
    pub compare_op: Option<CompareOp>,
    pub max_anisotropy: Option<f32>,
}

#[derive(Default, Clone, Copy)]
pub struct CommandBufferBeginDesc {
    pub one_time_submit: bool,
}

macro_rules! define_rhi_resources {
    ($($name:ident),*) => {
        $(#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
        pub struct $name;)*
    };
}

define_rhi_resources!(
    Surface,
    Queue,
    Swapchain,
    Image,
    ImageView,
    Sampler,
    Shader,
    PipelineLayout,
    RasterPipeline,
    RenderPass,
    Framebuffer,
    CommandPool,
    CommandBuffer,
    Semaphore,
    Fence,
    Buffer,
    DescriptorSetLayout,
    DescriptorPool,
    DescriptorSet
);
