use raw_window_handle::{RawDisplayHandle, RawWindowHandle};

use crate::{enums::*, pool::Handle, MAX_RENDER_TARGETS};

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
pub struct RHICreation<'a> {
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
pub struct SwapchainCreation {
    pub width: u32,
    pub height: u32,
    pub surface: Handle<Surface>,
    pub vsync: bool,
    pub format: Format,
}

pub const TEXTURE_DEFAULT_NAME: &str = "Texture_Default";

pub struct TextureCreation<'a> {
    pub name: &'a str,
    pub format: Format,
    pub extent: [u32; 3],
    pub texture_type: TextureType,
    pub texture_usage: TextureUsage,
    pub texture_tiling: TextureTiling,
    pub mip_levels: u32,
    pub array_layers: u32,
}

impl<'a> Default for TextureCreation<'a> {
    fn default() -> Self {
        Self {
            name: TEXTURE_DEFAULT_NAME,
            format: Format::R8G8B8A8_UNORM,
            extent: [1, 1, 0],
            texture_type: TextureType::Texture2D,
            texture_usage: TextureUsage::Sampled,
            texture_tiling: Default::default(),
            mip_levels: 1,
            array_layers: 1,
        }
    }
}

#[derive(Clone, Copy)]
pub struct TextureViewCreateDesc {
    pub format: Option<Format>,
    pub view_type: TextureViewType,
    pub base_mip_level: u8,
    pub level_count: u8,
    pub base_array_layer: u8,
    pub layer_count: u8,
    pub aspect_mask: TextureViewAspectMask,
    pub component_r: TextureComponentSwizzle,
    pub component_g: TextureComponentSwizzle,
    pub component_b: TextureComponentSwizzle,
    pub component_a: TextureComponentSwizzle,
}

impl Default for TextureViewCreateDesc {
    fn default() -> Self {
        Self {
            format: None,
            view_type: TextureViewType::Texture2D,
            base_mip_level: 0,
            level_count: 1,
            base_array_layer: 0,
            layer_count: 1,
            aspect_mask: TextureViewAspectMask::Color,
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
    pub stage: ShaderStage,
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
            cull_mode: CullMode::None,
            front_face: FrontFace::CounterClockwise,
            fill_mode: PolygonMode::Fill,
        }
    }
}
pub struct RasterPipelineCreateDesc<'a> {
    pub vertex_input_bindings: Option<&'a [VertexInputBinding<'a>]>,
    pub raster_state: RasterState,
    pub depth_state: DepthState,
    pub blend_states: &'a [BlendState],
    pub shader_stages: &'a [Handle<Shader>],
    pub render_pass_output: RenderPassOutput,
}

#[derive(Clone, Copy)]
pub struct ColorAttachment {
    pub view: Handle<TextureView>,
    pub clear_value: Option<[f32; 4]>,
    pub load_op: RenderTargetLoadAction,
    pub store_op: RenderTargetStoreAction,
}

#[derive(Clone, Copy)]
pub struct DepthStencilAttachment {
    pub view: Handle<TextureView>,
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
    pub color_views: &'a [Handle<TextureView>],
    pub depth_stencil_view: Option<Handle<TextureView>>,
}

pub struct QueueSubmitDesc<'a> {
    pub wait_semaphore: &'a [Handle<Semaphore>],
    pub wait_stage: &'a [PipelineStage],
    pub command_buffer: &'a [Handle<CommandBuffer>],
    pub finish_semaphore: &'a [Handle<Semaphore>],
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
    pub size: usize,
    pub usage: BufferUsage,
    pub sharing_mode: SharingMode,
    pub initial_data: Option<&'a [u8]>,
}

macro_rules! define_rhi_resources {
    ($($name:ident),*) => {
        $(#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
        pub struct $name;)*
    };
}

define_rhi_resources!(
    Device,
    Surface,
    Queue,
    Swapchain,
    Texture,
    TextureView,
    Shader,
    RasterPipeline,
    RenderPass,
    Framebuffer,
    CommandPool,
    CommandBuffer,
    Semaphore,
    Fence,
    Buffer
);
